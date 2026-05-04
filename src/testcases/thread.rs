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

/// pthread_create + pthread_join basic
pub struct PthreadCreateJoinTest;
impl SyscallTest for PthreadCreateJoinTest {
    fn name(&self) -> &str { "thread_pthread_create_join" }
    fn syscall(&self) -> &str { "clone/futex" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "pthread_create + pthread_join: thread sets a flag, join sees it" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        static mut FLAG: i32 = 0;
        extern "C" fn thread_fn(_: *mut c_void) -> *mut c_void {
            unsafe { FLAG = 42; }
            std::ptr::null_mut()
        }
        let mut tid: pthread_t = 0;
        let ret = unsafe { pthread_create(&mut tid, std::ptr::null(), thread_fn, std::ptr::null_mut()) };
        if ret != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        unsafe { pthread_join(tid, std::ptr::null_mut()) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if unsafe { FLAG } == 42 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 42, actual_ret: unsafe { FLAG } as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// pthread_mutex: lock/unlock basic
pub struct PthreadMutexTest;
impl SyscallTest for PthreadMutexTest {
    fn name(&self) -> &str { "thread_pthread_mutex" }
    fn syscall(&self) -> &str { "futex" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "pthread_mutex_lock + unlock should serialize increments from two threads" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        static mut COUNTER: i32 = 0;
        static mut MTX: pthread_mutex_t = PTHREAD_MUTEX_INITIALIZER;
        extern "C" fn inc_fn(_: *mut c_void) -> *mut c_void {
            for _ in 0..1000 {
                unsafe {
                    pthread_mutex_lock(&mut MTX);
                    COUNTER += 1;
                    pthread_mutex_unlock(&mut MTX);
                }
            }
            std::ptr::null_mut()
        }
        unsafe { COUNTER = 0; }
        let mut t1: pthread_t = 0;
        let mut t2: pthread_t = 0;
        let r1 = unsafe { pthread_create(&mut t1, std::ptr::null(), inc_fn, std::ptr::null_mut()) };
        let r2 = unsafe { pthread_create(&mut t2, std::ptr::null(), inc_fn, std::ptr::null_mut()) };
        if r1 != 0 || r2 != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        unsafe { pthread_join(t1, std::ptr::null_mut()); pthread_join(t2, std::ptr::null_mut()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if unsafe { COUNTER } == 2000 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 2000, actual_ret: unsafe { COUNTER } as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// gettid() returns a positive thread ID
pub struct GettidTest;
impl SyscallTest for GettidTest {
    fn name(&self) -> &str { "thread_gettid" }
    fn syscall(&self) -> &str { "gettid" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "gettid() should return a positive thread ID" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ret = unsafe { libc::syscall(libc::SYS_gettid) };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// thread-local storage via pthread_key_create
pub struct PthreadKeyTest;
impl SyscallTest for PthreadKeyTest {
    fn name(&self) -> &str { "thread_pthread_key" }
    fn syscall(&self) -> &str { "futex" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "pthread_key_create + setspecific/getspecific should isolate per-thread values" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut key: pthread_key_t = 0;
        let r = unsafe { pthread_key_create(&mut key, None) };
        if r != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let val: usize = 0xABCD;
        unsafe { pthread_setspecific(key, val as *const c_void) };
        let got = unsafe { pthread_getspecific(key) } as usize;
        unsafe { pthread_key_delete(key) };
        let dur = start.elapsed().as_micros() as u64;
        let s = if got == val { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: val as i64, actual_ret: got as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// futex WAIT+WAKE via raw syscall
pub struct FutexWakeWaitTest;
impl SyscallTest for FutexWakeWaitTest {
    fn name(&self) -> &str { "thread_futex_wake_wait" }
    fn syscall(&self) -> &str { "futex" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "futex FUTEX_WAKE on a word that nobody is waiting on should return 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut futex_word: i32 = 0;
        // FUTEX_WAKE with no waiters should return 0 (number of waiters woken)
        let ret = unsafe {
            libc::syscall(libc::SYS_futex,
                &mut futex_word as *mut i32 as c_long,
                libc::FUTEX_WAKE as c_long,
                1 as c_long, 0 as c_long, 0 as c_long, 0 as c_long)
        };
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// pthread_once: init function called exactly once
pub struct PthreadOnceTest;
impl SyscallTest for PthreadOnceTest {
    fn name(&self) -> &str { "thread_pthread_once" }
    fn syscall(&self) -> &str { "futex" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "pthread_once init function should be called exactly once across multiple calls" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        static mut ONCE_CTRL: pthread_once_t = PTHREAD_ONCE_INIT;
        static mut ONCE_COUNT: i32 = 0;
        extern "C" fn init_fn() { unsafe { ONCE_COUNT += 1; } }
        unsafe {
            pthread_once(&mut ONCE_CTRL, init_fn);
            pthread_once(&mut ONCE_CTRL, init_fn);
            pthread_once(&mut ONCE_CTRL, init_fn);
        }
        let dur = start.elapsed().as_micros() as u64;
        let s = if unsafe { ONCE_COUNT } == 1 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: unsafe { ONCE_COUNT } as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// pthread_cond: signal a waiting thread
pub struct PthreadCondTest;
impl SyscallTest for PthreadCondTest {
    fn name(&self) -> &str { "thread_pthread_cond_signal" }
    fn syscall(&self) -> &str { "futex" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Process }
    fn description(&self) -> &str { "pthread_cond_signal wakes a thread waiting on pthread_cond_wait" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        static mut MTX2: pthread_mutex_t = PTHREAD_MUTEX_INITIALIZER;
        static mut COND: pthread_cond_t = PTHREAD_COND_INITIALIZER;
        static mut READY: i32 = 0;

        extern "C" fn waiter(_: *mut c_void) -> *mut c_void {
            unsafe {
                pthread_mutex_lock(&mut MTX2);
                while READY == 0 {
                    pthread_cond_wait(&mut COND, &mut MTX2);
                }
                pthread_mutex_unlock(&mut MTX2);
            }
            std::ptr::null_mut()
        }
        unsafe { READY = 0; }
        let mut tid: pthread_t = 0;
        let r = unsafe { pthread_create(&mut tid, std::ptr::null(), waiter, std::ptr::null_mut()) };
        if r != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        // Give waiter time to start waiting
        unsafe { nanosleep(&timespec { tv_sec: 0, tv_nsec: 5_000_000 }, std::ptr::null_mut()); }
        unsafe {
            pthread_mutex_lock(&mut MTX2);
            READY = 1;
            pthread_cond_signal(&mut COND);
            pthread_mutex_unlock(&mut MTX2);
        }
        unsafe { pthread_join(tid, std::ptr::null_mut()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if unsafe { READY } == 1 { TestStatus::Pass }
        else { TestStatus::Error("cond_signal did not wake waiter".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
