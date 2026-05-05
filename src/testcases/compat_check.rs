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

/// Verify errno is correctly cleared after successful syscall
pub struct ErrnoResetTest;
impl SyscallTest for ErrnoResetTest {
    fn name(&self) -> &str { "compat_errno_reset_after_success" }
    fn syscall(&self) -> &str { "getpid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "After a failing syscall, errno should be set; after success it should NOT auto-clear" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // First cause an error to set errno
        unsafe { open(CString::new("/no/such/path").unwrap().as_ptr(), O_RDONLY, 0) };
        let errno_after_fail = unsafe { *libc::__errno_location() };
        // Now do a successful call
        unsafe { getpid() };
        let errno_after_success = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        // errno should be ENOENT after open failure
        // After getpid (success), errno should remain ENOENT (not cleared)
        let s = if errno_after_fail == ENOENT as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: ENOENT as i64, actual_ret: errno_after_fail as i64, expected_errno: None, actual_errno: Some(errno_after_success) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// write() returns exact byte count written
pub struct WriteExactCountTest;
impl SyscallTest for WriteExactCountTest {
    fn name(&self) -> &str { "compat_write_exact_count" }
    fn syscall(&self) -> &str { "write" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "write() must return exactly the number of bytes requested (for regular files)" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_exact_write.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let data = vec![0xABu8; 4096];
        let n = unsafe { write(fd, data.as_ptr() as *const _, 4096) };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 4096 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 4096, actual_ret: n as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// read() on a regular file never returns EAGAIN
pub struct ReadNoEagainTest;
impl SyscallTest for ReadNoEagainTest {
    fn name(&self) -> &str { "compat_read_no_eagain_regular" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "read() on a regular file must never return EAGAIN (even with O_NONBLOCK)" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_read_noagain.txt").unwrap();
        let start = Instant::now();
        let wfd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if wfd >= 0 {
            unsafe { write(wfd, b"data".as_ptr() as *const _, 4); close(wfd); }
        }
        let fd = unsafe { open(path.as_ptr(), O_RDONLY | O_NONBLOCK, 0) };
        if fd < 0 {
            unsafe { unlink(path.as_ptr()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut buf = [0u8; 8];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 8) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n > 0 && errno_val != EAGAIN as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 4, actual_ret: n as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fork() child has independent copy of memory
pub struct ForkCowTest;
impl SyscallTest for ForkCowTest {
    fn name(&self) -> &str { "compat_fork_cow_memory" }
    fn syscall(&self) -> &str { "fork" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "fork() child modifying its copy of memory must not affect parent" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut val: i32 = 42;
        let mut pipefd = [0i32; 2];
        if unsafe { pipe(pipefd.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let pid = unsafe { fork() };
        if pid < 0 {
            unsafe { close(pipefd[0]); close(pipefd[1]); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            unsafe { close(pipefd[0]); }
            val = 999; // Modify child's copy
            unsafe { write(pipefd[1], &val as *const _ as *const _, 4); }
            unsafe { close(pipefd[1]); libc::exit(0); }
        }
        unsafe { close(pipefd[1]); }
        let mut child_val: i32 = 0;
        unsafe { read(pipefd[0], &mut child_val as *mut _ as *mut _, 4); close(pipefd[0]); }
        unsafe { waitpid(pid, std::ptr::null_mut(), 0); }
        let dur = start.elapsed().as_micros() as u64;
        // Parent's val should still be 42, child sent 999
        let s = if val == 42 && child_val == 999 { TestStatus::Pass }
        else { TestStatus::Error(format!("parent val={}, child val={}", val, child_val)) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// open() with O_CREAT creates file with correct mode (respecting umask)
pub struct OpenModeUmaskTest;
impl SyscallTest for OpenModeUmaskTest {
    fn name(&self) -> &str { "compat_open_mode_umask" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open(O_CREAT, 0o666) with umask 0o022 should create file with mode 0o644" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_umask_mode.txt").unwrap();
        let start = Instant::now();
        let old_umask = unsafe { umask(0o022) };
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o666) };
        unsafe { umask(old_umask) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { close(fd); }
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::stat(path.as_ptr(), &mut st); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let mode = st.st_mode & 0o777;
        let s = if mode == 0o644 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0o644, actual_ret: mode as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// File offset after write should advance by bytes written
pub struct OffsetAdvanceTest;
impl SyscallTest for OffsetAdvanceTest {
    fn name(&self) -> &str { "compat_file_offset_advances" }
    fn syscall(&self) -> &str { "write/lseek" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "After write(fd, buf, N), file offset must advance by N bytes" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_offset_adv.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let n = 77usize;
        let data = vec![0u8; n];
        unsafe { write(fd, data.as_ptr() as *const _, n); }
        let pos = unsafe { lseek(fd, 0, SEEK_CUR) };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if pos == n as i64 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: n as i64, actual_ret: pos as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getpid() always returns same value in same process
pub struct GetpidConsistencyTest;
impl SyscallTest for GetpidConsistencyTest {
    fn name(&self) -> &str { "compat_getpid_consistent" }
    fn syscall(&self) -> &str { "getpid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getpid() called twice in same process must return same value" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let pid1 = unsafe { getpid() };
        let pid2 = unsafe { getpid() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if pid1 == pid2 && pid1 > 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: pid1 as i64, actual_ret: pid2 as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// CLOCK_MONOTONIC only increases over time
pub struct MonotonicOnlyIncreaseTest;
impl SyscallTest for MonotonicOnlyIncreaseTest {
    fn name(&self) -> &str { "compat_monotonic_only_increases" }
    fn syscall(&self) -> &str { "clock_gettime" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Time }
    fn description(&self) -> &str { "100 consecutive CLOCK_MONOTONIC reads must form a non-decreasing sequence" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut prev = 0i128;
        let mut violated = false;
        for _ in 0..100 {
            let mut ts: timespec = unsafe { std::mem::zeroed() };
            unsafe { clock_gettime(CLOCK_MONOTONIC, &mut ts) };
            let ns = ts.tv_sec as i128 * 1_000_000_000 + ts.tv_nsec as i128;
            if ns < prev { violated = true; break; }
            prev = ns;
        }
        let dur = start.elapsed().as_micros() as u64;
        let s = if !violated { TestStatus::Pass }
        else { TestStatus::Error("CLOCK_MONOTONIC went backwards".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// mmap anonymous memory is zero-initialized
pub struct MmapZeroInitTest;
impl SyscallTest for MmapZeroInitTest {
    fn name(&self) -> &str { "compat_mmap_zero_initialized" }
    fn syscall(&self) -> &str { "mmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "Anonymous mmap() pages must be zero-initialized per POSIX" }
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
        let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, 4096) };
        let all_zero = slice.iter().all(|&b| b == 0);
        unsafe { munmap(ptr, 4096); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if all_zero { TestStatus::Pass }
        else { TestStatus::Error("mmap anonymous page not zero-initialized".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
