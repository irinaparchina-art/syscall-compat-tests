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

/// pread64: read at offset without changing file position
pub struct Pread64OffsetTest;
impl SyscallTest for Pread64OffsetTest {
    fn name(&self) -> &str { "final_pread64_preserves_offset" }
    fn syscall(&self) -> &str { "pread" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "pread() at offset 10 should not change the file's current position" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_pread64.txt").unwrap();
        let data = b"0123456789ABCDEF";
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, data.as_ptr() as *const _, data.len()); }
        unsafe { lseek(fd, 3, SEEK_SET); } // position at 3
        let mut buf = [0u8; 4];
        unsafe { pread(fd, buf.as_mut_ptr() as *mut _, 4, 10) }; // read from 10
        let pos_after = unsafe { lseek(fd, 0, SEEK_CUR) };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        // position should still be 3
        let s = if pos_after == 3 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: pos_after as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// pwrite64: write at offset without changing file position
pub struct Pwrite64OffsetTest;
impl SyscallTest for Pwrite64OffsetTest {
    fn name(&self) -> &str { "final_pwrite64_preserves_offset" }
    fn syscall(&self) -> &str { "pwrite" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "pwrite() at offset 8 should not change the file's current position" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_pwrite64.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, b"AAAAAAAAAAAAAAAA".as_ptr() as *const _, 16); }
        unsafe { lseek(fd, 5, SEEK_SET); }
        unsafe { pwrite(fd, b"BBBB".as_ptr() as *const _, 4, 8) };
        let pos_after = unsafe { lseek(fd, 0, SEEK_CUR) };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if pos_after == 5 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 5, actual_ret: pos_after as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// write to /dev/null always succeeds and returns byte count
