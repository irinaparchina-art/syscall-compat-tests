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

pub struct SocketTcpCreateTest;
impl SyscallTest for SocketTcpCreateTest {
    fn name(&self) -> &str { "net_socket_tcp_create" }
    fn syscall(&self) -> &str { "socket" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "socket(AF_INET, SOCK_STREAM, 0) should return a valid fd" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); } }
        let dur = start.elapsed().as_micros() as u64;
        let status = if fd >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct SocketUdpCreateTest;
impl SyscallTest for SocketUdpCreateTest {
    fn name(&self) -> &str { "net_socket_udp_create" }
    fn syscall(&self) -> &str { "socket" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "socket(AF_INET, SOCK_DGRAM, 0) should return a valid fd" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); } }
        let dur = start.elapsed().as_micros() as u64;
        let status = if fd >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct SocketUnixCreateTest;
impl SyscallTest for SocketUnixCreateTest {
    fn name(&self) -> &str { "net_socket_unix_create" }
    fn syscall(&self) -> &str { "socket" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "socket(AF_UNIX, SOCK_STREAM, 0) should return a valid fd" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_UNIX, SOCK_STREAM, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        if fd >= 0 { unsafe { close(fd); } }
        let dur = start.elapsed().as_micros() as u64;
        let status = if fd >= 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: fd as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct SocketInvalidDomainTest;
impl SyscallTest for SocketInvalidDomainTest {
    fn name(&self) -> &str { "net_socket_invalid_domain" }
    fn syscall(&self) -> &str { "socket" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "socket(999, SOCK_STREAM, 0) with invalid domain should return -1 EINVAL or EAFNOSUPPORT" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(999, SOCK_STREAM, 0) } as i64;
        let errno_val = unsafe { *libc::__errno_location() };
        let dur = start.elapsed().as_micros() as u64;
        let status = if fd == -1 && (errno_val == EINVAL as i32 || errno_val == EAFNOSUPPORT as i32) { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: fd, expected_errno: Some(EAFNOSUPPORT as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct BindReuseAddrTest;
impl SyscallTest for BindReuseAddrTest {
    fn name(&self) -> &str { "net_bind_loopback" }
    fn syscall(&self) -> &str { "bind" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "bind() to 127.0.0.1:0 (auto-port) should succeed" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("socket failed".into()), start.elapsed().as_micros() as u64);
        }
        let opt: c_int = 1;
        unsafe { setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &opt as *const _ as *const _, std::mem::size_of_val(&opt) as u32) };
        let addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: 0u16.to_be(),
            sin_addr: in_addr { s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be() },
            sin_zero: [0; 8],
        };
        let ret = unsafe { bind(fd, &addr as *const _ as *const sockaddr, std::mem::size_of_val(&addr) as u32) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct ListenBasicTest;
impl SyscallTest for ListenBasicTest {
    fn name(&self) -> &str { "net_listen_basic" }
    fn syscall(&self) -> &str { "listen" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "listen() on a bound TCP socket should succeed" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("socket failed".into()), start.elapsed().as_micros() as u64);
        }
        let opt: c_int = 1;
        unsafe { setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &opt as *const _ as *const _, std::mem::size_of_val(&opt) as u32) };
        let addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: 0u16.to_be(),
            sin_addr: in_addr { s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be() },
            sin_zero: [0; 8],
        };
        unsafe { bind(fd, &addr as *const _ as *const sockaddr, std::mem::size_of_val(&addr) as u32) };
        let ret = unsafe { listen(fd, 5) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct GetsocknameTest;
impl SyscallTest for GetsocknameTest {
    fn name(&self) -> &str { "net_getsockname" }
    fn syscall(&self) -> &str { "getsockname" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "getsockname() after bind should return the bound address" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("socket failed".into()), start.elapsed().as_micros() as u64);
        }
        let addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: 0u16.to_be(),
            sin_addr: in_addr { s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be() },
            sin_zero: [0; 8],
        };
        unsafe { bind(fd, &addr as *const _ as *const sockaddr, std::mem::size_of_val(&addr) as u32) };
        let mut out_addr: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut len = std::mem::size_of::<sockaddr_in>() as u32;
        let ret = unsafe { getsockname(fd, &mut out_addr as *mut _ as *mut sockaddr, &mut len) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 && out_addr.sin_family == AF_INET as u16 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct SetsockoptReuseTest;
impl SyscallTest for SetsockoptReuseTest {
    fn name(&self) -> &str { "net_setsockopt_reuseaddr" }
    fn syscall(&self) -> &str { "setsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "setsockopt(SO_REUSEADDR) should succeed on a TCP socket" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("socket failed".into()), start.elapsed().as_micros() as u64);
        }
        let opt: c_int = 1;
        let ret = unsafe { setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &opt as *const _ as *const _, std::mem::size_of_val(&opt) as u32) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}

pub struct LoopbackConnectTest;
impl SyscallTest for LoopbackConnectTest {
    fn name(&self) -> &str { "net_loopback_connect_accept" }
    fn syscall(&self) -> &str { "connect/accept" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "connect() + accept() on loopback; send/recv a message" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        // Server socket
        let srv = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if srv < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("socket failed".into()), start.elapsed().as_micros() as u64);
        }
        let opt: c_int = 1;
        unsafe { setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &opt as *const _ as *const _, std::mem::size_of_val(&opt) as u32) };
        let addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: 0u16.to_be(),
            sin_addr: in_addr { s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be() },
            sin_zero: [0; 8],
        };
        if unsafe { bind(srv, &addr as *const _ as *const sockaddr, std::mem::size_of_val(&addr) as u32) } != 0 {
            unsafe { close(srv); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("bind failed".into()), start.elapsed().as_micros() as u64);
        }
        if unsafe { listen(srv, 1) } != 0 {
            unsafe { close(srv); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("listen failed".into()), start.elapsed().as_micros() as u64);
        }
        // Get assigned port
        let mut srv_addr: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut srv_len = std::mem::size_of::<sockaddr_in>() as u32;
        unsafe { getsockname(srv, &mut srv_addr as *mut _ as *mut sockaddr, &mut srv_len) };

        // Fork: child connects, parent accepts
        let pid = unsafe { fork() };
        if pid < 0 {
            unsafe { close(srv); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            // Child: connect and send
            unsafe { close(srv); }
            let cli = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
            unsafe { connect(cli, &srv_addr as *const _ as *const sockaddr, std::mem::size_of_val(&srv_addr) as u32) };
            unsafe { send(cli, b"hi\0".as_ptr() as *const _, 3, 0) };
            unsafe { close(cli); libc::exit(0); }
        }
        // Parent: accept and recv
        let mut cli_addr: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut cli_len = std::mem::size_of::<sockaddr_in>() as u32;
        let conn = unsafe { accept(srv, &mut cli_addr as *mut _ as *mut sockaddr, &mut cli_len) };
        let mut buf = [0u8; 8];
        let n = unsafe { recv(conn, buf.as_mut_ptr() as *mut _, buf.len(), 0) };
        unsafe { close(conn); close(srv); waitpid(pid, std::ptr::null_mut(), 0); }
        let dur = start.elapsed().as_micros() as u64;
        let status = if n == 3 && &buf[..3] == b"hi\0" { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 3, actual_ret: n as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), status, dur)
    }
}
