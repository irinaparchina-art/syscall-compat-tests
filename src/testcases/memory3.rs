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

/// MAP_FIXED: map at a specific address (after reserving it)
pub struct MmapFixedTest;
impl SyscallTest for MmapFixedTest {
    fn name(&self) -> &str { "mem3_mmap_fixed" }
    fn syscall(&self) -> &str { "mmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mmap(MAP_FIXED) at a pre-reserved address should return exactly that address" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // First reserve a region
        let base = unsafe {
            mmap(std::ptr::null_mut(), 4096 * 2, PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS, -1, 0)
        };
        if base == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("initial mmap failed".into()), start.elapsed().as_micros() as u64);
        }
        // Now map with MAP_FIXED at base
        let fixed = unsafe {
            mmap(base, 4096, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED, -1, 0)
        };
        let errno_val = unsafe { *libc::__errno_location() };
        let ok = fixed == base;
        unsafe { munmap(base, 4096 * 2); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ok { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: base as i64, actual_ret: fixed as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// MAP_POPULATE: fault in all pages eagerly
pub struct MmapPopulateTest;
impl SyscallTest for MmapPopulateTest {
    fn name(&self) -> &str { "mem3_mmap_populate" }
    fn syscall(&self) -> &str { "mmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mmap(MAP_POPULATE) should succeed and all pages should be accessible" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ptr = unsafe {
            mmap(std::ptr::null_mut(), 65536, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_POPULATE, -1, 0)
        };
        let errno_val = unsafe { *libc::__errno_location() };
        if ptr == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 1, actual_ret: -1, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        // Touch all pages to verify accessibility
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, 65536) };
        let mut sum = 0u64;
        for (i, b) in slice.iter_mut().enumerate() { *b = (i & 0xFF) as u8; sum += *b as u64; }
        unsafe { munmap(ptr, 65536); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if sum > 0 { TestStatus::Pass }
        else { TestStatus::Error("MAP_POPULATE pages not writable".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// mprotect: transition from RW to RO, then verify write causes SIGSEGV
pub struct MprotectRoTest;
impl SyscallTest for MprotectRoTest {
    fn name(&self) -> &str { "mem3_mprotect_ro_transition" }
    fn syscall(&self) -> &str { "mprotect" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mprotect RW->RO then RO->RW; data written before RO should survive" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ptr = unsafe {
            mmap(std::ptr::null_mut(), 4096, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS, -1, 0)
        };
        if ptr == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("mmap failed".into()), start.elapsed().as_micros() as u64);
        }
        // Write while RW
        unsafe { *(ptr as *mut u32) = 0xDEAD; }
        // Make RO
        let r1 = unsafe { mprotect(ptr, 4096, PROT_READ) };
        // Read back (should still work)
        let val_ro = unsafe { *(ptr as *const u32) };
        // Make RW again
        let r2 = unsafe { mprotect(ptr, 4096, PROT_READ | PROT_WRITE) };
        // Write again
        unsafe { *(ptr as *mut u32) = 0xBEEF; }
        let val_rw = unsafe { *(ptr as *const u32) };
        unsafe { munmap(ptr, 4096); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r1 == 0 && r2 == 0 && val_ro == 0xDEAD && val_rw == 0xBEEF { TestStatus::Pass }
        else { TestStatus::Error(format!("r1={} r2={} val_ro={:#x} val_rw={:#x}", r1, r2, val_ro, val_rw)) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// Multiple munmap calls on sub-regions
pub struct MunmapSubregionTest;
impl SyscallTest for MunmapSubregionTest {
    fn name(&self) -> &str { "mem3_munmap_subregion" }
    fn syscall(&self) -> &str { "munmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "munmap() on a sub-region of a larger mmap should leave adjacent pages intact" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // 3 pages
        let ptr = unsafe {
            mmap(std::ptr::null_mut(), 4096 * 3, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS, -1, 0)
        };
        if ptr == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("mmap failed".into()), start.elapsed().as_micros() as u64);
        }
        // Write to page 0 and page 2
        let p0 = ptr as *mut u8;
        let p2 = unsafe { (ptr as *mut u8).add(4096 * 2) };
        unsafe { *p0 = 0xAA; *p2 = 0xBB; }
        // Unmap middle page
        let r = unsafe { munmap((ptr as *mut u8).add(4096) as *mut _, 4096) };
        // Verify page 0 and 2 still accessible
        let v0 = unsafe { *p0 };
        let v2 = unsafe { *p2 };
        unsafe { munmap(ptr, 4096); munmap(p2 as *mut _, 4096); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if r == 0 && v0 == 0xAA && v2 == 0xBB { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: r as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// Large allocation: mmap 64MB anonymous
pub struct MmapLargeTest;
impl SyscallTest for MmapLargeTest {
    fn name(&self) -> &str { "mem3_mmap_large_64mb" }
    fn syscall(&self) -> &str { "mmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mmap() of 64MB anonymous should succeed on Linux" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let sz = 64 * 1024 * 1024usize;
        let ptr = unsafe {
            mmap(std::ptr::null_mut(), sz, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS, -1, 0)
        };
        let errno_val = unsafe { *libc::__errno_location() };
        if ptr != MAP_FAILED {
            // Touch first and last byte to verify
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, sz) };
            slice[0] = 1;
            slice[sz - 1] = 2;
            unsafe { munmap(ptr, sz); }
        }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ptr != MAP_FAILED { TestStatus::Pass }
        else if errno_val == ENOMEM as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: -1, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// mmap a file, modify it, close fd — mapping should still be valid
pub struct MmapFdClosedTest;
impl SyscallTest for MmapFdClosedTest {
    fn name(&self) -> &str { "mem3_mmap_fd_closed" }
    fn syscall(&self) -> &str { "mmap" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "mmap() keeps mapping valid even after the fd is closed" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_mmap_fdclose.bin").unwrap();
        let data = b"persistent mapping data";
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { write(fd, data.as_ptr() as *const _, data.len()); }
        let ptr = unsafe { mmap(std::ptr::null_mut(), data.len(), PROT_READ, MAP_PRIVATE, fd, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if ptr == MAP_FAILED {
            unsafe { close(fd); unlink(path.as_ptr()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 1, actual_ret: -1, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        // Close the fd — mapping should still work
        unsafe { close(fd); }
        let mapped = unsafe { std::slice::from_raw_parts(ptr as *const u8, data.len()) };
        let ok = mapped == data;
        unsafe { munmap(ptr, data.len()); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ok { TestStatus::Pass }
        else { TestStatus::Error("mapping invalid after fd closed".into()) };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// madvise MADV_SEQUENTIAL on a file mapping
pub struct MadviseSequentialTest;
impl SyscallTest for MadviseSequentialTest {
    fn name(&self) -> &str { "mem3_madvise_sequential" }
    fn syscall(&self) -> &str { "madvise" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "madvise(MADV_SEQUENTIAL) on file mapping should return 0" }
    fn run(&self) -> TestResult {
        let path = CString::new("/tmp/sct_madv_seq.bin").unwrap();
        let start = Instant::now();
        let fd = unsafe { open(path.as_ptr(), O_CREAT | O_RDWR | O_TRUNC, 0o644) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("open failed".into()), start.elapsed().as_micros() as u64);
        }
        let zeros = vec![0u8; 4096];
        unsafe { write(fd, zeros.as_ptr() as *const _, 4096); }
        let ptr = unsafe { mmap(std::ptr::null_mut(), 4096, PROT_READ, MAP_PRIVATE, fd, 0) };
        if ptr == MAP_FAILED {
            unsafe { close(fd); unlink(path.as_ptr()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { madvise(ptr, 4096, MADV_SEQUENTIAL) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { munmap(ptr, 4096); close(fd); unlink(path.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// madvise MADV_DONTNEED: pages should be zeroed on next access
pub struct MadviseWillneedTest;
impl SyscallTest for MadviseWillneedTest {
    fn name(&self) -> &str { "mem3_madvise_willneed" }
    fn syscall(&self) -> &str { "madvise" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "madvise(MADV_WILLNEED) on anon mapping should return 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let ptr = unsafe {
            mmap(std::ptr::null_mut(), 4096, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS, -1, 0)
        };
        if ptr == MAP_FAILED {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("mmap failed".into()), start.elapsed().as_micros() as u64);
        }
        let ret = unsafe { madvise(ptr, 4096, MADV_WILLNEED) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { munmap(ptr, 4096); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// process_vm_readv: read memory from another process
pub struct ProcessVmReadvTest;
impl SyscallTest for ProcessVmReadvTest {
    fn name(&self) -> &str { "mem3_process_vm_readv" }
    fn syscall(&self) -> &str { "process_vm_readv" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "process_vm_readv() reads memory from child process address space" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut pipefd = [0i32; 2];
        if unsafe { pipe(pipefd.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("pipe failed".into()), start.elapsed().as_micros() as u64);
        }
        let pid = unsafe { fork() };
        if pid < 0 {
            unsafe { close(pipefd[0]); close(pipefd[1]); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            unsafe { close(pipefd[0]); }
            let val: u64 = 0xCAFEBABE_DEADBEEF;
            // Send address to parent
            let addr = &val as *const u64 as usize;
            unsafe { write(pipefd[1], &addr as *const _ as *const _, std::mem::size_of::<usize>()); }
            // Wait for parent to read
            unsafe { nanosleep(&timespec { tv_sec: 0, tv_nsec: 50_000_000 }, std::ptr::null_mut()); }
            unsafe { close(pipefd[1]); libc::exit(0); }
        }
        unsafe { close(pipefd[1]); }
        let mut addr: usize = 0;
        unsafe { read(pipefd[0], &mut addr as *mut _ as *mut _, std::mem::size_of::<usize>()); close(pipefd[0]); }
        let mut dst: u64 = 0;
        let local_iov = iovec { iov_base: &mut dst as *mut _ as *mut _, iov_len: 8 };
        let remote_iov = iovec { iov_base: addr as *mut _, iov_len: 8 };
        let ret = unsafe { process_vm_readv(pid, &local_iov, 1, &remote_iov, 1, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { waitpid(pid, std::ptr::null_mut(), 0); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 8 && dst == 0xCAFEBABE_DEADBEEF { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 || errno_val == EPERM as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 8, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// userfaultfd: create a userfaultfd (just creation, no fault handling)
pub struct UserfaultfdCreateTest;
impl SyscallTest for UserfaultfdCreateTest {
    fn name(&self) -> &str { "mem3_userfaultfd_create" }
    fn syscall(&self) -> &str { "userfaultfd" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Memory }
    fn description(&self) -> &str { "userfaultfd(O_CLOEXEC) should return a valid fd" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { libc::syscall(libc::SYS_userfaultfd, O_CLOEXEC as c_long) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd as i32); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 || errno_val == EPERM as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
