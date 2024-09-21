//! Does all the magic to have you potentially long output piped through the
//! external pager. Similar to what git does for its output.
//!
//! # Quick Start
//!
//! ```rust
//! extern crate pager;
//! use pager::Pager;
//! fn main() {
//!     let _pager = Pager::new().setup();
//!     // The rest of your program goes here
//! }
//! ```
//!
//! Under the hood this forks the current process, connects child' stdout
//! to parent's stdin, and then replaces the parent with the pager of choice
//! (environment variable PAGER). The child just continues as normal. If PAGER
//! environment variable is not present `Pager` probes current PATH for `more`.
//! If found it is used as a default pager.
//!
//! You can control pager to a limited degree. For example you can change the
//! environment variable used for finding pager executable.
//!
//! ```rust
//! extern crate pager;
//! use pager::Pager;
//! fn main() {
//!     let _pager = Pager::with_env("MY_PAGER").setup();
//!     // The rest of your program goes here
//! }
//! ```
//!
//! Also you can set alternative default (fallback) pager to be used instead of
//! `more`. PAGER environment variable (if set) will still have precedence.
//!
//! ```rust
//! extern crate pager;
//! use pager::Pager;
//! fn main() {
//!     let _pager = Pager::with_default_pager("pager").setup();
//!     // The rest of your program goes here
//! }
//! ```
//! Alternatively you can specify directly the desired pager command, exactly
//! as it would appear in PAGER environment variable. This is useful if you
//! need some specific pager and/or flags (like "less -r") and would like to
//! avoid forcing your consumers into modifying their existing PAGER
//! configuration just for your application.
//!
//! ```rust
//! extern crate pager;
//! use pager::Pager;
//! fn main() {
//!     let _pager = Pager::with_pager("pager -r").setup();
//!     // The rest of your program goes here
//! }
//! ```
//!
//! If no suitable pager found `setup()` does nothing and your executable keeps
//! running as usual. `Pager` cleans after itself and doesn't leak resources in
//! case of setup failure.
//!
//! If you need to disable pager altogether set environment variable `NOPAGER` and `Pager::setup()`
//! will skip initialization. The host application will continue as normal. `Pager::is_on()` will
//! reflect the fact that no Pager is active.

#![doc(html_root_url = "https://docs.rs/pager/0.16.1")]
#![cfg_attr(feature = "pedantic", warn(clippy::pedantic))]
#![warn(clippy::use_self)]
#![warn(deprecated_in_future)]
#![warn(future_incompatible)]
#![warn(unreachable_pub)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
#![deny(warnings)]

use std::{
    env,
    ffi::{OsStr, OsString},
    fs::File,
    io::{stdout, Write},
    os::fd::{AsRawFd, RawFd},
    process::{Child, Command, Stdio},
};

/// Default pager environment variable
const DEFAULT_PAGER_ENV: &str = "PAGER";

/// Environment variable to disable pager altogether
const NOPAGER_ENV: &str = "NOPAGER";

/// Last resort pager. Should work everywhere.
const DEFAULT_PAGER: &str = "more";

/// Keeps track of the current pager state
#[derive(Debug)]
pub struct Pager {
    default_pager: Option<OsString>,
    pager: Option<OsString>,
    envs: Vec<(OsString, OsString)>,
}

#[derive(Debug)]
pub struct PagerProcess {
    pager: Option<Child>,
    old_fd: RawFd,
}

impl Default for Pager {
    fn default() -> Self {
        Self {
            default_pager: None,
            pager: env::var_os(DEFAULT_PAGER_ENV),
            envs: Vec::new(),
        }
    }
}

impl Pager {
    /// Creates new instance of `Pager` with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates new instance of pager using `env` environment variable instead of PAGER
    pub fn with_env(env: &str) -> Self {
        Self {
            pager: env::var_os(env),
            ..Self::default()
        }
    }

    #[deprecated(since = "0.12.0", note = "use with_env() instead")]
    pub fn env(env: &str) -> Self {
        Self::with_env(env)
    }

    /// Creates a new `Pager` instance with the specified default fallback
    pub fn with_default_pager<S>(pager: S) -> Self
    where
        S: Into<OsString>,
    {
        let default_pager = Some(pager.into());
        Self {
            default_pager,
            ..Self::default()
        }
    }

    /// Creates a new `Pager` instance directly specifying the desired pager
    pub fn with_pager(pager: &str) -> Self {
        Self {
            pager: Some(pager.into()),
            ..Self::default()
        }
    }

    /// Launch pager with the specified environment variables
    pub fn pager_envs(self, envs: impl IntoIterator<Item = impl Into<OsString>>) -> Self {
        let envs: Vec<(OsString, OsString)> = envs
            .into_iter()
            .map(|s| {
                let s: OsString = s.into();
                let pair = s
                    .to_str()
                    .expect("Environment variable is not UTF-8 compliant");
                let mut split = pair.split('=');
                let (Some(key), Some(value), None) = (split.next(), split.next(), split.next())
                else {
                    panic!(
                        "Invalid key-value pair for an environment variable: {}",
                        pair
                    );
                };
                (key.into(), value.into())
            })
            .collect();
        Self { envs, ..self }
    }

