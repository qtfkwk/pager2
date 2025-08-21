use {pager2::Pager, std::env};

#[test]
fn nopager() {
    unsafe {
        env::set_var("NOPAGER", "");
    }
    let mut pager = Pager::new();
    pager.setup();
    unsafe {
        env::remove_var("NOPAGER");
    }
    assert!(!pager.is_on());
}

#[test]
fn skip_on_notty() {
    let mut pager = Pager::new();
    pager.setup();
    assert!(!pager.is_on());
}

#[test]
fn no_skip() {
    let pager = Pager::new().no_skip();
    assert!(!pager.skip_on_notty());
}
