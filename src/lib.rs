//! Does all the magic to have you potentially long output piped through the
//! external pager. Similar to what git does for its output.
//!
//! Quick Start
//!
//! ```
//! extern crate pager;
//! use pager::Pager;
//! fn main() {
//!     Pager::new().setup();
//!     // The rest of your program goes here
//! }
//! ```
//!
//! Under the hood this forks the current process, connects child' stdout
//! to parent's stdin, and then replaces the parent with the pager of choice
//! (environment variable PAGER). The child just continues as normal.
//!
//! You can control pager to a limited degree. For example you can change the
//! environment variable used for finding pager executable.
//!
//! ```
//! extern crate pager;
//! use pager::Pager;
//! fn main() {
//!     Pager::env("MY_PAGER").setup();
//!     // The rest of your program goes here
//! }
//! ```
//!
//! If no PAGER is found `setup()` does nothing and your executable keeps
//! running as usual. `Pager` cleans after itself and doesn't leak resources in
//! case of setup failure.
//!

extern crate libc;

use std::env;
use std::ffi::CString;
use std::ptr;
use std::os::unix::ffi::OsStrExt;

const DEFAULT_PAGER_ENV: &'static str = "PAGER";

#[derive(Debug, Default)]
pub struct Pager {
    env: String,
    ok: bool,
}

impl Pager {
    pub fn new() -> Self {
        Pager {
            env: String::from(DEFAULT_PAGER_ENV),
            ok: true,
        }
    }

    pub fn env(env: &str) -> Self {
        Pager {
            env: String::from(env),
            ok: true,
        }
    }

    pub fn ok(&self) -> bool {
        self.ok
    }

    pub fn setup(&mut self) {
        if let Some(pager) = getenv(&self.env) {
            let (pager_stdin, main_stdout) = pipe();
            let pid = fork();
            match pid {
                -1 => {
                    // Fork failed
                    close(pager_stdin);
                    close(main_stdout);
                    self.ok = false
                }
                0 => {
                    // I am child
                    dup2(main_stdout, libc::STDOUT_FILENO);
                    close(pager_stdin);
                }
                _ => {
                    // I am parent
                    let argv = vec![pager.as_ptr(), ptr::null()];
                    dup2(pager_stdin, libc::STDIN_FILENO);
                    close(main_stdout);
                    execvp(argv);
                }
            }
        }
    }
}

// In C this would be simple getenv(). Not in Rust though
fn getenv(var: &str) -> Option<CString> {
    if let Some(value) = env::var_os(var) {
        let value = value.as_os_str().as_bytes();
        let value = CString::new(value);
        value.ok()
    } else {
        None
    }
    // let to_bytes = |x: &OsString| x.as_os_str().as_bytes();
    // let to_bytes = |x: &OsString| x.into::<Vec<u8>>();
    // env::var_os(&self.env).map(to_bytes).and_then(|x| CString::new(x).ok())
}


// Helper wrappers around libc::* API
fn fork() -> libc::pid_t {
    unsafe { libc::fork() }
}

fn execvp(argv: Vec<*const libc::c_char>) {
    assert!(unsafe { libc::execvp(argv[0], argv.as_ptr()) } > -1);
}

fn dup2(fd1: i32, fd2: i32) {
    assert!(unsafe { libc::dup2(fd1, fd2) } > -1);
}

fn close(fd: i32) {
    assert_eq!(unsafe { libc::close(fd) }, 0);
}

fn pipe() -> (i32, i32) {
    let mut fds = [0; 2];
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
    (fds[0], fds[1])
}
