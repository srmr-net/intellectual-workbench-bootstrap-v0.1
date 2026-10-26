//! ADR-003: Linux local-directory admission and exact, immutable releases.
//! Store is private to the trusted host UID on a local filesystem supporting fsync
//! and atomic rename. Source trees are hostile; the host store/tools are not.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    },
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use wb_semantic::{bounded_worker, CheckedMetadata};
pub const CONTRACT_VERSION: &str = "0.1";
static NEXT: AtomicU64 = AtomicU64::new(1);
const MAX_INDEX: u64 = 4096;
const MAX_RECORD: u64 = 4 * 1024 * 1024;
type Result<T> = std::result::Result<T, RegistryError>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasePin {
    pub id: String,
    pub version: String,
    pub sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionReceipt {
    pub pin: ReleasePin,
    pub record_digest: String,
}
#[derive(Clone, Debug)]
pub struct ValidatedHandle {
    receipt: AdmissionReceipt,
    store: (u64, u64),
}
impl ValidatedHandle {
    pub fn pin(&self) -> &ReleasePin {
        &self.receipt.pin
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentSummary {
    pub pin: ReleasePin,
    pub capabilities: Vec<String>,
    pub trust: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{code} at {location}")]
#[serde(deny_unknown_fields)]
pub struct RegistryError {
    pub code: String,
    pub location: String,
}
fn err(code: &str, location: &str) -> RegistryError {
    RegistryError {
        code: code.into(),
        location: location
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || "._/-#".contains(*c))
            .take(240)
            .collect(),
    }
}
fn io<T>(v: std::io::Result<T>) -> Result<T> {
    v.map_err(|_| err("REGISTRY_IO", "registry"))
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn slot(pin: &ReleasePin) -> String {
    hash(format!("{}\0{}", pin.id, pin.version).as_bytes())
}
fn hex(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Trusted native configuration, never package-controlled executable paths.
#[derive(Clone)]
pub struct Policy {
    pub allowed_roots: Vec<PathBuf>,
    pub python: PathBuf,
    pub semantic_worker: PathBuf,
    pub abi_worker: PathBuf,
    pub max_files: usize,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
    pub max_depth: usize,
    pub max_store_bytes: u64,
    pub max_releases: usize,
    pub seconds: u64,
    /// ADR-004 opt-in; default ADR-003 admission still rejects UI packages.
    pub semantic_ui_v01: bool,
}
impl Policy {
    pub fn local(
        allowed_roots: Vec<PathBuf>,
        python: PathBuf,
        semantic_worker: PathBuf,
        abi_worker: PathBuf,
    ) -> Self {
        Self {
            allowed_roots,
            python,
            semantic_worker,
            abi_worker,
            max_files: 64,
            max_file_bytes: 2 * 1024 * 1024,
            max_total_bytes: 8 * 1024 * 1024,
            max_depth: 8,
            max_store_bytes: 128 * 1024 * 1024,
            max_releases: 128,
            seconds: 30,
            semantic_ui_v01: false,
        }
    }
}
#[derive(Clone)]
pub struct Registry {
    root: Arc<File>,
    policy: Policy,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    format: u32,
    pin: ReleasePin,
    origin: String,
    checked: CheckedMetadata,
    files: BTreeMap<String, String>,
}
struct Lock(File);
impl Drop for Lock {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.0.as_raw_fd(), libc::LOCK_UN);
        }
    }
}
struct Stage(PathBuf);
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
/// Observer checkpoints allow real process-termination validation of write ordering.
/// Normal admission uses a no-op observer; observers cannot bypass any validator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checkpoint {
    Copied,
    Validated,
    FilesSynced,
    ObjectRenamed,
    IndexWritten,
    IndexRenamed,
    Committed,
}

// Open each path component with O_NOFOLLOW. Root/ancestor symlinks are rejected too.
fn open_at(parent: &File, name: &std::ffi::OsStr, directory: bool) -> Result<File> {
    use std::os::unix::ffi::OsStrExt;
    let name = CString::new(name.as_bytes()).map_err(|_| err("INVALID_PATH", "package"))?;
    let flags = libc::O_RDONLY
        | libc::O_NOFOLLOW
        | libc::O_CLOEXEC
        | libc::O_NONBLOCK
        | if directory { libc::O_DIRECTORY } else { 0 };
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
    if fd < 0 {
        return Err(err("INVALID_PATH", "package"));
    }
    // SAFETY: unique ownership of the descriptor returned by openat.
    Ok(unsafe { File::from_raw_fd(fd) })
}
fn absolute_dir(path: &Path) -> Result<File> {
    if !path.is_absolute() {
        return Err(err("INVALID_PATH", "source"));
    }
    let mut dir = io(File::open("/"))?;
    for part in path.components() {
        match part {
            Component::RootDir => {}
            Component::Normal(name) => dir = open_at(&dir, name, true)?,
            _ => return Err(err("INVALID_PATH", "source")),
        }
    }
    Ok(dir)
}
fn fd_path(file: &File) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()))
}
fn mkdir(path: &Path) -> Result<()> {
    match fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let m = io(fs::symlink_metadata(path))?;
            if !m.is_dir() || m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o077 != 0 {
                return Err(err("CAPABILITY_DENIED", "registry"));
            }
            Ok(())
        }
        Err(_) => Err(err("REGISTRY_IO", "registry")),
    }
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f = io(OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path))?;
    io(f.write_all(bytes))?;
    io(f.sync_all())
}
fn sync_dir(path: &Path) -> Result<()> {
    io(io(File::open(path))?.sync_all())
}
fn regular_read(path: &Path, max: u64) -> Result<Vec<u8>> {
    let file = io(OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path))?;
    let m = io(file.metadata())?;
    if !m.is_file() || m.nlink() != 1 || m.len() > max {
        return Err(err("INTEGRITY_FAILURE", "registry"));
    }
    let mut bytes = Vec::new();
    io(file.take(max + 1).read_to_end(&mut bytes))?;
    if bytes.len() as u64 > max {
        return Err(err("INTEGRITY_FAILURE", "registry"));
    }
    Ok(bytes)
}
impl Registry {
    pub fn open(root: &Path, policy: Policy) -> Result<Self> {
        if policy.seconds == 0 || policy.max_files == 0 || policy.max_total_bytes == 0 {
            return Err(err("QUOTA_EXCEEDED", "policy"));
        }
        // Parent must already exist; traversal of every ancestor is no-follow.
        let parent = absolute_dir(
            root.parent()
                .ok_or_else(|| err("INVALID_PATH", "registry"))?,
        )?;
        let name = root
            .file_name()
            .ok_or_else(|| err("INVALID_PATH", "registry"))?;
        mkdir(&fd_path(&parent).join(name))?;
        io(parent.sync_all())?;
        let root = Arc::new(open_at(&parent, name, true)?);
        let metadata = io(root.metadata())?;
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o077 != 0 {
            return Err(err("CAPABILITY_DENIED", "registry"));
        }
        let registry = Self { root, policy };
        let _lock = registry.lock()?;
        for name in ["objects", "releases", ".staging"] {
            mkdir(&registry.path().join(name))?;
        }
        io(registry.root.sync_all())?;
        registry.recover_locked()?;
        Ok(registry)
    }
    fn path(&self) -> PathBuf {
        fd_path(&self.root)
    }
    /// Native host identity check for separately opened adjacent storage. No path or grants.
    pub fn identity(&self) -> Result<(u64, u64)> {
        let m = io(self.root.metadata())?;
        Ok((m.dev(), m.ino()))
    }
    fn lock(&self) -> Result<Lock> {
        let f = io(OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(self.path().join(".lock")))?;
        let m = io(f.metadata())?;
        if !m.is_file() || m.nlink() != 1 {
            return Err(err("INTEGRITY_FAILURE", "registry"));
        }
        let deadline = Instant::now() + Duration::from_secs(self.policy.seconds);
        loop {
            if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
                return Ok(Lock(f));
            }
            if Instant::now() >= deadline {
                return Err(err("QUOTA_EXCEEDED", "registry-lock"));
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
    fn recover_locked(&self) -> Result<()> {
        // An uncommitted stage never grants visibility. Kernel releases all flock
        // ownership after SIGKILL; reopening serializes cleanup with installers.
        for entry in io(fs::read_dir(self.path().join(".staging")))? {
            let path = io(entry)?.path();
            if io(fs::symlink_metadata(&path))?.is_dir() {
                io(fs::remove_dir_all(path))?;
            } else {
                io(fs::remove_file(path))?;
            }
        }
        sync_dir(&self.path().join(".staging"))?;
        let mut live = BTreeSet::new();
        let mut uncertain = false;
        for e in io(fs::read_dir(self.path().join("releases")))? {
            let path = io(e)?.path();
            // If publication history is corrupt, do not destroy possibly pinned
            // objects. Manual host repair is required, never auto-substitution.
            let parsed = regular_read(&path, MAX_INDEX)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<AdmissionReceipt>(&bytes).ok());
            if let Some(receipt) = parsed {
                if !hex(&receipt.record_digest)
                    || path.file_name().and_then(|n| n.to_str()) != Some(&slot(&receipt.pin))
                    || self.load(&receipt.pin).is_err()
                {
                    uncertain = true;
                }
                live.insert(receipt.record_digest);
            } else {
                uncertain = true;
            }
        }
        // Complete durability of any marker observed after a prior process died
        // between rename and directory sync, before acknowledging recovered pins.
        sync_dir(&self.path().join("releases"))?;
        if uncertain {
            return Ok(());
        }
        for e in io(fs::read_dir(self.path().join("objects")))? {
            let e = io(e)?;
            if !live.contains(&e.file_name().to_string_lossy().into_owned()) {
                let m = io(fs::symlink_metadata(e.path()))?;
                if m.is_dir() {
                    io(fs::remove_dir_all(e.path()))?;
                } else {
                    io(fs::remove_file(e.path()))?;
                }
            }
        }
        sync_dir(&self.path().join("objects"))
    }
    pub fn admit_directory(&self, source: &Path) -> Result<AdmissionReceipt> {
        self.admit_observed(source, |_| {})
    }
    pub fn admit_observed(
        &self,
        source: &Path,
        mut observe: impl FnMut(Checkpoint),
    ) -> Result<AdmissionReceipt> {
        let _lock = self.lock()?;
        self.recover_locked()?;
        let source = self.open_source(source)?;
        let (used, _) = self.store_usage()?;
        // Reserve bounded staging + checked metadata + index bytes before writing.
        // Byte quota counts logical file contents, not filesystem block allocation.
        if used
            .saturating_add(self.policy.max_total_bytes)
            .saturating_add(4 * 1024 * 1024 + 4096)
            > self.policy.max_store_bytes
        {
            return Err(err("QUOTA_EXCEEDED", "registry"));
        }
        let stage = Stage(self.path().join(".staging").join(format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        mkdir(&stage.0)?;
        mkdir(&stage.0.join("payload"))?;
        let mut state = CopyState {
            files: BTreeMap::new(),
            total: 0,
            entries: 0,
            deadline: Instant::now() + Duration::from_secs(self.policy.seconds),
        };
        self.copy_tree(&source, &stage.0.join("payload"), "", 0, &mut state)?;
        observe(Checkpoint::Copied);
        let worker_stage = PathBuf::from(format!(
            "/proc/{}/fd/{}",
            std::process::id(),
            self.root.as_raw_fd()
        ))
        .join(".staging")
        .join(stage.0.file_name().unwrap());
        let payload = worker_stage.join("payload");
        let mut arguments = vec![self.policy.semantic_worker.as_path(), payload.as_path()];
        if self.policy.semantic_ui_v01 {
            arguments.push(Path::new("--semantic-ui-v0.1"));
        }
        let output = bounded_worker(&self.policy.python, &arguments, self.policy.seconds)
            .map_err(|code| err(code, "metadata-worker"))?;
        let reply: serde_json::Value = serde_json::from_slice(&output)
            .map_err(|_| err("INVALID_MANIFEST", "metadata-worker"))?;
        if let Some(error) = reply.get("error") {
            return Err(serde_json::from_value(error.clone())
                .map_err(|_| err("INVALID_MANIFEST", "metadata-worker"))?);
        }
        let checked: CheckedMetadata = serde_json::from_value(reply["ok"].clone())
            .map_err(|_| err("INVALID_MANIFEST", "metadata-worker"))?;
        self.check_ui(&checked, &stage.0.join("payload"), &state.files)?;
        if checked.trust != "unreviewed" {
            return Err(err("CAPABILITY_DENIED", "trust"));
        }
        write_new(
            &stage.0.join("checked.json"),
            &serde_json::to_vec(&checked).unwrap(),
        )?;
        let output = bounded_worker(
            &self.policy.abi_worker,
            &[&worker_stage],
            self.policy.seconds,
        )
        .map_err(|code| err(code, "abi-worker"))?;
        if output != b"VALID" {
            return Err(err("ABI_INCOMPATIBLE", "logic.wasm"));
        }
        io(fs::remove_file(stage.0.join("checked.json")))?;
        let m = &checked.manifest;
        let pin = ReleasePin {
            id: m["id"].as_str().unwrap().into(),
            version: m["version"].as_str().unwrap().into(),
            sha256: m["runtime"]["sha256"].as_str().unwrap().into(),
        };
        let record = Record {
            format: 1,
            pin: pin.clone(),
            origin: "local-directory-untrusted".into(),
            checked,
            files: state.files,
        };
        let bytes = serde_json::to_vec(&record).unwrap();
        let receipt = AdmissionReceipt {
            pin,
            record_digest: hash(&bytes),
        };
        if bytes.len() as u64 > MAX_RECORD
            || serde_json::to_vec(&receipt).unwrap().len() as u64 > MAX_INDEX
        {
            return Err(err("QUOTA_EXCEEDED", "release-record"));
        }
        let index = self.path().join("releases").join(slot(&receipt.pin));
        observe(Checkpoint::Validated);
        if index.exists() {
            let prior_bytes = regular_read(&index, MAX_INDEX)?;
            let prior_receipt: AdmissionReceipt = serde_json::from_slice(&prior_bytes)
                .map_err(|_| err("INTEGRITY_FAILURE", "registry"))?;
            if prior_receipt.pin != receipt.pin {
                return Err(err("RELEASE_CONFLICT", "component.json"));
            }
            let (prior, _) = self.load(&receipt.pin)?;
            if prior != receipt {
                return Err(err("RELEASE_CONFLICT", "component.json"));
            }
            sync_dir(&self.path().join("releases"))?;
            return Ok(prior);
        }
        // Quotas include all committed and uncommitted bytes before promotion.
        let (size, count) = self.store_usage()?;
        if size
            .saturating_add(state.total)
            .saturating_add(bytes.len() as u64)
            .saturating_add(MAX_INDEX)
            .saturating_add(self.policy.max_total_bytes)
            .saturating_add(MAX_RECORD + MAX_INDEX)
            > self.policy.max_store_bytes
            || count >= self.policy.max_releases
        {
            return Err(err("QUOTA_EXCEEDED", "registry"));
        }
        write_new(&stage.0.join("record.json"), &bytes)?;
        sync_dir(&stage.0)?;
        observe(Checkpoint::FilesSynced);
        let object = self.path().join("objects").join(&receipt.record_digest);
        if object.exists() {
            return Err(err("INTEGRITY_FAILURE", "registry"));
        }
        io(fs::rename(&stage.0, &object))?;
        sync_dir(&self.path().join("objects"))?;
        sync_dir(&self.path().join(".staging"))?;
        observe(Checkpoint::ObjectRenamed);
        let temp = self.path().join(".staging").join("index.tmp");
        write_new(&temp, &serde_json::to_vec(&receipt).unwrap())?;
        observe(Checkpoint::IndexWritten);
        io(fs::rename(&temp, &index))?;
        observe(Checkpoint::IndexRenamed);
        sync_dir(&self.path().join("releases"))?;
        sync_dir(&self.path().join(".staging"))?;
        observe(Checkpoint::Committed);
        Ok(receipt)
    }
    fn open_source(&self, source: &Path) -> Result<File> {
        if !source.is_absolute()
            || source
                .components()
                .any(|p| matches!(p, Component::ParentDir | Component::CurDir))
        {
            return Err(err("INVALID_PATH", "source"));
        }
        for root in &self.policy.allowed_roots {
            if let Ok(relative) = source.strip_prefix(root) {
                let mut fd = absolute_dir(root)?;
                for part in relative.components() {
                    let Component::Normal(name) = part else {
                        return Err(err("INVALID_PATH", "source"));
                    };
                    fd = open_at(&fd, name, true)?;
                }
                return Ok(fd);
            }
        }
        Err(err("CAPABILITY_DENIED", "source"))
    }
    fn copy_tree(
        &self,
        dir: &File,
        dest: &Path,
        prefix: &str,
        depth: usize,
        state: &mut CopyState,
    ) -> Result<()> {
        if depth > self.policy.max_depth {
            return Err(err("QUOTA_EXCEEDED", "package"));
        }
        let mut folded = BTreeSet::new();
        for entry in io(fs::read_dir(fd_path(dir)))? {
            state.entries += 1;
            if state.entries > self.policy.max_files || Instant::now() > state.deadline {
                return Err(err("QUOTA_EXCEEDED", "package"));
            }
            let entry = io(entry)?;
            let name = entry.file_name();
            let name = name
                .to_str()
                .ok_or_else(|| err("INVALID_PATH", "package"))?;
            if name.is_empty()
                || name == "."
                || name == ".."
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
                || !folded.insert(name.to_ascii_lowercase())
            {
                return Err(err("INVALID_PATH", "package"));
            }
            let relative = if prefix.is_empty() {
                name.into()
            } else {
                format!("{prefix}/{name}")
            };
            if relative.len() > 240 {
                return Err(err("INVALID_PATH", "package"));
            }
            // lstat only chooses open flags; openat + fstat are authoritative.
            let typ = io(entry.file_type())?;
            if !typ.is_file() && !typ.is_dir() {
                return Err(err("INVALID_PATH", &relative));
            }
            let mut file = open_at(dir, std::ffi::OsStr::new(name), typ.is_dir())?;
            let before = io(file.metadata())?;
            if typ.is_dir() {
                if !before.is_dir() {
                    return Err(err("INVALID_PATH", &relative));
                }
                mkdir(&dest.join(name))?;
                self.copy_tree(&file, &dest.join(name), &relative, depth + 1, state)?;
            } else {
                if !before.is_file() || before.nlink() != 1 {
                    return Err(err("INVALID_PATH", &relative));
                }
                if before.len() > self.policy.max_file_bytes {
                    return Err(err("QUOTA_EXCEEDED", &relative));
                }
                let mut bytes = Vec::new();
                io((&mut file)
                    .take(self.policy.max_file_bytes + 1)
                    .read_to_end(&mut bytes))?;
                let after = io(file.metadata())?;
                if before.ino() != after.ino()
                    || before.len() != after.len()
                    || before.mtime() != after.mtime()
                    || before.mtime_nsec() != after.mtime_nsec()
                    || before.ctime() != after.ctime()
                    || before.ctime_nsec() != after.ctime_nsec()
                    || after.nlink() != 1
                {
                    return Err(err("INVALID_PATH", &relative));
                }
                state.total += bytes.len() as u64;
                if bytes.len() as u64 > self.policy.max_file_bytes
                    || state.total > self.policy.max_total_bytes
                    || Instant::now() > state.deadline
                {
                    return Err(err("QUOTA_EXCEEDED", &relative));
                }
                write_new(&dest.join(name), &bytes)?;
                state.files.insert(relative, hash(&bytes));
            }
        }
        sync_dir(dest)
    }
    fn store_usage(&self) -> Result<(u64, usize)> {
        let mut total = 0;
        let mut count = 0;
        for entry in io(fs::read_dir(self.path().join("releases")))? {
            let bytes = regular_read(&io(entry)?.path(), MAX_INDEX)?;
            let receipt: AdmissionReceipt =
                serde_json::from_slice(&bytes).map_err(|_| err("INTEGRITY_FAILURE", "registry"))?;
            let (_, record) = self.load(&receipt.pin)?;
            for path in record.files.keys() {
                total += io(fs::metadata(
                    self.path()
                        .join("objects")
                        .join(&receipt.record_digest)
                        .join("payload")
                        .join(path),
                ))?
                .len();
            }
            total += serde_json::to_vec(&record).unwrap().len() as u64 + bytes.len() as u64;
            count += 1;
        }
        Ok((total, count))
    }
    fn load(&self, pin: &ReleasePin) -> Result<(AdmissionReceipt, Record)> {
        let index = self.path().join("releases").join(slot(pin));
        if !index.exists() {
            return Err(err("COMPONENT_NOT_FOUND", "pin"));
        }
        let integrity = || err("INTEGRITY_FAILURE", "registry");
        let bytes = regular_read(&index, MAX_INDEX).map_err(|_| integrity())?;
        let receipt: AdmissionReceipt = serde_json::from_slice(&bytes).map_err(|_| integrity())?;
        if !hex(&receipt.record_digest) || !hex(&receipt.pin.sha256) {
            return Err(integrity());
        }
        if &receipt.pin != pin {
            return Err(err("COMPONENT_NOT_FOUND", "pin"));
        }
        let object = self.path().join("objects").join(&receipt.record_digest);
        let bytes =
            regular_read(&object.join("record.json"), MAX_RECORD).map_err(|_| integrity())?;
        if hash(&bytes) != receipt.record_digest {
            return Err(integrity());
        }
        let record: Record = serde_json::from_slice(&bytes).map_err(|_| integrity())?;
        if record.format != 1 || record.pin != *pin || record.checked.trust != "unreviewed" {
            return Err(integrity());
        }
        for (path, digest) in &record.files {
            if !safe_relative(path) {
                return Err(integrity());
            }
            let bytes = regular_read(
                &object.join("payload").join(path),
                self.policy.max_file_bytes,
            )
            .map_err(|_| integrity())?;
            if hash(&bytes) != *digest {
                return Err(integrity());
            }
        }
        let module = record.checked.manifest["runtime"]["module"]
            .as_str()
            .ok_or_else(integrity)?;
        if record.files.get(module) != Some(&pin.sha256) {
            return Err(integrity());
        }
        self.check_ui(&record.checked, &object.join("payload"), &record.files)?;
        Ok((receipt, record))
    }
    fn check_ui(
        &self,
        checked: &CheckedMetadata,
        payload: &Path,
        files: &BTreeMap<String, String>,
    ) -> Result<()> {
        match (checked.manifest.get("ui"), &checked.ui) {
            (None, None) => Ok(()),
            (Some(path), Some(ui))
                if path.as_str() == Some(&ui.path)
                    && Path::new(&ui.path).file_name().and_then(|v| v.to_str())
                        == Some("ui.json")
                    && safe_relative(&ui.path)
                    && files.get(&ui.path) == Some(&ui.sha256)
                    && ui.schema_sha256 == wb_semantic::ui::schema_digest() =>
            {
                let bytes = regular_read(
                    &payload.join(&ui.path),
                    wb_semantic::ui::MAX_UI_BYTES as u64,
                )?;
                if hash(&bytes) != ui.sha256 {
                    return Err(err("INTEGRITY_FAILURE", "ui"));
                }
                let validated =
                    wb_semantic::ui::validate(&bytes, checked).map_err(|code| err(code, "ui"))?;
                if validated.document != ui.document {
                    return Err(err("INTEGRITY_FAILURE", "ui"));
                }
                Ok(())
            }
            _ => Err(err("INTEGRITY_FAILURE", "ui")),
        }
    }
    /// New ADR-004 render access. Generic integrity/resolve remains profile-neutral.
    pub fn read_verified_ui(
        &self,
        handle: &ValidatedHandle,
    ) -> Result<wb_semantic::ui::ValidatedUi> {
        if !self.policy.semantic_ui_v01 {
            return Err(err("UI_POLICY_DISABLED", "ui"));
        }
        let (checked, _) = self.read_verified_bundle(handle)?;
        let ui = checked
            .ui
            .as_ref()
            .ok_or_else(|| err("UI_UNAVAILABLE", "ui"))?;
        // Raw bytes are already hashed and compared with this descriptor under
        // the registry lock. Revalidate canonical typed metadata, not source paths.
        wb_semantic::ui::validate(&serde_json::to_vec(&ui.document).unwrap(), &checked)
            .map_err(|code| err(code, "ui"))
    }
    pub fn resolve_exact(&self, pin: &ReleasePin) -> Result<ValidatedHandle> {
        let _lock = self.lock()?;
        let (receipt, _) = self.load(pin)?;
        Ok(ValidatedHandle {
            receipt,
            store: self.identity()?,
        })
    }
    pub fn discover(&self, capability: Option<&str>) -> Result<Vec<ComponentSummary>> {
        let _lock = self.lock()?;
        let mut result = Vec::new();
        for e in io(fs::read_dir(self.path().join("releases")))? {
            let path = io(e)?.path();
            let Ok(bytes) = regular_read(&path, MAX_INDEX) else {
                continue;
            };
            let Ok(receipt) = serde_json::from_slice::<AdmissionReceipt>(&bytes) else {
                continue;
            };
            let Ok((_, record)) = self.load(&receipt.pin) else {
                continue;
            };
            if path.file_name().and_then(|p| p.to_str()) != Some(&slot(&receipt.pin)) {
                continue;
            }
            if capability.is_none_or(|q| record.checked.capabilities.iter().any(|v| v == q)) {
                result.push(ComponentSummary {
                    pin: record.pin,
                    capabilities: record.checked.capabilities,
                    trust: record.checked.trust,
                });
            }
        }
        result.sort_by(|a, b| (&a.pin.id, &a.pin.version).cmp(&(&b.pin.id, &b.pin.version)));
        Ok(result)
    }
    /// Revalidates all recorded bytes, including artifact, on every host boundary.
    pub fn read_verified_bundle(
        &self,
        handle: &ValidatedHandle,
    ) -> Result<(CheckedMetadata, Vec<u8>)> {
        let _lock = self.lock()?;
        if handle.store != self.identity()? {
            return Err(err("INTEGRITY_FAILURE", "handle"));
        }
        let (receipt, record) = self.load(handle.pin())?;
        if receipt != handle.receipt {
            return Err(err("INTEGRITY_FAILURE", "handle"));
        }
        let module = record.checked.manifest["runtime"]["module"]
            .as_str()
            .unwrap();
        let bytes = regular_read(
            &self
                .path()
                .join("objects")
                .join(receipt.record_digest)
                .join("payload")
                .join(module),
            self.policy.max_file_bytes,
        )?;
        if hash(&bytes) != handle.pin().sha256 {
            return Err(err("INTEGRITY_FAILURE", "artifact"));
        }
        Ok((record.checked, bytes))
    }
    pub fn read_verified_artifact(&self, handle: &ValidatedHandle) -> Result<Vec<u8>> {
        self.read_verified_bundle(handle).map(|(_, bytes)| bytes)
    }
}
fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 240
        && path.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        })
}
struct CopyState {
    files: BTreeMap<String, String>,
    total: u64,
    entries: usize,
    deadline: Instant,
}
