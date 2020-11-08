use std::ffi::{CString, OsString};
use std::os::unix::ffi::OsStringExt;
use std::ptr;

fn osstring2cstring(s: OsString) -> CString {
    unsafe { CString::from_vec_unchecked(s.into_vec()) }
}

fn split_string(s: &OsString) -> Vec<OsString> {
    match s.clone().into_string() {
        Ok(cmd) => cmd.split_whitespace().map(OsString::from).collect(),
        Err(cmd) => vec![cmd],
    }
}

// Helper wrappers around libc::* API
pub(crate) fn fork() -> libc::pid_t {
    unsafe { libc::fork() }
}

pub(crate) fn execvp(cmd: &OsString) {
    let cstrings = split_string(cmd)
        .into_iter()
        .map(osstring2cstring)
        .collect::<Vec<_>>();
    let mut args = cstrings.iter().map(|c| c.as_ptr()).collect::<Vec<_>>();
    args.push(ptr::null());
    errno::set_errno(errno::Errno(0));
    unsafe { libc::execvp(args[0], args.as_ptr()) };
}

pub(crate) fn execvpe(cmd: &OsString, envs: &[OsString]) {
    let cstrings = split_string(cmd)
        .into_iter()
        .map(osstring2cstring)
        .collect::<Vec<_>>();
    let mut args = cstrings.iter().map(|c| c.as_ptr()).collect::<Vec<_>>();
    args.push(ptr::null());

    let mut cstrings_envs = envs
        .into_iter()
        .map(|s| osstring2cstring(s.clone()))
        .collect::<Vec<_>>();
    for (mut k, v) in std::env::vars_os() {
        k.push("=");
        k.push(v);
        cstrings_envs.push(osstring2cstring(k));
    }
    let mut envs = cstrings_envs.iter().map(|c| c.as_ptr()).collect::<Vec<_>>();
    envs.push(ptr::null());

    errno::set_errno(errno::Errno(0));
    unsafe { libc::execvpe(args[0], args.as_ptr(), envs.as_ptr()) };
}

pub(crate) fn dup2(fd1: i32, fd2: i32) {
    assert!(unsafe { libc::dup2(fd1, fd2) } > -1);
}

pub(crate) fn close(fd: i32) {
    assert_eq!(unsafe { libc::close(fd) }, 0);
}

pub(crate) fn pipe() -> (i32, i32) {
    let mut fds = [0; 2];
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
    (fds[0], fds[1])
}

pub(crate) fn isatty(fd: i32) -> bool {
    unsafe { libc::isatty(fd) != 0 }
}
