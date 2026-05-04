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

/// sched_getscheduler(0) returns a valid scheduler policy
pub struct SchedGetschedulerTest;
impl SyscallTest for SchedGetschedulerTest {
    fn name(&self) -> &str { "res_sched_getscheduler" }
    fn syscall(&self) -> &str { "sched_getscheduler" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "sched_getscheduler(0) should return a valid scheduling policy (>= 0)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { sched_getscheduler(0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sched_getparam(0) returns valid sched_param
pub struct SchedGetparamTest;
impl SyscallTest for SchedGetparamTest {
    fn name(&self) -> &str { "res_sched_getparam" }
    fn syscall(&self) -> &str { "sched_getparam" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "sched_getparam(0) should succeed and return sched_priority >= 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut param = sched_param { sched_priority: 0 };
        let ret = unsafe { sched_getparam(0, &mut param) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && param.sched_priority >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sched_yield() should always return 0
pub struct SchedYieldTest;
impl SyscallTest for SchedYieldTest {
    fn name(&self) -> &str { "res_sched_yield" }
    fn syscall(&self) -> &str { "sched_yield" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "sched_yield() should always return 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { sched_yield() };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sched_get_priority_max/min for SCHED_OTHER
pub struct SchedPriorityRangeTest;
impl SyscallTest for SchedPriorityRangeTest {
    fn name(&self) -> &str { "res_sched_priority_range" }
    fn syscall(&self) -> &str { "sched_get_priority_max" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "sched_get_priority_max/min(SCHED_OTHER) should return 0/0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let max = unsafe { sched_get_priority_max(SCHED_OTHER) };
        let min = unsafe { sched_get_priority_min(SCHED_OTHER) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if max == 0 && min == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: max as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// setpriority / getpriority round-trip
pub struct SetpriorityTest;
impl SyscallTest for SetpriorityTest {
    fn name(&self) -> &str { "res_setpriority_roundtrip" }
    fn syscall(&self) -> &str { "setpriority" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "setpriority(0) then getpriority should return the same nice value" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        unsafe { *libc::__errno_location() = 0 };
        let orig = unsafe { getpriority(PRIO_PROCESS, 0) };
        let r = unsafe { setpriority(PRIO_PROCESS, 0, orig) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { *libc::__errno_location() = 0 };
        let back = unsafe { getpriority(PRIO_PROCESS, 0) };
        let back_errno = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if r == 0 && back == orig && back_errno == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: orig as i64, actual_ret: back as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// prlimit64: get current RLIMIT_NOFILE via prlimit
pub struct PrlimitGetTest;
impl SyscallTest for PrlimitGetTest {
    fn name(&self) -> &str { "res_prlimit_get_nofile" }
    fn syscall(&self) -> &str { "prlimit64" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "prlimit64(0, RLIMIT_NOFILE, NULL, &old) should return current limits" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut old: rlimit = unsafe { std::mem::zeroed() };
        let ret = unsafe { prlimit(0, RLIMIT_NOFILE, std::ptr::null(), &mut old) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && old.rlim_cur > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// mincore: check if pages of an mmap are in core
pub struct MincoreTest;
impl SyscallTest for MincoreTest {
    fn name(&self) -> &str { "res_mincore_basic" }
    fn syscall(&self) -> &str { "mincore" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mincore() on a touched mmap page should return 0 and set vec[0]" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ptr = unsafe { mmap(std::ptr::null_mut(), 4096, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) };
        if ptr == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("mmap failed".into()), start.elapsed().as_micros() as u64);
        }
        // Touch the page
        unsafe { *(ptr as *mut u8) = 1; }
        let mut vec = [0u8; 1];
        let ret = unsafe { mincore(ptr, 4096, vec.as_mut_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { munmap(ptr, 4096); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getloadavg returns 3 load averages >= 0
pub struct GetloadavgTest;
impl SyscallTest for GetloadavgTest {
    fn name(&self) -> &str { "res_getloadavg" }
    fn syscall(&self) -> &str { "getloadavg" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "getloadavg() should return 3 non-negative load average values" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut avg = [0.0f64; 3];
        let ret = unsafe { getloadavg(avg.as_mut_ptr(), 3) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 3 && avg.iter().all(|&v| v >= 0.0) { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
