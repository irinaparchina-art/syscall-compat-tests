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

/// clock_getres(CLOCK_REALTIME) should return resolution <= 1ms
pub struct ClockGetresRealtimeTest;
impl SyscallTest for ClockGetresRealtimeTest {
    fn name(&self) -> &str { "time2_clock_getres_realtime" }
    fn syscall(&self) -> &str { "clock_getres" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "clock_getres(CLOCK_REALTIME) should return resolution <= 1ms" }
    fn run(&self) -> TestResult {
        let mut ts: timespec = unsafe { std::mem::zeroed() };
        let start = Instant::now();
        let ret = unsafe { clock_getres(CLOCK_REALTIME, &mut ts) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && ts.tv_nsec <= 1_000_000 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// clock_getres(CLOCK_MONOTONIC) resolution check
pub struct ClockGetresMonotonicTest;
impl SyscallTest for ClockGetresMonotonicTest {
    fn name(&self) -> &str { "time2_clock_getres_monotonic" }
    fn syscall(&self) -> &str { "clock_getres" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "clock_getres(CLOCK_MONOTONIC) should succeed with tv_nsec > 0" }
    fn run(&self) -> TestResult {
        let mut ts: timespec = unsafe { std::mem::zeroed() };
        let start = Instant::now();
        let ret = unsafe { clock_getres(CLOCK_MONOTONIC, &mut ts) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && ts.tv_nsec >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// clock_gettime(CLOCK_PROCESS_CPUTIME_ID) should succeed
pub struct ClockGettimeCputimeTest;
impl SyscallTest for ClockGettimeCputimeTest {
    fn name(&self) -> &str { "time2_clock_gettime_cputime" }
    fn syscall(&self) -> &str { "clock_gettime" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "clock_gettime(CLOCK_PROCESS_CPUTIME_ID) should return 0" }
    fn run(&self) -> TestResult {
        let mut ts: timespec = unsafe { std::mem::zeroed() };
        let start = Instant::now();
        let ret = unsafe { clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &mut ts) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// Two consecutive CLOCK_MONOTONIC reads must be non-decreasing
pub struct MonotonicNonDecreasingTest;
impl SyscallTest for MonotonicNonDecreasingTest {
    fn name(&self) -> &str { "time2_monotonic_non_decreasing" }
    fn syscall(&self) -> &str { "clock_gettime" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "Two consecutive CLOCK_MONOTONIC reads must satisfy t2 >= t1" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut t1: timespec = unsafe { std::mem::zeroed() };
        let mut t2: timespec = unsafe { std::mem::zeroed() };
        unsafe { clock_gettime(CLOCK_MONOTONIC, &mut t1); }
        // Do some work
        let mut x = 0u64;
        for i in 0..10000u64 { x = x.wrapping_add(i); }
        let _ = x;
        unsafe { clock_gettime(CLOCK_MONOTONIC, &mut t2); }
        let dur = start.elapsed().as_micros() as u64;
        let ns1 = t1.tv_sec as i128 * 1_000_000_000 + t1.tv_nsec as i128;
        let ns2 = t2.tv_sec as i128 * 1_000_000_000 + t2.tv_nsec as i128;
        let s = if ns2 >= ns1 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: 0, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// setitimer / getitimer: set a virtual timer and read it back
pub struct SetitimerTest;
impl SyscallTest for SetitimerTest {
    fn name(&self) -> &str { "time2_setitimer_getitimer" }
    fn syscall(&self) -> &str { "setitimer" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "setitimer(ITIMER_REAL, 10s) then getitimer should return non-zero interval" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let val = itimerval {
            it_interval: timeval { tv_sec: 0, tv_usec: 0 },
            it_value: timeval { tv_sec: 10, tv_usec: 0 },
        };
        let r1 = unsafe { setitimer(ITIMER_REAL, &val, std::ptr::null_mut()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut out = itimerval { it_interval: timeval { tv_sec: 0, tv_usec: 0 }, it_value: timeval { tv_sec: 0, tv_usec: 0 } };
        let r2 = unsafe { getitimer(ITIMER_REAL, &mut out) };
        // Cancel timer
        let cancel = itimerval { it_interval: timeval { tv_sec: 0, tv_usec: 0 }, it_value: timeval { tv_sec: 0, tv_usec: 0 } };
        unsafe { setitimer(ITIMER_REAL, &cancel, std::ptr::null_mut()) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if r1 == 0 && r2 == 0 && out.it_value.tv_sec > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r1 as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// time() returns seconds since epoch > year 2020
pub struct TimeBasicTest;
impl SyscallTest for TimeBasicTest {
    fn name(&self) -> &str { "time2_time_basic" }
    fn syscall(&self) -> &str { "time" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "time(NULL) should return seconds since epoch > 2020-01-01" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { time(std::ptr::null_mut()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret > 1_577_836_800 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1_577_836_800, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// clock_nanosleep(CLOCK_MONOTONIC, 0, 1ms) should sleep at least ~1ms
pub struct ClockNanosleepTest;
impl SyscallTest for ClockNanosleepTest {
    fn name(&self) -> &str { "time2_clock_nanosleep_1ms" }
    fn syscall(&self) -> &str { "clock_nanosleep" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "clock_nanosleep(CLOCK_MONOTONIC, 0, 1ms) should sleep at least 1ms" }
    fn run(&self) -> TestResult {
        let req = timespec { tv_sec: 0, tv_nsec: 1_000_000 };
        let wall = Instant::now();
        let start = Instant::now();
        let ret = unsafe { clock_nanosleep(CLOCK_MONOTONIC, 0, &req, std::ptr::null_mut()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let elapsed = wall.elapsed().as_micros() as u64;
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && elapsed >= 900 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
