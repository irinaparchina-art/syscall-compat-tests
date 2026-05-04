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

/// SO_KEEPALIVE on TCP socket
pub struct SoKeepaliveTest;
impl SyscallTest for SoKeepaliveTest {
    fn name(&self) -> &str { "net3_so_keepalive" }
    fn syscall(&self) -> &str { "setsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "setsockopt(SO_KEEPALIVE, 1) then getsockopt should return 1" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let val: c_int = 1;
        unsafe { setsockopt(fd, SOL_SOCKET, SO_KEEPALIVE, &val as *const _ as *const _, std::mem::size_of_val(&val) as u32) };
        let mut got: c_int = 0;
        let mut len = std::mem::size_of::<c_int>() as u32;
        let ret = unsafe { getsockopt(fd, SOL_SOCKET, SO_KEEPALIVE, &mut got as *mut _ as *mut _, &mut len) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && got == 1 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: got as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// SO_RCVTIMEO: set receive timeout on socket
pub struct SoRcvtimeoTest;
impl SyscallTest for SoRcvtimeoTest {
    fn name(&self) -> &str { "net3_so_rcvtimeo" }
    fn syscall(&self) -> &str { "setsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "setsockopt(SO_RCVTIMEO) then recv on empty socket should return EAGAIN/EWOULDBLOCK" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut sv = [0i32; 2];
        if unsafe { socketpair(AF_UNIX, SOCK_STREAM, 0, sv.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let tv = timeval { tv_sec: 0, tv_usec: 1000 }; // 1ms
        unsafe { setsockopt(sv[0], SOL_SOCKET, SO_RCVTIMEO, &tv as *const _ as *const _, std::mem::size_of_val(&tv) as u32) };
        let mut buf = [0u8; 8];
        let ret = unsafe { recv(sv[0], buf.as_mut_ptr() as *mut _, buf.len(), 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(sv[0]); close(sv[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == -1 && (errno_val == EAGAIN as i32 || errno_val == EWOULDBLOCK as i32) { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: -1, actual_ret: ret as i64, expected_errno: Some(EAGAIN as i32), actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// TCP_NODELAY: disable Nagle algorithm
pub struct TcpNodelayTest;
impl SyscallTest for TcpNodelayTest {
    fn name(&self) -> &str { "net3_tcp_nodelay" }
    fn syscall(&self) -> &str { "setsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "setsockopt(TCP_NODELAY, 1) should succeed on TCP socket" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let val: c_int = 1;
        let ret = unsafe { setsockopt(fd, IPPROTO_TCP, TCP_NODELAY, &val as *const _ as *const _, std::mem::size_of_val(&val) as u32) };
        let errno_val = unsafe { *libc::__errno_location() };
        let mut got: c_int = 0;
        let mut len = std::mem::size_of::<c_int>() as u32;
        unsafe { getsockopt(fd, IPPROTO_TCP, TCP_NODELAY, &mut got as *mut _ as *mut _, &mut len) };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && got == 1 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: got as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// SO_LINGER: set linger option
pub struct SoLingerTest;
impl SyscallTest for SoLingerTest {
    fn name(&self) -> &str { "net3_so_linger" }
    fn syscall(&self) -> &str { "setsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "setsockopt(SO_LINGER) should persist and be readable via getsockopt" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let ling = linger { l_onoff: 1, l_linger: 5 };
        unsafe { setsockopt(fd, SOL_SOCKET, SO_LINGER, &ling as *const _ as *const _, std::mem::size_of_val(&ling) as u32) };
        let mut got: linger = unsafe { std::mem::zeroed() };
        let mut len = std::mem::size_of::<linger>() as u32;
        let ret = unsafe { getsockopt(fd, SOL_SOCKET, SO_LINGER, &mut got as *mut _ as *mut _, &mut len) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && got.l_onoff == 1 && got.l_linger == 5 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// SO_ERROR: get and clear pending socket error
pub struct SoErrorTest;
impl SyscallTest for SoErrorTest {
    fn name(&self) -> &str { "net3_so_error_clear" }
    fn syscall(&self) -> &str { "getsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "getsockopt(SO_ERROR) after successful connect should return 0" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut sv = [0i32; 2];
        if unsafe { socketpair(AF_UNIX, SOCK_STREAM, 0, sv.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let mut err: c_int = -1;
        let mut len = std::mem::size_of::<c_int>() as u32;
        let ret = unsafe { getsockopt(sv[0], SOL_SOCKET, SO_ERROR, &mut err as *mut _ as *mut _, &mut len) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(sv[0]); close(sv[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && err == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: err as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// recvmsg / sendmsg: scatter-gather I/O on socket
pub struct RecvmsgSendmsgTest;
impl SyscallTest for RecvmsgSendmsgTest {
    fn name(&self) -> &str { "net3_sendmsg_recvmsg" }
    fn syscall(&self) -> &str { "sendmsg/recvmsg" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "sendmsg() with two iov buffers; recvmsg() should reconstruct the message" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let mut sv = [0i32; 2];
        if unsafe { socketpair(AF_UNIX, SOCK_STREAM, 0, sv.as_mut_ptr()) } != 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let part1 = b"hello";
        let part2 = b"world";
        let send_iov = [
            iovec { iov_base: part1.as_ptr() as *mut _, iov_len: 5 },
            iovec { iov_base: part2.as_ptr() as *mut _, iov_len: 5 },
        ];
        let send_msg = msghdr {
            msg_name: std::ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: send_iov.as_ptr() as *mut _,
            msg_iovlen: 2,
            msg_control: std::ptr::null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        };
        let n_sent = unsafe { sendmsg(sv[0], &send_msg, 0) };
        let mut buf = [0u8; 10];
        let recv_iov = [iovec { iov_base: buf.as_mut_ptr() as *mut _, iov_len: 10 }];
        let mut recv_msg = msghdr {
            msg_name: std::ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: recv_iov.as_ptr() as *mut _,
            msg_iovlen: 1,
            msg_control: std::ptr::null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        };
        let n_recv = unsafe { recvmsg(sv[1], &mut recv_msg, 0) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(sv[0]); close(sv[1]); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if n_sent == 10 && n_recv == 10 && &buf == b"helloworld" { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 10, actual_ret: n_recv as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// IP_TTL: set and get time-to-live on UDP socket
pub struct IpTtlTest;
impl SyscallTest for IpTtlTest {
    fn name(&self) -> &str { "net3_ip_ttl" }
    fn syscall(&self) -> &str { "setsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "setsockopt(IP_TTL, 64) then getsockopt should return 64" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let ttl: c_int = 64;
        unsafe { setsockopt(fd, IPPROTO_IP, IP_TTL, &ttl as *const _ as *const _, std::mem::size_of_val(&ttl) as u32) };
        let mut got: c_int = 0;
        let mut len = std::mem::size_of::<c_int>() as u32;
        let ret = unsafe { getsockopt(fd, IPPROTO_IP, IP_TTL, &mut got as *mut _ as *mut _, &mut len) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 && got == 64 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 64, actual_ret: got as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// accept4 with SOCK_NONBLOCK flag
pub struct Accept4Test;
impl SyscallTest for Accept4Test {
    fn name(&self) -> &str { "net3_accept4_nonblock" }
    fn syscall(&self) -> &str { "accept4" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "accept4(SOCK_NONBLOCK) on listening socket with pending conn returns nonblocking fd" }
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
            unsafe { nanosleep(&timespec { tv_sec: 0, tv_nsec: 50_000_000 }, std::ptr::null_mut()); }
            unsafe { close(cli); libc::exit(0) };
        }
        let conn = unsafe { accept4(srv, std::ptr::null_mut(), std::ptr::null_mut(), SOCK_NONBLOCK) };
        let errno_val = unsafe { *libc::__errno_location() };
        let flags = if conn >= 0 { unsafe { fcntl(conn, F_GETFL) } } else { -1 };
        if conn >= 0 { unsafe { close(conn); } }
        unsafe { close(srv); waitpid(pid, std::ptr::null_mut(), 0); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if conn >= 0 && flags & O_NONBLOCK != 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 1, actual_ret: conn as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}

/// SO_BROADCAST on UDP socket
pub struct SoBroadcastTest;
impl SyscallTest for SoBroadcastTest {
    fn name(&self) -> &str { "net3_so_broadcast" }
    fn syscall(&self) -> &str { "setsockopt" }
    fn category(&self) -> SyscallCategory { SyscallCategory::Network }
    fn description(&self) -> &str { "setsockopt(SO_BROADCAST, 1) on UDP socket should succeed" }
    fn run(&self) -> TestResult {
        let start = Instant::now();
        let fd = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
        if fd < 0 {
            return make_result(self.name(), self.syscall(), self.category(), self.description(),
                TestStatus::Unimplemented, start.elapsed().as_micros() as u64);
        }
        let val: c_int = 1;
        let ret = unsafe { setsockopt(fd, SOL_SOCKET, SO_BROADCAST, &val as *const _ as *const _, std::mem::size_of_val(&val) as u32) };
        let errno_val = unsafe { *libc::__errno_location() };
        unsafe { close(fd); }
        let dur = start.elapsed().as_micros() as u64;
        let s = if ret == 0 { TestStatus::Pass }
        else if errno_val == ENOSYS as i32 { TestStatus::Unimplemented }
        else { TestStatus::Fail { expected_ret: 0, actual_ret: ret as i64, expected_errno: None, actual_errno: Some(errno_val) } };
        make_result(self.name(), self.syscall(), self.category(), self.description(), s, dur)
    }
}
