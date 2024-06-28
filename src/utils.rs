pub(crate) fn fork() -> libc::pid_t {
    unsafe { libc::fork() }
}

pub(crate) fn dup2(fd1: i32, fd2: i32) {
    assert!(unsafe { libc::dup2(fd1, fd2) } > -1);
}

pub(crate) fn close(fd: i32) {
    assert_eq!(unsafe { libc::close(fd) }, 0);
}

pub(crate) fn pipe() -> (i32, i32) {
    let mut fds = [0; 2];
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
    (fds[0], fds[1])
}

pub(crate) fn isatty(fd: i32) -> bool {
    unsafe { libc::isatty(fd) != 0 }
}
