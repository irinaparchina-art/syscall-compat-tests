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

// io_uring syscall numbers
const SYS_IO_URING_SETUP: i64 = 425;
const SYS_IO_URING_ENTER: i64 = 426;
const SYS_IO_URING_REGISTER: i64 = 427;

// io_uring setup parameters
#[repr(C)]
struct IoUringParams {
    sq_entries: u32,
    cq_entries: u32,
    flags: u32,
    sq_thread_cpu: u32,
    sq_thread_idle: u32,
    features: u32,
    wq_fd: u32,
    resv: [u32; 3],
    sq_off: SqRingOffsets,
    cq_off: CqRingOffsets,
}

#[repr(C)]
struct SqRingOffsets {
    head: u32, tail: u32, ring_mask: u32, ring_entries: u32,
    flags: u32, dropped: u32, array: u32, resv1: u32, resv2: u64,
}

#[repr(C)]
struct CqRingOffsets {
    head: u32, tail: u32, ring_mask: u32, ring_entries: u32,
    overflow: u32, cqes: u32, flags: u32, resv1: u32, resv2: u64,
}

impl Default for IoUringParams {
    fn default() -> Self { unsafe { std::mem::zeroed() } }
}

/// io_uring_setup: create an io_uring instance
pub struct IoUringSetupTest;
impl SyscallTest for IoUringSetupTest {
    fn name(&self) -> &str { "iou_setup_basic" }
    fn syscall(&self) -> &str { "io_uring_setup" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "io_uring_setup(8, params) should return a valid ring fd" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut params = IoUringParams::default();
        let fd = unsafe {
            libc::syscall(SYS_IO_URING_SETUP, 8 as c_long, &mut params as *mut _ as c_long)
        };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd as i32); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 || errno_val == EPERM as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// io_uring_setup with 0 entries should fail EINVAL
pub struct IoUringSetupZeroEntriesTest;
impl SyscallTest for IoUringSetupZeroEntriesTest {
    fn name(&self) -> &str { "iou_setup_zero_entries" }
    fn syscall(&self) -> &str { "io_uring_setup" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "io_uring_setup(0, params) should return -1 EINVAL" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut params = IoUringParams::default();
        let fd = unsafe {
            libc::syscall(SYS_IO_URING_SETUP, 0 as c_long, &mut params as *mut _ as c_long)
        };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd as i32); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd == -1 && errno_val == EINVAL as i32 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd, expected_errno: Some(EINVAL as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
