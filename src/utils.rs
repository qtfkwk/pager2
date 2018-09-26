use std::ffi::{CString, OsString};
use std::os::unix::ffi::OsStringExt;
use std::ptr;

use errno;
use libc;

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
pub fn fork() -> libc::pid_t {
    unsafe { libc::fork() }
}

pub fn execvp(cmd: &OsString) {
    let cstrings = split_string(cmd)
        .into_iter()
        .map(osstring2cstring)
        .collect::<Vec<_>>();
    let mut args = cstrings.iter().map(|c| c.as_ptr()).collect::<Vec<_>>();
    args.push(ptr::null());
    errno::set_errno(errno::Errno(0));
    unsafe { libc::execvp(args[0], args.as_ptr()) };
}

pub fn dup2(fd1: i32, fd2: i32) {
    assert!(unsafe { libc::dup2(fd1, fd2) } > -1);
}

pub fn close(fd: i32) {
    assert_eq!(unsafe { libc::close(fd) }, 0);
}

pub fn pipe() -> (i32, i32) {
    let mut fds = [0; 2];
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
    (fds[0], fds[1])
}

pub fn isatty(fd: i32) -> bool {
    let isatty = unsafe { libc::isatty(fd) };
    isatty != 0
}
