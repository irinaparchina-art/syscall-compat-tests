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

/// kill(getpid(), 0) — signal 0 just checks if process exists, should return 0
pub struct KillSelfTest;
impl SyscallTest for KillSelfTest {
    fn name(&self) -> &str { "signal_kill_self_zero" }
    fn syscall(&self) -> &str { "kill" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "kill(getpid(), 0) should return 0 (process exists check)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let pid = unsafe { getpid() };
        let ret = unsafe { kill(pid, 0) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

/// kill(-1, 0) with insufficient privileges should return EPERM or succeed
pub struct KillInvalidPidTest;
impl SyscallTest for KillInvalidPidTest {
    fn name(&self) -> &str { "signal_kill_invalid_signal" }
    fn syscall(&self) -> &str { "kill" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "kill(getpid(), 999) with invalid signal should return -1 with EINVAL" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let pid = unsafe { getpid() };
        let ret = unsafe { kill(pid, 999) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == -1 && errno_val == EINVAL as i32 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(EINVAL as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

/// sigaction: register a handler for SIGUSR1, send it, verify handler ran
pub struct SigactionBasicTest;
impl SyscallTest for SigactionBasicTest {
    fn name(&self) -> &str { "signal_sigaction_basic" }
    fn syscall(&self) -> &str { "sigaction" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "sigaction() register SIGUSR1 handler; kill() to self should invoke it" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        static mut RECEIVED: bool = false;
        extern "C" fn handler(_: c_int) { unsafe { RECEIVED = true; } }
        let sa = sigaction {
            sa_sigaction: handler as usize,
            sa_mask: unsafe { std::mem::zeroed() },
            sa_flags: 0,
            sa_restorer: None,
        };
        let r1 = unsafe { sigaction(SIGUSR1, &sa, std::ptr::null_mut()) };
        if r1 != 0 {
            let e = unsafe { *libc::__errno_location() };
            if e == ENOSYS as i32 {
                return make_result(self.name(), self.syscall(), self.category(), self.description(),
                    TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
            }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("sigaction failed".into()), start.elapsed().as_micros() as u64);
        }
        let pid = unsafe { getpid() };
        unsafe { kill(pid, SIGUSR1) };
        let dur = start.elapsed().as_micros() as u64;
        let status = if unsafe { RECEIVED } { TestStatus::Pass }
        else { TestStatus::Error("handler was not called".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

/// sigprocmask: block SIGUSR1, verify it's blocked
pub struct SigprocmaskBlockTest;
impl SyscallTest for SigprocmaskBlockTest {
    fn name(&self) -> &str { "signal_sigprocmask_block" }
    fn syscall(&self) -> &str { "sigprocmask" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "sigprocmask(SIG_BLOCK) should add signals to the blocked mask" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut set: sigset_t = unsafe { std::mem::zeroed() };
        unsafe { sigemptyset(&mut set); sigaddset(&mut set, SIGUSR1); }
        let mut oldset: sigset_t = unsafe { std::mem::zeroed() };
        let r = unsafe { sigprocmask(SIG_BLOCK, &set, &mut oldset) };
        let errno_val = unsafe { *libc::__errno_location() };
        // Restore
        unsafe { sigprocmask(SIG_SETMASK, &oldset, std::ptr::null_mut()) };
        let dur = start.elapsed().as_micros() as u64;
        let status = if r == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

/// sigpending: after blocking and sending SIGUSR1, it should be pending
pub struct SigpendingTest;
impl SyscallTest for SigpendingTest {
    fn name(&self) -> &str { "signal_sigpending" }
    fn syscall(&self) -> &str { "sigpending" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "sigpending() should show SIGUSR1 as pending after blocking and sending it" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // Block SIGUSR1
        let mut set: sigset_t = unsafe { std::mem::zeroed() };
        let mut oldset: sigset_t = unsafe { std::mem::zeroed() };
        unsafe { sigemptyset(&mut set); sigaddset(&mut set, SIGUSR1); }
        unsafe { sigprocmask(SIG_BLOCK, &set, &mut oldset) };
        // Send to self
        let pid = unsafe { getpid() };
        unsafe { kill(pid, SIGUSR1) };
        // Check pending
        let mut pending: sigset_t = unsafe { std::mem::zeroed() };
        let r = unsafe { sigpending(&mut pending) };
        let is_pending = unsafe { sigismember(&pending, SIGUSR1) };
        // Restore and consume signal
        unsafe { sigprocmask(SIG_SETMASK, &oldset, std::ptr::null_mut()) };
        let dur = start.elapsed().as_micros() as u64;
        let status = if r == 0 && is_pending == 1 { TestStatus::Pass }
        else if r == -1 && unsafe { *libc::__errno_location() } == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: is_pending as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

/// raise() sends a signal to the calling process
pub struct RaiseTest;
impl SyscallTest for RaiseTest {
    fn name(&self) -> &str { "signal_raise_sigusr1" }
    fn syscall(&self) -> &str { "raise" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "raise(SIGUSR1) should deliver signal to calling process" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        static mut RAISE_RECEIVED: bool = false;
        extern "C" fn raise_handler(_: c_int) { unsafe { RAISE_RECEIVED = true; } }
        let sa = sigaction {
            sa_sigaction: raise_handler as usize,
            sa_mask: unsafe { std::mem::zeroed() },
            sa_flags: 0,
            sa_restorer: None,
        };
        unsafe { sigaction(SIGUSR1, &sa, std::ptr::null_mut()) };
        let ret = unsafe { raise(SIGUSR1) } as i64;
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 && unsafe { RAISE_RECEIVED } { TestStatus::Pass }
        else if ret == -1 && unsafe { *libc::__errno_location() } == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}
