use crate::policy::{Failure, Kind, Secret};
use serde::Deserialize;
use std::{
    io::{Read, Write},
    os::{fd::AsRawFd, unix::process::CommandExt},
    path::PathBuf,
    process::{Child, Command, Stdio},
    task::Poll,
    time::{Duration, Instant},
};

pub struct HttpResponse {
    pub status: u16,
    pub retry_after_ms: Option<u64>,
    pub body: Vec<u8>,
}
impl Drop for HttpResponse {
    fn drop(&mut self) {
        self.body.fill(0);
    }
}
pub trait Exchange: Send {
    fn poll(&mut self) -> Poll<Result<HttpResponse, Failure>>;
}
pub trait Transport: Send + Sync {
    fn available(&self) -> bool {
        true
    }
    fn start(
        &self,
        body: &[u8],
        key: Secret,
        max_response: usize,
        timeout_ms: u64,
    ) -> Result<Box<dyn Exchange>, Failure>;
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Metadata {
    http_status: u16,
    failure: Option<Failure>,
    retry_after_ms: Option<u64>,
    body_length: usize,
}
pub struct TlsTransport {
    python: PathBuf,
    provider: Kind,
}
impl TlsTransport {
    pub fn for_provider(python: PathBuf, provider: Kind) -> Result<Self, Failure> {
        if !provider.is_remote() {
            return Err(Failure::NotConfigured);
        }
        let mut value = Self::new(python)?;
        value.provider = provider;
        Ok(value)
    }
    pub fn new(python: PathBuf) -> Result<Self, Failure> {
        if !python.is_absolute() || !python.is_file() {
            return Err(Failure::NotConfigured);
        }
        Ok(Self {
            python,
            provider: Kind::OpenaiResponses,
        })
    }
}
struct ProcessExchange {
    child: Child,
    bytes: Vec<u8>,
    send: Vec<u8>,
    written: usize,
    max: usize,
    done: bool,
}
impl Drop for ProcessExchange {
    fn drop(&mut self) {
        if !self.done {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        self.bytes.fill(0);
        self.send.fill(0);
    }
}
impl Exchange for ProcessExchange {
    fn poll(&mut self) -> Poll<Result<HttpResponse, Failure>> {
        if self.child.stdin.is_some() {
            for _ in 0..4 {
                if self.written == self.send.len() {
                    self.child.stdin.take();
                    self.send.fill(0);
                    self.send.clear();
                    break;
                }
                let end = (self.written + 4096).min(self.send.len());
                match self
                    .child
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write(&self.send[self.written..end])
                {
                    Ok(0) => return Poll::Ready(Err(Failure::Unavailable)),
                    Ok(n) => self.written += n,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => return Poll::Ready(Err(Failure::Unavailable)),
                }
            }
        }
        let mut chunk = [0u8; 4096];
        for _ in 0..4 {
            match self.child.stdout.as_mut().unwrap().read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    if self.bytes.len() + n > self.max + 1028 {
                        return Poll::Ready(Err(Failure::BudgetExceeded));
                    }
                    self.bytes.extend_from_slice(&chunk[..n]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => return Poll::Ready(Err(Failure::Unavailable)),
            }
        }
        let exited = match self.child.try_wait() {
            Ok(v) => v,
            Err(_) => return Poll::Ready(Err(Failure::Unavailable)),
        };
        if exited.is_none() {
            return Poll::Pending;
        }
        self.done = true;
        // Drain data buffered by the OS after child exit, still under the same hard cap.
        loop {
            match self.child.stdout.as_mut().unwrap().read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    if self.bytes.len() + n > self.max + 1028 {
                        return Poll::Ready(Err(Failure::BudgetExceeded));
                    }
                    self.bytes.extend_from_slice(&chunk[..n]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => return Poll::Ready(Err(Failure::Unavailable)),
            }
        }
        if !exited.unwrap().success() || self.bytes.len() < 4 {
            return Poll::Ready(Err(Failure::Unavailable));
        }
        let length = u32::from_be_bytes(self.bytes[..4].try_into().unwrap()) as usize;
        if length > 1024 || self.bytes.len() < 4 + length {
            return Poll::Ready(Err(Failure::MalformedResponse));
        }
        let metadata = match wb_contracts::ui::strict_json(&self.bytes[4..4 + length])
            .ok()
            .and_then(|v| serde_json::from_value::<Metadata>(v).ok())
        {
            Some(m) => m,
            None => return Poll::Ready(Err(Failure::MalformedResponse)),
        };
        if metadata.body_length > self.max || self.bytes.len() != 4 + length + metadata.body_length
        {
            return Poll::Ready(Err(Failure::BudgetExceeded));
        }
        if let Some(failure) = metadata.failure {
            return Poll::Ready(Err(failure));
        }
        Poll::Ready(Ok(HttpResponse {
            status: metadata.http_status,
            retry_after_ms: metadata.retry_after_ms,
            body: self.bytes[4 + length..].to_vec(),
        }))
    }
}
impl Transport for TlsTransport {
    fn available(&self) -> bool {
        cfg!(feature = "live-http")
    }
    fn start(
        &self,
        body: &[u8],
        key: Secret,
        max_response: usize,
        timeout_ms: u64,
    ) -> Result<Box<dyn Exchange>, Failure> {
        if !cfg!(feature = "live-http") {
            return Err(Failure::NotConfigured);
        }
        if body.len() > 65536 || max_response > 32768 {
            return Err(Failure::BudgetExceeded);
        }
        let frame=serde_json::to_vec(&serde_json::json!({"provider":self.provider.label(),"body":std::str::from_utf8(body).map_err(|_|Failure::MalformedResponse)?,"maxResponseBytes":max_response,"timeoutMs":timeout_ms})).map_err(|_|Failure::ProviderError)?;
        // Literal embedded trusted code; no shell, script path, proxy or secret argv.
        if frame.len() > 66000 {
            return Err(Failure::BudgetExceeded);
        }
        let mut command = Command::new(&self.python);
        command
            .args(["-I", "-S", "-u", "-c", include_str!("../transport.py")])
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let parent = unsafe { libc::getpid() };
        unsafe {
            command.pre_exec(move || {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0
                    || libc::getppid() != parent
                {
                    return Err(std::io::Error::from_raw_os_error(libc::ESRCH));
                }
                for (resource, limit) in [
                    (libc::RLIMIT_CORE, 0),
                    (libc::RLIMIT_AS, 256 * 1024 * 1024),
                    (libc::RLIMIT_CPU, 30),
                    (libc::RLIMIT_NOFILE, 64),
                ] {
                    let value = libc::rlimit {
                        rlim_cur: limit,
                        rlim_max: limit,
                    };
                    if libc::setrlimit(resource, &value) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }
        let mut child = command.spawn().map_err(|_| Failure::Unavailable)?;
        for fd in [
            child.stdout.as_ref().unwrap().as_raw_fd(),
            child.stdin.as_ref().unwrap().as_raw_fd(),
        ] {
            if unsafe { libc::fcntl(fd, libc::F_SETFL, libc::O_NONBLOCK) } < 0 {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Failure::Unavailable);
            }
        }
        let mut send = Vec::with_capacity(key.bytes().len() + frame.len() + 5);
        send.extend_from_slice(key.bytes());
        send.push(b'\n');
        send.extend_from_slice(&(frame.len() as u32).to_be_bytes());
        send.extend_from_slice(&frame);
        Ok(Box::new(ProcessExchange {
            child,
            bytes: Vec::new(),
            send,
            written: 0,
            max: max_response,
            done: false,
        }))
    }
}

/// Deterministic transport, available for direct native tests and feature-gated UI fixtures.
/// It never creates sockets. Synthetic test credentials are not API credentials.
#[derive(Clone)]
pub struct MockReply {
    pub status: u16,
    pub body: Vec<u8>,
    pub delay: Duration,
    pub retry_after_ms: Option<u64>,
    pub failure: Option<Failure>,
}
pub struct MockTransport {
    pub replies: std::sync::Mutex<std::collections::VecDeque<MockReply>>,
    pub sent: std::sync::Mutex<Vec<Vec<u8>>>,
    pub attempts: std::sync::atomic::AtomicU64,
    pub aborted: std::sync::Arc<std::sync::atomic::AtomicU64>,
}
impl MockTransport {
    pub fn new(replies: Vec<MockReply>) -> Self {
        Self {
            replies: std::sync::Mutex::new(replies.into()),
            sent: std::sync::Mutex::new(vec![]),
            attempts: 0.into(),
            aborted: Default::default(),
        }
    }
}
struct MockExchange {
    reply: Option<MockReply>,
    ready: Instant,
    done: bool,
    aborted: std::sync::Arc<std::sync::atomic::AtomicU64>,
}
impl Drop for MockExchange {
    fn drop(&mut self) {
        if !self.done {
            self.aborted
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
}
impl Exchange for MockExchange {
    fn poll(&mut self) -> Poll<Result<HttpResponse, Failure>> {
        if Instant::now() < self.ready {
            return Poll::Pending;
        }
        self.done = true;
        let r = self.reply.take().unwrap();
        Poll::Ready(match r.failure {
            Some(e) => Err(e),
            None => Ok(HttpResponse {
                status: r.status,
                retry_after_ms: r.retry_after_ms,
                body: r.body,
            }),
        })
    }
}
impl Transport for MockTransport {
    fn start(
        &self,
        body: &[u8],
        _: Secret,
        _: usize,
        _: u64,
    ) -> Result<Box<dyn Exchange>, Failure> {
        self.attempts
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.sent.lock().unwrap().push(body.to_vec());
        let r = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .ok_or(Failure::Unavailable)?;
        let ready = Instant::now() + r.delay;
        Ok(Box::new(MockExchange {
            reply: Some(r),
            ready,
            done: false,
            aborted: self.aborted.clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dropping_pending_process_exchange_kills_and_reaps_child() {
        let child = std::process::Command::new("/usr/bin/python3")
            .args(["-I", "-S", "-c", "import time; time.sleep(30)"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let pid = child.id() as libc::pid_t;
        let exchange = ProcessExchange {
            child,
            bytes: vec![],
            send: vec![],
            written: 0,
            max: 100,
            done: false,
        };
        drop(exchange);
        assert_eq!(
            unsafe { libc::waitpid(pid, std::ptr::null_mut(), libc::WNOHANG) },
            -1
        );
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ECHILD)
        );
    }
}
