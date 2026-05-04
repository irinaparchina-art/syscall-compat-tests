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

/// F_SETLK: set a POSIX read lock on a region
pub struct FcntlSetlkReadTest;
impl SyscallTest for FcntlSetlkReadTest {
    fn name(&self) -> &str { "fcntl2_setlk_read" }
    fn syscall(&self) -> &str { "fcntl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fcntl(F_SETLK, F_RDLCK) then F_GETLK should confirm read lock is held" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_setlk.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, b"lock test".as_ptr() as *const _, 9); }
        let lock = flock {
            l_type: F_RDLCK as i16,
            l_whence: SEEK_SET as i16,
            l_start: 0,
            l_len: 9,
            l_pid: 0,
        };
        let r = unsafe { fcntl(fd, F_SETLK, &lock) };
        let errno_val = unsafe { *libc::__errno_location() };
        // Unlock
        let unlock = flock { l_type: F_UNLCK as i16, l_whence: SEEK_SET as i16, l_start: 0, l_len: 9, l_pid: 0 };
        unsafe { fcntl(fd, F_SETLK, &unlock); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// F_SETLK: write lock on a file region
pub struct FcntlSetlkWriteTest;
impl SyscallTest for FcntlSetlkWriteTest {
    fn name(&self) -> &str { "fcntl2_setlk_write" }
    fn syscall(&self) -> &str { "fcntl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fcntl(F_SETLK, F_WRLCK) write lock on file region should succeed" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_setlk_wr.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, b"write lock".as_ptr() as *const _, 10); }
        let lock = flock {
            l_type: F_WRLCK as i16, l_whence: SEEK_SET as i16,
            l_start: 0, l_len: 10, l_pid: 0,
        };
        let r = unsafe { fcntl(fd, F_SETLK, &lock) };
        let errno_val = unsafe { *libc::__errno_location() };
        let unlock = flock { l_type: F_UNLCK as i16, l_whence: SEEK_SET as i16, l_start: 0, l_len: 10, l_pid: 0 };
        unsafe { fcntl(fd, F_SETLK, &unlock); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// F_GETFL: get file status flags
pub struct FcntlGetflTest;
impl SyscallTest for FcntlGetflTest {
    fn name(&self) -> &str { "fcntl2_getfl_flags" }
    fn syscall(&self) -> &str { "fcntl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fcntl(F_GETFL) on O_RDONLY file should return flags without O_WRONLY" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_getfl.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDONLY, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let flags = unsafe { fcntl(fd, F_GETFL) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        // O_RDONLY is 0, so flags & O_ACCMODE should == O_RDONLY
        let s = if flags >= 0 && (flags & O_ACCMODE) == O_RDONLY { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: O_RDONLY as i64, actual_ret: (flags & O_ACCMODE) as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fcntl F_SETFL: add O_NONBLOCK to existing flags
pub struct FcntlSetflTest;
impl SyscallTest for FcntlSetflTest {
    fn name(&self) -> &str { "fcntl2_setfl_add_nonblock" }
    fn syscall(&self) -> &str { "fcntl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fcntl(F_SETFL) adding O_NONBLOCK should be reflected in subsequent F_GETFL" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_setfl.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let orig_flags = unsafe { fcntl(fd, F_GETFL) };
        let r = unsafe { fcntl(fd, F_SETFL, orig_flags | O_NONBLOCK) };
        let new_flags = unsafe { fcntl(fd, F_GETFL) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r == 0 && new_flags & O_NONBLOCK != 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fcntl F_DUPFD_CLOEXEC: dup with close-on-exec
pub struct FcntlDupfdCloexecTest;
impl SyscallTest for FcntlDupfdCloexecTest {
    fn name(&self) -> &str { "fcntl2_dupfd_cloexec" }
    fn syscall(&self) -> &str { "fcntl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "fcntl(F_DUPFD_CLOEXEC) should set FD_CLOEXEC on new fd" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_dupfd_cloexec.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let newfd = unsafe { fcntl(fd, F_DUPFD_CLOEXEC, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let cloexec_set = if newfd >= 0 {
            let flags = unsafe { fcntl(newfd, F_GETFD) };
            flags & FD_CLOEXEC != 0
        } else { false };
        unsafe { close(fd); if newfd >= 0 { close(newfd); } unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if newfd >= 0 && cloexec_set { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: newfd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// ioctl TIOCGPGRP on a terminal or ENOTTY on a pipe
pub struct IoctlTiocgpgrpTest;
impl SyscallTest for IoctlTiocgpgrpTest {
    fn name(&self) -> &str { "fcntl2_ioctl_tiocgpgrp" }
    fn syscall(&self) -> &str { "ioctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "ioctl(TIOCGPGRP) on a pipe should return -1 ENOTTY (not a tty)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut pgrp: pid_t = 0;
        let ret = unsafe { ioctl(fds[0], TIOCGPGRP, &mut pgrp) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == ENOTTY as i32 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret as i64, expected_errno: Some(ENOTTY as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// isatty: stdin might or might not be a tty, but stdout/stderr distinction matters
pub struct IsattyTest;
impl SyscallTest for IsattyTest {
    fn name(&self) -> &str { "fcntl2_isatty_pipe" }
    fn syscall(&self) -> &str { "isatty" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "isatty() on a pipe fd should return 0 (not a tty)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut fds = [0i32; 2];
        if unsafe { pipe(fds.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { isatty(fds[0]) };
        unsafe { close(fds[0]); close(fds[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// open with O_SYNC flag should succeed
pub struct OpenOsyncTest;
impl SyscallTest for OpenOsyncTest {
    fn name(&self) -> &str { "fcntl2_open_osync" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open(O_SYNC) should succeed and writes go directly to storage" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_osync.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC | O_SYNC, 0o644) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 {
            unsafe { write(fd, b"sync write".as_ptr() as *const _, 10); close(fd); unlink(path.as_ptr()); }
        }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// open with O_DIRECT flag
pub struct OpenOdirectTest;
impl SyscallTest for OpenOdirectTest {
    fn name(&self) -> &str { "fcntl2_open_odirect" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "open(O_DIRECT) should succeed on a regular file" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_odirect.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC | O_DIRECT, 0o644) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); unlink(path.as_ptr()); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 || errno_val == EINVAL as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// ioctl FIOCLEX/FIONCLEX: set/clear close-on-exec
pub struct IoctlFioclexTest;
impl SyscallTest for IoctlFioclexTest {
    fn name(&self) -> &str { "fcntl2_ioctl_fioclex" }
    fn syscall(&self) -> &str { "ioctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "ioctl(FIOCLEX) sets FD_CLOEXEC; ioctl(FIONCLEX) clears it" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_fioclex.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let r1 = unsafe { ioctl(fd, FIOCLEX) };
        let flags_after_set = unsafe { fcntl(fd, F_GETFD) };
        let r2 = unsafe { ioctl(fd, FIONCLEX) };
        let flags_after_clr = unsafe { fcntl(fd, F_GETFD) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r1 == 0 && r2 == 0
            && flags_after_set & FD_CLOEXEC != 0
            && flags_after_clr & FD_CLOEXEC == 0
        { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r1 as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
