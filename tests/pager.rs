extern crate pager;

use std::env;

use pager::Pager;

#[test]
fn nopager() {
    env::set_var("NOPAGER", "");
    let pager = Pager::new().setup();
    env::remove_var("NOPAGER");
    assert!(!pager.is_on());
}

#[test]
fn skip_on_notty() {
    let pager = Pager::new().setup();
    assert!(!pager.is_on());
}
