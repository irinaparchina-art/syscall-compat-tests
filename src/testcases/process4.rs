use std::ffi::CString;
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

/// Child process has correct parent PID
pub struct ForkPpidTest;
impl SyscallTest for ForkPpidTest {
    fn name(&self) -> &str { "proc4_fork_child_ppid" }
    fn syscall(&self) -> &str { "fork/getppid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "Child's getppid() should equal parent's getpid()" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let parent_pid = unsafe { getpid() };
        let mut pipefd = [0i32; 2];
        if unsafe { pipe(pipefd.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let pid = unsafe { fork() };
        if pid < 0 {
            unsafe { close(pipefd[0]); close(pipefd[1]); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            unsafe { close(pipefd[0]); }
            let ppid = unsafe { getppid() };
            unsafe { write(pipefd[1], &ppid as *const _ as *const _, 4); close(pipefd[1]); libc::exit(0); }
        }
        unsafe { close(pipefd[1]); }
        let mut child_ppid: pid_t = 0;
        unsafe { read(pipefd[0], &mut child_ppid as *mut _ as *mut _, 4); close(pipefd[0]); }
        unsafe { waitpid(pid, std::ptr::null_mut(), 0); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if child_ppid == parent_pid { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: parent_pid as i64, actual_ret: child_ppid as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// environ is accessible (non-null)
pub struct EnvironTest;
impl SyscallTest for EnvironTest {
    fn name(&self) -> &str { "proc4_environ_accessible" }
    fn syscall(&self) -> &str { "environ" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "The environ global should be non-null and contain PATH" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let path_key = CString::new("PATH").unwrap();
        let val = unsafe { getenv(path_key.as_ptr()) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if !val.is_null() { TestStatus::Pass }
        else { TestStatus::Error("PATH not found in environ".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// putenv: set environment variable via putenv
pub struct PutenvTest;
impl SyscallTest for PutenvTest {
    fn name(&self) -> &str { "proc4_putenv" }
    fn syscall(&self) -> &str { "putenv" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "putenv(\"SCT_VAR=42\") then getenv(\"SCT_VAR\") should return \"42\"" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let env_str = CString::new("SCT_PUTENV_VAR=42").unwrap();
        let ret = unsafe { putenv(env_str.as_ptr() as *mut _) };
        let key = CString::new("SCT_PUTENV_VAR").unwrap();
        let val = unsafe { getenv(key.as_ptr()) };
        let dur = start.elapsed().as_micros() as u64;
        let val_str = if val.is_null() { "" } else { unsafe { std::ffi::CStr::from_ptr(val).to_str().unwrap_or("") } };
        let s = if ret == 0 && val_str == "42" { TestStatus::Pass }
        else { TestStatus::Error(format!("putenv ret={} val={}", ret, val_str)) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// clearenv: clear all environment variables
pub struct ClearenvTest;
impl SyscallTest for ClearenvTest {
    fn name(&self) -> &str { "proc4_clearenv" }
    fn syscall(&self) -> &str { "clearenv" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "clearenv() then setenv/getenv roundtrip should work on fresh environment" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // Save PATH first
        let path_key = CString::new("PATH").unwrap();
        let path_val = unsafe { getenv(path_key.as_ptr()) };
        let saved_path = if path_val.is_null() { None }
        else { Some(unsafe { std::ffi::CStr::from_ptr(path_val).to_owned() }) };
        // Clear
        let ret = unsafe { clearenv() };
        // Set a new var
        let k = CString::new("SCT_CLEAR_TEST").unwrap();
        let v = CString::new("yes").unwrap();
        unsafe { setenv(k.as_ptr(), v.as_ptr(), 1) };
        let got = unsafe { getenv(k.as_ptr()) };
        let got_str = if got.is_null() { "" } else { unsafe { std::ffi::CStr::from_ptr(got).to_str().unwrap_or("") } };
        // Restore PATH
        if let Some(p) = saved_path {
            unsafe { setenv(path_key.as_ptr(), p.as_ptr(), 1) };
        }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && got_str == "yes" { TestStatus::Pass }
        else { TestStatus::Error(format!("clearenv ret={} got={}", ret, got_str)) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// setsid: create a new session
pub struct SetsidTest;
impl SyscallTest for SetsidTest {
    fn name(&self) -> &str { "proc4_setsid_child" }
    fn syscall(&self) -> &str { "setsid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "setsid() in child should create new session; child's sid == child's pid" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut pipefd = [0i32; 2];
        if unsafe { pipe(pipefd.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let pid = unsafe { fork() };
        if pid < 0 {
            unsafe { close(pipefd[0]); close(pipefd[1]); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            unsafe { close(pipefd[0]); }
            let new_sid = unsafe { setsid() };
            let my_pid = unsafe { getpid() };
            let ok: i32 = if new_sid == my_pid { 1 } else { 0 };
            unsafe { write(pipefd[1], &ok as *const _ as *const _, 4); close(pipefd[1]); libc::exit(0); }
        }
        unsafe { close(pipefd[1]); }
        let mut result: i32 = 0;
        unsafe { read(pipefd[0], &mut result as *mut _ as *mut _, 4); close(pipefd[0]); }
        unsafe { waitpid(pid, std::ptr::null_mut(), 0); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if result == 1 { TestStatus::Pass }
        else { TestStatus::Error("setsid() did not set sid == pid".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// alarm: set a SIGALRM alarm
pub struct AlarmTest;
impl SyscallTest for AlarmTest {
    fn name(&self) -> &str { "proc4_alarm_cancel" }
    fn syscall(&self) -> &str { "alarm" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "alarm(10) then alarm(0) cancels it; previous value should be 10" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let prev1 = unsafe { alarm(10) }; // set 10s alarm, return previous (0)
        let prev2 = unsafe { alarm(0) };  // cancel, return previous (10)
        let dur = start.elapsed().as_micros() as u64;
        let s = if prev1 == 0 && prev2 == 10 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 10, actual_ret: prev2 as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// pause returns EINTR when signal arrives
pub struct PauseWithSignalTest;
impl SyscallTest for PauseWithSignalTest {
    fn name(&self) -> &str { "proc4_pause_eintr" }
    fn syscall(&self) -> &str { "pause" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "pause() interrupted by SIGUSR1 should return -1 EINTR" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        static mut PAUSE_GOT: bool = false;
        extern "C" fn pause_handler(_: c_int) { unsafe { PAUSE_GOT = true; } }
        let sa = sigaction {
            sa_sigaction: pause_handler as usize,
            sa_mask: unsafe { std::mem::zeroed() },
            sa_flags: 0,
            sa_restorer: None,
        };
        unsafe { sigaction(SIGUSR1, &sa, std::ptr::null_mut()); }
        let pid = unsafe { fork() };
        if pid < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            // Child sends SIGUSR1 to parent after 5ms
            unsafe { nanosleep(&timespec { tv_sec: 0, tv_nsec: 5_000_000 }, std::ptr::null_mut()); }
            let ppid = unsafe { getppid() };
            unsafe { kill(ppid, SIGUSR1); libc::exit(0); }
        }
        let ret = unsafe { pause() };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { waitpid(pid, std::ptr::null_mut(), 0); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EINTR as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret as i64, expected_errno: Some(EINTR as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getpgid(0) returns own process group
pub struct GetpgidTest;
impl SyscallTest for GetpgidTest {
    fn name(&self) -> &str { "proc4_getpgid_self" }
    fn syscall(&self) -> &str { "getpgid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getpgid(0) should return own process group ID (positive)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { getpgid(0) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
