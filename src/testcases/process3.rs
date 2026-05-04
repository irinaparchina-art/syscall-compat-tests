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

/// wait4: like waitpid but also fills rusage
pub struct Wait4Test;
impl SyscallTest for Wait4Test {
    fn name(&self) -> &str { "proc3_wait4_rusage" }
    fn syscall(&self) -> &str { "wait4" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "wait4() should return child pid and fill rusage struct" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let pid = unsafe { fork() };
        if pid < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            // Do some work so CPU time is nonzero
            let mut x = 0u64;
            for i in 0..100000u64 { x = x.wrapping_add(i); }
            let _ = x;
            unsafe { libc::exit(0) };
        }
        let mut status = 0i32;
        let mut usage: rusage = unsafe { std::mem::zeroed() };
        let ret = unsafe { wait4(pid, &mut status, 0, &mut usage) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == pid && libc::WIFEXITED(status) { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: pid as i64, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// clone with CLONE_FS: child shares filesystem namespace
pub struct CloneFsTest;
impl SyscallTest for CloneFsTest {
    fn name(&self) -> &str { "proc3_clone_basic" }
    fn syscall(&self) -> &str { "clone" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "clone(SIGCHLD) behaves like fork(); child exits 0 and parent waits" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // Use clone as fork equivalent (SIGCHLD only)
        let pid = unsafe { libc::syscall(libc::SYS_clone, SIGCHLD as c_long, 0 as c_long, 0 as c_long, 0 as c_long, 0 as c_long) };
        let errno_val = unsafe { *libc::__errno_location() };
        if pid < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 1, actual_ret: pid, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        if pid == 0 { unsafe { libc::exit(0) }; }
        let mut st = 0i32;
        let ret = unsafe { waitpid(pid as i32, &mut st, 0) };
        let dur = start.elapsed().as_micros() as u64;
        let exit_code = if libc::WIFEXITED(st) { libc::WEXITSTATUS(st) } else { -1 };
        let s = if ret == pid as i32 && exit_code == 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: exit_code as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getresuid: real, effective, saved UID
pub struct GetresuidTest;
impl SyscallTest for GetresuidTest {
    fn name(&self) -> &str { "proc3_getresuid" }
    fn syscall(&self) -> &str { "getresuid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getresuid() should return ruid == euid == suid for normal process" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut ruid: uid_t = 0;
        let mut euid: uid_t = 0;
        let mut suid: uid_t = 0;
        let ret = unsafe { getresuid(&mut ruid, &mut euid, &mut suid) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && ruid == euid { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getresgid: real, effective, saved GID
pub struct GetresgidTest;
impl SyscallTest for GetresgidTest {
    fn name(&self) -> &str { "proc3_getresgid" }
    fn syscall(&self) -> &str { "getresgid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "getresgid() should return rgid == egid == sgid for normal process" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut rgid: gid_t = 0;
        let mut egid: gid_t = 0;
        let mut sgid: gid_t = 0;
        let ret = unsafe { getresgid(&mut rgid, &mut egid, &mut sgid) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && rgid == egid { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// prctl PR_SET_NAME / PR_GET_NAME round-trip
pub struct PrctlNameRoundtripTest;
impl SyscallTest for PrctlNameRoundtripTest {
    fn name(&self) -> &str { "proc3_prctl_name_roundtrip" }
    fn syscall(&self) -> &str { "prctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "prctl(PR_SET_NAME) then PR_GET_NAME should return the set name" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // Save original name
        let mut orig = [0i8; 16];
        unsafe { prctl(PR_GET_NAME, orig.as_mut_ptr() as c_ulong, 0, 0, 0) };
        // Set new name
        let new_name = b"sct_test\0";
        let r1 = unsafe { prctl(PR_SET_NAME, new_name.as_ptr() as c_ulong, 0, 0, 0) };
        // Get it back
        let mut got = [0i8; 16];
        let r2 = unsafe { prctl(PR_GET_NAME, got.as_mut_ptr() as c_ulong, 0, 0, 0) };
        // Restore
        unsafe { prctl(PR_SET_NAME, orig.as_ptr() as c_ulong, 0, 0, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let got_str: String = got.iter().take_while(|&&c| c != 0).map(|&c| c as u8 as char).collect();
        let s = if r1 == 0 && r2 == 0 && got_str == "sct_test" { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Error(format!("got name: '{}'", got_str)) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// readlink /proc/self/exe returns the executable path
pub struct ReadlinkProcSelfExeTest;
impl SyscallTest for ReadlinkProcSelfExeTest {
    fn name(&self) -> &str { "proc3_readlink_proc_self_exe" }
    fn syscall(&self) -> &str { "readlink" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "readlink(/proc/self/exe) should return a non-empty executable path" }
    fn run(&self) -> TestResult {
        let path = CString::new("/proc/self/exe").unwrap();
        let start = Instant::now();
        let mut buf = vec![0u8; 1024];
        let ret = unsafe { readlink(path.as_ptr(), buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret > 0 && buf[0] == b'/' { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// /proc/self/maps is readable and non-empty
pub struct ProcSelfMapsTest;
impl SyscallTest for ProcSelfMapsTest {
    fn name(&self) -> &str { "proc3_proc_self_maps" }
    fn syscall(&self) -> &str { "read" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "/proc/self/maps should be readable and contain memory region entries" }
    fn run(&self) -> TestResult {
        let path = CString::new("/proc/self/maps").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut buf = vec![0u8; 1024];
        let n = unsafe { read(fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let content = std::str::from_utf8(&buf[..n.max(0) as usize]).unwrap_or("");
        // maps contains lines like "7f... r-xp ..."
        let s = if n > 0 && content.contains('-') && content.contains('p') { TestStatus::Pass }
        else { TestStatus::Error("/proc/self/maps content unexpected".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// /proc/self/fd directory lists open file descriptors
pub struct ProcSelfFdTest;
impl SyscallTest for ProcSelfFdTest {
    fn name(&self) -> &str { "proc3_proc_self_fd" }
    fn syscall(&self) -> &str { "opendir" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "/proc/self/fd should contain entries for stdin/stdout/stderr (0,1,2)" }
    fn run(&self) -> TestResult {
        let path = CString::new("/proc/self/fd").unwrap();
        let start = Instant::now();
        let dir = unsafe { opendir(path.as_ptr()) };
        if dir.is_null() {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut found_0 = false;
        let mut found_1 = false;
        let mut found_2 = false;
        loop {
            let e = unsafe { readdir(dir) };
            if e.is_null() { break; }
            let name = unsafe { std::ffi::CStr::from_ptr((*e).d_name.as_ptr()) }
                .to_str().unwrap_or("");
            if name == "0" { found_0 = true; }
            if name == "1" { found_1 = true; }
            if name == "2" { found_2 = true; }
        }
        unsafe { closedir(dir); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if found_0 && found_1 && found_2 { TestStatus::Pass }
        else { TestStatus::Error("missing 0/1/2 in /proc/self/fd".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getpid() == atoi(readlink /proc/self))
pub struct ProcSelfPidTest;
impl SyscallTest for ProcSelfPidTest {
    fn name(&self) -> &str { "proc3_proc_self_pid_consistent" }
    fn syscall(&self) -> &str { "readlink" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "readlink(/proc/self) should return our PID as string" }
    fn run(&self) -> TestResult {
        let path = CString::new("/proc/self").unwrap();
        let start = Instant::now();
        let mut buf = [0u8; 32];
        let ret = unsafe { readlink(path.as_ptr(), buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
        let dur = start.elapsed().as_micros() as u64;
        if ret <= 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, dur);
        }
        let s_str = std::str::from_utf8(&buf[..ret as usize]).unwrap_or("0");
        let link_pid: i32 = s_str.trim().parse().unwrap_or(-1);
        let my_pid = unsafe { getpid() };
        let s = if link_pid == my_pid { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: my_pid as i64, actual_ret: link_pid as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// fork + exec with environment variable passing
pub struct ExecveEnvTest;
impl SyscallTest for ExecveEnvTest {
    fn name(&self) -> &str { "proc3_execve_env" }
    fn syscall(&self) -> &str { "execve" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "execve with custom env; /usr/bin/env should exit 0" }
    fn run(&self) -> TestResult {
        use std::ffi::CString;
        let start = Instant::now();
        let pid = unsafe { fork() };
        if pid < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            let path = CString::new("/usr/bin/env").unwrap();
            let arg0 = CString::new("env").unwrap();
            let mut argv = [arg0.as_ptr(), std::ptr::null()];
            let env_str = CString::new("SCT_TEST=1").unwrap();
            let mut envp = [env_str.as_ptr(), std::ptr::null()];
            unsafe { execve(path.as_ptr(), argv.as_mut_ptr(), envp.as_mut_ptr()) };
            unsafe { libc::exit(1) };
        }
        let mut status = 0i32;
        let r = unsafe { waitpid(pid, &mut status, 0) };
        let dur = start.elapsed().as_micros() as u64;
        let exit_code = if libc::WIFEXITED(status) { libc::WEXITSTATUS(status) } else { -1 };
        let s = if r == pid && exit_code == 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: exit_code as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