    fn pager(&self) -> Option<OsString> {
        let fallback_pager = || Some(OsStr::new(DEFAULT_PAGER).into());

        if env::var_os(NOPAGER_ENV).is_some() {
            None
        } else {
            self.pager
                .clone()
                .or_else(|| self.default_pager.clone())
                .or_else(fallback_pager)
        }
    }

    /// Initiates Pager framework and sets up all the necessary environment for sending standard
    /// output to the activated pager.
    pub fn setup(self) -> PagerProcess {
        let isatty = unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 };
        let pager = self.pager();
        if isatty && pager.is_some() {
            let pager = pager.unwrap();
            let pager = pager.to_str().expect("pager path is not UTF-8 compliant");

            let args = shell_words::split(pager).unwrap_or_else(|err| {
                panic!("Can't parse pager arguments: {}", err);
            });
            let (pager_cmd, args) = match args.len() {
                0 => unreachable!(),
                1 => (&args[0], &[] as &[String]),
                _ => (&args[0], &args[1..]),
            };

            let pager = Command::new(pager_cmd)
                .args(args)
                .envs(self.envs)
                .stdin(Stdio::piped())
                .spawn()
                .unwrap();

            let fd = pager.stdin.as_ref().unwrap().as_raw_fd();
            let old_fd;
            unsafe {
                old_fd = libc::dup(libc::STDOUT_FILENO);
                assert!(old_fd >= 0);
                assert_eq!(libc::dup2(fd, libc::STDOUT_FILENO), libc::STDOUT_FILENO);
            }

            PagerProcess::new(Some(pager), old_fd)
        } else {
            PagerProcess::new(None, -1)
        }
    }
}

impl PagerProcess {
    fn new(pager: Option<Child>, old_fd: RawFd) -> Self {
        Self { pager, old_fd }
    }

    /// Gives quick assessment of successful `Pager` setup
    pub fn is_on(&self) -> bool {
        self.pager.is_some()
    }
}

impl Drop for PagerProcess {
    fn drop(&mut self) {
        if let Some(pager) = &mut self.pager {
            unsafe {
                // Unfortunately, we cannot drain the internal buffer of stdout(),
                // so we open /dev/null and flush into there.
                // If we don't do this, the remaining buffer will be printed to
                // stdout() after closing the pager.
                // We'll ignore if we can't open /dev/null.
                if let Ok(null) = File::open("/dev/null") {
                    assert_eq!(
                        libc::dup2(null.as_raw_fd(), libc::STDOUT_FILENO),
                        libc::STDOUT_FILENO
                    );
                    stdout().flush().unwrap();
                }
                assert_eq!(
                    libc::dup2(self.old_fd, libc::STDOUT_FILENO),
                    libc::STDOUT_FILENO
                );
            }
            pager.wait().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Drop;

    enum PagerEnv {
        Reinstate(OsString, OsString),
        Remove(OsString),
    }

    impl PagerEnv {
        fn new<S: AsRef<OsStr>>(env: S) -> Self {
            let env = env.as_ref().into();
            if let Some(value) = env::var_os(&env) {
                Self::Reinstate(env, value)
            } else {
                Self::Remove(env)
            }
        }

        fn set<S: AsRef<OsStr>>(&self, value: S) {
            match self {
                Self::Reinstate(env, _) | Self::Remove(env) => env::set_var(env, value),
            }
        }

        fn remove(&self) {
            match self {
                Self::Reinstate(env, _) | Self::Remove(env) => env::remove_var(env),
            }
        }
    }

    impl Drop for PagerEnv {
        fn drop(&mut self) {
            match self {
                Self::Reinstate(env, value) => env::set_var(env, value),
                Self::Remove(env) => env::remove_var(env),
            }
        }
    }

    fn assert_pager(pager: &Pager, result: &str) {
        assert_eq!(pager.pager(), Some(OsStr::new(result).into()));
    }

    #[test]
    fn nopager() {
        let nopager = PagerEnv::new(NOPAGER_ENV);
        nopager.set("");

        let pager = Pager::new();
        assert!(pager.pager().is_none());
    }

    #[test]
    fn fallback_uses_more() {
        let pager = Pager::new();
        assert_pager(&pager, DEFAULT_PAGER);
    }

    #[test]
    fn with_default_pager_without_env() {
        let pagerenv = PagerEnv::new(DEFAULT_PAGER_ENV);
        pagerenv.remove();

        let pager = Pager::with_default_pager("more_or_less");
        assert_pager(&pager, "more_or_less");
    }

    #[test]
    fn with_default_pager_with_env() {
        let pagerenv = PagerEnv::new(DEFAULT_PAGER_ENV);
        pagerenv.set("something_else");

        let pager = Pager::with_default_pager("more_or_less");
        assert_pager(&pager, "something_else");
    }

    #[test]
    fn with_default_pager() {
        let pager = Pager::with_default_pager("more_or_less");
        assert_pager(&pager, "more_or_less");
    }

    #[test]
    fn with_pager() {
        let pager = Pager::with_pager("now_or_never");
        assert_pager(&pager, "now_or_never");
    }
}
