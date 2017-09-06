//! Does all the magic to have you potentially long output piped through the
//! external pager. Similar to what git does for its output.
//!
//! # Quick Start
//!
//! ```rust
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
//!     Pager::with_env("MY_PAGER").setup();
//!     // The rest of your program goes here
//! }
//! ```
//!
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
//!     Pager::with_pager("less -r").setup();
//!     // The rest of your program goes here
//! }
//! ```
//!
//! If no suitable pager found `setup()` does nothing and your executable keeps
//! running as usual. `Pager` cleans after itself and doesn't leak resources in
//! case of setup failure.
//!
//! Sometimes you may want to bypass pager if the output of you executable is not a `tty`.
//! If this case you may use `.skip_on_notty()` to get the desirable effect.
//!
//! ```rust
//! extern crate pager;
//! use pager::Pager;
//! fn main() {
//!     Pager::new().skip_on_notty().setup();
//!     // The rest of your program goes here
//! }
//! ```
//!
//! If you need to disable pager altogether set environment variable `NOPAGER` and `Pager::setup()`
//! will skip initialization. The host application will continue as normal. `Pager::is_on()` will
//! reflect the fact that no Pager is active.

extern crate errno;
extern crate libc;

mod utils;

use std::ffi::OsString;

const DEFAULT_PAGER_ENV: &str = "PAGER";

#[derive(Debug, Default)]
pub struct Pager {
    pager: Option<OsString>,
    env: Option<String>,
    on: bool,
    skip_on_notty: bool,
}

impl Pager {
    /// Creates new instance of pager with default settings
    pub fn new() -> Self {
        Pager::with_env(DEFAULT_PAGER_ENV)
    }

    /// Creates new instance of pager using `env` environment variable instead of PAGER
    pub fn with_env(env: &str) -> Self {
        let pager = utils::find_pager(env);

        Pager {
            pager: pager,
            env: String::from(env).into(),
            on: true,
            skip_on_notty: false,
        }
    }

    #[deprecated(since = "0.12.0", note = "use with_env() instead")]
    pub fn env(env: &str) -> Self {
        Pager::with_env(env)
    }

    /// Creates a new pager instance directly specifying the desired pager
    pub fn with_pager(pager: &str) -> Self {
        Pager {
            pager: OsString::from(pager).into(),
            env: None,
            on: true,
            skip_on_notty: false,
        }
    }

    /// Instructs `Pager` to bypass invoking pager if output is not a `tty`
    pub fn skip_on_notty(self) -> Self {
        Pager {
            skip_on_notty: true,
            ..self
        }
    }

    /// Gives quick assessment of successful Pager setup
    pub fn is_on(&self) -> bool {
        self.on
    }

    /// Initiates Pager framework and sets up all the necessary environment for sending standard
    /// output to the activated pager.
    pub fn setup(&mut self) {
        if self.skip_on_notty && !utils::isatty(libc::STDOUT_FILENO) {
            self.on = false;
            return;
        }
        if let Some(ref pager) = self.pager {
            let (pager_stdin, main_stdout) = utils::pipe();
            let pid = utils::fork();
            match pid {
                -1 => {
                    // Fork failed
                    utils::close(pager_stdin);
                    utils::close(main_stdout);
                    self.on = false
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
                    utils::execvp(pager);
                }
            }
        } else {
            self.on = false;
        }
    }
}
