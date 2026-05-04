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

/// opendir("/tmp") should return non-null
pub struct OpendirTest;
impl SyscallTest for OpendirTest {
    fn name(&self) -> &str { "dir_opendir_tmp" }
    fn syscall(&self) -> &str { "opendir" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "opendir(\"/tmp\") should return a non-null DIR pointer" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let path = CString::new("/tmp").unwrap();
        let dir = unsafe { opendir(path.as_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        if !dir.is_null() { unsafe { closedir(dir); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if !dir.is_null() { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: 0, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// readdir on /tmp should return at least "." and ".."
pub struct ReaddirTest;
impl SyscallTest for ReaddirTest {
    fn name(&self) -> &str { "dir_readdir_dot_entries" }
    fn syscall(&self) -> &str { "readdir" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "readdir() on /tmp should yield entries including \".\" and \"..\"" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let path = CString::new("/tmp").unwrap();
        let dir = unsafe { opendir(path.as_ptr()) };
        if dir.is_null() {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("opendir failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut found_dot = false;
        let mut found_dotdot = false;
        loop {
            let entry = unsafe { readdir(dir) };
            if entry.is_null() { break; }
            let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }
                .to_str().unwrap_or("");
            if name == "." { found_dot = true; }
            if name == ".." { found_dotdot = true; }
            if found_dot && found_dotdot { break; }
        }
        unsafe { closedir(dir); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if found_dot && found_dotdot { TestStatus::Pass }
        else { TestStatus::Error("missing . or .. entries".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// mkdir + opendir + readdir: create a dir with files, verify they appear
pub struct ReaddirCreatedFilesTest;
impl SyscallTest for ReaddirCreatedFilesTest {
    fn name(&self) -> &str { "dir_readdir_created_files" }
    fn syscall(&self) -> &str { "readdir" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "readdir() on a dir with known files should enumerate them all" }
    fn run(&self) -> TestResult {
        let dir_path = CString::new("/tmp/sct_dirent_test").unwrap();
        let start = Instant::now();
        unsafe { mkdir(dir_path.as_ptr(), 0o755); }
        // Create 3 files
        for i in 0..3usize {
            let p = CString::new(format!("/tmp/sct_dirent_test/file{}", i)).unwrap();
            let fd = unsafe { open(p.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
            if fd >= 0 { unsafe { close(fd); } }
        }
        let dir = unsafe { opendir(dir_path.as_ptr()) };
        let mut count = 0usize;
        if !dir.is_null() {
            loop {
                let e = unsafe { readdir(dir) };
                if e.is_null() { break; }
                let name = unsafe { std::ffi::CStr::from_ptr((*e).d_name.as_ptr()) }
                    .to_str().unwrap_or("");
                if name != "." && name != ".." { count += 1; }
            }
            unsafe { closedir(dir); }
        }
        // Cleanup
        for i in 0..3usize {
            let p = CString::new(format!("/tmp/sct_dirent_test/file{}", i)).unwrap();
            unsafe { unlink(p.as_ptr()); }
        }
        unsafe { rmdir(dir_path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if count == 3 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: count as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// rewinddir: read entries, rewind, read again — same count
pub struct RewinddirTest;
impl SyscallTest for RewinddirTest {
    fn name(&self) -> &str { "dir_rewinddir" }
    fn syscall(&self) -> &str { "rewinddir" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "rewinddir() should reset position; second pass should yield same entry count" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let path = CString::new("/tmp").unwrap();
        let dir = unsafe { opendir(path.as_ptr()) };
        if dir.is_null() {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("opendir failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut c1 = 0usize;
        loop { let e = unsafe { readdir(dir) }; if e.is_null() { break; } c1 += 1; }
        unsafe { rewinddir(dir); }
        let mut c2 = 0usize;
        loop { let e = unsafe { readdir(dir) }; if e.is_null() { break; } c2 += 1; }
        unsafe { closedir(dir); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if c1 == c2 && c1 > 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: c1 as i64, actual_ret: c2 as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getdents64 via raw syscall on a directory fd
pub struct Getdents64Test;
impl SyscallTest for Getdents64Test {
    fn name(&self) -> &str { "dir_getdents64" }
    fn syscall(&self) -> &str { "getdents64" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "getdents64() on /tmp fd should return > 0 bytes of directory entries" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY | O_DIRECTORY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open dir failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut buf = vec![0u8; 4096];
        let ret = unsafe { syscall(SYS_getdents64, fd as c_long, buf.as_mut_ptr() as c_long, 4096 as c_long) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// chdir changes the working directory; getcwd confirms it
pub struct ChdirTest;
impl SyscallTest for ChdirTest {
    fn name(&self) -> &str { "dir_chdir_getcwd" }
    fn syscall(&self) -> &str { "chdir" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "chdir(\"/tmp\") then getcwd() should return \"/tmp\"" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // Save current dir
        let mut orig = vec![0i8; 4096];
        unsafe { getcwd(orig.as_mut_ptr(), orig.len()); }
        let tmp = CString::new("/tmp").unwrap();
        let ret = unsafe { chdir(tmp.as_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut cwd = vec![0i8; 4096];
        unsafe { getcwd(cwd.as_mut_ptr(), cwd.len()); }
        // Restore
        unsafe { chdir(orig.as_ptr()); }
        let cwd_str: String = cwd.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && cwd_str == "/tmp" { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// openat with AT_FDCWD opens relative to cwd
pub struct OpenatTest;
impl SyscallTest for OpenatTest {
    fn name(&self) -> &str { "dir_openat_cwd" }
    fn syscall(&self) -> &str { "openat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "openat(AT_FDCWD, path, O_CREAT) should behave like open()" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_openat.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { openat(AT_FDCWD, path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644u32) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); unlink(path.as_ptr()); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// mkdirat with AT_FDCWD
pub struct MkdiratTest;
impl SyscallTest for MkdiratTest {
    fn name(&self) -> &str { "dir_mkdirat_cwd" }
    fn syscall(&self) -> &str { "mkdirat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "mkdirat(AT_FDCWD, path, mode) should create a directory" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_mkdirat_test").unwrap();
        let start = Instant::now();
        unsafe { rmdir(path.as_ptr()); }
        let ret = unsafe { mkdirat(AT_FDCWD, path.as_ptr(), 0o755) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut st: stat = unsafe { std::mem::zeroed() };
        let sr = unsafe { libc::stat(path.as_ptr(), &mut st) };
        unsafe { rmdir(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && sr == 0 && (st.st_mode & S_IFMT) == S_IFDIR { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
