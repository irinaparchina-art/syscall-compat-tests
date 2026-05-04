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

/// writev: scatter write from multiple buffers
pub struct WritevBasicTest;
impl SyscallTest for WritevBasicTest {
    fn name(&self) -> &str { "io_writev_basic" }
    fn syscall(&self) -> &str { "writev" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "writev() two buffers to a file; total bytes written should equal sum of buffer sizes" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_writev.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let buf1 = b"hello ";
        let buf2 = b"world";
        let iov = [
            iovec { iov_base: buf1.as_ptr() as *mut _, iov_len: buf1.len() },
            iovec { iov_base: buf2.as_ptr() as *mut _, iov_len: buf2.len() },
        ];
        let ret = unsafe { writev(fd, iov.as_ptr(), 2) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { lseek(fd, 0, SEEK_SET); }
        let mut rbuf = [0u8; 11];
        unsafe { read(fd, rbuf.as_mut_ptr() as *mut _, 11); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 11 && &rbuf == b"hello world" { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 11, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// readv: gather read into multiple buffers
pub struct ReadvBasicTest;
impl SyscallTest for ReadvBasicTest {
    fn name(&self) -> &str { "io_readv_basic" }
    fn syscall(&self) -> &str { "readv" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "readv() into two buffers should split file content correctly" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_readv.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, b"helloworld".as_ptr() as *const _, 10); lseek(fd, 0, SEEK_SET); }
        let mut buf1 = [0u8; 5];
        let mut buf2 = [0u8; 5];
        let iov = [
            iovec { iov_base: buf1.as_mut_ptr() as *mut _, iov_len: 5 },
            iovec { iov_base: buf2.as_mut_ptr() as *mut _, iov_len: 5 },
        ];
        let ret = unsafe { readv(fd, iov.as_ptr(), 2) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 10 && &buf1 == b"hello" && &buf2 == b"world" { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 10, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// pread: read at specific offset without seeking
pub struct PreadBasicTest;
impl SyscallTest for PreadBasicTest {
    fn name(&self) -> &str { "io_pread_basic" }
    fn syscall(&self) -> &str { "pread" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "pread() at offset 5 should read from that position without changing file offset" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_pread.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, b"helloworld".as_ptr() as *const _, 10); }
        let mut buf = [0u8; 5];
        let ret = unsafe { pread(fd, buf.as_mut_ptr() as *mut _, 5, 5) };
        let errno_val = unsafe { *libc::__errno_location() };
        // Check file offset wasn't moved (lseek returns current pos)
        let pos = unsafe { lseek(fd, 0, SEEK_CUR) };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 5 && &buf == b"world" && pos == 10 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 5, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// pwrite: write at specific offset without seeking
pub struct PwriteBasicTest;
impl SyscallTest for PwriteBasicTest {
    fn name(&self) -> &str { "io_pwrite_basic" }
    fn syscall(&self) -> &str { "pwrite" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "pwrite() at offset 5 should write without changing file offset" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_pwrite.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, b"helloXXXXX".as_ptr() as *const _, 10); }
        let ret = unsafe { pwrite(fd, b"world".as_ptr() as *const _, 5, 5) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { lseek(fd, 0, SEEK_SET); }
        let mut buf = [0u8; 10];
        unsafe { read(fd, buf.as_mut_ptr() as *mut _, 10); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 5 && &buf == b"helloworld" { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 5, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sendfile: copy between two file descriptors in kernel space
pub struct SendfileBasicTest;
impl SyscallTest for SendfileBasicTest {
    fn name(&self) -> &str { "io_sendfile_basic" }
    fn syscall(&self) -> &str { "sendfile" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "sendfile() should copy bytes from src fd to dst fd in kernel space" }
    fn run(&self) -> TestResult {
        let src_path = CString::new("/tmp/sct_sf_src.txt").unwrap();
        let dst_path = CString::new("/tmp/sct_sf_dst.txt").unwrap();
        let data = b"sendfile test data";
        let start = Instant::now();
        let src = unsafe { open(src_path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        let dst = unsafe { open(dst_path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if src < 0 || dst < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(src, data.as_ptr() as *const _, data.len()); lseek(src, 0, SEEK_SET); }
        let mut offset: off_t = 0;
        let ret = unsafe { sendfile(dst, src, &mut offset, data.len()) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { lseek(dst, 0, SEEK_SET); }
        let mut buf = vec![0u8; data.len()];
        unsafe { read(dst, buf.as_mut_ptr() as *mut _, data.len()); }
        unsafe { close(src); close(dst); unlink(src_path.as_ptr()); unlink(dst_path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == data.len() as isize && &buf[..] == data { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: data.len() as i64, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// splice: move data between pipe and file
pub struct SpliceBasicTest;
impl SyscallTest for SpliceBasicTest {
    fn name(&self) -> &str { "io_splice_pipe_to_file" }
    fn syscall(&self) -> &str { "splice" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "splice() from pipe to file should transfer data without user-space copy" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_splice.txt").unwrap();
        let start = Instant::now();
        let mut pfd = [0i32; 2];
        if unsafe { pipe(pfd.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let msg = b"splice_test";
        unsafe { write(pfd[1], msg.as_ptr() as *const _, msg.len()); close(pfd[1]); }
        let out = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if out < 0 {
            unsafe { close(pfd[0]); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { splice(pfd[0], std::ptr::null_mut(), out, std::ptr::null_mut(), msg.len(), 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { lseek(out, 0, SEEK_SET); }
        let mut buf = vec![0u8; msg.len()];
        unsafe { read(out, buf.as_mut_ptr() as *mut _, msg.len()); close(pfd[0]); close(out); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == msg.len() as isize && &buf[..] == msg { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: msg.len() as i64, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fsync: flush file data to storage
pub struct FsyncTest;
impl SyscallTest for FsyncTest {
    fn name(&self) -> &str { "io_fsync_basic" }
    fn syscall(&self) -> &str { "fsync" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fsync() on an open fd should return 0" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_fsync.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, b"fsync test".as_ptr() as *const _, 10); }
        let ret = unsafe { fsync(fd) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fdatasync: flush file data (no metadata) to storage
pub struct FdatasyncTest;
impl SyscallTest for FdatasyncTest {
    fn name(&self) -> &str { "io_fdatasync_basic" }
    fn syscall(&self) -> &str { "fdatasync" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fdatasync() on an open fd should return 0" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_fdatasync.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, b"fdatasync".as_ptr() as *const _, 9); }
        let ret = unsafe { fdatasync(fd) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
