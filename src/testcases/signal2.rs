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

/// sigaltstack: set an alternate signal stack
pub struct SigaltstackTest;
impl SyscallTest for SigaltstackTest {
    fn name(&self) -> &str { "sig2_sigaltstack_set_get" }
    fn syscall(&self) -> &str { "sigaltstack" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "sigaltstack() set a new alt stack, then get it back and verify size" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let stack_mem = vec![0u8; SIGSTKSZ];
        let new_stack = stack_t {
            ss_sp: stack_mem.as_ptr() as *mut _,
            ss_flags: 0,
            ss_size: SIGSTKSZ,
        };
        let mut old_stack: stack_t = unsafe { std::mem::zeroed() };
        let r1 = unsafe { sigaltstack(&new_stack, &mut old_stack) };
        let errno_val = unsafe { *libc::__errno_location() };
        // Read it back
        let mut cur_stack: stack_t = unsafe { std::mem::zeroed() };
        let r2 = unsafe { sigaltstack(std::ptr::null(), &mut cur_stack) };
        // Restore original
        if old_stack.ss_flags & SS_DISABLE == 0 {
            unsafe { sigaltstack(&old_stack, std::ptr::null_mut()); }
        } else {
            let disable = stack_t { ss_sp: std::ptr::null_mut(), ss_flags: SS_DISABLE, ss_size: 0 };
            unsafe { sigaltstack(&disable, std::ptr::null_mut()); }
        }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r1 == 0 && r2 == 0 && cur_stack.ss_size >= SIGSTKSZ { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r1 as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sigsuspend: block SIGUSR1, then suspend; parent wakes us
pub struct SigsuspendTest;
impl SyscallTest for SigsuspendTest {
    fn name(&self) -> &str { "sig2_sigsuspend_wakeup" }
    fn syscall(&self) -> &str { "sigsuspend" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "sigsuspend() atomically unblocks SIGUSR1 and waits; kill() from parent wakes it" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        static mut SIG_GOT: bool = false;
        extern "C" fn handler(_: c_int) { unsafe { SIG_GOT = true; } }
        let sa = sigaction {
            sa_sigaction: handler as usize,
            sa_mask: unsafe { std::mem::zeroed() },
            sa_flags: 0,
            sa_restorer: None,
        };
        unsafe { sigaction(SIGUSR1, &sa, std::ptr::null_mut()) };
        // Block SIGUSR1
        let mut block: sigset_t = unsafe { std::mem::zeroed() };
        let mut orig: sigset_t = unsafe { std::mem::zeroed() };
        unsafe { sigemptyset(&mut block); sigaddset(&mut block, SIGUSR1); }
        unsafe { sigprocmask(SIG_BLOCK, &block, &mut orig) };
        let pid = unsafe { fork() };
        if pid < 0 {
            unsafe { sigprocmask(SIG_SETMASK, &orig, std::ptr::null_mut()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            // Child: send SIGUSR1 to parent after 5ms
            unsafe { nanosleep(&timespec { tv_sec: 0, tv_nsec: 5_000_000 }, std::ptr::null_mut()); }
            let ppid = unsafe { getppid() };
            unsafe { kill(ppid, SIGUSR1); libc::exit(0); }
        }
        // Parent: suspend waiting for SIGUSR1 (mask has SIGUSR1 unblocked)
        let mut wait_mask: sigset_t = unsafe { std::mem::zeroed() };
        unsafe { sigemptyset(&mut wait_mask); }
        unsafe { sigsuspend(&wait_mask) }; // returns -1/EINTR when signal arrives
        unsafe { sigprocmask(SIG_SETMASK, &orig, std::ptr::null_mut()) };
        unsafe { waitpid(pid, std::ptr::null_mut(), 0) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if unsafe { SIG_GOT } { TestStatus::Pass }
        else { TestStatus::Error("sigsuspend did not receive SIGUSR1".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// signalfd: receive SIGUSR2 via file descriptor
pub struct SignalfdTest;
impl SyscallTest for SignalfdTest {
    fn name(&self) -> &str { "sig2_signalfd_basic" }
    fn syscall(&self) -> &str { "signalfd" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "signalfd() + block SIGUSR2 + kill self; read from sfd should return signal info" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut mask: sigset_t = unsafe { std::mem::zeroed() };
        unsafe { sigemptyset(&mut mask); sigaddset(&mut mask, SIGUSR2); }
        unsafe { sigprocmask(SIG_BLOCK, &mask, std::ptr::null_mut()) };
        let sfd = unsafe { signalfd(-1, &mask, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if sfd < 0 {
            unsafe { sigprocmask(SIG_UNBLOCK, &mask, std::ptr::null_mut()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 3, actual_ret: sfd as i64, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        let pid = unsafe { getpid() };
        unsafe { kill(pid, SIGUSR2) };
        let mut sinfo: signalfd_siginfo = unsafe { std::mem::zeroed() };
        let n = unsafe { read(sfd, &mut sinfo as *mut _ as *mut _, std::mem::size_of::<signalfd_siginfo>()) };
        unsafe { close(sfd); sigprocmask(SIG_UNBLOCK, &mask, std::ptr::null_mut()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == std::mem::size_of::<signalfd_siginfo>() as isize && sinfo.ssi_signo == SIGUSR2 as u32 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: SIGUSR2 as i64, actual_ret: sinfo.ssi_signo as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// signal mask inheritance: child inherits parent's blocked signals
pub struct SigmaskInheritTest;
impl SyscallTest for SigmaskInheritTest {
    fn name(&self) -> &str { "sig2_sigmask_inherit_fork" }
    fn syscall(&self) -> &str { "sigprocmask" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "Child process inherits parent's signal mask after fork()" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut block: sigset_t = unsafe { std::mem::zeroed() };
        let mut orig: sigset_t = unsafe { std::mem::zeroed() };
        unsafe { sigemptyset(&mut block); sigaddset(&mut block, SIGUSR1); }
        unsafe { sigprocmask(SIG_BLOCK, &block, &mut orig) };
        let mut pipefd = [0i32; 2];
        unsafe { pipe(pipefd.as_mut_ptr()) };
        let pid = unsafe { fork() };
        if pid < 0 {
            unsafe { sigprocmask(SIG_SETMASK, &orig, std::ptr::null_mut()); close(pipefd[0]); close(pipefd[1]); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            unsafe { close(pipefd[0]); }
            let mut child_mask: sigset_t = unsafe { std::mem::zeroed() };
            unsafe { sigprocmask(SIG_BLOCK, std::ptr::null(), &mut child_mask) };
            let is_blocked = unsafe { sigismember(&child_mask, SIGUSR1) };
            let val: u8 = if is_blocked == 1 { 1 } else { 0 };
            unsafe { write(pipefd[1], &val as *const _ as *const _, 1); close(pipefd[1]); libc::exit(0); }
        }
        unsafe { close(pipefd[1]); }
        let mut result = [0u8; 1];
        unsafe { read(pipefd[0], result.as_mut_ptr() as *mut _, 1); close(pipefd[0]); }
        unsafe { waitpid(pid, std::ptr::null_mut(), 0); sigprocmask(SIG_SETMASK, &orig, std::ptr::null_mut()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if result[0] == 1 { TestStatus::Pass }
        else { TestStatus::Error("child did not inherit blocked signal mask".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// sigqueue: send signal with value using sigqueue()
pub struct SigqueueTest;
impl SyscallTest for SigqueueTest {
    fn name(&self) -> &str { "sig2_sigqueue_value" }
    fn syscall(&self) -> &str { "sigqueue" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "sigqueue() delivers SIGUSR1 with sigval; SA_SIGINFO handler receives the value" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        static mut QUEUE_VALUE: i32 = 0;
        extern "C" fn siginfo_handler(_: c_int, si: *mut siginfo_t, _: *mut c_void) {
            if !si.is_null() { unsafe { QUEUE_VALUE = (*si).si_value().sival_ptr as i32; } }
        }
        let mut sa: sigaction = unsafe { std::mem::zeroed() };
        sa.sa_sigaction = siginfo_handler as usize;
        sa.sa_flags = SA_SIGINFO;
        let r = unsafe { sigaction(SIGUSR1, &sa, std::ptr::null_mut()) };
        if r != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let pid = unsafe { getpid() };
        let sv = sigval { sival_ptr: 12345 as *mut libc::c_void };
        let r2 = unsafe { sigqueue(pid, SIGUSR1, sv) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if r2 == 0 && unsafe { QUEUE_VALUE } == 12345 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 12345, actual_ret: unsafe { QUEUE_VALUE as i64 }, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// SIGCHLD is delivered when child exits
pub struct SigchldTest;
impl SyscallTest for SigchldTest {
    fn name(&self) -> &str { "sig2_sigchld_on_exit" }
    fn syscall(&self) -> &str { "signal" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Signal }
    fn description(&self) -> &str { "SIGCHLD should be delivered to parent when child exits" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        static mut CHLD_RECEIVED: bool = false;
        extern "C" fn chld_handler(_: c_int) { unsafe { CHLD_RECEIVED = true; } }
        let sa = sigaction {
            sa_sigaction: chld_handler as usize,
            sa_mask: unsafe { std::mem::zeroed() },
            sa_flags: SA_NOCLDSTOP as i32,
            sa_restorer: None,
        };
        unsafe { sigaction(SIGCHLD, &sa, std::ptr::null_mut()) };
        let pid = unsafe { fork() };
        if pid < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 { unsafe { libc::exit(0) }; }
        unsafe { waitpid(pid, std::ptr::null_mut(), 0) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if unsafe { CHLD_RECEIVED } { TestStatus::Pass }
        else { TestStatus::Error("SIGCHLD not received".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
