use std::env;
use std::ffi::{CString, OsString};
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
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

fn which(exec: &str) -> Option<PathBuf> {
    if let Some(path) = env::var_os("PATH") {
        let paths = env::split_paths(&path);
        for path in paths {
            let candidate = path.join(exec);
            if path.join(exec).exists() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn find_pager(env: &str) -> Option<OsString> {
    if env::var_os("NOPAGER").is_some() {
        return None;
    }
    let default_pager = || which("more").map(|p| PathBuf::from(format!("{} -r", p.display())).into_os_string());
    env::var_os(env).or_else(default_pager)
}

#[cfg(test)]
mod tests {
    use super::{find_pager, which};
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[cfg(target_os = "linux")]
    const MORE: &'static str = "/bin/more";

    #[cfg(target_os = "macos")]
    const MORE: &'static str = "/usr/bin/more";

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
        assert_eq!(which("more"), Some(PathBuf::from(MORE)));
    }

    #[test]
    fn usr_bin_more_default_pager() {
        assert_eq!(find_pager("__RANDOM_NAME"), Some(OsString::from(MORE)));
    }
}
