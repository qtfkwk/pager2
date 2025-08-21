#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "pedantic", warn(clippy::pedantic))]
#![warn(clippy::use_self)]
#![warn(deprecated_in_future)]
#![warn(future_incompatible)]
#![warn(unreachable_pub)]
#![warn(missing_debug_implementations)]
#![warn(unused)]
#![deny(warnings)]

mod utils;

use std::env;
use std::ffi::{OsStr, OsString};

/// Default pager environment variable
const DEFAULT_PAGER_ENV: &str = "PAGER";

/// Environment variable to disable pager altogether
const NOPAGER_ENV: &str = "NOPAGER";

/// Last resort pager. Should work everywhere.
const DEFAULT_PAGER: &str = "more";

/// Keeps track of the current pager state
#[derive(Debug)]
pub struct Pager {
    default: Option<OsString>,
    env: Option<OsString>,
    envs: Vec<OsString>,
    on: bool,
    pub skip_on_notty: bool,
}

impl Default for Pager {
    fn default() -> Self {
        Self {
            default: None,
            env: env::var_os(DEFAULT_PAGER_ENV),
            envs: Vec::new(),
            on: true,
            skip_on_notty: true,
        }
    }
}

impl Pager {
    /// Creates new instance of `Pager` with default settings
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates new instance of pager using `env` environment variable instead of PAGER
    #[must_use]
    pub fn with_env(env: &str) -> Self {
        Self {
            env: env::var_os(env),
            ..Self::default()
        }
    }

    /// Creates a new `Pager` instance with the specified default fallback
    pub fn with_default_pager<S>(pager: S) -> Self
    where
        S: Into<OsString>,
    {
        let default_pager = Some(pager.into());
        Self {
            default: default_pager,
            ..Self::default()
        }
    }

    /// Creates a new `Pager` instance directly specifying the desired pager
    #[must_use]
    pub fn with_pager(pager: &str) -> Self {
        Self {
            env: Some(pager.into()),
            ..Self::default()
        }
    }

    /// Launch pager with the specified environment variables
    #[must_use]
    pub fn pager_envs(self, envs: impl IntoIterator<Item = impl Into<OsString>>) -> Self {
        let envs = envs.into_iter().map(std::convert::Into::into).collect();
        Self { envs, ..self }
    }

    /// Gives quick assessment of successful `Pager` setup
    #[must_use]
    pub fn is_on(&self) -> bool {
        self.on
    }

    fn pager(&self) -> Option<OsString> {
        let fallback_pager = || Some(OsStr::new(DEFAULT_PAGER).into());

        if env::var_os(NOPAGER_ENV).is_some() {
            None
        } else {
            self.env
                .clone()
                .or_else(|| self.default.clone())
                .or_else(fallback_pager)
        }
    }

    /// Convert this pager to one that will not be skipped if stdout is not a TTY
    #[must_use]
    pub fn no_skip(mut self) -> Pager {
        self.skip_on_notty = false;
        self
    }

    /// Initiates Pager framework and sets up all the necessary environment for sending standard
    /// output to the activated pager.
    pub fn setup(&mut self) {
        if self.skip_on_notty && !utils::isatty(libc::STDOUT_FILENO) {
            self.on = false;
            return;
        }
        if let Some(ref pager) = self.pager() {
            let (pager_stdin, main_stdout) = utils::pipe();
            let pid = utils::fork();
            match pid {
                -1 => {
                    // Fork failed
                    utils::close(pager_stdin);
                    utils::close(main_stdout);
                    self.on = false;
                }
                0 => {
                    // I am child
                    utils::dup2(main_stdout, libc::STDOUT_FILENO);
                    utils::close(pager_stdin);
                }
                _ => {
                    // I am parent
                    utils::dup2(pager_stdin, libc::STDIN_FILENO);
                    utils::close(main_stdout);
                    utils::execvpe(pager, &self.envs);
                }
            }
        } else {
            self.on = false;
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
            unsafe {
                match self {
                    Self::Reinstate(env, _) | Self::Remove(env) => env::set_var(env, value),
                }
            }
        }

        fn remove(&self) {
            unsafe {
                match self {
                    Self::Reinstate(env, _) | Self::Remove(env) => env::remove_var(env),
                }
            }
        }
    }

    impl Drop for PagerEnv {
        fn drop(&mut self) {
            unsafe {
                match self {
                    Self::Reinstate(env, value) => env::set_var(env, value),
                    Self::Remove(env) => env::remove_var(env),
                }
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
