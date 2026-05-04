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

/// clock_gettime CLOCK_THREAD_CPUTIME_ID
pub struct ClockThreadCputimeTest;
impl SyscallTest for ClockThreadCputimeTest {
    fn name(&self) -> &str { "time3_clock_thread_cputime" }
    fn syscall(&self) -> &str { "clock_gettime" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "clock_gettime(CLOCK_THREAD_CPUTIME_ID) should return 0 and non-negative value" }
    fn run(&self) -> TestResult {
        let mut ts: timespec = unsafe { std::mem::zeroed() };
        let start = Instant::now();
        let ret = unsafe { clock_gettime(CLOCK_THREAD_CPUTIME_ID, &mut ts) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && ts.tv_sec >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// CLOCK_MONOTONIC_RAW: hardware monotonic clock (no NTP adjustment)
pub struct ClockMonotonicRawTest;
impl SyscallTest for ClockMonotonicRawTest {
    fn name(&self) -> &str { "time3_clock_monotonic_raw" }
    fn syscall(&self) -> &str { "clock_gettime" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "clock_gettime(CLOCK_MONOTONIC_RAW) should return non-negative value" }
    fn run(&self) -> TestResult {
        let mut ts: timespec = unsafe { std::mem::zeroed() };
        let start = Instant::now();
        let ret = unsafe { clock_gettime(CLOCK_MONOTONIC_RAW, &mut ts) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && ts.tv_sec >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// CLOCK_BOOTTIME: time since boot including suspend time
pub struct ClockBoottimeTest;
impl SyscallTest for ClockBoottimeTest {
    fn name(&self) -> &str { "time3_clock_boottime" }
    fn syscall(&self) -> &str { "clock_gettime" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "clock_gettime(CLOCK_BOOTTIME) should return uptime >= 0" }
    fn run(&self) -> TestResult {
        let mut ts: timespec = unsafe { std::mem::zeroed() };
        let start = Instant::now();
        let ret = unsafe { clock_gettime(CLOCK_BOOTTIME, &mut ts) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && ts.tv_sec >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// timerfd with CLOCK_REALTIME
pub struct TimerfdRealtimeTest;
impl SyscallTest for TimerfdRealtimeTest {
    fn name(&self) -> &str { "time3_timerfd_realtime" }
    fn syscall(&self) -> &str { "timerfd_create" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "timerfd_create(CLOCK_REALTIME) + settime(1ms) + read should return expiry >= 1" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { timerfd_create(CLOCK_REALTIME, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        let spec = itimerspec {
            it_interval: timespec { tv_sec: 0, tv_nsec: 0 },
            it_value: timespec { tv_sec: 0, tv_nsec: 2_000_000 },
        };
        unsafe { timerfd_settime(fd, 0, &spec, std::ptr::null_mut()) };
        let mut count: u64 = 0;
        let n = unsafe { read(fd, &mut count as *mut _ as *mut _, 8) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 8 && count >= 1 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: count as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// timerfd_gettime: verify timer value after settime
pub struct TimerfdGettimeTest;
impl SyscallTest for TimerfdGettimeTest {
    fn name(&self) -> &str { "time3_timerfd_gettime" }
    fn syscall(&self) -> &str { "timerfd_gettime" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "timerfd_gettime() after settime should return non-zero remaining value" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { timerfd_create(CLOCK_MONOTONIC, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let spec = itimerspec {
            it_interval: timespec { tv_sec: 0, tv_nsec: 0 },
            it_value: timespec { tv_sec: 60, tv_nsec: 0 }, // 60s future
        };
        unsafe { timerfd_settime(fd, 0, &spec, std::ptr::null_mut()) };
        let mut cur = itimerspec { it_interval: timespec { tv_sec: 0, tv_nsec: 0 }, it_value: timespec { tv_sec: 0, tv_nsec: 0 } };
        let ret = unsafe { timerfd_gettime(fd, &mut cur) };
        let errno_val = unsafe { *libc::__errno_location() };
        // Cancel timer
        let cancel = itimerspec { it_interval: timespec { tv_sec: 0, tv_nsec: 0 }, it_value: timespec { tv_sec: 0, tv_nsec: 0 } };
        unsafe { timerfd_settime(fd, 0, &cancel, std::ptr::null_mut()); close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && cur.it_value.tv_sec > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// Repeated nanosleep: multiple 1ms sleeps accumulate correctly
pub struct NanosleepAccumulateTest;
impl SyscallTest for NanosleepAccumulateTest {
    fn name(&self) -> &str { "time3_nanosleep_accumulate" }
    fn syscall(&self) -> &str { "nanosleep" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "5 x 1ms nanosleep calls should total >= 5ms elapsed wall time" }
    fn run(&self) -> TestResult {
        let req = timespec { tv_sec: 0, tv_nsec: 1_000_000 };
        let wall = Instant::now();
        let start = Instant::now();
        for _ in 0..5 {
            unsafe { nanosleep(&req, std::ptr::null_mut()); }
        }
        let elapsed_us = wall.elapsed().as_micros() as u64;
        let dur = start.elapsed().as_micros() as u64;
        let s = if elapsed_us >= 4500 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 5000, actual_ret: elapsed_us as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// clock_nanosleep with absolute time (TIMER_ABSTIME)
pub struct ClockNanosleepAbsTest;
impl SyscallTest for ClockNanosleepAbsTest {
    fn name(&self) -> &str { "time3_clock_nanosleep_abstime" }
    fn syscall(&self) -> &str { "clock_nanosleep" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "clock_nanosleep(CLOCK_MONOTONIC, TIMER_ABSTIME, now+1ms) should sleep ~1ms" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut now: timespec = unsafe { std::mem::zeroed() };
        unsafe { clock_gettime(CLOCK_MONOTONIC, &mut now) };
        // Absolute time = now + 1ms
        let abs_time = timespec {
            tv_sec: now.tv_sec,
            tv_nsec: now.tv_nsec + 1_000_000,
        };
        let wall = Instant::now();
        let ret = unsafe { clock_nanosleep(CLOCK_MONOTONIC, TIMER_ABSTIME, &abs_time, std::ptr::null_mut()) };
        let elapsed = wall.elapsed().as_micros() as u64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && elapsed >= 500 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// ITIMER_VIRTUAL: user-space CPU timer
pub struct ItimerVirtualTest;
impl SyscallTest for ItimerVirtualTest {
    fn name(&self) -> &str { "time3_itimer_virtual" }
    fn syscall(&self) -> &str { "setitimer" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "setitimer(ITIMER_VIRTUAL, 1s) then getitimer should show non-zero value" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let val = itimerval {
            it_interval: timeval { tv_sec: 0, tv_usec: 0 },
            it_value: timeval { tv_sec: 1, tv_usec: 0 },
        };
        let r1 = unsafe { setitimer(ITIMER_VIRTUAL, &val, std::ptr::null_mut()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut out = itimerval { it_interval: timeval { tv_sec: 0, tv_usec: 0 }, it_value: timeval { tv_sec: 0, tv_usec: 0 } };
        unsafe { getitimer(ITIMER_VIRTUAL, &mut out) };
        // Cancel
        let cancel = itimerval { it_interval: timeval { tv_sec: 0, tv_usec: 0 }, it_value: timeval { tv_sec: 0, tv_usec: 0 } };
        unsafe { setitimer(ITIMER_VIRTUAL, &cancel, std::ptr::null_mut()) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if r1 == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r1 as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
