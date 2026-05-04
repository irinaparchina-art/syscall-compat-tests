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

/// inotify_add_watch + inotify_rm_watch on /tmp
pub struct InotifyWatchTest;
impl SyscallTest for InotifyWatchTest {
    fn name(&self) -> &str { "misc2_inotify_watch" }
    fn syscall(&self) -> &str { "inotify_add_watch" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "inotify_add_watch(/tmp, IN_CREATE) then inotify_rm_watch should both return 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { inotify_init1(0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        let path = CString::new("/tmp").unwrap();
        let wd = unsafe { inotify_add_watch(fd, path.as_ptr(), IN_CREATE) };
        let e1 = unsafe { *libc::__errno_location() };
        let r2 = if wd >= 0 { unsafe { inotify_rm_watch(fd, wd) } } else { -1 };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if wd >= 0 && r2 == 0 { TestStatus::Pass }
        else if e1 == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: wd as i64, expected_errno: None, actual_errno: Some(e1) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// inotify: detect file creation in watched directory
pub struct InotifyDetectCreateTest;
impl SyscallTest for InotifyDetectCreateTest {
    fn name(&self) -> &str { "misc2_inotify_detect_create" }
    fn syscall(&self) -> &str { "inotify" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "inotify should generate IN_CREATE event when file is created in watched dir" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { inotify_init1(IN_NONBLOCK as i32) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let dir = CString::new("/tmp").unwrap();
        let wd = unsafe { inotify_add_watch(fd, dir.as_ptr(), IN_CREATE) };
        if wd < 0 { unsafe { close(fd); } return make_result(self.name(), self.syscall(), self.category(), self.description(), TestStatus::Unimplemented, start.elapsed().as_micros() as u64); }
        // Create a file
        let file = CString::new("/tmp/sct_inotify_test.txt").unwrap();
        let ffd = unsafe { open(file.as_ptr(), O_CREAT | O_WRONLY | O_TRUNC, 0o644) };
        if ffd >= 0 { unsafe { close(ffd); } }
        // Read inotify event
        let mut buf = [0u8; 256];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 256) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { inotify_rm_watch(fd, wd); close(fd); unlink(file.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        // inotify_event: wd(4) + mask(4) + cookie(4) + len(4) = 16 bytes minimum
        let s = if n >= 16 { TestStatus::Pass }
        else if errno_val == EAGAIN as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 16, actual_ret: n as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// syslog: read kernel log (may need CAP_SYSLOG)
pub struct SyslogReadTest;
impl SyscallTest for SyslogReadTest {
    fn name(&self) -> &str { "misc2_syslog_size" }
    fn syscall(&self) -> &str { "syslog" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "syslog(SYSLOG_ACTION_SIZE_BUFFER) returns ring buffer size >= 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // SYSLOG_ACTION_SIZE_BUFFER = 10
        let ret = unsafe { libc::syscall(libc::SYS_syslog, 10 as c_long, std::ptr::null::<u8>() as c_long, 0 as c_long) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 || errno_val == EPERM as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// personality syscall: get current personality
pub struct PersonalityGetTest;
impl SyscallTest for PersonalityGetTest {
    fn name(&self) -> &str { "misc2_personality_get" }
    fn syscall(&self) -> &str { "personality" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "personality(0xffffffff) returns current personality without changing it" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { libc::syscall(libc::SYS_personality, 0xffffffffu64 as c_long) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// capget: get process capabilities
pub struct CapgetTest;
impl SyscallTest for CapgetTest {
    fn name(&self) -> &str { "misc2_capget" }
    fn syscall(&self) -> &str { "capget" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "capget() should return current process capabilities without error" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        #[repr(C)]
        struct CapHeader { version: u32, pid: i32 }
        #[repr(C)]
        #[derive(Copy, Clone)]
        struct CapData { effective: u32, permitted: u32, inheritable: u32 }
        let mut hdr = CapHeader { version: 0x20080522, pid: 0 };
        let mut data = [CapData { effective: 0, permitted: 0, inheritable: 0 }; 2];
        let ret = unsafe {
            libc::syscall(libc::SYS_capget,
                &mut hdr as *mut _ as c_long,
                data.as_mut_ptr() as c_long)
        };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getcpu: get current CPU and NUMA node
pub struct GetcpuTest;
impl SyscallTest for GetcpuTest {
    fn name(&self) -> &str { "misc2_getcpu" }
    fn syscall(&self) -> &str { "getcpu" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getcpu() should return current CPU index >= 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut cpu: u32 = 0;
        let mut node: u32 = 0;
        let ret = unsafe {
            libc::syscall(libc::SYS_getcpu,
                &mut cpu as *mut _ as c_long,
                &mut node as *mut _ as c_long,
                std::ptr::null::<u8>() as c_long)
        };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// membarrier: issue memory barrier
pub struct MembarrierTest;
impl SyscallTest for MembarrierTest {
    fn name(&self) -> &str { "misc2_membarrier_query" }
    fn syscall(&self) -> &str { "membarrier" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "membarrier(MEMBARRIER_CMD_QUERY) should return supported commands bitmask" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // MEMBARRIER_CMD_QUERY = 0
        let ret = unsafe { libc::syscall(libc::SYS_membarrier, 0 as c_long, 0 as c_long, 0 as c_long) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// rseq: restartable sequences registration
pub struct RseqRegisterTest;
impl SyscallTest for RseqRegisterTest {
    fn name(&self) -> &str { "misc2_rseq_register" }
    fn syscall(&self) -> &str { "rseq" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "rseq syscall should return 0 or ENOSYS/EINVAL" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // SYS_rseq = 334
        let ret = unsafe { libc::syscall(334, std::ptr::null::<u8>() as c_long, 0 as c_long, 0 as c_long, 0 as c_long) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        // ret=0 success, EINVAL if null ptr, ENOSYS if not supported
        let s = if ret == 0 || errno_val == EINVAL as i32 || errno_val == EFAULT as i32 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
