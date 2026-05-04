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

/// getpgrp() should return a positive process group ID
pub struct GetpgrpTest;
impl SyscallTest for GetpgrpTest {
    fn name(&self) -> &str { "proc2_getpgrp" }
    fn syscall(&self) -> &str { "getpgrp" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getpgrp() should return a positive process group ID" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { getpgrp() } as i64;
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret > 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

/// getsid(0) returns the session ID of the calling process
pub struct GetsidTest;
impl SyscallTest for GetsidTest {
    fn name(&self) -> &str { "proc2_getsid" }
    fn syscall(&self) -> &str { "getsid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getsid(0) should return a positive session ID" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { getsid(0) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

/// geteuid() should return a valid effective UID
pub struct GeteuidTest;
impl SyscallTest for GeteuidTest {
    fn name(&self) -> &str { "proc2_geteuid" }
    fn syscall(&self) -> &str { "geteuid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "geteuid() should return a valid effective UID (>= 0)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { geteuid() } as i64;
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret >= 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

/// getegid() should return a valid effective GID
pub struct GetegidTest;
impl SyscallTest for GetegidTest {
    fn name(&self) -> &str { "proc2_getegid" }
    fn syscall(&self) -> &str { "getegid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getegid() should return a valid effective GID (>= 0)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { getegid() } as i64;
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret >= 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

/// fork() + waitpid(): child exits 0, parent waits and gets status 0
pub struct ForkWaitTest;
impl SyscallTest for ForkWaitTest {
    fn name(&self) -> &str { "proc2_fork_wait" }
    fn syscall(&self) -> &str { "fork/waitpid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "fork() child exits 0; waitpid() in parent should return child pid with status 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let pid = unsafe { fork() };
        if pid < 0 {
            let e = unsafe { *libc::__errno_location() };
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if e == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Error("fork failed".into()) },
                start.elapsed().as_micros() as u64);
        }
        if pid == 0 { unsafe { libc::exit(0) }; }
        let mut status = 0i32;
        let r = unsafe { waitpid(pid, &mut status, 0) };
        let dur = start.elapsed().as_micros() as u64;
        let exit_code = if libc::WIFEXITED(status) { libc::WEXITSTATUS(status) } else { -1 };
        let s = if r == pid && exit_code == 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: exit_code as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// waitpid with WNOHANG on no children should return 0 or ECHILD
pub struct WaitpidNohangTest;
impl SyscallTest for WaitpidNohangTest {
    fn name(&self) -> &str { "proc2_waitpid_wnohang" }
    fn syscall(&self) -> &str { "waitpid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "waitpid(-1, WNOHANG) with no children should return -1 ECHILD or 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut status = 0i32;
        let ret = unsafe { waitpid(-1, &mut status, WNOHANG) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 || (ret == -1 && errno_val == ECHILD as i32) { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(ECHILD as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getrusage(RUSAGE_SELF) should succeed
pub struct GetrusageTest;
impl SyscallTest for GetrusageTest {
    fn name(&self) -> &str { "proc2_getrusage_self" }
    fn syscall(&self) -> &str { "getrusage" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getrusage(RUSAGE_SELF) should return 0 and fill ru_utime" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut usage: rusage = unsafe { std::mem::zeroed() };
        let ret = unsafe { getrusage(RUSAGE_SELF, &mut usage) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// times() should return a non-negative clock value
pub struct TimesTest;
impl SyscallTest for TimesTest {
    fn name(&self) -> &str { "proc2_times" }
    fn syscall(&self) -> &str { "times" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "times() should return a non-negative clock value" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut buf: tms = unsafe { std::mem::zeroed() };
        let ret = unsafe { times(&mut buf) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// setpgid(0, 0) sets process group to own PID, should return 0
pub struct SetpgidTest;
impl SyscallTest for SetpgidTest {
    fn name(&self) -> &str { "proc2_setpgid" }
    fn syscall(&self) -> &str { "setpgid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "setpgid(0, 0) should set process group to own PID and return 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { setpgid(0, 0) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fork + execve: run /bin/true, expect exit code 0
pub struct ExecveBasicTest;
impl SyscallTest for ExecveBasicTest {
    fn name(&self) -> &str { "proc2_execve_true" }
    fn syscall(&self) -> &str { "execve" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "fork+execve('/bin/true') child should exit with status 0" }
    fn run(&self) -> TestResult {
        use std::ffi::CString;
        let start = Instant::now();
        let pid = unsafe { fork() };
        if pid < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            let path = CString::new("/bin/true").unwrap();
            let arg0 = CString::new("true").unwrap();
            let mut argv = [arg0.as_ptr(), std::ptr::null()];
            let mut envp: [*const libc::c_char; 1] = [std::ptr::null()];
            unsafe { execve(path.as_ptr(), argv.as_mut_ptr(), envp.as_mut_ptr()) };
            unsafe { libc::exit(1) };
        }
        let mut status = 0i32;
        let r = unsafe { waitpid(pid, &mut status, 0) };
        let dur = start.elapsed().as_micros() as u64;
        let exit_code = if libc::WIFEXITED(status) { libc::WEXITSTATUS(status) } else { -1 };
        let s = if r == pid && exit_code == 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: exit_code as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fork + execve /bin/false should exit with non-zero status
pub struct ExecveFalseTest;
impl SyscallTest for ExecveFalseTest {
    fn name(&self) -> &str { "proc2_execve_false" }
    fn syscall(&self) -> &str { "execve" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "fork+execve('/bin/false') child should exit with non-zero status" }
    fn run(&self) -> TestResult {
        use std::ffi::CString;
        let start = Instant::now();
        let pid = unsafe { fork() };
        if pid < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            let path = CString::new("/bin/false").unwrap();
            let arg0 = CString::new("false").unwrap();
            let mut argv = [arg0.as_ptr(), std::ptr::null()];
            let mut envp: [*const libc::c_char; 1] = [std::ptr::null()];
            unsafe { execve(path.as_ptr(), argv.as_mut_ptr(), envp.as_mut_ptr()) };
            unsafe { libc::exit(0) };
        }
        let mut status = 0i32;
        let r = unsafe { waitpid(pid, &mut status, 0) };
        let dur = start.elapsed().as_micros() as u64;
        let exit_code = if libc::WIFEXITED(status) { libc::WEXITSTATUS(status) } else { -1 };
        let s = if r == pid && exit_code != 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: exit_code as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getpriority(PRIO_PROCESS, 0) should return current nice value without error
pub struct GetpriorityTest;
impl SyscallTest for GetpriorityTest {
    fn name(&self) -> &str { "proc2_getpriority" }
    fn syscall(&self) -> &str { "getpriority" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getpriority(PRIO_PROCESS, 0) should succeed (errno stays 0)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        unsafe { *libc::__errno_location() = 0 };
        let _ret = unsafe { getpriority(PRIO_PROCESS, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if errno_val == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: errno_val as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
