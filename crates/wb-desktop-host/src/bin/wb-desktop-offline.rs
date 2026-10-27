//! Native conformance launcher: deny IPv4/IPv6 socket creation before real app exec.
//! Never linked into or imported by a guest. No model/network fallback permitted.
use std::{os::unix::process::CommandExt, process::Command};
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() {
        std::process::exit(2);
    }
    // seccomp_data nr at0, args[0] at16. Deny only socket(AF_INET/AF_INET6),
    // preserving local Unix Qt/DBus IPC. Kernel filter is inherited across exec.
    let mut filter = [
        libc::sock_filter {
            code: 0x20,
            jt: 0,
            jf: 0,
            k: 0,
        },
        libc::sock_filter {
            code: 0x15,
            jt: 0,
            jf: 4,
            k: libc::SYS_socket as u32,
        },
        libc::sock_filter {
            code: 0x20,
            jt: 0,
            jf: 0,
            k: 16,
        },
        libc::sock_filter {
            code: 0x15,
            jt: 1,
            jf: 0,
            k: libc::AF_INET as u32,
        },
        libc::sock_filter {
            code: 0x15,
            jt: 0,
            jf: 1,
            k: libc::AF_INET6 as u32,
        },
        libc::sock_filter {
            code: 0x06,
            jt: 0,
            jf: 0,
            k: 0x00050000 | libc::EPERM as u32,
        },
        libc::sock_filter {
            code: 0x06,
            jt: 0,
            jf: 0,
            k: 0x7fff0000,
        },
    ];
    let program = libc::sock_fprog {
        len: filter.len() as u16,
        filter: filter.as_mut_ptr(),
    };
    if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0
        || unsafe { libc::prctl(libc::PR_SET_SECCOMP, 2, &program) } != 0
    {
        eprintln!("OFFLINE_FILTER_FAILED");
        std::process::exit(1);
    }
    for family in [libc::AF_INET, libc::AF_INET6] {
        let fd = unsafe { libc::socket(family, libc::SOCK_STREAM, 0) };
        if fd >= 0 || std::io::Error::last_os_error().raw_os_error() != Some(libc::EPERM) {
            eprintln!("OFFLINE_PROBE_FAILED");
            std::process::exit(1);
        }
    }
    println!("OFFLINE_IPV4_IPV6_DENIED");
    let error = Command::new(&args[0]).args(&args[1..]).exec();
    eprintln!("OFFLINE_EXEC_FAILED: {}", error.kind());
    std::process::exit(1);
}
