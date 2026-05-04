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

pub struct ChmodBasicTest;
impl SyscallTest for ChmodBasicTest {
    fn name(&self) -> &str { "fs_chmod_basic" }
    fn syscall(&self) -> &str { "chmod" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "chmod() should change file permissions; stat() should reflect new mode" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_chmod.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { close(fd); }
        let ret = unsafe { chmod(path.as_ptr(), 0o600) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::stat(path.as_ptr(), &mut st) };
        unsafe { unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 && (st.st_mode & 0o777) == 0o600 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct TruncateBasicTest;
impl SyscallTest for TruncateBasicTest {
    fn name(&self) -> &str { "fs_truncate_basic" }
    fn syscall(&self) -> &str { "truncate" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "truncate() should shrink file to given size; stat() should confirm" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_truncate.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, b"hello world!".as_ptr() as *const _, 12); close(fd); }
        let ret = unsafe { truncate(path.as_ptr(), 5) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::stat(path.as_ptr(), &mut st); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 && st.st_size == 5 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 5, actual_ret: st.st_size, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct FtruncateTest;
impl SyscallTest for FtruncateTest {
    fn name(&self) -> &str { "fs_ftruncate_basic" }
    fn syscall(&self) -> &str { "ftruncate" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "ftruncate() on an open fd should resize file to given length" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_ftruncate.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, b"abcdefghij".as_ptr() as *const _, 10); }
        let ret = unsafe { ftruncate(fd, 4) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::fstat(fd, &mut st); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 && st.st_size == 4 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 4, actual_ret: st.st_size, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct FstatTest;
impl SyscallTest for FstatTest {
    fn name(&self) -> &str { "fs_fstat_basic" }
    fn syscall(&self) -> &str { "fstat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fstat() on open fd should return correct file size" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_fstat.txt").unwrap();
        let data = b"fstat test";
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, data.as_ptr() as *const _, data.len()); }
        let mut st: stat = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::fstat(fd, &mut st) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 && st.st_size == data.len() as i64 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: data.len() as i64, actual_ret: st.st_size, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct AccessReadableTest;
impl SyscallTest for AccessReadableTest {
    fn name(&self) -> &str { "fs_access_readable" }
    fn syscall(&self) -> &str { "access" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "access(path, R_OK) on a readable file should return 0" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_access.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { close(fd); }
        let ret = unsafe { access(path.as_ptr(), R_OK) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct AccessNotExistTest;
impl SyscallTest for AccessNotExistTest {
    fn name(&self) -> &str { "fs_access_nonexistent" }
    fn syscall(&self) -> &str { "access" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "access() on nonexistent file should return -1 with ENOENT" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_no_such_access_xyz.txt").unwrap();
        let start = Instant::now();
        let ret = unsafe { access(path.as_ptr(), F_OK) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == -1 && errno_val == ENOENT as i32 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret as i64, expected_errno: Some(ENOENT as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct SymlinkTest;
impl SyscallTest for SymlinkTest {
    fn name(&self) -> &str { "fs_symlink_basic" }
    fn syscall(&self) -> &str { "symlink" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "symlink() creates a symlink; lstat() confirms it is a symlink" }
    fn run(&self) -> TestResult {
        let target = CString::new("/tmp/sct_symlink_target.txt").unwrap();
        let link = CString::new("/tmp/sct_symlink_link.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(target.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { close(fd); unlink(link.as_ptr()); }
        let ret = unsafe { symlink(target.as_ptr(), link.as_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        let lr = unsafe { lstat(link.as_ptr(), &mut st) };
        unsafe { unlink(link.as_ptr()); unlink(target.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 && lr == 0 && (st.st_mode & S_IFMT) == S_IFLNK { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct ReadlinkTest;
impl SyscallTest for ReadlinkTest {
    fn name(&self) -> &str { "fs_readlink_basic" }
    fn syscall(&self) -> &str { "readlink" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "readlink() on a symlink should return the target path" }
    fn run(&self) -> TestResult {
        let target = CString::new("/tmp/sct_rl_target.txt").unwrap();
        let link = CString::new("/tmp/sct_rl_link.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(target.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { close(fd); unlink(link.as_ptr()); symlink(target.as_ptr(), link.as_ptr()); }
        let mut buf = vec![0i8; 256];
        let ret = unsafe { readlink(link.as_ptr(), buf.as_mut_ptr(), buf.len()) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { unlink(link.as_ptr()); unlink(target.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let read_path: String = buf[..ret.max(0) as usize].iter().map(|&c| c as u8 as char).collect();
        let status = if ret > 0 && read_path == "/tmp/sct_rl_target.txt" { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: target.as_bytes().len() as i64, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct HardlinkTest;
impl SyscallTest for HardlinkTest {
    fn name(&self) -> &str { "fs_link_hardlink" }
    fn syscall(&self) -> &str { "link" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "link() creates a hard link; both paths should have st_nlink == 2" }
    fn run(&self) -> TestResult {
        let src = CString::new("/tmp/sct_link_src.txt").unwrap();
        let dst = CString::new("/tmp/sct_link_dst.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(src.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { close(fd); unlink(dst.as_ptr()); }
        let ret = unsafe { link(src.as_ptr(), dst.as_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::stat(src.as_ptr(), &mut st); }
        unsafe { unlink(src.as_ptr()); unlink(dst.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 && st.st_nlink >= 2 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct StatfsTest;
impl SyscallTest for StatfsTest {
    fn name(&self) -> &str { "fs_statfs_tmp" }
    fn syscall(&self) -> &str { "statfs" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "statfs(\"/tmp\") should succeed and return non-zero block size" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp").unwrap();
        let mut buf: statfs = unsafe { std::mem::zeroed() };
        let start = Instant::now();
        let ret = unsafe { statfs(path.as_ptr(), &mut buf) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 && buf.f_bsize > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}
