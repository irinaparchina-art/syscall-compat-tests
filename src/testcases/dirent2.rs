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

/// fchdir: change directory using fd
pub struct FchdirTest;
impl SyscallTest for FchdirTest {
    fn name(&self) -> &str { "dir2_fchdir" }
    fn syscall(&self) -> &str { "fchdir" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fchdir(fd_of_/tmp) then getcwd() should return /tmp" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut orig = vec![0u8; 4096];
        unsafe { getcwd(orig.as_mut_ptr() as *mut c_char, orig.len()); }
        let dir = CString::new("/tmp").unwrap();
        let fd = unsafe { open(dir.as_ptr(), O_RDONLY | O_DIRECTORY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open /tmp failed".into()), start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { fchdir(fd) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut cwd = vec![0u8; 4096];
        unsafe { getcwd(cwd.as_mut_ptr() as *mut c_char, cwd.len()); }
        // Restore
        unsafe { chdir(orig.as_ptr() as *const c_char); close(fd); }
        let cwd_str: String = cwd.iter().take_while(|c| **c != 0).map(|&c| c as char).collect();
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && cwd_str == "/tmp" { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fchmod: change permissions via fd
pub struct FchmodTest;
impl SyscallTest for FchmodTest {
    fn name(&self) -> &str { "dir2_fchmod" }
    fn syscall(&self) -> &str { "fchmod" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fchmod(fd, 0o600) then fstat should confirm new mode" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_fchmod.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { fchmod(fd, 0o600) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::fstat(fd, &mut st); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && (st.st_mode & 0o777) == 0o600 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fchmodat: change mode using dirfd
pub struct FchmodatTest;
impl SyscallTest for FchmodatTest {
    fn name(&self) -> &str { "dir2_fchmodat" }
    fn syscall(&self) -> &str { "fchmodat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fchmodat(AT_FDCWD, path, 0o755) should change mode" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_fchmodat.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        let ret = unsafe { fchmodat(AT_FDCWD, path.as_ptr(), 0o755, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::stat(path.as_ptr(), &mut st); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && (st.st_mode & 0o777) == 0o755 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// lchown: change ownership of symlink itself (not target)
pub struct LchownTest;
impl SyscallTest for LchownTest {
    fn name(&self) -> &str { "dir2_lchown_symlink" }
    fn syscall(&self) -> &str { "lchown" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "lchown() on a symlink should return 0 (or EPERM if not root)" }
    fn run(&self) -> TestResult {
        let target = CString::new("/tmp/sct_lchown_tgt.txt").unwrap();
        let link = CString::new("/tmp/sct_lchown_lnk").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(target.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        unsafe { unlink(link.as_ptr()); symlink(target.as_ptr(), link.as_ptr()); }
        let uid = unsafe { getuid() };
        let gid = unsafe { getgid() };
        let ret = unsafe { lchown(link.as_ptr(), uid, gid) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { unlink(link.as_ptr()); unlink(target.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 || errno_val == EPERM as i32 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// utimes: set file access and modification times
pub struct UtimesTest;
impl SyscallTest for UtimesTest {
    fn name(&self) -> &str { "dir2_utimes" }
    fn syscall(&self) -> &str { "utimes" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "utimes() sets atime and mtime; stat should reflect new times" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_utimes.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        let times = [
            timeval { tv_sec: 1000000, tv_usec: 0 },
            timeval { tv_sec: 2000000, tv_usec: 0 },
        ];
        let ret = unsafe { utimes(path.as_ptr(), times.as_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::stat(path.as_ptr(), &mut st); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && st.st_mtime == 2000000 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// futimens: set times via fd
pub struct FutimensTest;
impl SyscallTest for FutimensTest {
    fn name(&self) -> &str { "dir2_futimens" }
    fn syscall(&self) -> &str { "futimens" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "futimens() with UTIME_NOW should update mtime to current time" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_futimens.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let times = [
            timespec { tv_sec: 0, tv_nsec: UTIME_NOW },
            timespec { tv_sec: 0, tv_nsec: UTIME_NOW },
        ];
        let ret = unsafe { futimens(fd, times.as_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::fstat(fd, &mut st); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && st.st_mtime > 1_577_836_800 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// statvfs: get filesystem statistics
pub struct StatvfsTest;
impl SyscallTest for StatvfsTest {
    fn name(&self) -> &str { "dir2_statvfs_tmp" }
    fn syscall(&self) -> &str { "statvfs" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "statvfs(\"/tmp\") should return valid filesystem info with f_bsize > 0" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp").unwrap();
        let start = Instant::now();
        let mut st: statvfs = unsafe { std::mem::zeroed() };
        let ret = unsafe { statvfs(path.as_ptr(), &mut st) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && st.f_bsize > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
