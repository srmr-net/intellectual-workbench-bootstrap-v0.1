//! Bounded native metadata/inspection workers. No guest privileges or execution.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    os::unix::process::CommandExt,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
pub const CONTRACT_VERSION: &str = "0.1";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedMetadata {
    pub manifest: Value,
    pub schemas: BTreeMap<String, Value>,
    pub capabilities: Vec<String>,
    pub trust: String,
}

/// Trusted executable and arguments only. Child cannot inherit stdin or diagnostics.
/// Linux process quotas are separate from ADR-001 guest execution budgets.
pub fn bounded_worker(
    program: &Path,
    args: &[&Path],
    seconds: u64,
) -> Result<Vec<u8>, &'static str> {
    let mut output: File = tempfile::tempfile().map_err(|_| "REGISTRY_IO")?;
    let mut command = Command::new(program);
    command
        .args(args)
        .env_clear()
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(output.try_clone().map_err(|_| "REGISTRY_IO")?);
    // SAFETY: only async-signal-safe libc calls between fork/exec. No allocation,
    // locks or Rust code with destructors in the child callback.
    let parent_pid = unsafe { libc::getpid() };
    unsafe {
        command.pre_exec(move || {
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0
                || libc::getppid() != parent_pid
            {
                return Err(std::io::Error::from_raw_os_error(libc::ESRCH));
            }
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            for (resource, limit) in [
                (libc::RLIMIT_AS, 2 * 1024 * 1024 * 1024),
                (libc::RLIMIT_CPU, seconds),
                (libc::RLIMIT_FSIZE, 4 * 1024 * 1024),
                (libc::RLIMIT_NOFILE, 64),
                (libc::RLIMIT_CORE, 0),
            ] {
                let limits = libc::rlimit {
                    rlim_cur: limit,
                    rlim_max: limit,
                };
                if libc::setrlimit(resource, &limits) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            Ok(())
        });
    }
    let mut child = command.spawn().map_err(|_| "REGISTRY_IO")?;
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(Some(_)) => return Err("QUOTA_EXCEEDED"),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            _ => {
                // Kill the process group too; no orphan compiler/helper survives timeout.
                unsafe { libc::kill(-(child.id() as i32), libc::SIGKILL) };
                let _ = child.wait();
                return Err("QUOTA_EXCEEDED");
            }
        }
    }
    output.seek(SeekFrom::Start(0)).map_err(|_| "REGISTRY_IO")?;
    let mut bytes = Vec::new();
    output
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "REGISTRY_IO")?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("QUOTA_EXCEEDED");
    }
    Ok(bytes)
}
