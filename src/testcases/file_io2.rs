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

/// open with O_EXCL on existing file should fail with EEXIST
pub struct OpenExclExistsTest;
impl SyscallTest for OpenExclExistsTest {
    fn name(&self) -> &str { "fio2_open_excl_exists" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open(O_CREAT|O_EXCL) on existing file should return -1 EEXIST" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_excl.txt").unwrap();
        let start = Instant::now();
        let fd1 = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd1 >= 0 { unsafe { close(fd1); } }
        let fd2 = unsafe { open(path.as_ptr(), O_CREAT | O_EXCL | O_WRONLY, 0o644) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd2 >= 0 { unsafe { close(fd2); } }
        unsafe { unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd2 == -1 && errno_val == EEXIST as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd2 as i64, expected_errno: Some(EEXIST as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// write to read-only fd should fail with EBADF
pub struct WriteReadonlyFdTest;
impl SyscallTest for WriteReadonlyFdTest {
    fn name(&self) -> &str { "fio2_write_readonly_fd" }
    fn syscall(&self) -> &str { "write" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "write() to a read-only fd should return -1 EBADF" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_rdonly.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        let rdfd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        let ret = unsafe { write(rdfd, b"x".as_ptr() as *const _, 1) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(rdfd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EBADF as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(EBADF as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// read from write-only fd should fail with EBADF
pub struct ReadWriteonlyFdTest;
impl SyscallTest for ReadWriteonlyFdTest {
    fn name(&self) -> &str { "fio2_read_writeonly_fd" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "read() from a write-only fd should return -1 EBADF" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_wronly.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        let mut buf = [0u8; 8];
        let ret = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 8) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EBADF as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(EBADF as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// lseek past end of file then write creates a hole
pub struct LseekHoleTest;
impl SyscallTest for LseekHoleTest {
    fn name(&self) -> &str { "fio2_lseek_hole_write" }
    fn syscall(&self) -> &str { "lseek" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "lseek past EOF then write; file size should equal offset + written bytes" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_hole.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { lseek(fd, 1000, SEEK_SET); }
        unsafe { write(fd, b"end".as_ptr() as *const _, 3); }
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::fstat(fd, &mut st); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if st.st_size == 1003 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 1003, actual_ret: st.st_size, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// open nonexistent path with O_RDONLY should fail ENOENT
pub struct OpenNoentTest;
impl SyscallTest for OpenNoentTest {
    fn name(&self) -> &str { "fio2_open_noent_rdonly" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open() nonexistent path O_RDONLY should return -1 ENOENT" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_noent_xyz_123.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd == -1 && errno_val == ENOENT as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd as i64, expected_errno: Some(ENOENT as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// flock: exclusive lock on a file
pub struct FlockExclusiveTest;
impl SyscallTest for FlockExclusiveTest {
    fn name(&self) -> &str { "fio2_flock_exclusive" }
    fn syscall(&self) -> &str { "flock" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "flock(LOCK_EX) then flock(LOCK_UN) on same fd should both return 0" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_flock.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let r1 = unsafe { flock(fd, LOCK_EX) };
        let e1 = unsafe { *libc::__errno_location() };
        let r2 = unsafe { flock(fd, LOCK_UN) };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r1 == 0 && r2 == 0 { TestStatus::Pass }
        else if e1 == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r1 as i64, expected_errno: None, actual_errno: Some(e1) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fallocate: pre-allocate space in file
pub struct FallocateTest;
impl SyscallTest for FallocateTest {
    fn name(&self) -> &str { "fio2_fallocate_basic" }
    fn syscall(&self) -> &str { "fallocate" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fallocate(0, 0, 4096) should succeed and file size should be >= 4096" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_fallocate.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { fallocate(fd, 0, 0, 4096) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::fstat(fd, &mut st); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && st.st_size >= 4096 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 || errno_val == EOPNOTSUPP as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// copy_file_range: copy bytes between two open files
pub struct CopyFileRangeTest;
impl SyscallTest for CopyFileRangeTest {
    fn name(&self) -> &str { "fio2_copy_file_range" }
    fn syscall(&self) -> &str { "copy_file_range" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "copy_file_range() should copy bytes between fds in kernel space" }
    fn run(&self) -> TestResult {
        let src_path = CString::new("/tmp/sct_cfr_src.txt").unwrap();
        let dst_path = CString::new("/tmp/sct_cfr_dst.txt").unwrap();
        let data = b"copy_file_range_data";
        let start = Instant::now();
        let src = unsafe { open(src_path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        let dst = unsafe { open(dst_path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if src < 0 || dst < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(src, data.as_ptr() as *const _, data.len()); }
        let mut src_off: loff_t = 0;
        let mut dst_off: loff_t = 0;
        let ret = unsafe {
            libc::syscall(libc::SYS_copy_file_range,
                src as c_long, &mut src_off as *mut _ as c_long,
                dst as c_long, &mut dst_off as *mut _ as c_long,
                data.len() as c_long, 0 as c_long)
        };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { lseek(dst, 0, SEEK_SET); }
        let mut buf = vec![0u8; data.len()];
        unsafe { read(dst, buf.as_mut_ptr() as *mut _, data.len()); }
        unsafe { close(src); close(dst); unlink(src_path.as_ptr()); unlink(dst_path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == data.len() as i64 && &buf[..] == data { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 || errno_val == EXDEV as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: data.len() as i64, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// linkat: create hard link using dirfd
pub struct LinkatTest;
impl SyscallTest for LinkatTest {
    fn name(&self) -> &str { "fio2_linkat_cwd" }
    fn syscall(&self) -> &str { "linkat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "linkat(AT_FDCWD, src, AT_FDCWD, dst) should create a hard link" }
    fn run(&self) -> TestResult {
        let src = CString::new("/tmp/sct_lat_src.txt").unwrap();
        let dst = CString::new("/tmp/sct_lat_dst.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(src.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        unsafe { unlink(dst.as_ptr()); }
        let ret = unsafe { linkat(AT_FDCWD, src.as_ptr(), AT_FDCWD, dst.as_ptr(), 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::stat(src.as_ptr(), &mut st); }
        unsafe { unlink(src.as_ptr()); unlink(dst.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && st.st_nlink >= 2 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// unlinkat: remove file using dirfd
pub struct UnlinkatTest;
impl SyscallTest for UnlinkatTest {
    fn name(&self) -> &str { "fio2_unlinkat_cwd" }
    fn syscall(&self) -> &str { "unlinkat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "unlinkat(AT_FDCWD, path, 0) should remove a file" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_unlinkat.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        let ret = unsafe { unlinkat(AT_FDCWD, path.as_ptr(), 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        let sr = unsafe { libc::stat(path.as_ptr(), &mut st) };
        let se = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && sr == -1 && se == ENOENT as i32 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// renameat: rename file using dirfd
pub struct RenameatTest;
impl SyscallTest for RenameatTest {
    fn name(&self) -> &str { "fio2_renameat_cwd" }
    fn syscall(&self) -> &str { "renameat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "renameat(AT_FDCWD, src, AT_FDCWD, dst) should rename a file" }
    fn run(&self) -> TestResult {
        let src = CString::new("/tmp/sct_rat_src.txt").unwrap();
        let dst = CString::new("/tmp/sct_rat_dst.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(src.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        unsafe { unlink(dst.as_ptr()); }
        let ret = unsafe { renameat(AT_FDCWD, src.as_ptr(), AT_FDCWD, dst.as_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        let sr = unsafe { libc::stat(dst.as_ptr(), &mut st) };
        unsafe { unlink(src.as_ptr()); unlink(dst.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && sr == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// readahead: advisory read-ahead on a file
pub struct ReadaheadTest;
impl SyscallTest for ReadaheadTest {
    fn name(&self) -> &str { "fio2_readahead" }
    fn syscall(&self) -> &str { "readahead" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "readahead() on a valid fd should return 0" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_readahead.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let data = vec![0u8; 4096];
        unsafe { write(fd, data.as_ptr() as *const _, 4096); }
        let ret = unsafe { readahead(fd, 0, 4096) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
