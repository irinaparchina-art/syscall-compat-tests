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

/// tee: duplicate data between two pipes without consuming
pub struct TeeBasicTest;
impl SyscallTest for TeeBasicTest {
    fn name(&self) -> &str { "pipe2_tee_basic" }
    fn syscall(&self) -> &str { "tee" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "tee() copies data from pipe1 to pipe2; both should contain the same data" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut p1 = [0i32; 2];
        let mut p2 = [0i32; 2];
        if unsafe { pipe(p1.as_mut_ptr()) != 0 || pipe(p2.as_mut_ptr()) != 0 } {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let msg = b"tee_test";
        unsafe { write(p1[1], msg.as_ptr() as *const _, msg.len()); }
        let ret = unsafe { tee(p1[0], p2[1], msg.len(), SPLICE_F_NONBLOCK) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut buf1 = [0u8; 8];
        let mut buf2 = [0u8; 8];
        let r1 = unsafe { read(p1[0], buf1.as_mut_ptr() as *mut _, 8) };
        let r2 = unsafe { read(p2[0], buf2.as_mut_ptr() as *mut _, 8) };
        unsafe { close(p1[0]); close(p1[1]); close(p2[0]); close(p2[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == msg.len() as isize && r1 == 8 && r2 == 8 && buf1 == buf2 && &buf1 == msg { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: msg.len() as i64, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// vmsplice: transfer user-space buffer to pipe
pub struct VmspliceTest;
impl SyscallTest for VmspliceTest {
    fn name(&self) -> &str { "pipe2_vmsplice_basic" }
    fn syscall(&self) -> &str { "vmsplice" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "vmsplice() transfers a user buffer into a pipe; read should retrieve it" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let data = b"vmsplice_data";
        let iov = iovec { iov_base: data.as_ptr() as *mut _, iov_len: data.len() };
        let ret = unsafe { vmsplice(fds[1], &iov, 1, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut buf = vec![0u8; data.len()];
        let n = unsafe { read(fds[0], buf.as_mut_ptr() as *mut _, data.len()) };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == data.len() as isize && n == data.len() as isize && &buf[..] == data { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: data.len() as i64, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// pipe capacity: PIPE_BUF worth of atomic writes
pub struct PipeBufAtomicTest;
impl SyscallTest for PipeBufAtomicTest {
    fn name(&self) -> &str { "pipe2_pipe_buf_atomic" }
    fn syscall(&self) -> &str { "write" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "write() <= PIPE_BUF bytes to pipe should be atomic and fully written" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        // PIPE_BUF is 4096 on Linux
        let data = vec![0xABu8; 4096];
        let n_write = unsafe { write(fds[1], data.as_ptr() as *const _, 4096) };
        let mut buf = vec![0u8; 4096];
        let n_read = unsafe { read(fds[0], buf.as_mut_ptr() as *mut _, 4096) };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n_write == 4096 && n_read == 4096 && buf.iter().all(|&b| b == 0xAB) { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 4096, actual_ret: n_write as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fcntl F_DUPFD: duplicate fd to >= minimum value
pub struct FcntlDupfdTest;
impl SyscallTest for FcntlDupfdTest {
    fn name(&self) -> &str { "pipe2_fcntl_dupfd" }
    fn syscall(&self) -> &str { "fcntl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "fcntl(fd, F_DUPFD, 100) should return a new fd >= 100" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_dupfd.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { fcntl(fd, F_DUPFD, 100) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); if ret >= 0 { close(ret); } unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret >= 100 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 100, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// F_GETPIPE_SZ / F_SETPIPE_SZ: get and set pipe capacity
pub struct PipeSizeTest;
impl SyscallTest for PipeSizeTest {
    fn name(&self) -> &str { "pipe2_pipe_capacity" }
    fn syscall(&self) -> &str { "fcntl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "fcntl(F_GETPIPE_SZ) should return pipe capacity >= 4096" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let size = unsafe { fcntl(fds[0], F_GETPIPE_SZ) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if size >= 4096 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 4096, actual_ret: size as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// inotify_init1: create an inotify instance
pub struct InotifyInitTest;
impl SyscallTest for InotifyInitTest {
    fn name(&self) -> &str { "pipe2_inotify_init" }
    fn syscall(&self) -> &str { "inotify_init1" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "inotify_init1(0) should return a valid inotify fd" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { inotify_init1(0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
