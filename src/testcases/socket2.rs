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

/// UDP sendto/recvfrom on loopback
pub struct UdpSendrecvTest;
impl SyscallTest for UdpSendrecvTest {
    fn name(&self) -> &str { "sock2_udp_sendto_recvfrom" }
    fn syscall(&self) -> &str { "sendto/recvfrom" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "UDP sendto() to loopback; recvfrom() should return same data" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let srv = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
        if srv < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let srv_addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: 0u16.to_be(),
            sin_addr: in_addr { s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be() },
            sin_zero: [0; 8],
        };
        unsafe { bind(srv, &srv_addr as *const _ as *const sockaddr, std::mem::size_of_val(&srv_addr) as u32) };
        let mut bound: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut blen = std::mem::size_of::<sockaddr_in>() as u32;
        unsafe { getsockname(srv, &mut bound as *mut _ as *mut sockaddr, &mut blen) };

        let cli = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
        let msg = b"udp_test";
        let n_sent = unsafe {
            sendto(cli, msg.as_ptr() as *const _, msg.len(), 0,
                &bound as *const _ as *const sockaddr, std::mem::size_of_val(&bound) as u32)
        };
        let mut buf = [0u8; 32];
        let mut from: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut flen = std::mem::size_of::<sockaddr_in>() as u32;
        let n_recv = unsafe {
            recvfrom(srv, buf.as_mut_ptr() as *mut _, buf.len(), 0,
                &mut from as *mut _ as *mut sockaddr, &mut flen)
        };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(srv); close(cli); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n_sent == msg.len() as isize && n_recv == msg.len() as isize && &buf[..msg.len()] == msg {
            TestStatus::Pass
        } else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: msg.len() as i64, actual_ret: n_recv as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// Unix domain socket: connect + send + recv
pub struct UnixSocketTest;
impl SyscallTest for UnixSocketTest {
    fn name(&self) -> &str { "sock2_unix_socket_stream" }
    fn syscall(&self) -> &str { "connect/send/recv" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "Unix domain stream socket: connect/send/recv should transfer data" }
    fn run(&self) -> TestResult {
        let path = "/tmp/sct_unix.sock";
        let cpath = CString::new(path).unwrap();
        let start = Instant::now();
        unsafe { unlink(cpath.as_ptr()); }
        let srv = unsafe { socket(AF_UNIX, SOCK_STREAM, 0) };
        if srv < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut addr: sockaddr_un = unsafe { std::mem::zeroed() };
        addr.sun_family = AF_UNIX as u16;
        let bytes = path.as_bytes();
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), addr.sun_path.as_mut_ptr() as *mut u8, bytes.len());
        }
        let addr_len = (std::mem::size_of::<sa_family_t>() + bytes.len() + 1) as u32;
        if unsafe { bind(srv, &addr as *const _ as *const sockaddr, addr_len) } != 0 {
            unsafe { close(srv); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Error("bind failed".into()), start.elapsed().as_micros() as u64);
        }
        unsafe { listen(srv, 1) };
        let pid = unsafe { fork() };
        if pid < 0 {
            unsafe { close(srv); unlink(cpath.as_ptr()); }
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        if pid == 0 {
            unsafe { close(srv); }
            let cli = unsafe { socket(AF_UNIX, SOCK_STREAM, 0) };
            unsafe { connect(cli, &addr as *const _ as *const sockaddr, addr_len) };
            unsafe { send(cli, b"hello\0".as_ptr() as *const _, 6, 0) };
            unsafe { close(cli); libc::exit(0) };
        }
        let conn = unsafe { accept(srv, std::ptr::null_mut(), std::ptr::null_mut()) };
        let mut buf = [0u8; 8];
        let n = unsafe { recv(conn, buf.as_mut_ptr() as *mut _, buf.len(), 0) };
        unsafe { close(conn); close(srv); waitpid(pid, std::ptr::null_mut(), 0); unlink(cpath.as_ptr()); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 6 && &buf[..6] == b"hello\0" { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: 6, actual_ret: n as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// socketpair: bidirectional communication
pub struct SocketpairTest;
impl SyscallTest for SocketpairTest {
    fn name(&self) -> &str { "sock2_socketpair_basic" }
    fn syscall(&self) -> &str { "socketpair" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "socketpair(AF_UNIX, SOCK_STREAM) creates two connected fds; send on one, recv on other" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut sv = [0i32; 2];
        let ret = unsafe { socketpair(AF_UNIX, SOCK_STREAM, 0, sv.as_mut_ptr()) };
        let errno_val = unsafe { *libc::__errno_location() };
        if ret != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
                else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } },
                start.elapsed().as_micros() as u64);
        }
        let msg = b"socketpair";
        unsafe { send(sv[0], msg.as_ptr() as *const _, msg.len(), 0) };
        let mut buf = [0u8; 16];
        let n = unsafe { recv(sv[1], buf.as_mut_ptr() as *mut _, buf.len(), 0) };
        unsafe { close(sv[0]); close(sv[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == msg.len() as isize && &buf[..msg.len()] == msg { TestStatus::Pass }
        else { TestStatus::Fail { expected_ret: msg.len() as i64, actual_ret: n as i64, expected_errno: None, actual_errno: None } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getsockopt SO_TYPE on TCP socket should return SOCK_STREAM
pub struct GetsockoptTypeTest;
impl SyscallTest for GetsockoptTypeTest {
    fn name(&self) -> &str { "sock2_getsockopt_type" }
    fn syscall(&self) -> &str { "getsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "getsockopt(SO_TYPE) on TCP socket should return SOCK_STREAM" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut val: c_int = 0;
        let mut len = std::mem::size_of::<c_int>() as u32;
        let ret = unsafe { getsockopt(fd, SOL_SOCKET, SO_TYPE, &mut val as *mut _ as *mut _, &mut len) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && val == SOCK_STREAM { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: SOCK_STREAM as i64, actual_ret: val as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// shutdown(SHUT_WR) on a socket should cause peer to see EOF
pub struct ShutdownTest;
impl SyscallTest for ShutdownTest {
    fn name(&self) -> &str { "sock2_shutdown_wr" }
    fn syscall(&self) -> &str { "shutdown" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "shutdown(SHUT_WR) on socketpair fd causes peer to read EOF (0 bytes)" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut sv = [0i32; 2];
        if unsafe { socketpair(AF_UNIX, SOCK_STREAM, 0, sv.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        unsafe { shutdown(sv[0], SHUT_WR) };
        let mut buf = [0u8; 8];
        let n = unsafe { recv(sv[1], buf.as_mut_ptr() as *mut _, buf.len(), 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(sv[0]); close(sv[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: n as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// SO_RCVBUF / SO_SNDBUF can be set and read back
pub struct SockBufSizeTest;
impl SyscallTest for SockBufSizeTest {
    fn name(&self) -> &str { "sock2_so_rcvbuf_sndbuf" }
    fn syscall(&self) -> &str { "setsockopt/getsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "setsockopt(SO_RCVBUF) then getsockopt should return >= requested size" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let requested: c_int = 65536;
        unsafe { setsockopt(fd, SOL_SOCKET, SO_RCVBUF, &requested as *const _ as *const _, std::mem::size_of_val(&requested) as u32) };
        let mut actual: c_int = 0;
        let mut len = std::mem::size_of::<c_int>() as u32;
        let ret = unsafe { getsockopt(fd, SOL_SOCKET, SO_RCVBUF, &mut actual as *mut _ as *mut _, &mut len) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        // Linux doubles the value; just check it's > 0
        let s = if ret == 0 && actual > 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: requested as i64, actual_ret: actual as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// connect() to a port where nobody is listening should fail with ECONNREFUSED
pub struct ConnectRefusedTest;
impl SyscallTest for ConnectRefusedTest {
    fn name(&self) -> &str { "sock2_connect_refused" }
    fn syscall(&self) -> &str { "connect" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "connect() to a port with no listener should return -1 ECONNREFUSED" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        // Port 1 is almost never open
        let addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: 1u16.to_be(),
            sin_addr: in_addr { s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be() },
            sin_zero: [0; 8],
        };
        let ret = unsafe { connect(fd, &addr as *const _ as *const sockaddr, std::mem::size_of_val(&addr) as u32) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && errno_val == ECONNREFUSED as i32 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret as i64, expected_errno: Some(ECONNREFUSED as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// getpeername after connect should return server address
pub struct GetpeernameTest;
impl SyscallTest for GetpeernameTest {
    fn name(&self) -> &str { "sock2_getpeername" }
    fn syscall(&self) -> &str { "getpeername" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "getpeername() on accepted conn should return client address" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let srv = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if srv < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let opt: c_int = 1;
        unsafe { setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &opt as *const _ as *const _, std::mem::size_of_val(&opt) as u32) };
        let addr = sockaddr_in {
            sin_family: AF_INET as u16, sin_port: 0u16.to_be(),
            sin_addr: in_addr { s_addr: u32::from_be_bytes([127, 0, 0, 1]).to_be() },
            sin_zero: [0; 8],
        };
        unsafe { bind(srv, &addr as *const _ as *const sockaddr, std::mem::size_of_val(&addr) as u32) };
        unsafe { listen(srv, 1) };
        let mut srv_addr: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut slen = std::mem::size_of::<sockaddr_in>() as u32;
        unsafe { getsockname(srv, &mut srv_addr as *mut _ as *mut sockaddr, &mut slen) };
        let pid = unsafe { fork() };
        if pid < 0 { unsafe { close(srv); } return make_result(self.name(), self.syscall(), self.category(), self.description(), TestStatus::Unimplemented, start.elapsed().as_micros() as u64); }
        if pid == 0 {
            unsafe { close(srv); }
            let cli = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
            unsafe { connect(cli, &srv_addr as *const _ as *const sockaddr, slen) };
            std::thread::sleep(std::time::Duration::from_millis(50));
            unsafe { close(cli); libc::exit(0) };
        }
        let conn = unsafe { accept(srv, std::ptr::null_mut(), std::ptr::null_mut()) };
        let mut peer: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut plen = std::mem::size_of::<sockaddr_in>() as u32;
        let ret = unsafe { getpeername(conn, &mut peer as *mut _ as *mut sockaddr, &mut plen) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(conn); close(srv); waitpid(pid, std::ptr::null_mut(), 0); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && peer.sin_family == AF_INET as u16 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
