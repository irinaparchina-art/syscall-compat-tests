use std::ffi::CString;
use std::time::Instant;

use libc::*;

use crate::runner::{SyscallCategory, SyscallTest, TestResult, TestStatus};

/// Helper: run a closure, capture return value and errno, measure time.
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

// ─── open ────────────────────────────────────────────────────────────────────

pub struct OpenBasicTest;
impl SyscallTest for OpenBasicTest {
    fn name(&self) -> &str { "file_open_basic" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "open() a new file with O_CREAT|O_WRONLY should return a non-negative fd"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_open_basic.txt").unwrap();
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) as i64 }
        });
        if ret >= 0 {
            unsafe { close(ret as i32) };
            unsafe { unlink(path.as_ptr()) };
        }
        let status = if ret >= 0 {
            TestStatus::Pass
        } else if -errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 3, // any positive fd
                actual_ret: ret,
                expected_errno: None,
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct OpenNonexistentTest;
impl SyscallTest for OpenNonexistentTest {
    fn name(&self) -> &str { "file_open_nonexistent" }
    fn syscall(&self) -> &str { "open" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "open() a nonexistent file without O_CREAT should return -1 with ENOENT"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_this_does_not_exist_xyz.txt").unwrap();
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { open(path.as_ptr(), O_RDONLY) as i64 }
        });
        let status = if ret == -1 && errno_val == ENOENT as i32 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1,
                actual_ret: ret,
                expected_errno: Some(ENOENT as i32),
                actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── read / write ─────────────────────────────────────────────────────────────

pub struct ReadWriteTest;
impl SyscallTest for ReadWriteTest {
    fn name(&self) -> &str { "file_read_write_basic" }
    fn syscall(&self) -> &str { "read/write" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "write() some bytes to a file then read() them back; data must match"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_rw_test.txt").unwrap();
        let data = b"hello syscall-compat-tests\n";
        let start = Instant::now();

        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            let e = unsafe { *libc::__errno_location() };
            if e == ENOSYS as i32 {
                return make_result(self.name(), self.syscall(), self.category(), self.description(),
                    TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
            }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error(format!("open failed: errno={}", e)),
                start.elapsed().as_micros() as u64);
        }

