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

pub struct GetpidTest;
impl SyscallTest for GetpidTest {
    fn name(&self) -> &str { "proc_getpid" }
    fn syscall(&self) -> &str { "getpid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getpid() should return a positive PID" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { getpid() } as i64;
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret > 0 { TestStatus::Pass } else {
            TestStatus::Fail { expected_ret: 1, actual_ret: ret, expected_errno: None, actual_errno: None }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct GetppidTest;
impl SyscallTest for GetppidTest {
    fn name(&self) -> &str { "proc_getppid" }
    fn syscall(&self) -> &str { "getppid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getppid() should return a non-negative value" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { getppid() } as i64;
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret >= 0 { TestStatus::Pass } else {
            TestStatus::Fail { expected_ret: 1, actual_ret: ret, expected_errno: None, actual_errno: None }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct GetuidTest;
impl SyscallTest for GetuidTest {
    fn name(&self) -> &str { "proc_getuid" }
    fn syscall(&self) -> &str { "getuid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getuid() should return a valid UID (>= 0)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { getuid() } as i64;
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret >= 0 { TestStatus::Pass } else {
            TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: None }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct GetgidTest;
impl SyscallTest for GetgidTest {
    fn name(&self) -> &str { "proc_getgid" }
    fn syscall(&self) -> &str { "getgid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getgid() should return a valid GID (>= 0)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { getgid() } as i64;
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret >= 0 { TestStatus::Pass } else {
            TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: None }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct UmaskTest;
impl SyscallTest for UmaskTest {
    fn name(&self) -> &str { "proc_umask" }
    fn syscall(&self) -> &str { "umask" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str {
        "umask(0o022) should return the previous mask (always succeeds)"
    }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let prev = unsafe { umask(0o022) };
        // Restore
        unsafe { umask(prev) };
        let dur = start.elapsed().as_micros() as u64;
        // umask always succeeds, just check it returned something reasonable (< 0o777)
        let status = if (prev as u32) < 0o777 {
            TestStatus::Pass
        } else {
            TestStatus::Fail {
                expected_ret: 0o022, actual_ret: prev as i64,
                expected_errno: None, actual_errno: None,
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct ExitCodeTest;
impl SyscallTest for ExitCodeTest {
    fn name(&self) -> &str { "proc_exit_code" }
    fn syscall(&self) -> &str { "exit" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str {
        "A child process calling exit(42) should be reaped with status 42 via waitpid()"
    }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let pid = unsafe { fork() };
        if pid < 0 {
            let e = unsafe { *libc::__errno_location() };
            if e == ENOSYS as i32 {
                return make_result(self.name(), self.syscall(), self.category(), self.description(),
                    TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
            }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("fork failed".into()), start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            // Child
            unsafe { libc::exit(42) };
        }
        // Parent
        let mut status: i32 = 0;
        let r = unsafe { waitpid(pid, &mut status, 0) };
        let dur = start.elapsed().as_micros() as u64;
        let exit_code = if libc::WIFEXITED(status) { libc::WEXITSTATUS(status) } else { -1 };
        let result_status = if r == pid && exit_code == 42 {
            TestStatus::Pass
        } else {
            TestStatus::Fail {
                expected_ret: 42, actual_ret: exit_code as i64,
                expected_errno: None, actual_errno: None,
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), result_status, dur)
    }
}
