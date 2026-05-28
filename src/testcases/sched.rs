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

// ─── sched_yield ────────────────────────────────────────────────────────────

pub struct SchedYieldTest;
impl SyscallTest for SchedYieldTest {
    fn name(&self) -> &str { "sched_yield_basic" }
    fn syscall(&self) -> &str { "sched_yield" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str {
        "sched_yield() should return 0 on success"
    }
    fn run(&self) -> TestResult {
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { sched_yield() as i64 }
        });
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

// ─── sched_getaffinity ──────────────────────────────────────────────────────

pub struct SchedGetaffinityTest;
impl SyscallTest for SchedGetaffinityTest {
    fn name(&self) -> &str { "sched_getaffinity_self" }
    fn syscall(&self) -> &str { "sched_getaffinity" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str {
        "sched_getaffinity(0) for self should return 0 with at least one CPU set"
    }
    fn run(&self) -> TestResult {
        let mut cpuset: cpu_set_t = unsafe { std::mem::zeroed() };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { sched_getaffinity(0, std::mem::size_of::<cpu_set_t>(), &mut cpuset) as i64 }
        });
        let status = if ret == 0 {
            let any_set = (0..CPU_SETSIZE as usize).any(|i| unsafe { CPU_ISSET(i, &cpuset) });
            if any_set {
                TestStatus::Pass
            } else {
                TestStatus::Fail {
                    expected_ret: 0,
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

pub struct SchedGetaffinityBadPidTest;
impl SyscallTest for SchedGetaffinityBadPidTest {
    fn name(&self) -> &str { "sched_getaffinity_bad_pid" }
    fn syscall(&self) -> &str { "sched_getaffinity" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str {
        "sched_getaffinity with non-existent PID should return ESRCH"
    }
    fn run(&self) -> TestResult {
        let mut cpuset: cpu_set_t = unsafe { std::mem::zeroed() };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { sched_getaffinity(999999, std::mem::size_of::<cpu_set_t>(), &mut cpuset) as i64 }
        });
        let status = if ret == -1 && errno_val == ESRCH {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1,
                actual_ret: ret,
                expected_errno: Some(ESRCH),
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── sched_setaffinity ──────────────────────────────────────────────────────

pub struct SchedSetaffinitySelfTest;
impl SyscallTest for SchedSetaffinitySelfTest {
    fn name(&self) -> &str { "sched_setaffinity_self" }
    fn syscall(&self) -> &str { "sched_setaffinity" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str {
        "sched_setaffinity(0) to CPU 0 should succeed"
    }
    fn run(&self) -> TestResult {
        let mut cpuset: cpu_set_t = unsafe { std::mem::zeroed() };
        unsafe { CPU_SET(0, &mut cpuset) };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { sched_setaffinity(0, std::mem::size_of::<cpu_set_t>(), &cpuset) as i64 }
        });
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

// ─── sched_getscheduler / sched_getparam ────────────────────────────────────

pub struct SchedGetschedulerTest;
impl SyscallTest for SchedGetschedulerTest {
    fn name(&self) -> &str { "sched_getscheduler_self" }
    fn syscall(&self) -> &str { "sched_getscheduler" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str {
        "sched_getscheduler(0) should return a valid scheduling policy"
    }
    fn run(&self) -> TestResult {
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { sched_getscheduler(0) as i64 }
        });
        let status = if ret >= 0 && (ret == SCHED_OTHER as i64 || ret == SCHED_FIFO as i64
            || ret == SCHED_RR as i64 || ret == SCHED_BATCH as i64)
        {
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

pub struct SchedGetparamTest;
impl SyscallTest for SchedGetparamTest {
    fn name(&self) -> &str { "sched_getparam_self" }
    fn syscall(&self) -> &str { "sched_getparam" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str {
        "sched_getparam(0) should return 0 with sched_priority filled"
    }
    fn run(&self) -> TestResult {
        let mut param: sched_param = unsafe { std::mem::zeroed() };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { sched_getparam(0, &mut param) as i64 }
        });
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