pub struct DevNullWriteTest;
impl SyscallTest for DevNullWriteTest {
    fn name(&self) -> &str { "final_dev_null_write" }
    fn syscall(&self) -> &str { "write" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "write() to /dev/null should return the requested byte count" }
    fn run(&self) -> TestResult {
        let path = CString::new("/dev/null").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_WRONLY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let data = vec![0u8; 4096];
        let n = unsafe { write(fd, data.as_ptr() as *const _, 4096) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 4096 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 4096, actual_ret: n as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// read from /dev/null always returns 0 (EOF)
pub struct DevNullReadTest;
impl SyscallTest for DevNullReadTest {
    fn name(&self) -> &str { "final_dev_null_read" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "read() from /dev/null should immediately return 0 (EOF)" }
    fn run(&self) -> TestResult {
        let path = CString::new("/dev/null").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut buf = [0u8; 64];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 64) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: n as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// /dev/zero: read returns zero bytes
pub struct DevZeroReadTest;
impl SyscallTest for DevZeroReadTest {
    fn name(&self) -> &str { "final_dev_zero_read" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "read() from /dev/zero should fill buffer with zeros" }
    fn run(&self) -> TestResult {
        let path = CString::new("/dev/zero").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut buf = [0xFFu8; 16];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 16) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 16 && buf.iter().all(|&b| b == 0) { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 16, actual_ret: n as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// Two files opened separately should have independent offsets
pub struct IndependentFileOffsetTest;
impl SyscallTest for IndependentFileOffsetTest {
    fn name(&self) -> &str { "final_independent_file_offsets" }
    fn syscall(&self) -> &str { "open/lseek" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "Two fds for same file have independent offsets" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_indep_offset.txt").unwrap();
        let start = Instant::now();
        let fd1 = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd1 < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd1, b"ABCDEFGH".as_ptr() as *const _, 8); }
        let fd2 = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        // Move fd1 to position 4
        unsafe { lseek(fd1, 4, SEEK_SET); }
        // fd2 should still be at 0
        let pos1 = unsafe { lseek(fd1, 0, SEEK_CUR) };
        let pos2 = unsafe { lseek(fd2, 0, SEEK_CUR) };
        unsafe { close(fd1); close(fd2); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if pos1 == 4 && pos2 == 0 { TestStatus::Pass }
        else { TestStatus::Error(format!("pos1={} pos2={}", pos1, pos2)) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// dup() fds share file offset
pub struct DupSharedOffsetTest;
impl SyscallTest for DupSharedOffsetTest {
    fn name(&self) -> &str { "final_dup_shared_offset" }
    fn syscall(&self) -> &str { "dup" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "dup() fds share the same file offset: seek on one affects the other" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_dup_offset.txt").unwrap();
        let start = Instant::now();
        let fd1 = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd1 < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd1, b"ABCDEFGH".as_ptr() as *const _, 8); }
        let fd2 = unsafe { dup(fd1) };
        // Seek fd1 to 4
        unsafe { lseek(fd1, 4, SEEK_SET); }
        // fd2 should also be at 4 (shared offset)
        let pos2 = unsafe { lseek(fd2, 0, SEEK_CUR) };
        unsafe { close(fd1); close(fd2); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if pos2 == 4 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 4, actual_ret: pos2 as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getpid() != getppid() in normal process
pub struct PidNotEqualPpidTest;
impl SyscallTest for PidNotEqualPpidTest {
    fn name(&self) -> &str { "final_pid_ne_ppid" }
    fn syscall(&self) -> &str { "getpid/getppid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getpid() should not equal getppid() in a normal process" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let pid = unsafe { getpid() };
        let ppid = unsafe { getppid() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if pid != ppid && pid > 0 && ppid > 0 { TestStatus::Pass }
        else { TestStatus::Error(format!("pid={} ppid={}", pid, ppid)) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// Verify SEEK_END works on a file with known content
pub struct SeekEndTest;
impl SyscallTest for SeekEndTest {
    fn name(&self) -> &str { "final_lseek_seek_end" }
    fn syscall(&self) -> &str { "lseek" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "lseek(SEEK_END, 0) on a 100-byte file should return 100" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_seekend100.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let data = vec![0u8; 100];
        unsafe { write(fd, data.as_ptr() as *const _, 100); }
        let pos = unsafe { lseek(fd, 0, SEEK_END) };
        unsafe { close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if pos == 100 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 100, actual_ret: pos as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// Multiple forks in sequence all succeed
pub struct MultiForkTest;
impl SyscallTest for MultiForkTest {
    fn name(&self) -> &str { "final_multi_fork_sequence" }
    fn syscall(&self) -> &str { "fork" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "5 sequential fork+wait pairs should all succeed" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut all_ok = true;
        for _ in 0..5 {
            let pid = unsafe { fork() };
            if pid < 0 { all_ok = false; break; }
            if pid == 0 { unsafe { libc::exit(0) }; }
            let mut st = 0i32;
            let r = unsafe { waitpid(pid, &mut st, 0) };
            if r != pid || !libc::WIFEXITED(st) || libc::WEXITSTATUS(st) != 0 {
                all_ok = false; break;
            }
        }
        let dur = start.elapsed().as_micros() as u64;
        let s = if all_ok { TestStatus::Pass }
        else { TestStatus::Error("one of 5 fork/wait cycles failed".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// stat st_nlink on a new file is 1
pub struct StatNlinkTest;
impl SyscallTest for StatNlinkTest {
    fn name(&self) -> &str { "final_stat_nlink_new_file" }
    fn syscall(&self) -> &str { "stat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "stat() on a newly created file should show st_nlink == 1" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_nlink1.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if fd >= 0 { unsafe { close(fd); } }
        let mut st: stat = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::stat(path.as_ptr(), &mut st) };
        unsafe { unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && st.st_nlink == 1 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: st.st_nlink as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// stat st_size reflects bytes written
pub struct StatSizeWrittenTest;
impl SyscallTest for StatSizeWrittenTest {
    fn name(&self) -> &str { "final_stat_size_after_write" }
    fn syscall(&self) -> &str { "stat" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "stat().st_size after writing 128 bytes should return 128" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_statsize.txt").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let data = vec![0u8; 128];
        unsafe { write(fd, data.as_ptr() as *const _, 128); close(fd); }
        let mut st: stat = unsafe { std::mem::zeroed() };
        unsafe { libc::stat(path.as_ptr(), &mut st); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if st.st_size == 128 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 128, actual_ret: st.st_size, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getuid() == geteuid() for non-setuid process
pub struct UidEqualsEuidTest;
impl SyscallTest for UidEqualsEuidTest {
    fn name(&self) -> &str { "final_uid_equals_euid" }
    fn syscall(&self) -> &str { "getuid/geteuid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "For a non-setuid process, getuid() should equal geteuid()" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let uid = unsafe { getuid() };
        let euid = unsafe { geteuid() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if uid == euid { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: uid as i64, actual_ret: euid as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
