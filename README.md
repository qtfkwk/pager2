# Pager2 - long output best friend

[![Crates.io](https://img.shields.io/crates/v/pager2.svg)](https://crates.io/crates/pager2)
[![Docs.rs](https://docs.rs/pager2/badge.svg)](https://docs.rs/pager2)

> [!NOTE]
> Pager2 is a fork of the [pager crate](https://crates.io/crates/pager) @
> [2b7dc4d0/2022-09-22](https://gitlab.com/imp/pager-rs/-/commit/2b7dc4d0bdd8a0c7cde19e8c7fbc2e5cd75c78b0)
> on 2025-08-19 due to [this issue](https://gitlab.com/imp/pager-rs/-/issues/8#note_2695988275); see
> also the [Forcing the pager](#forcing-the-pager) example.

Does all the magic to have you potentially long output piped through the external pager.
Similar to what `git` does for its output.

## Quick Start

```rust
# #[allow(needless_doctest_main)]
use pager2::Pager;

fn main() {
    Pager::new().setup();
    // The rest of your program goes here
}
```

Under the hood this forks the current process, connects child's stdout to parent's stdin, and then
replaces the parent with the pager of choice (environment variable `PAGER`).
The child just continues as normal.
If `PAGER` environment variable is not present [`Pager`] probes current PATH for `more`.
If found it is used as a default pager.

## Custom environment variable

You can control pager to a limited degree.
For example you can change the environment variable used for finding pager executable.

```rust
# #[allow(needless_doctest_main)]
use pager2::Pager;

fn main() {
    Pager::with_env("MY_PAGER").setup();
    // The rest of your program goes here
}
```

## Alternative fallback

Also you can set alternative default (fallback) pager to be used instead of `more`.
`PAGER` environment variable (if set) will still have precedence.

```rust
# #[allow(needless_doctest_main)]
use pager2::Pager;

fn main() {
    Pager::with_default_pager("pager").setup();
    // The rest of your program goes here
}
```

If no suitable pager is found, [`Pager::setup()`] does nothing and your executable keeps running as
usual.
[`Pager`] cleans after itself and doesn't leak resources in case of setup failure.

## Custom pager command

Alternatively you can specify directly the desired pager command, exactly as it would appear in
`PAGER` environment variable.
This is useful if you need some specific pager and/or flags (like `less -r`) and would like to avoid
forcing your consumers into modifying their existing `PAGER` configuration just for your
application.

```rust
# #[allow(needless_doctest_main)]
use pager2::Pager;

fn main() {
    Pager::with_pager("less -r").setup();
    // The rest of your program goes here
}
```

## Disabling the pager

If you need to disable pager altogether set environment variable `NOPAGER` and [`Pager::setup()`]
will skip initialization.
The host application will continue as normal.
[`Pager::is_on()`] will reflect the fact that no [`Pager`] is active.

## Forcing the pager

Sometimes you may want to force the pager to be set even if the output of your executable is not a
`tty`, for example when your executable has a `--color always` option.

```rust
# #[allow(needless_doctest_main)]
use {
    clap::{Parser, ValueEnum},
    pager2::Pager,
    std::io::IsTerminal,
    which::which,
};

#[derive(Parser)]
struct Options {
    /// Color
    #[arg(long, default_value = "auto")]
    color: Color,
}

#[derive(Clone, PartialEq, ValueEnum)]
enum Color {
    Auto,
    Always,
    Never,
}

fn main() {
    let options = Options::parse();
    if which("bat").is_ok() {
        let mut bat = String::from("bat -p");
        let always = options.color == Color::Always;
        if (options.color == Color::Auto && std::io::stdout().is_terminal()) || always {
            // Syntax highlight JSON output if stdout is a TTY or color set to always
            bat.push_str(" -l json");
        }
        if always {
            // Force color
            bat.push_str(" --color always");

            Pager::with_pager(&bat).no_skip().setup();
        } else {
            Pager::with_pager(&bat).setup();
        }
    }

    // The rest of your program goes here
}
```

