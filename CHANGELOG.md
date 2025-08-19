# CHANGELOG

* 0.1.0 (2025-08-19)
    * Forked from <https://gitlab.com/imp/pager-rs/-/commit/2b7dc4d0bdd8a0c7cde19e8c7fbc2e5cd75c78b0> to <https://github.com/qtfkwk/pager2>
    * Updated dependencies
    * Apply clippy fixes
    * Leave `.skip_on_notty()` as the default but enable users to force invoking the pager via:

        ```rust
        use pager2::Pager;

        let mut pager = Pager::with_pager("bat -pl md --color always");

        pager.skip_on_notty = false;

        pager.setup();
        ```

* 0.1.1 (2025-08-19)
    * Fix readme

