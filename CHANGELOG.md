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

* 0.1.1 (2025-08-19): Fix readme
* 0.1.2 (2025-08-19): Remove docsrs links; fix changelog
* 0.2.0 (2025-08-19): Update to `2024` edition
* 0.3.0 (2025-08-20): Remove deprecated and 2018 edition references; add better example to readme; include readme as the crate doc; fix changelog, readme; apply pedantic clippy fixes
* 0.4.0 (2025-08-21): Add `.no_skip()` method to simplify the usage to:

    ```rust
    use pager2::Pager;
    Pager::with_pager("bat -pl md --color always").no_skip().setup();
    ```

* 0.4.1 (2025-08-21): Add doc for the `.no_skip()` method
* 0.5.0 (2025-08-21): Add the `.skip_on_notty()` method and make the `.skip_on_notty` field private
* 0.5.1 (2025-08-21): Scrub doc

