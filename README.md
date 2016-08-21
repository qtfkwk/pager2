## Pager - long output best friend

 branch | status
:-------|:------:
  master| ![Build Status](https://gitlab.com/imp/pager-rs/badges/master/build.svg)
 develop| ![Build Status](https://gitlab.com/imp/pager-rs/badges/develop/build.svg)


master ![Build Status](https://gitlab.com/imp/pager-rs/badges/master/build.svg) develop ![Build Status](https://gitlab.com/imp/pager-rs/badges/develop/build.svg)
Does all the magic to have you potentially long output piped through the
external pager. Similar to what `git` does for its output.

# Quick Start

```rust
extern crate pager;

use pager::Pager;

fn main() {
    Pager::new().setup();
    // The rest of your program goes here
}
```

Under the hood this forks the current process, connects child' stdout
to parent's stdin, and then replaces the parent with the pager of choice
(environment variable PAGER). The child just continues as normal. If PAGER
environment variable is not present `Pager` probes current PATH for `more`.
If found it is used as a default pager.

You can control pager to a limited degree. For example you can change the
environment variable used for finding pager executable.

```rust
extern crate pager;

use pager::Pager;

fn main() {
    Pager::env("MY_PAGER").setup();
    // The rest of your program goes here
}
```

If no suitable pager found `setup()` does nothing and your executable keeps
running as usual. `Pager` cleans after itself and doesn't leak resources in
case of setup failure.
