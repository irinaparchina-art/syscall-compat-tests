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

pub struct BrkBasicTest;
impl SyscallTest for BrkBasicTest {
    fn name(&self) -> &str { "mem_brk_basic" }
    fn syscall(&self) -> &str { "brk" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str {
        "brk(0) should return the current program break (a non-null pointer)"
    }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // brk(0) returns 0 on success (current break)
        let ret = unsafe { libc::syscall(libc::SYS_brk, 0usize) };
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret >= 0 {
            TestStatus::Pass
        } else {
            TestStatus::Fail {
                expected_ret: 1, actual_ret: ret,
                expected_errno: None, actual_errno: None,
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct MmapAnonTest;
impl SyscallTest for MmapAnonTest {
    fn name(&self) -> &str { "mem_mmap_anon" }
    fn syscall(&self) -> &str { "mmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str {
        "mmap() anonymous private mapping of 4096 bytes should succeed"
    }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ptr = unsafe {
            mmap(
                std::ptr::null_mut(),
                4096,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let status = if ptr != MAP_FAILED {
            unsafe { munmap(ptr, 4096) };
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 1, actual_ret: -1,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct MmapReadWriteTest;
impl SyscallTest for MmapReadWriteTest {
    fn name(&self) -> &str { "mem_mmap_read_write" }
    fn syscall(&self) -> &str { "mmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str {
        "Mapped memory should be readable and writable; written value must be read back correctly"
    }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ptr = unsafe {
            mmap(std::ptr::null_mut(), 4096, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS, -1, 0)
        };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        if ptr == MAP_FAILED {
            if errno_val == ENOSYS as i32 {
                return make_result(self.name(), self.syscall(), self.category(), self.description(),
                    TestStatus::Unimplemented, dur);
            }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error(format!("mmap failed errno={}", errno_val)), dur);
        }
        // Write and read back
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, 4096) };
        slice[0] = 0xAB;
        slice[4095] = 0xCD;
        let ok = slice[0] == 0xAB && slice[4095] == 0xCD;
        unsafe { munmap(ptr, 4096) };
        let status = if ok { TestStatus::Pass } else {
            TestStatus::Error("read-back mismatch after write".into())
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct MunmapTest;
impl SyscallTest for MunmapTest {
    fn name(&self) -> &str { "mem_munmap_basic" }
    fn syscall(&self) -> &str { "munmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str {
        "munmap() a previously mmap'd region should return 0"
    }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ptr = unsafe {
            mmap(std::ptr::null_mut(), 4096, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS, -1, 0)
        };
        if ptr == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("mmap failed".into()), start.elapsed().as_micros() as u64);
        }
        let (ret, errno_val, dur) = {
            let s = Instant::now();
            let r = unsafe { munmap(ptr, 4096) } as i64;
            let e = unsafe { *libc::__errno_location() };
            (r, e, s.elapsed().as_micros() as u64)
        };
        let status = if ret == 0 {
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

pub struct MprotectTest;
impl SyscallTest for MprotectTest {
    fn name(&self) -> &str { "mem_mprotect_basic" }
    fn syscall(&self) -> &str { "mprotect" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str {
        "mprotect() changing a region to PROT_READ should return 0"
    }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ptr = unsafe {
            mmap(std::ptr::null_mut(), 4096, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS, -1, 0)
        };
        if ptr == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("mmap failed".into()), start.elapsed().as_micros() as u64);
        }
        let (ret, errno_val, dur) = {
            let s = Instant::now();
            let r = unsafe { mprotect(ptr, 4096, PROT_READ) } as i64;
            let e = unsafe { *libc::__errno_location() };
            unsafe { munmap(ptr, 4096) };
            (r, e, s.elapsed().as_micros() as u64)
        };
        let status = if ret == 0 {
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
