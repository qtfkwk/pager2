use std::env;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use libc;

// In C this would be simple getenv(). Not in Rust though
pub fn getenv(var: &str) -> Option<CString> {
    if let Some(value) = env::var_os(var) {
        let value = value.as_os_str().as_bytes();
        CString::new(value).ok()
    } else {
        None
    }
    // let to_bytes = |x: &OsString| x.as_os_str().as_bytes();
    // let to_bytes = |x: &OsString| x.into::<Vec<u8>>();
    // env::var_os(&self.env).map(to_bytes).and_then(|x| CString::new(x).ok())
}


// Helper wrappers around libc::* API
pub fn fork() -> libc::pid_t {
    unsafe { libc::fork() }
}

pub fn execvp(argv: Vec<*const libc::c_char>) {
    assert!(unsafe { libc::execvp(argv[0], argv.as_ptr()) } > -1);
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

pub fn default_pager() -> Option<CString> {
    // let pathbuf2cstring = |p: PathBuf| CString::new(p.as_os_str().as_bytes()).ok();
    let str2cstring = |x: &str| CString::new(x).ok();
    let pathbuf2cstring = |p: PathBuf| p.to_str().and_then(str2cstring);
    which("more").and_then(pathbuf2cstring)
}

#[cfg(test)]
mod tests {
    use super::{default_pager, which};
    use std::ffi::CString;
    use std::path::PathBuf;

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
        assert_eq!(which("more"), Some(PathBuf::from("/usr/bin/more")));
    }

    #[test]
    fn usr_bin_more_default_pager() {
        let usr_bin_more = CString::new("/usr/bin/more").unwrap();
        assert_eq!(default_pager(), Some(usr_bin_more));
    }
}
