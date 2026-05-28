use std::time::Instant;

use libc::*;

use crate::runner::{SyscallCategory, SyscallTest, TestResult, TestStatus};

macro_rules! timed_syscall {
    ($body:expr) => {{
        let start = Instant::now();
        let ret: i64 = $body;
        let errno_val: i32 = unsafe { *libc::__errno_location() };
        let duration_us = start.elapsed().as_micros() as u64;
        (ret, errno_val, duration_us)
    }};
}

fn make_result(
    name: &str,
    syscall: &str,
    category: SyscallCategory,
    description: &str,
    status: TestStatus,
    duration_us: u64,
) -> TestResult {
    TestResult {
        name: name.to_string(),
        syscall: syscall.to_string(),
        category,
        status,
        description: description.to_string(),
        duration_us,
    }
}

// ─── epoll_create1 ──────────────────────────────────────────────────────────

pub struct EpollCreate1BasicTest;
impl SyscallTest for EpollCreate1BasicTest {
    fn name(&self) -> &str { "epoll_create1_basic" }
    fn syscall(&self) -> &str { "epoll_create1" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "epoll_create1(0) should return a valid epoll fd"
    }
    fn run(&self) -> TestResult {
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { epoll_create1(0) as i64 }
        });
        if ret >= 0 {
            unsafe { close(ret as i32) };
        }
        let status = if ret >= 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct EpollCreate1CloexecTest;
impl SyscallTest for EpollCreate1CloexecTest {
    fn name(&self) -> &str { "epoll_create1_cloexec" }
    fn syscall(&self) -> &str { "epoll_create1" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "epoll_create1(EPOLL_CLOEXEC) should return a valid fd with close-on-exec set"
    }
    fn run(&self) -> TestResult {
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { epoll_create1(EPOLL_CLOEXEC) as i64 }
        });
        let status = if ret >= 0 {
            let flags = unsafe { fcntl(ret as i32, F_GETFD) };
            unsafe { close(ret as i32) };
            if flags & FD_CLOEXEC != 0 {
                TestStatus::Pass
            } else {
                TestStatus::Fail {
                    expected_ret: 1,
                    actual_ret: 0,
                    expected_errno: None,
                    actual_errno: None,
                }
            }
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── epoll_ctl ──────────────────────────────────────────────────────────────

pub struct EpollCtlAddTest;
impl SyscallTest for EpollCtlAddTest {
    fn name(&self) -> &str { "epoll_ctl_add" }
    fn syscall(&self) -> &str { "epoll_ctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "epoll_ctl(EPOLL_CTL_ADD) should register a pipe fd for EPOLLIN"
    }
    fn run(&self) -> TestResult {
        let epfd = unsafe { epoll_create1(0) };
        let mut pipefd = [0i32; 2];
        unsafe { pipe(pipefd.as_mut_ptr()) };

        let mut ev = epoll_event { events: EPOLLIN as u32, u64: pipefd[0] as u64 };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, pipefd[0], &mut ev) as i64 }
        });

        unsafe { close(pipefd[0]); close(pipefd[1]); close(epfd); }

        let status = if ret == 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct EpollCtlDelTest;
impl SyscallTest for EpollCtlDelTest {
    fn name(&self) -> &str { "epoll_ctl_del" }
    fn syscall(&self) -> &str { "epoll_ctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "epoll_ctl(EPOLL_CTL_DEL) should remove a previously added fd"
    }
    fn run(&self) -> TestResult {
        let epfd = unsafe { epoll_create1(0) };
        let mut pipefd = [0i32; 2];
        unsafe { pipe(pipefd.as_mut_ptr()) };

        let mut ev = epoll_event { events: EPOLLIN as u32, u64: pipefd[0] as u64 };
        unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, pipefd[0], &mut ev) };

        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { epoll_ctl(epfd, EPOLL_CTL_DEL, pipefd[0], std::ptr::null_mut()) as i64 }
        });

        unsafe { close(pipefd[0]); close(pipefd[1]); close(epfd); }

        let status = if ret == 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct EpollCtlModTest;
impl SyscallTest for EpollCtlModTest {
    fn name(&self) -> &str { "epoll_ctl_mod" }
    fn syscall(&self) -> &str { "epoll_ctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "epoll_ctl(EPOLL_CTL_MOD) should modify events on a registered fd"
    }
    fn run(&self) -> TestResult {
        let epfd = unsafe { epoll_create1(0) };
        let mut pipefd = [0i32; 2];
        unsafe { pipe(pipefd.as_mut_ptr()) };

        let mut ev = epoll_event { events: EPOLLIN as u32, u64: pipefd[0] as u64 };
        unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, pipefd[0], &mut ev) };

        ev.events = (EPOLLIN | EPOLLOUT) as u32;
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { epoll_ctl(epfd, EPOLL_CTL_MOD, pipefd[0], &mut ev) as i64 }
        });

        unsafe { close(pipefd[0]); close(pipefd[1]); close(epfd); }

        let status = if ret == 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct EpollCtlBadFdTest;
