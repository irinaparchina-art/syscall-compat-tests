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

/// mmap a file, verify content matches what was written
pub struct MmapFileTest;
impl SyscallTest for MmapFileTest {
    fn name(&self) -> &str { "mem2_mmap_file" }
    fn syscall(&self) -> &str { "mmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mmap() a file with PROT_READ; mapped content should match file content" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_mmap_file.txt").unwrap();
        let data = b"mmap file test data";
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, data.as_ptr() as *const _, data.len()); }
        let ptr = unsafe { mmap(std::ptr::null_mut(), data.len(), PROT_READ, MAP_PRIVATE, fd, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if ptr == MAP_FAILED {
            unsafe { close(fd); unlink(path.as_ptr()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 1, actual_ret: -1, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        let mapped = unsafe { std::slice::from_raw_parts(ptr as *const u8, data.len()) };
        let ok = mapped == data;
        unsafe { munmap(ptr, data.len()); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ok { TestStatus::Pass }
        else { TestStatus::Error("mapped content mismatch".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// mmap MAP_SHARED + msync: write through mapping, verify file on disk
pub struct MsyncTest;
impl SyscallTest for MsyncTest {
    fn name(&self) -> &str { "mem2_msync_shared" }
    fn syscall(&self) -> &str { "msync" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "msync(MS_SYNC) on MAP_SHARED mapping should flush writes to disk" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_msync.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        // Pre-fill to page size
        let zeros = vec![0u8; 4096];
        unsafe { write(fd, zeros.as_ptr() as *const _, 4096); }
        let ptr = unsafe { mmap(std::ptr::null_mut(), 4096, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0) };
        if ptr == MAP_FAILED {
            unsafe { close(fd); unlink(path.as_ptr()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        // Write through mapping
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, 5) };
        slice.copy_from_slice(b"hello");
        let ret = unsafe { msync(ptr, 4096, MS_SYNC) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { munmap(ptr, 4096); }
        // Verify from disk
        unsafe { lseek(fd, 0, SEEK_SET); }
        let mut buf = [0u8; 5];
        unsafe { read(fd, buf.as_mut_ptr() as *mut _, 5); }
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && &buf == b"hello" { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// madvise(MADV_NORMAL) on a valid mapping should return 0
pub struct MadviseTest;
impl SyscallTest for MadviseTest {
    fn name(&self) -> &str { "mem2_madvise_normal" }
    fn syscall(&self) -> &str { "madvise" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "madvise(MADV_NORMAL) on anon mapping should return 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ptr = unsafe { mmap(std::ptr::null_mut(), 4096, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) };
        if ptr == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("mmap failed".into()), start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { madvise(ptr, 4096, MADV_NORMAL) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { munmap(ptr, 4096); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// mlock / munlock: lock a page in memory
pub struct MlockTest;
impl SyscallTest for MlockTest {
    fn name(&self) -> &str { "mem2_mlock_munlock" }
    fn syscall(&self) -> &str { "mlock" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mlock() then munlock() on an anon mapping should both return 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ptr = unsafe { mmap(std::ptr::null_mut(), 4096, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) };
        if ptr == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("mmap failed".into()), start.elapsed().as_micros() as u64);
        }
        let r1 = unsafe { mlock(ptr, 4096) };
        let e1 = unsafe { *libc::__errno_location() };
        let r2 = unsafe { munlock(ptr, 4096) };
        unsafe { munmap(ptr, 4096); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r1 == 0 && r2 == 0 { TestStatus::Pass }
        else if e1 == ENOSYS as i32 || e1 == EPERM as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r1 as i64, expected_errno: None, actual_errno: Some(e1) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// mmap with offset: map second page of a file
pub struct MmapOffsetTest;
impl SyscallTest for MmapOffsetTest {
    fn name(&self) -> &str { "mem2_mmap_offset" }
    fn syscall(&self) -> &str { "mmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mmap() with non-zero page-aligned offset should map correct file region" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_mmap_offset.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        // Write 2 pages: first with 'A', second with 'B'
        let page1 = vec![b'A'; 4096];
        let page2 = vec![b'B'; 4096];
        unsafe { write(fd, page1.as_ptr() as *const _, 4096); write(fd, page2.as_ptr() as *const _, 4096); }
        // Map second page
        let ptr = unsafe { mmap(std::ptr::null_mut(), 4096, PROT_READ, MAP_PRIVATE, fd, 4096) };
        let errno_val = unsafe { *libc::__errno_location() };
        if ptr == MAP_FAILED {
            unsafe { close(fd); unlink(path.as_ptr()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 1, actual_ret: -1, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        let first_byte = unsafe { *(ptr as *const u8) };
        unsafe { munmap(ptr, 4096); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if first_byte == b'B' { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: b'B' as i64, actual_ret: first_byte as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// mremap: resize an existing mapping
pub struct MremapTest;
impl SyscallTest for MremapTest {
    fn name(&self) -> &str { "mem2_mremap_grow" }
    fn syscall(&self) -> &str { "mremap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mremap() to grow an anon mapping should succeed and preserve data" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ptr = unsafe { mmap(std::ptr::null_mut(), 4096, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) };
        if ptr == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("mmap failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { *(ptr as *mut u8) = 0xAB; }
        let new_ptr = unsafe { mremap(ptr, 4096, 8192, MREMAP_MAYMOVE) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        if new_ptr == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 1, actual_ret: -1, expected_errno: None, actual_errno: Some(errno_val) } }, dur);
        }
        let val = unsafe { *(new_ptr as *const u8) };
        unsafe { munmap(new_ptr, 8192); }
        let s = if val == 0xAB { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0xAB, actual_ret: val as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
