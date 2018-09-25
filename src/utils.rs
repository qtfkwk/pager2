use std::env;
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

fn which(exec: &str) -> Option<OsString> {
    if let Some(path) = env::var_os("PATH") {
        for path in env::split_paths(&path) {
            let candidate = path.join(exec);
            if path.join(exec).exists() {
                return Some(candidate.into_os_string());
            }
        }
    }
    None
}

pub fn isatty(fd: i32) -> bool {
    let isatty = unsafe { libc::isatty(fd) };
    isatty != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const MORE: &str = "/bin/more";

    fn assert_ends_with_bin_more(more: OsString) {
        let good = more.to_str().map(|m| m.ends_with(MORE)).unwrap_or(false);
        assert!(good, "{:?} doesn't end with {}", more, MORE);
    }

    #[test]
    fn more_found_in_path() {
        assert!(which("more").is_some())
    }

    #[test]
    fn erom_not_found_in_path() {
        assert!(which("erom").is_none())
    }

    #[test]
    fn which_more() {
        which("more").map(assert_ends_with_bin_more);
    }

    #[test]
    fn usr_bin_more_default_pager() {
        find_pager("__RANDOM_NAME").map(assert_ends_with_bin_more);
    }

    #[test]
    fn nopager() {
        env::set_var("NOPAGER", "");
        assert!(find_pager("more").is_none());
        env::remove_var("NOPAGER");
    }
}