        let w = unsafe { write(fd, data.as_ptr() as *const _, data.len()) };
        if w != data.len() as isize {
            unsafe { close(fd); unlink(path.as_ptr()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Fail {
                    expected_ret: data.len() as i64, actual_ret: w as i64,
                    expected_errno: None, actual_errno: None
                }, start.elapsed().as_micros() as u64);
        }

        unsafe { lseek(fd, 0, SEEK_SET) };
        let mut buf = vec![0u8; data.len()];
        let r = unsafe { read(fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        unsafe { close(fd); unlink(path.as_ptr()); }

        let dur = start.elapsed().as_micros() as u64;
        let status = if r == data.len() as isize && &buf[..] == data {
            TestStatus::Pass
        } else {
            TestStatus::Fail {
                expected_ret: data.len() as i64, actual_ret: r as i64,
                expected_errno: None, actual_errno: None,
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct ReadZeroTest;
impl SyscallTest for ReadZeroTest {
    fn name(&self) -> &str { "file_read_zero_bytes" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "read() 0 bytes from a valid fd should return 0 immediately"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_read_zero.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDONLY, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let mut buf = [0u8; 0];
        let ret = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 0) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        unsafe { close(fd as i32); unlink(path.as_ptr()); }
        let (ret, errno_val, dur) = (ret, errno_val, dur);
        let status = if ret == 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0, actual_ret: ret,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct WriteAppendTest;
impl SyscallTest for WriteAppendTest {
    fn name(&self) -> &str { "file_write_append" }
    fn syscall(&self) -> &str { "write" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "open() with O_APPEND: writes should always go to end of file"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_append.txt").unwrap();
        let start = Instant::now();

        let fd1 = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd1 < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("initial open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd1, b"hello\n".as_ptr() as *const _, 6); close(fd1); }

        let fd2 = unsafe { open(path.as_ptr(), O_WRONLY | O_APPEND, 0o644) };
        if fd2 < 0 {
            unsafe { unlink(path.as_ptr()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd2, b"world\n".as_ptr() as *const _, 6); close(fd2); }

        // Verify size is 12
        let mut st: stat = unsafe { std::mem::zeroed() };
        let r = unsafe { libc::stat(path.as_ptr(), &mut st) };
        unsafe { unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;

        let status = if r == 0 && st.st_size == 12 {
            TestStatus::Pass
        } else {
            TestStatus::Fail {
                expected_ret: 12, actual_ret: if r == 0 { st.st_size } else { -1 },
                expected_errno: None, actual_errno: None,
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── close ───────────────────────────────────────────────────────────────────

pub struct CloseInvalidFdTest;
impl SyscallTest for CloseInvalidFdTest {
    fn name(&self) -> &str { "file_close_invalid_fd" }
    fn syscall(&self) -> &str { "close" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "close(-1) should return -1 with EBADF"
    }
    fn run(&self) -> TestResult {
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { close(-1) as i64 }
        });
        let status = if ret == -1 && errno_val == EBADF as i32 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1, actual_ret: ret,
                expected_errno: Some(EBADF as i32), actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── stat ────────────────────────────────────────────────────────────────────

pub struct StatBasicTest;
impl SyscallTest for StatBasicTest {
    fn name(&self) -> &str { "file_stat_basic" }
    fn syscall(&self) -> &str { "stat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "stat() on an existing file should succeed and report correct size"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_stat_test.txt").unwrap();
        let data = b"stat test data";
        let start = Instant::now();

        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, data.as_ptr() as *const _, data.len()); close(fd); }

        let mut st: stat = unsafe { std::mem::zeroed() };
        let r = unsafe { libc::stat(path.as_ptr(), &mut st) };
        unsafe { unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;

        let status = if r == 0 && st.st_size == data.len() as i64 {
            TestStatus::Pass
        } else if r == -1 && unsafe { *libc::__errno_location() } == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: data.len() as i64,
                actual_ret: if r == 0 { st.st_size } else { -1 },
                expected_errno: None, actual_errno: None,
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct StatNonexistentTest;
impl SyscallTest for StatNonexistentTest {
    fn name(&self) -> &str { "file_stat_nonexistent" }
    fn syscall(&self) -> &str { "stat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "stat() on a nonexistent path should return -1 with ENOENT"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_no_such_file_xyz.txt").unwrap();
        let mut st: stat = unsafe { std::mem::zeroed() };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { libc::stat(path.as_ptr(), &mut st) as i64 }
        });
        let status = if ret == -1 && errno_val == ENOENT as i32 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1, actual_ret: ret,
                expected_errno: Some(ENOENT as i32), actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── lseek ───────────────────────────────────────────────────────────────────

pub struct LseekBasicTest;
impl SyscallTest for LseekBasicTest {
    fn name(&self) -> &str { "file_lseek_basic" }
    fn syscall(&self) -> &str { "lseek" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "lseek() SEEK_SET to offset 0 on an open file should return 0"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_lseek.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, b"abcdef".as_ptr() as *const _, 6); }
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { lseek(fd, 0, SEEK_SET) as i64 }
        });
        unsafe { close(fd); unlink(path.as_ptr()); }
        let status = if ret == 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0, actual_ret: ret,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct LseekInvalidTest;
impl SyscallTest for LseekInvalidTest {
    fn name(&self) -> &str { "file_lseek_invalid_whence" }
    fn syscall(&self) -> &str { "lseek" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "lseek() with invalid whence (99) should return -1 with EINVAL"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_lseek_inv.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { lseek(fd, 0, 99) as i64 }
        });
        unsafe { close(fd); unlink(path.as_ptr()); }
        let status = if ret == -1 && errno_val == EINVAL as i32 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: -1, actual_ret: ret,
                expected_errno: Some(EINVAL as i32), actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── unlink / rename ─────────────────────────────────────────────────────────

pub struct UnlinkTest;
impl SyscallTest for UnlinkTest {
    fn name(&self) -> &str { "file_unlink_basic" }
    fn syscall(&self) -> &str { "unlink" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "unlink() an existing file should succeed; subsequent stat() should give ENOENT"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_unlink.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { close(fd); }
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { unlink(path.as_ptr()) as i64 }
        });
        let status = if ret == 0 {
            // Verify file is gone
            let mut st: stat = unsafe { std::mem::zeroed() };
            let sr = unsafe { libc::stat(path.as_ptr(), &mut st) };
            let se = unsafe { *libc::__errno_location() };
            if sr == -1 && se == ENOENT as i32 { TestStatus::Pass }
            else { TestStatus::Error("file still exists after unlink".into()) }
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0, actual_ret: ret,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct RenameTest;
impl SyscallTest for RenameTest {
    fn name(&self) -> &str { "file_rename_basic" }
    fn syscall(&self) -> &str { "rename" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "rename() an existing file to a new name should succeed"
    }
    fn run(&self) -> TestResult {
        let src = CString::new("/tmp/sct_rename_src.txt").unwrap();
        let dst = CString::new("/tmp/sct_rename_dst.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(src.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { close(fd); }
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { rename(src.as_ptr(), dst.as_ptr()) as i64 }
        });
        unsafe { unlink(dst.as_ptr()); }
        let status = if ret == 0 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0, actual_ret: ret,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── mkdir / rmdir ───────────────────────────────────────────────────────────

pub struct MkdirRmdirTest;
impl SyscallTest for MkdirRmdirTest {
    fn name(&self) -> &str { "file_mkdir_rmdir" }
    fn syscall(&self) -> &str { "mkdir/rmdir" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "mkdir() then rmdir() on a new directory should both succeed"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_testdir_xyz").unwrap();
        let start = Instant::now();
        let r1 = unsafe { mkdir(path.as_ptr(), 0o755) };
        let e1 = unsafe { *libc::__errno_location() };
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { rmdir(path.as_ptr()) as i64 }
        });
        let status = if r1 == 0 && ret == 0 {
            TestStatus::Pass
        } else if e1 == ENOSYS as i32 || errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 0,
                actual_ret: if r1 != 0 { r1 as i64 } else { ret },
                expected_errno: None,
                actual_errno: Some(if r1 != 0 { e1 } else { errno_val }),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── getcwd ──────────────────────────────────────────────────────────────────

pub struct GetcwdTest;
impl SyscallTest for GetcwdTest {
    fn name(&self) -> &str { "file_getcwd_basic" }
    fn syscall(&self) -> &str { "getcwd" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "getcwd() should return a non-empty path starting with '/'"
    }
    fn run(&self) -> TestResult {
        let mut buf = vec![0u8; 4096];
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { getcwd(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) as i64 }
        });
        let status = if ret != 0 && buf[0] == b'/' {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: 1, actual_ret: ret,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

// ─── dup / dup2 ──────────────────────────────────────────────────────────────

pub struct DupTest;
impl SyscallTest for DupTest {
    fn name(&self) -> &str { "file_dup_basic" }
    fn syscall(&self) -> &str { "dup" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "dup() a valid fd should return a new non-negative fd"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_dup.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { dup(fd) as i64 }
        });
        unsafe { close(fd); if ret >= 0 { close(ret as i32); } unlink(path.as_ptr()); }
        let status = if ret >= 0 && ret != fd as i64 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: fd as i64 + 1, actual_ret: ret,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct Dup2Test;
impl SyscallTest for Dup2Test {
    fn name(&self) -> &str { "file_dup2_basic" }
    fn syscall(&self) -> &str { "dup2" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str {
        "dup2(fd, newfd) should return newfd"
    }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_dup2.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let newfd = 50i32;
        let (ret, errno_val, dur) = timed_syscall!({
            unsafe { dup2(fd, newfd) as i64 }
        });
        unsafe { close(fd); if ret >= 0 { close(newfd); } unlink(path.as_ptr()); }
        let status = if ret == newfd as i64 {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 {
            TestStatus::Unimplemented
        } else {
            TestStatus::Fail {
                expected_ret: newfd as i64, actual_ret: ret,
                expected_errno: None, actual_errno: Some(errno_val),
            }
        };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}
