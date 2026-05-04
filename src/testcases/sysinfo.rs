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

/// uname() should fill sysname and nodename
pub struct UnameTest;
impl SyscallTest for UnameTest {
    fn name(&self) -> &str { "sysinfo_uname" }
    fn syscall(&self) -> &str { "uname" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "uname() should succeed and return non-empty sysname" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut buf: utsname = unsafe { std::mem::zeroed() };
        let ret = unsafe { uname(&mut buf) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let sysname_len = buf.sysname.iter().take_while(|&&c| c != 0).count();
        let s = if ret == 0 && sysname_len > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sysinfo() should return total RAM > 0
pub struct SysinfoTest;
impl SyscallTest for SysinfoTest {
    fn name(&self) -> &str { "sysinfo_sysinfo" }
    fn syscall(&self) -> &str { "sysinfo" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "sysinfo() should return totalram > 0 and uptime >= 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut info: sysinfo = unsafe { std::mem::zeroed() };
        let ret = unsafe { sysinfo(&mut info) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && info.totalram > 0 && info.uptime >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getrlimit(RLIMIT_NOFILE) should return a positive soft limit
pub struct GetrlimitNofileTest;
impl SyscallTest for GetrlimitNofileTest {
    fn name(&self) -> &str { "sysinfo_getrlimit_nofile" }
    fn syscall(&self) -> &str { "getrlimit" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "getrlimit(RLIMIT_NOFILE) soft limit should be > 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut rl: rlimit = unsafe { std::mem::zeroed() };
        let ret = unsafe { getrlimit(RLIMIT_NOFILE, &mut rl) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && rl.rlim_cur > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getrlimit(RLIMIT_STACK) should return a positive limit
pub struct GetrlimitStackTest;
impl SyscallTest for GetrlimitStackTest {
    fn name(&self) -> &str { "sysinfo_getrlimit_stack" }
    fn syscall(&self) -> &str { "getrlimit" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "getrlimit(RLIMIT_STACK) soft limit should be > 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut rl: rlimit = unsafe { std::mem::zeroed() };
        let ret = unsafe { getrlimit(RLIMIT_STACK, &mut rl) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && rl.rlim_cur > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// setrlimit + getrlimit: set RLIMIT_CORE to 0 and verify
pub struct SetrlimitTest;
impl SyscallTest for SetrlimitTest {
    fn name(&self) -> &str { "sysinfo_setrlimit_core" }
    fn syscall(&self) -> &str { "setrlimit" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "setrlimit(RLIMIT_CORE, 0) then getrlimit should confirm soft limit is 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut orig: rlimit = unsafe { std::mem::zeroed() };
        unsafe { getrlimit(RLIMIT_CORE, &mut orig) };
        let new_rl = rlimit { rlim_cur: 0, rlim_max: orig.rlim_max };
        let r1 = unsafe { setrlimit(RLIMIT_CORE, &new_rl) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut check: rlimit = unsafe { std::mem::zeroed() };
        unsafe { getrlimit(RLIMIT_CORE, &mut check) };
        unsafe { setrlimit(RLIMIT_CORE, &orig) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if r1 == 0 && check.rlim_cur == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r1 as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getpagesize() should return a power of 2 >= 4096
pub struct GetpagesizeTest;
impl SyscallTest for GetpagesizeTest {
    fn name(&self) -> &str { "sysinfo_getpagesize" }
    fn syscall(&self) -> &str { "getpagesize" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "getpagesize() should return a power of 2 >= 4096" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as i32 };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret >= 4096 && (ret & (ret - 1)) == 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 4096, actual_ret: ret as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sysconf(_SC_NPROCESSORS_ONLN) should return >= 1
pub struct SysconfNprocsTest;
impl SyscallTest for SysconfNprocsTest {
    fn name(&self) -> &str { "sysinfo_sysconf_nprocs" }
    fn syscall(&self) -> &str { "sysconf" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "sysconf(_SC_NPROCESSORS_ONLN) should return >= 1" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { sysconf(_SC_NPROCESSORS_ONLN) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret >= 1 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sysconf(_SC_PAGE_SIZE) should match getpagesize()
pub struct SysconfPageSizeTest;
impl SyscallTest for SysconfPageSizeTest {
    fn name(&self) -> &str { "sysinfo_sysconf_pagesize" }
    fn syscall(&self) -> &str { "sysconf" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "sysconf(_SC_PAGE_SIZE) should match getpagesize()" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let sc = unsafe { sysconf(_SC_PAGE_SIZE) };
        let gp = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as i32 } as i64;
        let dur = start.elapsed().as_micros() as u64;
        let s = if sc == gp { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: gp, actual_ret: sc as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getgroups(0, NULL) returns number of supplementary groups
pub struct GetgroupsTest;
impl SyscallTest for GetgroupsTest {
    fn name(&self) -> &str { "sysinfo_getgroups" }
    fn syscall(&self) -> &str { "getgroups" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "getgroups(0, NULL) should return the number of supplementary groups (>= 0)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { getgroups(0, std::ptr::null_mut()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// hostname: gethostname returns non-empty string
pub struct GethostnameTest;
impl SyscallTest for GethostnameTest {
    fn name(&self) -> &str { "sysinfo_gethostname" }
    fn syscall(&self) -> &str { "gethostname" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "gethostname() should return a non-empty hostname string" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut buf = vec![0i8; 256];
        let ret = unsafe { gethostname(buf.as_mut_ptr(), buf.len()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let len = buf.iter().take_while(|&&c| c != 0).count();
        let s = if ret == 0 && len > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// /proc/self/status readable and contains "Name:"
pub struct ProcSelfStatusTest;
impl SyscallTest for ProcSelfStatusTest {
    fn name(&self) -> &str { "sysinfo_proc_self_status" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "/proc/self/status should be readable and contain 'Name:' field" }
    fn run(&self) -> TestResult {
        let path = CString::new("/proc/self/status").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut buf = vec![0u8; 512];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let content = std::str::from_utf8(&buf[..n.max(0) as usize]).unwrap_or("");
        let s = if content.contains("Name:") { TestStatus::Pass }
        else { TestStatus::Error("/proc/self/status missing 'Name:' field".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
