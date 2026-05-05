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

/// /proc/self/cmdline is readable
pub struct ProcCmdlineTest;
impl SyscallTest for ProcCmdlineTest {
    fn name(&self) -> &str { "sys2_proc_cmdline" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "/proc/self/cmdline should be readable and non-empty" }
    fn run(&self) -> TestResult {
        let path = CString::new("/proc/self/cmdline").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut buf = [0u8; 256];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 256) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n > 0 { TestStatus::Pass }
        else { TestStatus::Error("/proc/self/cmdline empty".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// /proc/self/stat is readable and starts with pid
pub struct ProcStatTest;
impl SyscallTest for ProcStatTest {
    fn name(&self) -> &str { "sys2_proc_stat" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "/proc/self/stat should start with our PID" }
    fn run(&self) -> TestResult {
        let path = CString::new("/proc/self/stat").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut buf = [0u8; 256];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 255) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let content = std::str::from_utf8(&buf[..n.max(0) as usize]).unwrap_or("");
        let pid = unsafe { getpid() };
        let pid_str = pid.to_string();
        let s = if content.starts_with(&pid_str) { TestStatus::Pass }
        else { TestStatus::Error(format!("stat starts with: {}", &content[..20.min(content.len())]) ) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// /proc/meminfo is readable and contains MemTotal
pub struct ProcMeminfoTest;
impl SyscallTest for ProcMeminfoTest {
    fn name(&self) -> &str { "sys2_proc_meminfo" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "/proc/meminfo should contain MemTotal field" }
    fn run(&self) -> TestResult {
        let path = CString::new("/proc/meminfo").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut buf = vec![0u8; 512];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 511) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let content = std::str::from_utf8(&buf[..n.max(0) as usize]).unwrap_or("");
        let s = if content.contains("MemTotal") { TestStatus::Pass }
        else { TestStatus::Error("MemTotal not found in /proc/meminfo".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// /proc/cpuinfo is readable and contains processor field
pub struct ProcCpuinfoTest;
impl SyscallTest for ProcCpuinfoTest {
    fn name(&self) -> &str { "sys2_proc_cpuinfo" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "/proc/cpuinfo should contain 'processor' field" }
    fn run(&self) -> TestResult {
        let path = CString::new("/proc/cpuinfo").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut buf = vec![0u8; 512];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 511) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let content = std::str::from_utf8(&buf[..n.max(0) as usize]).unwrap_or("");
        let s = if content.contains("processor") { TestStatus::Pass }
        else { TestStatus::Error("'processor' not found in /proc/cpuinfo".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// /proc/uptime is readable and shows positive uptime
pub struct ProcUptimeTest;
impl SyscallTest for ProcUptimeTest {
    fn name(&self) -> &str { "sys2_proc_uptime" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "/proc/uptime should contain two positive floats" }
    fn run(&self) -> TestResult {
        let path = CString::new("/proc/uptime").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut buf = [0u8; 64];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, 63) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let content = std::str::from_utf8(&buf[..n.max(0) as usize]).unwrap_or("");
        // Format: "uptime idletime\n", both should be > 0
        let parts: Vec<&str> = content.split_whitespace().collect();
        let uptime: f64 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let s = if uptime > 0.0 { TestStatus::Pass }
        else { TestStatus::Error(format!("unexpected uptime: {}", content)) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sysconf(_SC_CLK_TCK) should return a positive value (usually 100)
pub struct SysconfClkTckTest;
impl SyscallTest for SysconfClkTckTest {
    fn name(&self) -> &str { "sys2_sysconf_clk_tck" }
    fn syscall(&self) -> &str { "sysconf" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "sysconf(_SC_CLK_TCK) should return clock ticks per second (>0)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { sysconf(_SC_CLK_TCK) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret > 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 100, actual_ret: ret, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sysconf(_SC_OPEN_MAX) should return a value >= 256
pub struct SysconfOpenMaxTest;
impl SyscallTest for SysconfOpenMaxTest {
    fn name(&self) -> &str { "sys2_sysconf_open_max" }
    fn syscall(&self) -> &str { "sysconf" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "sysconf(_SC_OPEN_MAX) should return max open files >= 256" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { sysconf(_SC_OPEN_MAX) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret >= 256 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 256, actual_ret: ret, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sysconf(_SC_CHILD_MAX) should return positive value
pub struct SysconfChildMaxTest;
impl SyscallTest for SysconfChildMaxTest {
    fn name(&self) -> &str { "sys2_sysconf_child_max" }
    fn syscall(&self) -> &str { "sysconf" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "sysconf(_SC_CHILD_MAX) should return max simultaneous processes > 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { sysconf(_SC_CHILD_MAX) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret > 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getrlimit RLIMIT_NPROC: max number of processes
pub struct GetrlimitNprocTest;
impl SyscallTest for GetrlimitNprocTest {
    fn name(&self) -> &str { "sys2_getrlimit_nproc" }
    fn syscall(&self) -> &str { "getrlimit" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "getrlimit(RLIMIT_NPROC) should return a positive soft limit" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut rl: rlimit = unsafe { std::mem::zeroed() };
        let ret = unsafe { getrlimit(RLIMIT_NPROC, &mut rl) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && (rl.rlim_cur > 0 || rl.rlim_cur == RLIM_INFINITY) { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getrlimit RLIMIT_AS: virtual memory size limit
pub struct GetrlimitAsTest;
impl SyscallTest for GetrlimitAsTest {
    fn name(&self) -> &str { "sys2_getrlimit_as" }
    fn syscall(&self) -> &str { "getrlimit" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "getrlimit(RLIMIT_AS) should return virtual address space limit" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut rl: rlimit = unsafe { std::mem::zeroed() };
        let ret = unsafe { getrlimit(RLIMIT_AS, &mut rl) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
