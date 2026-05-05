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

/// socket with SOCK_NONBLOCK flag
pub struct SocketNonblockTest;
impl SyscallTest for SocketNonblockTest {
    fn name(&self) -> &str { "net4_socket_nonblock" }
    fn syscall(&self) -> &str { "socket" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "socket(SOCK_STREAM|SOCK_NONBLOCK) should create a non-blocking socket" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 {
            let flags = unsafe { fcntl(fd, F_GETFL) };
            let ok = flags & O_NONBLOCK != 0;
            unsafe { close(fd); }
            let dur = start.elapsed().as_micros() as u64;
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if ok { TestStatus::Pass } else { TestStatus::Fail { expected_ret: 1, actual_ret: 0, expected_errno: None, actual_errno: None } }, dur);
        }
        let dur = start.elapsed().as_micros() as u64;
        let s = if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// socket with SOCK_CLOEXEC flag
pub struct SocketCloexecTest;
impl SyscallTest for SocketCloexecTest {
    fn name(&self) -> &str { "net4_socket_cloexec" }
    fn syscall(&self) -> &str { "socket" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "socket(SOCK_STREAM|SOCK_CLOEXEC) should set FD_CLOEXEC on the socket fd" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM | SOCK_CLOEXEC, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 {
            let flags = unsafe { fcntl(fd, F_GETFD) };
            let ok = flags & FD_CLOEXEC != 0;
            unsafe { close(fd); }
            let dur = start.elapsed().as_micros() as u64;
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if ok { TestStatus::Pass } else { TestStatus::Fail { expected_ret: FD_CLOEXEC as i64, actual_ret: flags as i64, expected_errno: None, actual_errno: None } }, dur);
        }
        let dur = start.elapsed().as_micros() as u64;
        let s = if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// UDP socket: bind to port 0 gets assigned port
pub struct UdpBindAutoPortTest;
impl SyscallTest for UdpBindAutoPortTest {
    fn name(&self) -> &str { "net4_udp_bind_auto_port" }
    fn syscall(&self) -> &str { "bind/getsockname" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "UDP bind(port=0) should assign a non-zero ephemeral port" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let addr = sockaddr_in {
            sin_family: AF_INET as u16, sin_port: 0u16.to_be(),
            sin_addr: in_addr { s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be() },
            sin_zero: [0; 8],
        };
        unsafe { bind(fd, &addr as *const _ as *const sockaddr, std::mem::size_of_val(&addr) as u32) };
        let mut bound: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut len = std::mem::size_of::<sockaddr_in>() as u32;
        unsafe { getsockname(fd, &mut bound as *mut _ as *mut sockaddr, &mut len) };
        let port = u16::from_be(bound.sin_port);
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if port > 0 { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: port as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// TCP SO_REUSEPORT allows multiple sockets on same port
pub struct SoReuseportTest;
impl SyscallTest for SoReuseportTest {
    fn name(&self) -> &str { "net4_so_reuseport" }
    fn syscall(&self) -> &str { "setsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "setsockopt(SO_REUSEPORT) should allow binding same port twice" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd1 = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd1 < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let opt: c_int = 1;
        unsafe { setsockopt(fd1, SOL_SOCKET, SO_REUSEADDR, &opt as *const _ as *const _, std::mem::size_of_val(&opt) as u32) };
        unsafe { setsockopt(fd1, SOL_SOCKET, SO_REUSEPORT, &opt as *const _ as *const _, std::mem::size_of_val(&opt) as u32) };
        let addr = sockaddr_in {
            sin_family: AF_INET as u16, sin_port: 0u16.to_be(),
            sin_addr: in_addr { s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be() },
            sin_zero: [0; 8],
        };
        unsafe { bind(fd1, &addr as *const _ as *const sockaddr, std::mem::size_of_val(&addr) as u32) };
        let mut bound: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut blen = std::mem::size_of::<sockaddr_in>() as u32;
        unsafe { getsockname(fd1, &mut bound as *mut _ as *mut sockaddr, &mut blen) };
        let fd2 = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        unsafe { setsockopt(fd2, SOL_SOCKET, SO_REUSEADDR, &opt as *const _ as *const _, std::mem::size_of_val(&opt) as u32) };
        unsafe { setsockopt(fd2, SOL_SOCKET, SO_REUSEPORT, &opt as *const _ as *const _, std::mem::size_of_val(&opt) as u32) };
        let ret = unsafe { bind(fd2, &bound as *const _ as *const sockaddr, blen) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd1); close(fd2); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// connect to INADDR_LOOPBACK (127.0.0.1) vs INADDR_ANY (0.0.0.0) distinction
pub struct InAddrLoopbackTest;
impl SyscallTest for InAddrLoopbackTest {
    fn name(&self) -> &str { "net4_inaddr_loopback_listen" }
    fn syscall(&self) -> &str { "bind/listen" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "Server bound to INADDR_ANY should accept connections from loopback" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let srv = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if srv < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let opt: c_int = 1;
        unsafe { setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &opt as *const _ as *const _, std::mem::size_of_val(&opt) as u32) };
        // Bind to INADDR_ANY
        let addr = sockaddr_in {
            sin_family: AF_INET as u16, sin_port: 0u16.to_be(),
            sin_addr: in_addr { s_addr: INADDR_ANY.to_be() },
            sin_zero: [0; 8],
        };
        unsafe { bind(srv, &addr as *const _ as *const sockaddr, std::mem::size_of_val(&addr) as u32) };
        unsafe { listen(srv, 1) };
        let mut srv_addr: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut slen = std::mem::size_of::<sockaddr_in>() as u32;
        unsafe { getsockname(srv, &mut srv_addr as *mut _ as *mut sockaddr, &mut slen) };
        // Connect from loopback
        srv_addr.sin_addr.s_addr = u32::from_be_bytes([127, 0, 0, 1]).to_be();
        let pid = unsafe { fork() };
        if pid < 0 { unsafe { close(srv); } return make_result(self.name(), self.syscall(), self.category(), self.description(), TestStatus::Unimplemented, start.elapsed().as_micros() as u64); }
        if pid == 0 {
            unsafe { close(srv); }
            let cli = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
            unsafe { connect(cli, &srv_addr as *const _ as *const sockaddr, slen) };
            unsafe { close(cli); libc::exit(0) };
        }
        let conn = unsafe { accept(srv, std::ptr::null_mut(), std::ptr::null_mut()) };
        let ok = conn >= 0;
        if ok { unsafe { close(conn); } }
        unsafe { close(srv); waitpid(pid, std::ptr::null_mut(), 0); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ok { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: conn as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// SO_SNDBUF: set and verify send buffer size
pub struct SoSndbufTest;
impl SyscallTest for SoSndbufTest {
    fn name(&self) -> &str { "net4_so_sndbuf" }
    fn syscall(&self) -> &str { "setsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "setsockopt(SO_SNDBUF) then getsockopt should return a positive value" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let requested: c_int = 32768;
        unsafe { setsockopt(fd, SOL_SOCKET, SO_SNDBUF, &requested as *const _ as *const _, std::mem::size_of_val(&requested) as u32) };
        let mut got: c_int = 0;
        let mut len = std::mem::size_of::<c_int>() as u32;
        let ret = unsafe { getsockopt(fd, SOL_SOCKET, SO_SNDBUF, &mut got as *mut _ as *mut _, &mut len) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && got > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: got as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// IPV6 socket creation
pub struct Ipv6SocketTest;
impl SyscallTest for Ipv6SocketTest {
    fn name(&self) -> &str { "net4_ipv6_socket_create" }
    fn syscall(&self) -> &str { "socket" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "socket(AF_INET6, SOCK_STREAM) should return a valid fd on IPv6-capable systems" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET6, SOCK_STREAM, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); } }
        let dur = start.elapsed().as_micros() as u64;
        let s = if fd >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 || errno_val == EAFNOSUPPORT as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// recvfrom with NULL addr on connected UDP socket
pub struct RecvfromNullAddrTest;
impl SyscallTest for RecvfromNullAddrTest {
    fn name(&self) -> &str { "net4_recvfrom_null_addr" }
    fn syscall(&self) -> &str { "recvfrom" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "recvfrom() with NULL addr/addrlen on socketpair should succeed" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut sv = [0i32; 2];
        if unsafe { socketpair(AF_UNIX, SOCK_DGRAM, 0, sv.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        unsafe { send(sv[0], b"hi".as_ptr() as *const _, 2, 0) };
        let mut buf = [0u8; 8];
        let n = unsafe { recvfrom(sv[1], buf.as_mut_ptr() as *mut _, 8, 0, std::ptr::null_mut(), std::ptr::null_mut()) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(sv[0]); close(sv[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 2 && &buf[..2] == b"hi" { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 2, actual_ret: n as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
