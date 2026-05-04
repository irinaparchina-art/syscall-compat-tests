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

pub struct PipeBasicTest;
impl SyscallTest for PipeBasicTest {
    fn name(&self) -> &str { "pipe_basic" }
    fn syscall(&self) -> &str { "pipe" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "pipe() should create two valid fds; write to write-end, read from read-end" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        let r = unsafe { pipe(fds.as_mut_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        if r != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 0, actual_ret: r as i64, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        let msg = b"hello pipe";
        let w = unsafe { write(fds[1], msg.as_ptr() as *const _, msg.len()) };
        let mut buf = vec![0u8; msg.len()];
        let rd = unsafe { read(fds[0], buf.as_mut_ptr() as *mut _, buf.len()) };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if w == msg.len() as isize && rd == msg.len() as isize && &buf[..] == msg {
            TestStatus::Pass
        } else {
            TestStatus::Fail { expected_ret: msg.len() as i64, actual_ret: rd as i64, expected_errno: None, actual_errno: None }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct PipeReadEofTest;
impl SyscallTest for PipeReadEofTest {
    fn name(&self) -> &str { "pipe_read_eof_on_close" }
    fn syscall(&self) -> &str { "pipe" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "read() on pipe returns 0 (EOF) after write-end is closed" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        let r = unsafe { pipe(fds.as_mut_ptr()) };
        if r != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { close(fds[1]) }; // close write end
        let mut buf = [0u8; 16];
        let rd = unsafe { read(fds[0], buf.as_mut_ptr() as *mut _, buf.len()) };
        unsafe { close(fds[0]); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if rd == 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: rd as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct Pipe2CloseOnExecTest;
impl SyscallTest for Pipe2CloseOnExecTest {
    fn name(&self) -> &str { "pipe2_cloexec" }
    fn syscall(&self) -> &str { "pipe2" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "pipe2(O_CLOEXEC) should set FD_CLOEXEC on both fds" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        let r = unsafe { pipe2(fds.as_mut_ptr(), O_CLOEXEC) };
        let errno_val = unsafe { *libc::__errno_location() };
        if r != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 0, actual_ret: r as i64, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        let flags0 = unsafe { fcntl(fds[0], F_GETFD) };
        let flags1 = unsafe { fcntl(fds[1], F_GETFD) };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if flags0 & FD_CLOEXEC != 0 && flags1 & FD_CLOEXEC != 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: FD_CLOEXEC as i64, actual_ret: flags0 as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct FcntlGetFdTest;
impl SyscallTest for FcntlGetFdTest {
    fn name(&self) -> &str { "fcntl_get_fd_flags" }
    fn syscall(&self) -> &str { "fcntl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "fcntl(fd, F_GETFD) on a valid fd should return >= 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { fcntl(1, F_GETFD) } as i64; // stdout
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct FcntlSetNonblockTest;
impl SyscallTest for FcntlSetNonblockTest {
    fn name(&self) -> &str { "fcntl_set_nonblock" }
    fn syscall(&self) -> &str { "fcntl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "fcntl(F_SETFL, O_NONBLOCK) should set non-blocking mode on a pipe fd" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let r = unsafe { fcntl(fds[0], F_SETFL, O_NONBLOCK) };
        let errno_val = unsafe { *libc::__errno_location() };
        let flags = unsafe { fcntl(fds[0], F_GETFL) };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if r == 0 && flags & O_NONBLOCK != 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct ReadNonblockTest;
impl SyscallTest for ReadNonblockTest {
    fn name(&self) -> &str { "pipe_read_nonblock_eagain" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "read() on empty non-blocking pipe should return -1 with EAGAIN" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { fcntl(fds[0], F_SETFL, O_NONBLOCK) };
        let mut buf = [0u8; 16];
        let ret = unsafe { read(fds[0], buf.as_mut_ptr() as *mut _, buf.len()) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == -1 && (errno_val == EAGAIN as i32 || errno_val == EWOULDBLOCK as i32) { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(EAGAIN as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}
