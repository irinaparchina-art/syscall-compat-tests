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

/// mmap with negative offset should fail EINVAL
pub struct MmapNegativeOffsetTest;
impl SyscallTest for MmapNegativeOffsetTest {
    fn name(&self) -> &str { "err2_mmap_negative_offset" }
    fn syscall(&self) -> &str { "mmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mmap() with non-page-aligned offset should return MAP_FAILED EINVAL" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_mmap_off.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let zeros = vec![0u8; 8192];
        unsafe { write(fd, zeros.as_ptr() as *const _, 8192); }
        // offset 1 is not page-aligned
        let ptr = unsafe { mmap(std::ptr::null_mut(), 4096, PROT_READ, MAP_PRIVATE, fd, 1) };
        let errno_val = unsafe { *libc::__errno_location() };
        if ptr != MAP_FAILED { unsafe { munmap(ptr, 4096); } }
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ptr == MAP_FAILED && errno_val == EINVAL as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ptr as i64, expected_errno: Some(EINVAL as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// munmap on address not from mmap should fail EINVAL
pub struct MunmapInvalidAddrTest;
impl SyscallTest for MunmapInvalidAddrTest {
    fn name(&self) -> &str { "err2_munmap_stack_addr" }
    fn syscall(&self) -> &str { "munmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "munmap() with length=0 should return -1 EINVAL" }
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
        // length=0 is EINVAL
        let ret = unsafe { munmap(ptr, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { munmap(ptr, 4096); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EINVAL as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret as i64, expected_errno: Some(EINVAL as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// waitpid for nonexistent PID should give ECHILD
pub struct WaitpidEchildTest;
impl SyscallTest for WaitpidEchildTest {
    fn name(&self) -> &str { "err2_waitpid_echild" }
    fn syscall(&self) -> &str { "waitpid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "waitpid(99999) on non-existent PID should return -1 ECHILD" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut status = 0i32;
        let ret = unsafe { waitpid(99999, &mut status, 0) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == ECHILD as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(ECHILD as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// mkdir with existing path should give EEXIST
pub struct MkdirEexistTest;
impl SyscallTest for MkdirEexistTest {
    fn name(&self) -> &str { "err2_mkdir_eexist" }
    fn syscall(&self) -> &str { "mkdir" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "mkdir() on existing directory should return -1 EEXIST" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp").unwrap();
        let start = Instant::now();
        let ret = unsafe { mkdir(path.as_ptr(), 0o755) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EEXIST as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret as i64, expected_errno: Some(EEXIST as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// rename src to itself should succeed (no-op)
pub struct RenameSelfTest;
impl SyscallTest for RenameSelfTest {
    fn name(&self) -> &str { "err2_rename_self" }
    fn syscall(&self) -> &str { "rename" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "rename(path, path) renaming file to itself should return 0" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_rename_self.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        let ret = unsafe { rename(path.as_ptr(), path.as_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// unlink on directory should give EISDIR
pub struct UnlinkDirTest;
impl SyscallTest for UnlinkDirTest {
    fn name(&self) -> &str { "err2_unlink_directory" }
    fn syscall(&self) -> &str { "unlink" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "unlink() on a directory should return -1 EISDIR or EPERM" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_unlink_dir").unwrap();
        let start = Instant::now();
        unsafe { mkdir(path.as_ptr(), 0o755); }
        let ret = unsafe { unlink(path.as_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { rmdir(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && (errno_val == EISDIR as i32 || errno_val == EPERM as i32) { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret as i64, expected_errno: Some(EISDIR as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// ftruncate on read-only fd should give EBADF
pub struct FtruncateReadonlyTest;
impl SyscallTest for FtruncateReadonlyTest {
    fn name(&self) -> &str { "err2_ftruncate_readonly" }
    fn syscall(&self) -> &str { "ftruncate" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "ftruncate() on a read-only fd should return -1 EINVAL or EBADF" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_ftrunc_ro.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { write(fd, b"data".as_ptr() as *const _, 4); close(fd); } }
        let rdfd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        let ret = unsafe { ftruncate(rdfd, 2) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(rdfd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && (errno_val == EINVAL as i32 || errno_val == EBADF as i32) { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret as i64, expected_errno: Some(EINVAL as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// pipe2 with invalid flags should give EINVAL
pub struct Pipe2InvalidFlagsTest;
impl SyscallTest for Pipe2InvalidFlagsTest {
    fn name(&self) -> &str { "err2_pipe2_invalid_flags" }
    fn syscall(&self) -> &str { "pipe2" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "pipe2() with invalid flags (0xFFFF) should return -1 EINVAL" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        let ret = unsafe { pipe2(fds.as_mut_ptr(), 0xFFFF) };
        let errno_val = unsafe { *libc::__errno_location() };
        if ret == 0 { unsafe { close(fds[0]); close(fds[1]); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EINVAL as i32 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret as i64, expected_errno: Some(EINVAL as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// dup2 fd to itself should return the same fd
pub struct Dup2SelfTest;
impl SyscallTest for Dup2SelfTest {
    fn name(&self) -> &str { "err2_dup2_self" }
    fn syscall(&self) -> &str { "dup2" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "dup2(fd, fd) duplicating fd to itself should return fd unchanged" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_dup2self.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { dup2(fd, fd) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == fd { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: fd as i64, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// open with O_RDWR | O_TRUNC on non-writable file gives EACCES (non-root)
pub struct OpenTruncNowriteTest;
impl SyscallTest for OpenTruncNowriteTest {
    fn name(&self) -> &str { "err2_open_trunc_no_write" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open(O_RDWR|O_TRUNC) on mode-444 file as non-root gives EACCES" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_trunc_ro.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o444) };
        if fd >= 0 { unsafe { close(fd); } }
        let uid = unsafe { getuid() };
        let fd2 = unsafe { open(path.as_ptr(), O_RDWR | O_TRUNC, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd2 >= 0 { unsafe { close(fd2); } }
        unsafe { unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if uid == 0 { TestStatus::Unimplemented }
        else if fd2 == -1 && errno_val == EACCES as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd2 as i64, expected_errno: Some(EACCES as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
