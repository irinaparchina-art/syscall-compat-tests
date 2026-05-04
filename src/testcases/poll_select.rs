use std::time::Instant;
use libc::*;

use crate::runner::{SyscallCategory, SyscallTest, TestResult, TestStatus};

fn make_result(
    name: &str, syscall: &str, category: SyscallCategory,
    description: &str, status: TestStatus, duration_us: u64,
) -> TestResult {
    TestResult {
        name: name.to_string(), syscall: syscall.to_string(),
        category, status, description: description.to_string(), duration_us,
    }
}

/// poll() on a pipe write-end should report POLLOUT immediately
pub struct PollWriteReadyTest;
impl SyscallTest for PollWriteReadyTest {
    fn name(&self) -> &str { "poll_write_ready" }
    fn syscall(&self) -> &str { "poll" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "poll() on pipe write-end with POLLOUT should return 1 immediately" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut pfd = pollfd { fd: fds[1], events: POLLOUT, revents: 0 };
        let ret = unsafe { poll(&mut pfd, 1, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 1 && pfd.revents & POLLOUT != 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// poll() with timeout=0 on empty pipe read-end should return 0 (timeout)
pub struct PollTimeoutTest;
impl SyscallTest for PollTimeoutTest {
    fn name(&self) -> &str { "poll_timeout_empty_pipe" }
    fn syscall(&self) -> &str { "poll" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "poll() on empty pipe read-end with timeout=0 should return 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut pfd = pollfd { fd: fds[0], events: POLLIN, revents: 0 };
        let ret = unsafe { poll(&mut pfd, 1, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// poll() after writing to pipe: read-end should show POLLIN
pub struct PollReadReadyTest;
impl SyscallTest for PollReadReadyTest {
    fn name(&self) -> &str { "poll_read_ready_after_write" }
    fn syscall(&self) -> &str { "poll" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "poll() on pipe read-end should show POLLIN after data is written" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fds[1], b"x".as_ptr() as *const _, 1); }
        let mut pfd = pollfd { fd: fds[0], events: POLLIN, revents: 0 };
        let ret = unsafe { poll(&mut pfd, 1, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 1 && pfd.revents & POLLIN != 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// poll() with invalid fd should set POLLNVAL in revents
pub struct PollInvalidFdTest;
impl SyscallTest for PollInvalidFdTest {
    fn name(&self) -> &str { "poll_invalid_fd" }
    fn syscall(&self) -> &str { "poll" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "poll() on invalid fd should return 1 and set POLLNVAL in revents" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut pfd = pollfd { fd: 9999, events: POLLIN, revents: 0 };
        let ret = unsafe { poll(&mut pfd, 1, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 1 && pfd.revents & POLLNVAL != 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// select() on write-end of pipe should show it writable
pub struct SelectWriteReadyTest;
impl SyscallTest for SelectWriteReadyTest {
    fn name(&self) -> &str { "select_write_ready" }
    fn syscall(&self) -> &str { "select" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "select() write-fd-set on pipe write-end should return 1 immediately" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut wset: fd_set = unsafe { std::mem::zeroed() };
        unsafe { FD_ZERO(&mut wset); FD_SET(fds[1], &mut wset); }
        let mut tv = timeval { tv_sec: 0, tv_usec: 0 };
        let ret = unsafe { select(fds[1] + 1, std::ptr::null_mut(), &mut wset, std::ptr::null_mut(), &mut tv) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 1 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// select() with zero timeout on empty read-end returns 0
pub struct SelectTimeoutTest;
impl SyscallTest for SelectTimeoutTest {
    fn name(&self) -> &str { "select_timeout_empty" }
    fn syscall(&self) -> &str { "select" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "select() on empty pipe read-end with zero timeout should return 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut rset: fd_set = unsafe { std::mem::zeroed() };
        unsafe { FD_ZERO(&mut rset); FD_SET(fds[0], &mut rset); }
        let mut tv = timeval { tv_sec: 0, tv_usec: 0 };
        let ret = unsafe { select(fds[0] + 1, &mut rset, std::ptr::null_mut(), std::ptr::null_mut(), &mut tv) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// epoll_create1 should return a valid fd
pub struct EpollCreateTest;
impl SyscallTest for EpollCreateTest {
    fn name(&self) -> &str { "epoll_create1_basic" }
    fn syscall(&self) -> &str { "epoll_create1" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "epoll_create1(0) should return a valid epoll fd" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { epoll_create1(0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// epoll_ctl + epoll_wait: add pipe read-end, write data, epoll_wait should fire
pub struct EpollWaitTest;
impl SyscallTest for EpollWaitTest {
    fn name(&self) -> &str { "epoll_ctl_wait_basic" }
    fn syscall(&self) -> &str { "epoll_ctl/epoll_wait" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "epoll_ctl(ADD) + epoll_wait should detect EPOLLIN on pipe after write" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let epfd = unsafe { epoll_create1(0) };
        if epfd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut pfd = [0i32; 2];
        if unsafe { pipe(pfd.as_mut_ptr()) } != 0 {
            unsafe { close(epfd); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut ev = epoll_event { events: EPOLLIN as u32, u64: pfd[0] as u64 };
        unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, pfd[0], &mut ev); }
        unsafe { write(pfd[1], b"hi".as_ptr() as *const _, 2); }
        let mut events = [epoll_event { events: 0, u64: 0 }; 4];
        let ret = unsafe { epoll_wait(epfd, events.as_mut_ptr(), 4, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(epfd); close(pfd[0]); close(pfd[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 1 && events[0].events & EPOLLIN as u32 != 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// epoll_ctl DEL: remove fd from epoll interest list
pub struct EpollCtlDelTest;
impl SyscallTest for EpollCtlDelTest {
    fn name(&self) -> &str { "epoll_ctl_del" }
    fn syscall(&self) -> &str { "epoll_ctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "epoll_ctl(DEL) should remove fd; epoll_wait should return 0 after removal" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let epfd = unsafe { epoll_create1(0) };
        if epfd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut pfd = [0i32; 2];
        if unsafe { pipe(pfd.as_mut_ptr()) } != 0 {
            unsafe { close(epfd); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut ev = epoll_event { events: EPOLLIN as u32, u64: pfd[0] as u64 };
        unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, pfd[0], &mut ev); }
        // Delete it
        let r = unsafe { epoll_ctl(epfd, EPOLL_CTL_DEL, pfd[0], std::ptr::null_mut()) };
        let errno_val = unsafe { *libc::__errno_location() };
        // Write and wait - should get 0 events
        unsafe { write(pfd[1], b"x".as_ptr() as *const _, 1); }
        let mut events = [epoll_event { events: 0, u64: 0 }; 4];
        let n = unsafe { epoll_wait(epfd, events.as_mut_ptr(), 4, 0) };
        unsafe { close(epfd); close(pfd[0]); close(pfd[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r == 0 && n == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
