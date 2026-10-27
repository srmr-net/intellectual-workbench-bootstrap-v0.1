//! Internal VSLICE002 snapshot v1. Data only; no runtime grants or guest memory.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
    path::{Component, Path, PathBuf},
};
use wb_contracts::{ipc::RoomSnapshot, ui::strict_json, ComponentRef};
pub const MAX_BYTES: usize = 1024 * 1024;
pub const MAX_ROOMS: usize = 16;
pub type Result<T> = std::result::Result<T, &'static str>;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UiPin {
    pub path: String,
    pub sha256: String,
    pub schema_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Checkpoint {
    pub room_id: String,
    pub label: String,
    pub pinned: bool,
    pub component: ComponentRef,
    pub ui: UiPin,
    pub room: RoomSnapshot,
    pub outputs: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Workspace {
    pub format_version: u32,
    pub content_sha256: String,
    pub revision: u64,
    pub selected_room: Option<String>,
    pub rooms: Vec<Checkpoint>,
}
impl Default for Workspace {
    fn default() -> Self {
        Self {
            format_version: 1,
            content_sha256: String::new(),
            revision: 0,
            selected_room: None,
            rooms: vec![],
        }
    }
}
pub fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
impl Workspace {
    pub fn validate(&self) -> Result<()> {
        if self.format_version != 1 {
            return Err("UNSUPPORTED_SNAPSHOT_VERSION");
        }
        static VALIDATOR: std::sync::OnceLock<jsonschema::Validator> = std::sync::OnceLock::new();
        let validator = VALIDATOR.get_or_init(|| {
            let schema: Value = serde_json::from_slice(include_bytes!(
                "../../../fixtures/vslice-002/snapshot-v1.schema.json"
            ))
            .unwrap();
            let composition: Value =
                serde_json::from_slice(include_bytes!("../../../schemas/composition.schema.json"))
                    .unwrap();
            let room: Value =
                serde_json::from_slice(include_bytes!("../../../schemas/room.schema.json"))
                    .unwrap();
            jsonschema::options()
                .with_resource(
                    composition["$id"].as_str().unwrap(),
                    jsonschema::Resource::from_contents(composition.clone()).unwrap(),
                )
                .with_resource(
                    room["$id"].as_str().unwrap(),
                    jsonschema::Resource::from_contents(room.clone()).unwrap(),
                )
                .build(&schema)
                .unwrap()
        });
        if !validator.is_valid(&serde_json::to_value(self).map_err(|_| "INVALID_SNAPSHOT")?) {
            return Err("INVALID_SNAPSHOT");
        }

        if self.rooms.len() > MAX_ROOMS {
            return Err("SNAPSHOT_QUOTA");
        }
        let mut ids = BTreeSet::new();
        for r in &self.rooms {
            if !identifier(&r.room_id)
                || !ids.insert(&r.room_id)
                || r.label.trim().is_empty()
                || r.label.chars().count() > 160
                || r.label.chars().any(char::is_control)
            {
                return Err("INVALID_SNAPSHOT");
            }
            if !digest(&r.component.sha256)
                || !digest(&r.ui.sha256)
                || !digest(&r.ui.schema_sha256)
                || r.ui.path.len() > 240
                || !r.ui.path.ends_with("ui.json")
                || r.ui
                    .path
                    .split('/')
                    .any(|p| p.is_empty() || p == "." || p == "..")
                || r.ui.path.contains(['\\', ':', '\0'])
            {
                return Err("INVALID_SNAPSHOT");
            }
            let s = &r.room;
            if s.revision == 0
                || s.name != r.label
                || s.room_format_version != "0.1"
                || s.room_id != r.room_id
                || s.components.len() != 1
                || !s.connections.is_empty()
                || s.state.len() > 1
                || r.outputs.len() > 32
            {
                return Err("INVALID_SNAPSHOT");
            }
            let c = &s.components[0];
            if c.instance_id != "component"
                || c.id != r.component.id
                || c.version != r.component.version
                || c.sha256 != r.component.sha256
                || s.ui.definition_ref != format!("{}/{}", r.component.id, r.ui.path)
            {
                return Err("INVALID_SNAPSHOT");
            }
            if r.component.id.is_empty()
                || r.component.id.len() > 200
                || r.component.version.len() > 64
                || r.component.version.is_empty()
            {
                return Err("INVALID_SNAPSHOT");
            }
        }
        if self
            .selected_room
            .as_ref()
            .is_some_and(|id| !ids.contains(id))
        {
            return Err("INVALID_SNAPSHOT");
        }
        Ok(())
    }
    fn checksum(&self) -> Result<String> {
        let mut value = serde_json::to_value(self).map_err(|_| "INVALID_SNAPSHOT")?;
        value.as_object_mut().unwrap().remove("contentSha256");
        Ok(format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&value).map_err(|_| "INVALID_SNAPSHOT")?)
        ))
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err("SNAPSHOT_QUOTA");
        }
        let value = strict_json(bytes).map_err(|_| "INVALID_SNAPSHOT")?;
        let result: Self = serde_json::from_value(value).map_err(|_| "INVALID_SNAPSHOT")?;
        result.validate()?;
        if result.content_sha256 != result.checksum()? {
            return Err("SNAPSHOT_CHECKSUM");
        }
        Ok(result)
    }
}
fn cstring(s: &std::ffi::OsStr) -> Result<CString> {
    CString::new(s.as_bytes()).map_err(|_| "SNAPSHOT_PATH")
}
fn at(dir: &File, name: &std::ffi::OsStr, flags: i32, mode: u32) -> Result<File> {
    let c = cstring(name)?;
    // SAFETY: owned descriptor from openat, fixed flags, NUL-terminated leaf.
    let fd = unsafe {
        libc::openat(
            dir.as_raw_fd(),
            c.as_ptr(),
            flags | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
            mode,
        )
    };
    if fd < 0 {
        return Err("SNAPSHOT_IO");
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}
fn private(file: &File, directory: bool) -> Result<()> {
    let m = file.metadata().map_err(|_| "SNAPSHOT_IO")?;
    if m.uid() != unsafe { libc::geteuid() }
        || m.mode() & 0o077 != 0
        || if directory {
            !m.is_dir()
        } else {
            !m.is_file() || m.nlink() != 1
        }
    {
        return Err("SNAPSHOT_PATH");
    }
    Ok(())
}
pub struct FileStore {
    dir: File,
    _lock: File,
}
impl FileStore {
    /// Absolute caller-selected native location; reject every symlink ancestor.
    pub fn open(path: &Path) -> Result<Self> {
        if !path.is_absolute() {
            return Err("SNAPSHOT_PATH");
        }
        let mut dir = File::open("/").map_err(|_| "SNAPSHOT_IO")?;
        let parts: Vec<_> = path.components().collect();
        for (index, part) in parts.iter().enumerate() {
            match part {
                Component::RootDir => {}
                Component::Normal(name) => {
                    if index == parts.len() - 1 {
                        let name_c = cstring(name)?;
                        let status =
                            unsafe { libc::mkdirat(dir.as_raw_fd(), name_c.as_ptr(), 0o700) };
                        if status != 0
                            && std::io::Error::last_os_error().kind()
                                != std::io::ErrorKind::AlreadyExists
                        {
                            return Err("SNAPSHOT_IO");
                        }
                    }
                    dir = at(&dir, name, libc::O_RDONLY | libc::O_DIRECTORY, 0)?;
                }
                _ => return Err("SNAPSHOT_PATH"),
            }
        }
        private(&dir, true)?;
        let lock = at(
            &dir,
            ".workspace.lock".as_ref(),
            libc::O_RDWR | libc::O_CREAT,
            0o600,
        )?;
        private(&lock, false)?;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err("WORKSPACE_BUSY");
        }
        Ok(Self { dir, _lock: lock })
    }
    fn anchored(&self, name: &str) -> PathBuf {
        PathBuf::from(format!("/proc/self/fd/{}", self.dir.as_raw_fd())).join(name)
    }
    pub fn load(&self) -> Result<Option<Workspace>> {
        let name = "workspace.json";
        // lstat avoids treating symlinks (including dangling ones) as absent.
        match std::fs::symlink_metadata(self.anchored(name)) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("SNAPSHOT_IO"),
            Ok(_) => {}
        }
        let file = at(&self.dir, name.as_ref(), libc::O_RDONLY, 0)?;
        private(&file, false)?;
        if file.metadata().map_err(|_| "SNAPSHOT_IO")?.len() > MAX_BYTES as u64 {
            return Err("SNAPSHOT_QUOTA");
        }
        let mut bytes = vec![];
        file.take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "SNAPSHOT_IO")?;
        Workspace::decode(&bytes).map(Some)
    }
    /// Single-writer lock plus on-disk CAS. Never overwrite corrupt/unreadable data.
    pub fn commit(&self, expected: u64, candidate: &Workspace) -> Result<Workspace> {
        let current = self.load()?.unwrap_or_default();
        if current.revision != expected {
            return Err("STATE_CONFLICT");
        }
        let mut next = candidate.clone();
        next.revision = expected.checked_add(1).ok_or("SNAPSHOT_QUOTA")?;
        next.content_sha256 = next.checksum()?;
        next.validate()?;
        let bytes = serde_json::to_vec(&next).map_err(|_| "INVALID_SNAPSHOT")?;
        if bytes.len() > MAX_BYTES {
            return Err("SNAPSHOT_QUOTA");
        }
        // A crashed writer may leave a temp file. Never overwrite or reuse it.
        let mut nonce = [0u8; 16];
        if unsafe { libc::getrandom(nonce.as_mut_ptr().cast(), nonce.len(), 0) }
            != nonce.len() as isize
        {
            return Err("SNAPSHOT_IO");
        }
        let temp = format!(
            ".workspace-{}-{:032x}.tmp",
            std::process::id(),
            u128::from_ne_bytes(nonce)
        );
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(self.anchored(&temp))
            .map_err(|_| "SNAPSHOT_IO")?;
        let result = (|| {
            file.write_all(&bytes).map_err(|_| "SNAPSHOT_IO")?;
            file.sync_all().map_err(|_| "SNAPSHOT_IO")?;
            std::fs::rename(self.anchored(&temp), self.anchored("workspace.json"))
                .map_err(|_| "SNAPSHOT_IO")?;
            self.dir.sync_all().map_err(|_| "SNAPSHOT_IO")
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(self.anchored(&temp));
        }
        result?;
        Ok(next)
    }
}
