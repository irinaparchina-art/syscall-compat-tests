use std::ffi::CString;
use std::time::Instant;

use libc::*;

use crate::runner::{SyscallCategory, SyscallTest, TestResult, TestStatus};

macro_rules! timed_syscall {
    ($body:expr) => {{
        let start = Instant::now();
        let ret: i64 = $body;
        let errno_val: i32 = unsafe { *libc::__errno_location() };
        let duration_us = start.elapsed().as_micros() as u64;
        (ret, errno_val, duration_us)
    }};
}

fn make_result(
    name: &str,
    syscall: &str,
    category: SyscallCategory,
    description: &str,
    status: TestStatus,
    duration_us: u64,
) -> TestResult {
    TestResult {
        name: name.to_string(),
        syscall: syscall.to_string(),
        category,
        status,
        description: description.to_string(),
        duration_us,
    }
}

// ─── umask ──────────────────────────────────────────────────────────────────

pub struct UmaskGetTest;
impl SyscallTest for UmaskGetTest {
    fn name(&self) -> &str { "umask_get" }
    fn syscall(&self) -> &str { "umask" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "umask() should return the previous umask value"
    }
    fn run(&self) -> TestResult {
        let (ret, _errno_val, dur) = timed_syscall!({
            let old = unsafe { umask(0o022) };
            unsafe { umask(old) };
            old as i64
        });
        let status = if ret >= 0 && ret <= 0o777 {
            TestStatus::Pass
        } else {
            TestStatus::Fail {
                expected_ret: 0o022,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: None,
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct UmaskRoundtripTest;
impl SyscallTest for UmaskRoundtripTest {
    fn name(&self) -> &str { "umask_roundtrip" }
    fn syscall(&self) -> &str { "umask" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "Setting umask to 0o077 and back should correctly roundtrip"
    }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let old = unsafe { umask(0o077) };
        let got = unsafe { umask(old) };
        let dur = start.elapsed().as_micros() as u64;

        let status = if got == 0o077 {
            TestStatus::Pass
        } else {
            TestStatus::Fail {
                expected_ret: 0o077,
                actual_ret: got as i64,
                expected_errno: None,
                actual_errno: None,
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── dup3 ───────────────────────────────────────────────────────────────────

pub struct Dup3BasicTest;
impl SyscallTest for Dup3BasicTest {
    fn name(&self) -> &str { "dup3_basic" }
    fn syscall(&self) -> &str { "dup3" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "dup3(fd, newfd, 0) should duplicate fd to newfd"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_dup3_test.txt").unwrap();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("failed to create test file".into()), 0);
        }

        let newfd = 200;
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { dup3(fd, newfd, 0) as i64 }
        });

        unsafe { close(fd); close(newfd); unlink(path.as_ptr()); }

        let status = if ret == newfd as i64 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: newfd as i64,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct Dup3CloexecTest;
impl SyscallTest for Dup3CloexecTest {
    fn name(&self) -> &str { "dup3_cloexec" }
    fn syscall(&self) -> &str { "dup3" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "dup3(fd, newfd, O_CLOEXEC) should set close-on-exec on newfd"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_dup3_cloexec.txt").unwrap();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("failed to create test file".into()), 0);
        }

        let newfd = 201;
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { dup3(fd, newfd, O_CLOEXEC) as i64 }
        });

        let status = if ret == newfd as i64 {
            let flags = unsafe { fcntl(newfd, F_GETFD) };
            if flags & FD_CLOEXEC != 0 {
                TestStatus::Pass
            } else {
                TestStatus::Fail {
                    expected_ret: 1,
                    actual_ret: 0,
                    expected_errno: None,
                    actual_errno: None,
                }
            }
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: newfd as i64,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };

        unsafe { close(fd); close(newfd); unlink(path.as_ptr()); }
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct Dup3SameFdTest;
impl SyscallTest for Dup3SameFdTest {
    fn name(&self) -> &str { "dup3_same_fd" }
    fn syscall(&self) -> &str { "dup3" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "dup3(fd, fd, 0) where oldfd == newfd should return EINVAL"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_dup3_same.txt").unwrap();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("failed to create test file".into()), 0);
        }

        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { dup3(fd, fd, 0) as i64 }
        });

        unsafe { close(fd); unlink(path.as_ptr()); }

        let status = if ret == -1 && errno_val == EINVAL {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1,
                actual_ret: ret,
                expected_errno: Some(EINVAL),
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── chown / fchown ─────────────────────────────────────────────────────────

pub struct ChownSelfTest;
impl SyscallTest for ChownSelfTest {
    fn name(&self) -> &str { "chown_self" }
    fn syscall(&self) -> &str { "chown" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "chown() to current uid/gid should succeed"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_chown_test.txt").unwrap();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("failed to create test file".into()), 0);
        }
        unsafe { close(fd) };

        let uid = unsafe { getuid() };
        let gid = unsafe { getgid() };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { chown(path.as_ptr(), uid, gid) as i64 }
        });

        unsafe { unlink(path.as_ptr()) };

        let status = if ret == 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct FchownSelfTest;
impl SyscallTest for FchownSelfTest {
    fn name(&self) -> &str { "fchown_self" }
    fn syscall(&self) -> &str { "fchown" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "fchown(fd, uid, gid) with current uid/gid should succeed"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_fchown_test.txt").unwrap();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("failed to create test file".into()), 0);
        }

        let uid = unsafe { getuid() };
        let gid = unsafe { getgid() };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { fchown(fd, uid, gid) as i64 }
        });

        unsafe { close(fd); unlink(path.as_ptr()); }

        let status = if ret == 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct ChownNoentTest;
impl SyscallTest for ChownNoentTest {
    fn name(&self) -> &str { "chown_noent" }
    fn syscall(&self) -> &str { "chown" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "chown() on a nonexistent path should return ENOENT"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_chown_noent_xyz.txt").unwrap();
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { chown(path.as_ptr(), 0, 0) as i64 }
        });
        let status = if ret == -1 && errno_val == ENOENT {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1,
                actual_ret: ret,
                expected_errno: Some(ENOENT),
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── faccessat ──────────────────────────────────────────────────────────────

pub struct FaccessatReadTest;
impl SyscallTest for FaccessatReadTest {
    fn name(&self) -> &str { "faccessat_read" }
    fn syscall(&self) -> &str { "faccessat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "faccessat(AT_FDCWD, '/tmp', R_OK, 0) should succeed"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp").unwrap();
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { faccessat(AT_FDCWD, path.as_ptr(), R_OK, 0) as i64 }
        });
        let status = if ret == 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct FaccessatNoentTest;
impl SyscallTest for FaccessatNoentTest {
    fn name(&self) -> &str { "faccessat_noent" }
    fn syscall(&self) -> &str { "faccessat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "faccessat() on a non-existent path should return ENOENT"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_faccessat_noent_xyz").unwrap();
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { faccessat(AT_FDCWD, path.as_ptr(), F_OK, 0) as i64 }
        });
        let status = if ret == -1 && errno_val == ENOENT {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1,
                actual_ret: ret,
                expected_errno: Some(ENOENT),
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── fstatat ────────────────────────────────────────────────────────────────

pub struct FstatatBasicTest;
impl SyscallTest for FstatatBasicTest {
    fn name(&self) -> &str { "fstatat_basic" }
    fn syscall(&self) -> &str { "fstatat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "fstatat(AT_FDCWD, path, &stat, 0) should succeed on an existing file"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_fstatat_test.txt").unwrap();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 {
            unsafe { write(fd, b"test".as_ptr() as *const _, 4) };
            unsafe { close(fd) };
        }

        let mut st: stat = unsafe { std::mem::zeroed() };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { fstatat(AT_FDCWD, path.as_ptr(), &mut st, 0) as i64 }
        });

        unsafe { unlink(path.as_ptr()) };

        let status = if ret == 0 && st.st_size == 4 {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0,
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct FstatatNoentTest;
impl SyscallTest for FstatatNoentTest {
    fn name(&self) -> &str { "fstatat_noent" }
    fn syscall(&self) -> &str { "fstatat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "fstatat() on a non-existent path should return ENOENT"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_fstatat_noent_xyz.txt").unwrap();
        let mut st: stat = unsafe { std::mem::zeroed() };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { fstatat(AT_FDCWD, path.as_ptr(), &mut st, 0) as i64 }
        });
        let status = if ret == -1 && errno_val == ENOENT {
            TestStatus::Pass
        } else if errno_val == ENOSYS {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1,
                actual_ret: ret,
                expected_errno: Some(ENOENT),
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}
