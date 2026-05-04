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

pub struct ClockGettimeRealtimeTest;
impl SyscallTest for ClockGettimeRealtimeTest {
    fn name(&self) -> &str { "time_clock_gettime_realtime" }
    fn syscall(&self) -> &str { "clock_gettime" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str {
        "clock_gettime(CLOCK_REALTIME) should succeed and return a timestamp after year 2020"
    }
    fn run(&self) -> TestResult {
        let mut ts: timespec = unsafe { std::mem::zeroed() };
        let (ret, errno_val, dur) = {
            let s = Instant::now();
            let r = unsafe { clock_gettime(CLOCK_REALTIME, &mut ts) } as i64;
            let e = unsafe { *libc::__errno_location() };
            (r, e, s.elapsed().as_micros() as u64)
        };
        // Jan 1 2020 in unix time
        let status = if ret == 0 && ts.tv_sec > 1_577_836_800 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0, actual_ret: ret,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct ClockGettimeMonotonicTest;
impl SyscallTest for ClockGettimeMonotonicTest {
    fn name(&self) -> &str { "time_clock_gettime_monotonic" }
    fn syscall(&self) -> &str { "clock_gettime" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str {
        "clock_gettime(CLOCK_MONOTONIC) should succeed and return a non-negative value"
    }
    fn run(&self) -> TestResult {
        let mut ts: timespec = unsafe { std::mem::zeroed() };
        let (ret, errno_val, dur) = {
            let s = Instant::now();
            let r = unsafe { clock_gettime(CLOCK_MONOTONIC, &mut ts) } as i64;
            let e = unsafe { *libc::__errno_location() };
            (r, e, s.elapsed().as_micros() as u64)
        };
        let status = if ret == 0 && ts.tv_sec >= 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0, actual_ret: ret,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct GettimeofdayTest;
impl SyscallTest for GettimeofdayTest {
    fn name(&self) -> &str { "time_gettimeofday" }
    fn syscall(&self) -> &str { "gettimeofday" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str {
        "gettimeofday() should return 0 and fill tv_sec with a value after year 2020"
    }
    fn run(&self) -> TestResult {
        let mut tv: timeval = unsafe { std::mem::zeroed() };
        let (ret, errno_val, dur) = {
            let s = Instant::now();
            let r = unsafe { gettimeofday(&mut tv, std::ptr::null_mut()) } as i64;
            let e = unsafe { *libc::__errno_location() };
            (r, e, s.elapsed().as_micros() as u64)
        };
        let status = if ret == 0 && tv.tv_sec > 1_577_836_800 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0, actual_ret: ret,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct NanosleepBasicTest;
impl SyscallTest for NanosleepBasicTest {
    fn name(&self) -> &str { "time_nanosleep_basic" }
    fn syscall(&self) -> &str { "nanosleep" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str {
        "nanosleep() for 1ms should return 0 and actually take >= 1ms"
    }
    fn run(&self) -> TestResult {
        let req = timespec { tv_sec: 0, tv_nsec: 1_000_000 }; // 1ms
        let mut rem: timespec = unsafe { std::mem::zeroed() };
        let wall_start = Instant::now();
        let (ret, errno_val, dur) = {
            let s = Instant::now();
            let r = unsafe { nanosleep(&req, &mut rem) } as i64;
            let e = unsafe { *libc::__errno_location() };
            (r, e, s.elapsed().as_micros() as u64)
        };
        let elapsed_us = wall_start.elapsed().as_micros() as u64;
        let status = if ret == 0 && elapsed_us >= 900 {
            // at least ~1ms elapsed
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0, actual_ret: ret,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct NanosleepInvalidTest;
impl SyscallTest for NanosleepInvalidTest {
    fn name(&self) -> &str { "time_nanosleep_invalid" }
    fn syscall(&self) -> &str { "nanosleep" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str {
        "nanosleep() with tv_nsec >= 1e9 should return -1 with EINVAL"
    }
    fn run(&self) -> TestResult {
        let req = timespec { tv_sec: 0, tv_nsec: 2_000_000_000 }; // invalid
        let mut rem: timespec = unsafe { std::mem::zeroed() };
        let (ret, errno_val, dur) = {
            let s = Instant::now();
            let r = unsafe { nanosleep(&req, &mut rem) } as i64;
            let e = unsafe { *libc::__errno_location() };
            (r, e, s.elapsed().as_micros() as u64)
        };
        let status = if ret == -1 && errno_val == EINVAL as i32 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1, actual_ret: ret,
                expected_errno: Some(EINVAL as i32), actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}
