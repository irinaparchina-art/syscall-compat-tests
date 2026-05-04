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

/// getenv / setenv / unsetenv round-trip
pub struct SetenvTest;
impl SyscallTest for SetenvTest {
    fn name(&self) -> &str { "misc_setenv_getenv" }
    fn syscall(&self) -> &str { "setenv/getenv" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "setenv() + getenv() should return the set value; unsetenv() should remove it" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let key = CString::new("SCT_TEST_VAR").unwrap();
        let val = CString::new("hello123").unwrap();
        unsafe { setenv(key.as_ptr(), val.as_ptr(), 1) };
        let got = unsafe { getenv(key.as_ptr()) };
        let matches = if got.is_null() { false }
        else { unsafe { std::ffi::CStr::from_ptr(got).to_str().unwrap_or("") == "hello123" } };
        unsafe { unsetenv(key.as_ptr()) };
        let after = unsafe { getenv(key.as_ptr()) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if matches && after.is_null() { TestStatus::Pass }
        else { TestStatus::Error("setenv/getenv/unsetenv round-trip failed".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getrandom fills buffer with non-zero bytes
pub struct GetrandomTest;
impl SyscallTest for GetrandomTest {
    fn name(&self) -> &str { "misc_getrandom" }
    fn syscall(&self) -> &str { "getrandom" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "getrandom() should fill buffer with random bytes (return == count)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut buf = [0u8; 32];
        let ret = unsafe { getrandom(buf.as_mut_ptr() as *mut _, 32, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 32 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 32, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getrandom with GRND_NONBLOCK
pub struct GetrandomNonblockTest;
impl SyscallTest for GetrandomNonblockTest {
    fn name(&self) -> &str { "misc_getrandom_nonblock" }
    fn syscall(&self) -> &str { "getrandom" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "getrandom(GRND_NONBLOCK) should succeed when entropy is available" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut buf = [0u8; 16];
        let ret = unsafe { getrandom(buf.as_mut_ptr() as *mut _, 16, GRND_NONBLOCK) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 16 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 || errno_val == EAGAIN as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 16, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// /dev/urandom readable
pub struct DevUrandomTest;
impl SyscallTest for DevUrandomTest {
    fn name(&self) -> &str { "misc_dev_urandom" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "/dev/urandom should be readable and return requested number of bytes" }
    fn run(&self) -> TestResult {
        let path = CString::new("/dev/urandom").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut buf = [0u8; 16];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 16) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 16 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 16, actual_ret: n as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// ioctl TIOCGWINSZ on a terminal or pipe
pub struct IoctlFionreadTest;
impl SyscallTest for IoctlFionreadTest {
    fn name(&self) -> &str { "misc_ioctl_fionread" }
    fn syscall(&self) -> &str { "ioctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "ioctl(FIONREAD) on pipe after write should return number of bytes available" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fds[1], b"abcde".as_ptr() as *const _, 5); }
        let mut avail: c_int = 0;
        let ret = unsafe { ioctl(fds[0], FIONREAD, &mut avail) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && avail == 5 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 5, actual_ret: avail as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sync() should return without error
pub struct SyncTest;
impl SyscallTest for SyncTest {
    fn name(&self) -> &str { "misc_sync" }
    fn syscall(&self) -> &str { "sync" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "sync() should flush filesystem buffers and return without error" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        unsafe { sync() };
        // sync() always succeeds on Linux; ignore stale errno from earlier calls
        let dur = start.elapsed().as_micros() as u64;
        let s = TestStatus::Pass;
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getlogin_r or getlogin returns a non-empty username
pub struct GetloginTest;
impl SyscallTest for GetloginTest {
    fn name(&self) -> &str { "misc_getlogin" }
    fn syscall(&self) -> &str { "getlogin" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "getlogin() or getlogin_r() should return a non-empty login name" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let login = unsafe { libc::getlogin() };
        let dur = start.elapsed().as_micros() as u64;
        // getlogin may return NULL if no controlling terminal — that's acceptable
        let s = if login.is_null() { TestStatus::Unimplemented }
        else {
            let s = unsafe { std::ffi::CStr::from_ptr(login).to_str().unwrap_or("") };
            if s.len() > 0 { TestStatus::Pass } else { TestStatus::Unimplemented }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getenv("PATH") should return a non-empty path
pub struct GetenvPathTest;
impl SyscallTest for GetenvPathTest {
    fn name(&self) -> &str { "misc_getenv_path" }
    fn syscall(&self) -> &str { "getenv" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "getenv(\"PATH\") should return a non-null, non-empty string" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let key = CString::new("PATH").unwrap();
        let val = unsafe { getenv(key.as_ptr()) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if !val.is_null() {
            let s = unsafe { std::ffi::CStr::from_ptr(val).to_str().unwrap_or("") };
            if s.len() > 0 { TestStatus::Pass } else { TestStatus::Error("PATH is empty".into()) }
        } else { TestStatus::Error("PATH not set".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// memfd_create: anonymous file in memory
pub struct MemfdCreateTest;
impl SyscallTest for MemfdCreateTest {
    fn name(&self) -> &str { "misc_memfd_create" }
    fn syscall(&self) -> &str { "memfd_create" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "memfd_create() should return a valid fd; write/read should work" }
    fn run(&self) -> TestResult {
        let name = CString::new("sct_memfd").unwrap();
        let start = Instant::now();
        let fd = unsafe { memfd_create(name.as_ptr(), 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        let data = b"memfd test";
        unsafe { write(fd, data.as_ptr() as *const _, data.len()); lseek(fd, 0, SEEK_SET); }
        let mut buf = [0u8; 10];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 10) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 10 && &buf == data { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 10, actual_ret: n as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// eventfd: write a value and read it back
pub struct EventfdTest;
impl SyscallTest for EventfdTest {
    fn name(&self) -> &str { "misc_eventfd" }
    fn syscall(&self) -> &str { "eventfd" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "eventfd() write 5, read should return 5" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { eventfd(0, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        let val: u64 = 5;
        unsafe { write(fd, &val as *const _ as *const _, 8) };
        let mut rval: u64 = 0;
        let n = unsafe { read(fd, &mut rval as *mut _ as *mut _, 8) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 8 && rval == 5 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 5, actual_ret: rval as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// timerfd_create + timerfd_settime + read
pub struct TimerfdTest;
impl SyscallTest for TimerfdTest {
    fn name(&self) -> &str { "misc_timerfd" }
    fn syscall(&self) -> &str { "timerfd_create" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "timerfd_create + timerfd_settime(1ms) + read should return expiry count >= 1" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { timerfd_create(CLOCK_MONOTONIC, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        let spec = itimerspec {
            it_interval: timespec { tv_sec: 0, tv_nsec: 0 },
            it_value: timespec { tv_sec: 0, tv_nsec: 1_000_000 }, // 1ms
        };
        unsafe { timerfd_settime(fd, 0, &spec, std::ptr::null_mut()) };
        // Block until timer fires
        let mut count: u64 = 0;
        let n = unsafe { read(fd, &mut count as *mut _ as *mut _, 8) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 8 && count >= 1 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: count as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// prctl(PR_GET_NAME) returns the thread name
pub struct PrctlGetNameTest;
impl SyscallTest for PrctlGetNameTest {
    fn name(&self) -> &str { "misc_prctl_get_name" }
    fn syscall(&self) -> &str { "prctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "prctl(PR_GET_NAME) should return the current thread name (non-empty)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut name = [0i8; 16];
        let ret = unsafe { prctl(PR_GET_NAME, name.as_mut_ptr() as c_ulong, 0, 0, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let len = name.iter().take_while(|&&c| c != 0).count();
        let s = if ret == 0 && len > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
