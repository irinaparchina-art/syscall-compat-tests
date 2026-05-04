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

// ─── EBADF family ────────────────────────────────────────────────────────────

/// read from closed fd should give EBADF
pub struct ReadClosedFdTest;
impl SyscallTest for ReadClosedFdTest {
    fn name(&self) -> &str { "errno_read_closed_fd" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "read() on a closed fd (999) should return -1 EBADF" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut buf = [0u8; 8];
        let ret = unsafe { read(999, buf.as_mut_ptr() as *mut _, 8) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EBADF as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(EBADF as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// write to closed fd should give EBADF
pub struct WriteClosedFdTest;
impl SyscallTest for WriteClosedFdTest {
    fn name(&self) -> &str { "errno_write_closed_fd" }
    fn syscall(&self) -> &str { "write" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "write() to a closed fd (998) should return -1 EBADF" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { write(998, b"x".as_ptr() as *const _, 1) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EBADF as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(EBADF as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// lseek on closed fd should give EBADF
pub struct LseekClosedFdTest;
impl SyscallTest for LseekClosedFdTest {
    fn name(&self) -> &str { "errno_lseek_closed_fd" }
    fn syscall(&self) -> &str { "lseek" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "lseek() on a closed fd should return -1 EBADF" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { lseek(997, 0, SEEK_SET) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EBADF as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(EBADF as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

// ─── EINVAL family ───────────────────────────────────────────────────────────

/// mmap with bad prot flags should give EINVAL
pub struct MmapBadProtTest;
impl SyscallTest for MmapBadProtTest {
    fn name(&self) -> &str { "errno_mmap_bad_prot" }
    fn syscall(&self) -> &str { "mmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mmap() with PROT_READ|PROT_EXEC on zero-length should return MAP_FAILED EINVAL" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // length = 0 is always EINVAL
        let ptr = unsafe { mmap(std::ptr::null_mut(), 0, PROT_READ, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ptr == MAP_FAILED && errno_val == EINVAL as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ptr as i64, expected_errno: Some(EINVAL as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// kill with invalid signal number gives EINVAL
pub struct KillInvalidSigTest;
impl SyscallTest for KillInvalidSigTest {
    fn name(&self) -> &str { "errno_kill_invalid_signal" }
    fn syscall(&self) -> &str { "kill" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "kill(getpid(), 200) with out-of-range signal should give EINVAL" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let pid = unsafe { getpid() };
        let ret = unsafe { kill(pid, 200) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EINVAL as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(EINVAL as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fcntl with invalid command gives EINVAL
pub struct FcntlInvalidCmdTest;
impl SyscallTest for FcntlInvalidCmdTest {
    fn name(&self) -> &str { "errno_fcntl_invalid_cmd" }
    fn syscall(&self) -> &str { "fcntl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fcntl(fd, 9999) with invalid command should return -1 EINVAL" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { fcntl(1, 9999) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EINVAL as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(EINVAL as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

// ─── ENOENT / ENOTDIR family ─────────────────────────────────────────────────

/// open path with nonexistent parent dir gives ENOENT
pub struct OpenDeepNoentTest;
impl SyscallTest for OpenDeepNoentTest {
    fn name(&self) -> &str { "errno_open_deep_noent" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open(\"/tmp/no_such_dir/file\") should return -1 ENOENT" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_no_such_dir_xyz/file.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd == -1 && errno_val == ENOENT as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd as i64, expected_errno: Some(ENOENT as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// rmdir on non-empty directory should give ENOTEMPTY
pub struct RmdirNotemptyTest;
impl SyscallTest for RmdirNotemptyTest {
    fn name(&self) -> &str { "errno_rmdir_notempty" }
    fn syscall(&self) -> &str { "rmdir" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "rmdir() on a non-empty directory should return -1 ENOTEMPTY" }
    fn run(&self) -> TestResult {
        let dir = CString::new("/tmp/sct_notempty_dir").unwrap();
        let file = CString::new("/tmp/sct_notempty_dir/file.txt").unwrap();
        let start = Instant::now();
        unsafe { mkdir(dir.as_ptr(), 0o755); }
        let fd = unsafe { open(file.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        let ret = unsafe { rmdir(dir.as_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        // Cleanup
        unsafe { unlink(file.as_ptr()); rmdir(dir.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == ENOTEMPTY as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret as i64, expected_errno: Some(ENOTEMPTY as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// open path where component is a file, not dir, gives ENOTDIR
pub struct OpenEnotdirTest;
impl SyscallTest for OpenEnotdirTest {
    fn name(&self) -> &str { "errno_open_enotdir" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open(\"/tmp/regular_file/sub\") where component is a file gives ENOTDIR" }
    fn run(&self) -> TestResult {
        let file = CString::new("/tmp/sct_notdir_file.txt").unwrap();
        let path = CString::new("/tmp/sct_notdir_file.txt/sub.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(file.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        let fd2 = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { unlink(file.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd2 == -1 && errno_val == ENOTDIR as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd2 as i64, expected_errno: Some(ENOTDIR as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

// ─── EPIPE / ECONNRESET ───────────────────────────────────────────────────────

/// write to pipe after read end closed gives EPIPE (with SIGPIPE ignored)
pub struct WritePipeEpipeTest;
impl SyscallTest for WritePipeEpipeTest {
    fn name(&self) -> &str { "errno_write_pipe_epipe" }
    fn syscall(&self) -> &str { "write" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "write() to pipe after read-end is closed should return -1 EPIPE (SIGPIPE ignored)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        // Ignore SIGPIPE so write returns EPIPE instead of terminating
        let sa = sigaction {
            sa_sigaction: SIG_IGN,
            sa_mask: unsafe { std::mem::zeroed() },
            sa_flags: 0,
            sa_restorer: None,
        };
        unsafe { sigaction(SIGPIPE, &sa, std::ptr::null_mut()); }
        unsafe { close(fds[0]); } // close read end
        let ret = unsafe { write(fds[1], b"x".as_ptr() as *const _, 1) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == EPIPE as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret, expected_errno: Some(EPIPE as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

// ─── EACCES family ───────────────────────────────────────────────────────────

/// open /root should give EACCES for non-root
pub struct OpenEaccesTest;
impl SyscallTest for OpenEaccesTest {
    fn name(&self) -> &str { "errno_open_eacces_root" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open(\"/root/sct_test.txt\", O_CREAT) as non-root should give EACCES" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let uid = unsafe { getuid() };
        if uid == 0 {
            // Running as root - skip
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let path = CString::new("/root/sct_eacces_test.txt").unwrap();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY, 0o644) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd == -1 && errno_val == EACCES as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd as i64, expected_errno: Some(EACCES as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// chmod a file to 000 then open it should give EACCES (non-root)
pub struct ChmodEaccesTest;
impl SyscallTest for ChmodEaccesTest {
    fn name(&self) -> &str { "errno_chmod_eacces_open" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "chmod(000) then open(O_RDONLY) as non-root should give EACCES" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_chmod000.txt").unwrap();
        let start = Instant::now();
        let uid = unsafe { getuid() };
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        unsafe { chmod(path.as_ptr(), 0o000); }
        let fd2 = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        // Restore and cleanup
        unsafe { chmod(path.as_ptr(), 0o644); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        // Root can open anything
        let s = if uid == 0 { TestStatus::Unimplemented }
        else if fd2 == -1 && errno_val == EACCES as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd2 as i64, expected_errno: Some(EACCES as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

// ─── EISDIR / ELOOP ──────────────────────────────────────────────────────────

/// open a directory with O_WRONLY should give EISDIR
pub struct OpenIsdirTest;
impl SyscallTest for OpenIsdirTest {
    fn name(&self) -> &str { "errno_open_eisdir" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open(\"/tmp\", O_WRONLY) should return -1 EISDIR" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_WRONLY, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd == -1 && errno_val == EISDIR as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd as i64, expected_errno: Some(EISDIR as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// symlink loop should give ELOOP
pub struct SymlinkLoopTest;
impl SyscallTest for SymlinkLoopTest {
    fn name(&self) -> &str { "errno_symlink_loop" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open() on a symlink loop should return -1 ELOOP" }
    fn run(&self) -> TestResult {
        let a = CString::new("/tmp/sct_loop_a").unwrap();
        let b = CString::new("/tmp/sct_loop_b").unwrap();
        let start = Instant::now();
        unsafe { unlink(a.as_ptr()); unlink(b.as_ptr()); }
        // a -> b -> a
        unsafe { symlink(b.as_ptr(), a.as_ptr()); symlink(a.as_ptr(), b.as_ptr()); }
        let fd = unsafe { open(a.as_ptr(), O_RDONLY, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); } }
        unsafe { unlink(a.as_ptr()); unlink(b.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd == -1 && errno_val == ELOOP as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd as i64, expected_errno: Some(ELOOP as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

// ─── ENAMETOOLONG ────────────────────────────────────────────────────────────

/// open with a path longer than PATH_MAX should give ENAMETOOLONG
pub struct OpenNametoolongTest;
impl SyscallTest for OpenNametoolongTest {
    fn name(&self) -> &str { "errno_open_enametoolong" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open() with path > PATH_MAX bytes should return -1 ENAMETOOLONG" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let long_name: String = std::iter::repeat('a').take(4097).collect();
        let path = CString::new(long_name).unwrap();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd == -1 && errno_val == ENAMETOOLONG as i32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd as i64, expected_errno: Some(ENAMETOOLONG as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

// ─── Boundary conditions ──────────────────────────────────────────────────────

/// write 0 bytes should return 0
pub struct WriteZeroBytesTest;
impl SyscallTest for WriteZeroBytesTest {
    fn name(&self) -> &str { "errno_write_zero_bytes" }
    fn syscall(&self) -> &str { "write" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "write(fd, buf, 0) should return 0 (no-op)" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_write0.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { write(fd, b"".as_ptr() as *const _, 0) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// lseek SEEK_END on an empty file should return 0
pub struct LseekEndEmptyFileTest;
impl SyscallTest for LseekEndEmptyFileTest {
    fn name(&self) -> &str { "errno_lseek_end_empty_file" }
    fn syscall(&self) -> &str { "lseek" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "lseek(fd, 0, SEEK_END) on an empty file should return 0" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_seekend.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { lseek(fd, 0, SEEK_END) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// stat on a symlink follows it; lstat does not
pub struct StatVsLstatTest;
impl SyscallTest for StatVsLstatTest {
    fn name(&self) -> &str { "errno_stat_vs_lstat_symlink" }
    fn syscall(&self) -> &str { "stat/lstat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "stat() follows symlinks; lstat() does not — modes should differ" }
    fn run(&self) -> TestResult {
        let target = CString::new("/tmp/sct_slstat_tgt.txt").unwrap();
        let link = CString::new("/tmp/sct_slstat_lnk").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(target.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        unsafe { unlink(link.as_ptr()); symlink(target.as_ptr(), link.as_ptr()); }
        let mut st_stat: stat = unsafe { std::mem::zeroed() };
        let mut st_lstat: stat = unsafe { std::mem::zeroed() };
        let r1 = unsafe { libc::stat(link.as_ptr(), &mut st_stat) };
        let r2 = unsafe { lstat(link.as_ptr(), &mut st_lstat) };
        unsafe { unlink(link.as_ptr()); unlink(target.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        // stat should see regular file; lstat should see symlink
        let stat_is_reg = (st_stat.st_mode & S_IFMT) == S_IFREG;
        let lstat_is_lnk = (st_lstat.st_mode & S_IFMT) == S_IFLNK;
        let s = if r1 == 0 && r2 == 0 && stat_is_reg && lstat_is_lnk { TestStatus::Pass }
        else { TestStatus::Error(format!("stat_mode={:o} lstat_mode={:o}", st_stat.st_mode, st_lstat.st_mode)) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fstat and stat on same path should agree on size
pub struct FstatStatConsistencyTest;
impl SyscallTest for FstatStatConsistencyTest {
    fn name(&self) -> &str { "errno_fstat_stat_consistency" }
    fn syscall(&self) -> &str { "fstat/stat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fstat() and stat() on same file should return identical st_size" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_fstat_stat.txt").unwrap();
        let data = b"consistency check data";
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, data.as_ptr() as *const _, data.len()); }
        let mut fst: stat = unsafe { std::mem::zeroed() };
        let mut pst: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::fstat(fd, &mut fst); libc::stat(path.as_ptr(), &mut pst); }
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fst.st_size == pst.st_size && fst.st_size == data.len() as i64 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: data.len() as i64, actual_ret: fst.st_size, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// read returns 0 at EOF
pub struct ReadEofTest;
impl SyscallTest for ReadEofTest {
    fn name(&self) -> &str { "errno_read_at_eof" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "read() at EOF should return 0" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_eof.txt").unwrap();
        let data = b"EOF";
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, data.as_ptr() as *const _, data.len()); lseek(fd, 0, SEEK_SET); }
        let mut buf = [0u8; 16];
        let r1 = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 16) }; // reads "EOF"
        let r2 = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 16) }; // should return 0
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r1 == data.len() as isize && r2 == 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r2 as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