impl SyscallTest for EpollCtlBadFdTest {
    fn name(&self) -> &str { "epoll_ctl_bad_fd" }
    fn syscall(&self) -> &str { "epoll_ctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "epoll_ctl(EPOLL_CTL_ADD) with an invalid fd should return EBADF"
    }
    fn run(&self) -> TestResult {
        let epfd = unsafe { epoll_create1(0) };
        let mut ev = epoll_event { events: EPOLLIN as u32, u64: 9999 };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, 9999, &mut ev) as i64 }
        });
        unsafe { close(epfd) };

        let status = if ret == -1 && errno_val == EBADF {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1,
                actual_ret: ret,
                expected_errno: Some(EBADF),
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── epoll_wait ─────────────────────────────────────────────────────────────

pub struct EpollWaitReadyTest;
impl SyscallTest for EpollWaitReadyTest {
    fn name(&self) -> &str { "epoll_wait_ready" }
    fn syscall(&self) -> &str { "epoll_wait" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "epoll_wait should return 1 event when a pipe has data ready to read"
    }
    fn run(&self) -> TestResult {
        let epfd = unsafe { epoll_create1(0) };
        let mut pipefd = [0i32; 2];
        unsafe { pipe(pipefd.as_mut_ptr()) };

        let mut ev = epoll_event { events: EPOLLIN as u32, u64: pipefd[0] as u64 };
        unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, pipefd[0], &mut ev) };

        // Write data so pipe is ready
        unsafe { write(pipefd[1], b"x".as_ptr() as *const _, 1) };

        let mut events = [epoll_event { events: 0, u64: 0 }; 4];
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { epoll_wait(epfd, events.as_mut_ptr(), 4, 100) as i64 }
        });

        unsafe { close(pipefd[0]); close(pipefd[1]); close(epfd); }

        let status = if ret == 1 && (events[0].events & EPOLLIN as u32) != 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 1,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct EpollWaitTimeoutTest;
impl SyscallTest for EpollWaitTimeoutTest {
    fn name(&self) -> &str { "epoll_wait_timeout" }
    fn syscall(&self) -> &str { "epoll_wait" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "epoll_wait with no ready fds and 0 timeout should return 0 immediately"
    }
    fn run(&self) -> TestResult {
        let epfd = unsafe { epoll_create1(0) };
        let mut pipefd = [0i32; 2];
        unsafe { pipe(pipefd.as_mut_ptr()) };

        let mut ev = epoll_event { events: EPOLLIN as u32, u64: pipefd[0] as u64 };
        unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, pipefd[0], &mut ev) };

        let mut events = [epoll_event { events: 0, u64: 0 }; 4];
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { epoll_wait(epfd, events.as_mut_ptr(), 4, 0) as i64 }
        });

        unsafe { close(pipefd[0]); close(pipefd[1]); close(epfd); }

        let status = if ret == 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct EpollCtlDuplicateTest;
impl SyscallTest for EpollCtlDuplicateTest {
    fn name(&self) -> &str { "epoll_ctl_duplicate" }
    fn syscall(&self) -> &str { "epoll_ctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "epoll_ctl(EPOLL_CTL_ADD) for an already-added fd should return EEXIST"
    }
    fn run(&self) -> TestResult {
        let epfd = unsafe { epoll_create1(0) };
        let mut pipefd = [0i32; 2];
        unsafe { pipe(pipefd.as_mut_ptr()) };

        let mut ev = epoll_event { events: EPOLLIN as u32, u64: pipefd[0] as u64 };
        unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, pipefd[0], &mut ev) };

        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, pipefd[0], &mut ev) as i64 }
        });

        unsafe { close(pipefd[0]); close(pipefd[1]); close(epfd); }

        let status = if ret == -1 && errno_val == EEXIST {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1,
                actual_ret: ret,
                expected_errno: Some(EEXIST),
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct EpollWaitEpolloutTest;
impl SyscallTest for EpollWaitEpolloutTest {
    fn name(&self) -> &str { "epoll_wait_epollout" }
    fn syscall(&self) -> &str { "epoll_wait" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "epoll_wait should report EPOLLOUT when a pipe write end is ready"
    }
    fn run(&self) -> TestResult {
        let epfd = unsafe { epoll_create1(0) };
        let mut pipefd = [0i32; 2];
        unsafe { pipe(pipefd.as_mut_ptr()) };

        let mut ev = epoll_event { events: EPOLLOUT as u32, u64: pipefd[1] as u64 };
        unsafe { epoll_ctl(epfd, EPOLL_CTL_ADD, pipefd[1], &mut ev) };

        let mut events = [epoll_event { events: 0, u64: 0 }; 4];
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { epoll_wait(epfd, events.as_mut_ptr(), 4, 100) as i64 }
        });

        unsafe { close(pipefd[0]); close(pipefd[1]); close(epfd); }

        let status = if ret >= 1 && (events[0].events & EPOLLOUT as u32) != 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 1,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}
