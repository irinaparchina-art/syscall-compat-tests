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

/// shmget creates a shared memory segment
pub struct ShmgetCreateTest;
impl SyscallTest for ShmgetCreateTest {
    fn name(&self) -> &str { "ipc_shmget_create" }
    fn syscall(&self) -> &str { "shmget" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "shmget(IPC_PRIVATE, 4096, IPC_CREAT) should return a valid shmid" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let shmid = unsafe { shmget(IPC_PRIVATE, 4096, IPC_CREAT | 0o600) };
        let errno_val = unsafe { *libc::__errno_location() };
        if shmid >= 0 { unsafe { shmctl(shmid, IPC_RMID, std::ptr::null_mut()); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if shmid >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: shmid as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// shmat attaches shared memory; write then detach and re-attach should persist
pub struct ShmatShmdetTest;
impl SyscallTest for ShmatShmdetTest {
    fn name(&self) -> &str { "ipc_shmat_shmdt" }
    fn syscall(&self) -> &str { "shmat/shmdt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "shmat() + write + shmdt() + shmat() again should see the written value" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let shmid = unsafe { shmget(IPC_PRIVATE, 4096, IPC_CREAT | 0o600) };
        if shmid < 0 {
            let e = unsafe { *libc::__errno_location() };
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if e == ENOSYS as i32 { TestStatus::Unimplemented } else { TestStatus::Error("shmget failed".into()) },
                start.elapsed().as_micros() as u64);
        }
        let ptr = unsafe { shmat(shmid, std::ptr::null(), 0) };
        if ptr as isize == -1 {
            unsafe { shmctl(shmid, IPC_RMID, std::ptr::null_mut()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        unsafe { *(ptr as *mut u32) = 0xDEADBEEF; }
        unsafe { shmdt(ptr); }
        // Re-attach
        let ptr2 = unsafe { shmat(shmid, std::ptr::null(), 0) };
        let val = unsafe { *(ptr2 as *const u32) };
        unsafe { shmdt(ptr2); shmctl(shmid, IPC_RMID, std::ptr::null_mut()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if val == 0xDEADBEEF { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0xDEADBEEF, actual_ret: val as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// shmctl IPC_STAT returns valid info about a segment
pub struct ShmctlStatTest;
impl SyscallTest for ShmctlStatTest {
    fn name(&self) -> &str { "ipc_shmctl_stat" }
    fn syscall(&self) -> &str { "shmctl" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "shmctl(IPC_STAT) should return segment info with correct size" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let shmid = unsafe { shmget(IPC_PRIVATE, 4096, IPC_CREAT | 0o600) };
        if shmid < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut buf: shmid_ds = unsafe { std::mem::zeroed() };
        let ret = unsafe { shmctl(shmid, IPC_STAT, &mut buf) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { shmctl(shmid, IPC_RMID, std::ptr::null_mut()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && buf.shm_segsz >= 4096 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// msgget creates a message queue
pub struct MsggetCreateTest;
impl SyscallTest for MsggetCreateTest {
    fn name(&self) -> &str { "ipc_msgget_create" }
    fn syscall(&self) -> &str { "msgget" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "msgget(IPC_PRIVATE, IPC_CREAT) should return a valid msqid" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let msqid = unsafe { msgget(IPC_PRIVATE, IPC_CREAT | 0o600) };
        let errno_val = unsafe { *libc::__errno_location() };
        if msqid >= 0 { unsafe { msgctl(msqid, IPC_RMID, std::ptr::null_mut()); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if msqid >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: msqid as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// msgsnd + msgrcv: send a message and receive it back
pub struct MsgsndRcvTest;
impl SyscallTest for MsgsndRcvTest {
    fn name(&self) -> &str { "ipc_msgsnd_msgrcv" }
    fn syscall(&self) -> &str { "msgsnd/msgrcv" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "msgsnd() then msgrcv() should deliver the same message content" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let msqid = unsafe { msgget(IPC_PRIVATE, IPC_CREAT | 0o600) };
        if msqid < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        #[repr(C)]
        struct Msg { mtype: c_long, mtext: [u8; 8] }
        let mut send_msg = Msg { mtype: 1, mtext: *b"testdata" };
        let r1 = unsafe { msgsnd(msqid, &send_msg as *const _ as *const _, 8, 0) };
        let mut recv_msg = Msg { mtype: 0, mtext: [0u8; 8] };
        let r2 = unsafe { msgrcv(msqid, &mut recv_msg as *mut _ as *mut _, 8, 0, 0) };
        unsafe { msgctl(msqid, IPC_RMID, std::ptr::null_mut()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r1 == 0 && r2 == 8 && &recv_msg.mtext == b"testdata" { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 8, actual_ret: r2 as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// semget creates a semaphore set
pub struct SemgetCreateTest;
impl SyscallTest for SemgetCreateTest {
    fn name(&self) -> &str { "ipc_semget_create" }
    fn syscall(&self) -> &str { "semget" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "semget(IPC_PRIVATE, 1, IPC_CREAT) should return a valid semid" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let semid = unsafe { semget(IPC_PRIVATE, 1, IPC_CREAT | 0o600) };
        let errno_val = unsafe { *libc::__errno_location() };
        if semid >= 0 { unsafe { semctl(semid, 0, IPC_RMID); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if semid >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: semid as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// semop: post and wait on a semaphore
pub struct SemopTest;
impl SyscallTest for SemopTest {
    fn name(&self) -> &str { "ipc_semop_basic" }
    fn syscall(&self) -> &str { "semop" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Other }
    fn description(&self) -> &str { "semctl(SETVAL=1) then semop(-1) should decrement and return 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let semid = unsafe { semget(IPC_PRIVATE, 1, IPC_CREAT | 0o600) };
        if semid < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        // Set semaphore to 1
        let r1 = unsafe { semctl(semid, 0, SETVAL, 1i32) };
        // Decrement by 1 (non-blocking)
        let mut sop = sembuf { sem_num: 0, sem_op: -1, sem_flg: IPC_NOWAIT as i16 };
        let r2 = unsafe { semop(semid, &mut sop, 1) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { semctl(semid, 0, IPC_RMID); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r1 == 0 && r2 == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r2 as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
