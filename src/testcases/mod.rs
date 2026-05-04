pub mod dirent;
pub mod dirent2;
pub mod errno_edge;
pub mod errno_edge2;
pub mod file_io;
pub mod file_io2;
pub mod filestat;
pub mod io_advanced;
pub mod ipc;
pub mod memory;
pub mod memory2;
pub mod misc;
pub mod network;
pub mod pipe;
pub mod poll_select;
pub mod process;
pub mod process2;
pub mod resource;
pub mod signal;
pub mod memory3;
pub mod network3;
pub mod fcntl2;
pub mod io_uring;
pub mod misc2;
pub mod pipe2;
pub mod time3;
pub mod process3;
pub mod signal2;
pub mod socket2;
pub mod sysinfo;
pub mod thread;
pub mod time;
pub mod time2;

use crate::runner::TestRunner;

pub fn register_all(runner: &mut TestRunner) {
    // File I/O basic (16)
    runner.register(Box::new(file_io::OpenBasicTest));
    runner.register(Box::new(file_io::OpenNonexistentTest));
    runner.register(Box::new(file_io::ReadWriteTest));
    runner.register(Box::new(file_io::ReadZeroTest));
    runner.register(Box::new(file_io::WriteAppendTest));
    runner.register(Box::new(file_io::CloseInvalidFdTest));
    runner.register(Box::new(file_io::StatBasicTest));
    runner.register(Box::new(file_io::StatNonexistentTest));
    runner.register(Box::new(file_io::LseekBasicTest));
    runner.register(Box::new(file_io::LseekInvalidTest));
    runner.register(Box::new(file_io::UnlinkTest));
    runner.register(Box::new(file_io::RenameTest));
    runner.register(Box::new(file_io::MkdirRmdirTest));
    runner.register(Box::new(file_io::GetcwdTest));
    runner.register(Box::new(file_io::DupTest));
    runner.register(Box::new(file_io::Dup2Test));
    // File I/O advanced (12)
    runner.register(Box::new(file_io2::OpenExclExistsTest));
    runner.register(Box::new(file_io2::WriteReadonlyFdTest));
    runner.register(Box::new(file_io2::ReadWriteonlyFdTest));
    runner.register(Box::new(file_io2::LseekHoleTest));
    runner.register(Box::new(file_io2::OpenNoentTest));
    runner.register(Box::new(file_io2::FlockExclusiveTest));
    runner.register(Box::new(file_io2::FallocateTest));
    runner.register(Box::new(file_io2::CopyFileRangeTest));
    runner.register(Box::new(file_io2::LinkatTest));
    runner.register(Box::new(file_io2::UnlinkatTest));
    runner.register(Box::new(file_io2::RenameatTest));
    runner.register(Box::new(file_io2::ReadaheadTest));
    // Errno edge cases (17)
    runner.register(Box::new(errno_edge::ReadClosedFdTest));
    runner.register(Box::new(errno_edge::WriteClosedFdTest));
    runner.register(Box::new(errno_edge::LseekClosedFdTest));
    runner.register(Box::new(errno_edge::MmapBadProtTest));
    runner.register(Box::new(errno_edge::KillInvalidSigTest));
    runner.register(Box::new(errno_edge::FcntlInvalidCmdTest));
    runner.register(Box::new(errno_edge::OpenDeepNoentTest));
    runner.register(Box::new(errno_edge::RmdirNotemptyTest));
    runner.register(Box::new(errno_edge::OpenEnotdirTest));
    runner.register(Box::new(errno_edge::WritePipeEpipeTest));
    runner.register(Box::new(errno_edge::OpenEaccesTest));
    runner.register(Box::new(errno_edge::ChmodEaccesTest));
    runner.register(Box::new(errno_edge::OpenIsdirTest));
    runner.register(Box::new(errno_edge::SymlinkLoopTest));
    runner.register(Box::new(errno_edge::OpenNametoolongTest));
    runner.register(Box::new(errno_edge::WriteZeroBytesTest));
    runner.register(Box::new(errno_edge::LseekEndEmptyFileTest));
    runner.register(Box::new(errno_edge::StatVsLstatTest));
    runner.register(Box::new(errno_edge::FstatStatConsistencyTest));
    runner.register(Box::new(errno_edge::ReadEofTest));
    // Errno edge cases 2 (10)
    runner.register(Box::new(errno_edge2::MmapNegativeOffsetTest));
    runner.register(Box::new(errno_edge2::MunmapInvalidAddrTest));
    runner.register(Box::new(errno_edge2::WaitpidEchildTest));
    runner.register(Box::new(errno_edge2::MkdirEexistTest));
    runner.register(Box::new(errno_edge2::RenameSelfTest));
    runner.register(Box::new(errno_edge2::UnlinkDirTest));
    runner.register(Box::new(errno_edge2::FtruncateReadonlyTest));
    runner.register(Box::new(errno_edge2::Pipe2InvalidFlagsTest));
    runner.register(Box::new(errno_edge2::Dup2SelfTest));
    runner.register(Box::new(errno_edge2::OpenTruncNowriteTest));
        // File stat / permissions (10)
    runner.register(Box::new(filestat::ChmodBasicTest));
    runner.register(Box::new(filestat::TruncateBasicTest));
    runner.register(Box::new(filestat::FtruncateTest));
    runner.register(Box::new(filestat::FstatTest));
    runner.register(Box::new(filestat::AccessReadableTest));
    runner.register(Box::new(filestat::AccessNotExistTest));
    runner.register(Box::new(filestat::SymlinkTest));
    runner.register(Box::new(filestat::ReadlinkTest));
    runner.register(Box::new(filestat::HardlinkTest));
    runner.register(Box::new(filestat::StatfsTest));
    // Advanced I/O (8)
    runner.register(Box::new(io_advanced::WritevBasicTest));
    runner.register(Box::new(io_advanced::ReadvBasicTest));
    runner.register(Box::new(io_advanced::PreadBasicTest));
    runner.register(Box::new(io_advanced::PwriteBasicTest));
    runner.register(Box::new(io_advanced::SendfileBasicTest));
    runner.register(Box::new(io_advanced::SpliceBasicTest));
    runner.register(Box::new(io_advanced::FsyncTest));
    runner.register(Box::new(io_advanced::FdatasyncTest));
    // Memory (5)
    runner.register(Box::new(memory::BrkBasicTest));
    runner.register(Box::new(memory::MmapAnonTest));
    runner.register(Box::new(memory::MmapReadWriteTest));
    runner.register(Box::new(memory::MunmapTest));
    runner.register(Box::new(memory::MprotectTest));
    // Memory advanced (6)
    runner.register(Box::new(memory2::MmapFileTest));
    runner.register(Box::new(memory2::MsyncTest));
    runner.register(Box::new(memory2::MadviseTest));
    runner.register(Box::new(memory2::MlockTest));
    runner.register(Box::new(memory2::MmapOffsetTest));
    runner.register(Box::new(memory2::MremapTest));
    // Network (9)
    runner.register(Box::new(network::SocketTcpCreateTest));
    runner.register(Box::new(network::SocketUdpCreateTest));
    runner.register(Box::new(network::SocketUnixCreateTest));
    runner.register(Box::new(network::SocketInvalidDomainTest));
    runner.register(Box::new(network::BindReuseAddrTest));
    runner.register(Box::new(network::ListenBasicTest));
    runner.register(Box::new(network::GetsocknameTest));
    runner.register(Box::new(network::SetsockoptReuseTest));
    runner.register(Box::new(network::LoopbackConnectTest));
    // Socket advanced (8)
    runner.register(Box::new(socket2::UdpSendrecvTest));
    runner.register(Box::new(socket2::UnixSocketTest));
    runner.register(Box::new(socket2::SocketpairTest));
    runner.register(Box::new(socket2::GetsockoptTypeTest));
    runner.register(Box::new(socket2::ShutdownTest));
    runner.register(Box::new(socket2::SockBufSizeTest));
    runner.register(Box::new(socket2::ConnectRefusedTest));
    runner.register(Box::new(socket2::GetpeernameTest));
    // Pipe / fcntl (6)
    runner.register(Box::new(pipe::PipeBasicTest));
    runner.register(Box::new(pipe::PipeReadEofTest));
    runner.register(Box::new(pipe::Pipe2CloseOnExecTest));
    runner.register(Box::new(pipe::FcntlGetFdTest));
    runner.register(Box::new(pipe::FcntlSetNonblockTest));
    runner.register(Box::new(pipe::ReadNonblockTest));
    // Poll / select / epoll (9)
    runner.register(Box::new(poll_select::PollWriteReadyTest));
    runner.register(Box::new(poll_select::PollTimeoutTest));
    runner.register(Box::new(poll_select::PollReadReadyTest));
    runner.register(Box::new(poll_select::PollInvalidFdTest));
    runner.register(Box::new(poll_select::SelectWriteReadyTest));
    runner.register(Box::new(poll_select::SelectTimeoutTest));
    runner.register(Box::new(poll_select::EpollCreateTest));
    runner.register(Box::new(poll_select::EpollWaitTest));
    runner.register(Box::new(poll_select::EpollCtlDelTest));
    // Process (6)
    runner.register(Box::new(process::GetpidTest));
    runner.register(Box::new(process::GetppidTest));
    runner.register(Box::new(process::GetuidTest));
    runner.register(Box::new(process::GetgidTest));
    runner.register(Box::new(process::UmaskTest));
    runner.register(Box::new(process::ExitCodeTest));
    // Process advanced (12)
    runner.register(Box::new(process2::GetpgrpTest));
    runner.register(Box::new(process2::GetsidTest));
    runner.register(Box::new(process2::GeteuidTest));
    runner.register(Box::new(process2::GetegidTest));
    runner.register(Box::new(process2::ForkWaitTest));
    runner.register(Box::new(process2::WaitpidNohangTest));
    runner.register(Box::new(process2::GetrusageTest));
    runner.register(Box::new(process2::TimesTest));
    runner.register(Box::new(process2::SetpgidTest));
    runner.register(Box::new(process2::ExecveBasicTest));
    runner.register(Box::new(process2::ExecveFalseTest));
    runner.register(Box::new(process2::GetpriorityTest));
    // Thread (7)
    runner.register(Box::new(thread::PthreadCreateJoinTest));
    runner.register(Box::new(thread::PthreadMutexTest));
    runner.register(Box::new(thread::GettidTest));
    runner.register(Box::new(thread::PthreadKeyTest));
    runner.register(Box::new(thread::FutexWakeWaitTest));
    runner.register(Box::new(thread::PthreadOnceTest));
    runner.register(Box::new(thread::PthreadCondTest));
    // Resource / scheduling (8)
    runner.register(Box::new(resource::SchedGetschedulerTest));
    runner.register(Box::new(resource::SchedGetparamTest));
    runner.register(Box::new(resource::SchedYieldTest));
    runner.register(Box::new(resource::SchedPriorityRangeTest));
    runner.register(Box::new(resource::SetpriorityTest));
    runner.register(Box::new(resource::PrlimitGetTest));
    runner.register(Box::new(resource::MincoreTest));
    runner.register(Box::new(resource::GetloadavgTest));
    // Signal basic (6)
    runner.register(Box::new(signal::KillSelfTest));
    runner.register(Box::new(signal::KillInvalidPidTest));
    runner.register(Box::new(signal::SigactionBasicTest));
    runner.register(Box::new(signal::SigprocmaskBlockTest));
    runner.register(Box::new(signal::SigpendingTest));
    runner.register(Box::new(signal::RaiseTest));
    // Signal advanced (6)
    runner.register(Box::new(signal2::SigaltstackTest));
    runner.register(Box::new(signal2::SigsuspendTest));
    runner.register(Box::new(signal2::SignalfdTest));
    runner.register(Box::new(signal2::SigmaskInheritTest));
    runner.register(Box::new(signal2::SigqueueTest));
    runner.register(Box::new(signal2::SigchldTest));
    // Sysinfo (11)
    runner.register(Box::new(sysinfo::UnameTest));
    runner.register(Box::new(sysinfo::SysinfoTest));
    runner.register(Box::new(sysinfo::GetrlimitNofileTest));
    runner.register(Box::new(sysinfo::GetrlimitStackTest));
    runner.register(Box::new(sysinfo::SetrlimitTest));
    runner.register(Box::new(sysinfo::GetpagesizeTest));
    runner.register(Box::new(sysinfo::SysconfNprocsTest));
    runner.register(Box::new(sysinfo::SysconfPageSizeTest));
    runner.register(Box::new(sysinfo::GetgroupsTest));
    runner.register(Box::new(sysinfo::GethostnameTest));
    runner.register(Box::new(sysinfo::ProcSelfStatusTest));
    // Time (5)
    runner.register(Box::new(time::ClockGettimeRealtimeTest));
    runner.register(Box::new(time::ClockGettimeMonotonicTest));
    runner.register(Box::new(time::GettimeofdayTest));
    runner.register(Box::new(time::NanosleepBasicTest));
    runner.register(Box::new(time::NanosleepInvalidTest));
    // Time advanced (7)
    runner.register(Box::new(time2::ClockGetresRealtimeTest));
    runner.register(Box::new(time2::ClockGetresMonotonicTest));
    runner.register(Box::new(time2::ClockGettimeCputimeTest));
    runner.register(Box::new(time2::MonotonicNonDecreasingTest));
    runner.register(Box::new(time2::SetitimerTest));
    runner.register(Box::new(time2::TimeBasicTest));
    runner.register(Box::new(time2::ClockNanosleepTest));
    // IPC (7)
    runner.register(Box::new(ipc::ShmgetCreateTest));
    runner.register(Box::new(ipc::ShmatShmdetTest));
    runner.register(Box::new(ipc::ShmctlStatTest));
    runner.register(Box::new(ipc::MsggetCreateTest));
    runner.register(Box::new(ipc::MsgsndRcvTest));
    runner.register(Box::new(ipc::SemgetCreateTest));
    runner.register(Box::new(ipc::SemopTest));
    // Directory (8)
    runner.register(Box::new(dirent::OpendirTest));
    runner.register(Box::new(dirent::ReaddirTest));
    runner.register(Box::new(dirent::ReaddirCreatedFilesTest));
    runner.register(Box::new(dirent::RewinddirTest));
    runner.register(Box::new(dirent::Getdents64Test));
    runner.register(Box::new(dirent::ChdirTest));
    runner.register(Box::new(dirent::OpenatTest));
    runner.register(Box::new(dirent::MkdiratTest));
    // Memory advanced v3 (10)
    runner.register(Box::new(memory3::MmapFixedTest));
    runner.register(Box::new(memory3::MmapPopulateTest));
    runner.register(Box::new(memory3::MprotectRoTest));
    runner.register(Box::new(memory3::MunmapSubregionTest));
    runner.register(Box::new(memory3::MmapLargeTest));
    runner.register(Box::new(memory3::MmapFdClosedTest));
    runner.register(Box::new(memory3::MadviseSequentialTest));
    runner.register(Box::new(memory3::MadviseWillneedTest));
    runner.register(Box::new(memory3::ProcessVmReadvTest));
    runner.register(Box::new(memory3::UserfaultfdCreateTest));
    // Process advanced v3 (10)
    runner.register(Box::new(process3::Wait4Test));
    runner.register(Box::new(process3::CloneFsTest));
    runner.register(Box::new(process3::GetresuidTest));
    runner.register(Box::new(process3::GetresgidTest));
    runner.register(Box::new(process3::PrctlNameRoundtripTest));
    runner.register(Box::new(process3::ReadlinkProcSelfExeTest));
    runner.register(Box::new(process3::ProcSelfMapsTest));
    runner.register(Box::new(process3::ProcSelfFdTest));
    runner.register(Box::new(process3::ProcSelfPidTest));
    runner.register(Box::new(process3::ExecveEnvTest));
        // Network advanced v3 (9)
    runner.register(Box::new(network3::SoKeepaliveTest));
    runner.register(Box::new(network3::SoRcvtimeoTest));
    runner.register(Box::new(network3::TcpNodelayTest));
    runner.register(Box::new(network3::SoLingerTest));
    runner.register(Box::new(network3::SoErrorTest));
    runner.register(Box::new(network3::RecvmsgSendmsgTest));
    runner.register(Box::new(network3::IpTtlTest));
    runner.register(Box::new(network3::Accept4Test));
    runner.register(Box::new(network3::SoBroadcastTest));
    // Time advanced v3 (8)
    runner.register(Box::new(time3::ClockThreadCputimeTest));
    runner.register(Box::new(time3::ClockMonotonicRawTest));
    runner.register(Box::new(time3::ClockBoottimeTest));
    runner.register(Box::new(time3::TimerfdRealtimeTest));
    runner.register(Box::new(time3::TimerfdGettimeTest));
    runner.register(Box::new(time3::NanosleepAccumulateTest));
    runner.register(Box::new(time3::ClockNanosleepAbsTest));
    runner.register(Box::new(time3::ItimerVirtualTest));
        // Pipe advanced (5)
    runner.register(Box::new(pipe2::TeeBasicTest));
    runner.register(Box::new(pipe2::VmspliceTest));
    runner.register(Box::new(pipe2::PipeBufAtomicTest));
    runner.register(Box::new(pipe2::FcntlDupfdTest));
    runner.register(Box::new(pipe2::PipeSizeTest));
    runner.register(Box::new(pipe2::InotifyInitTest));
        // fcntl advanced (10)
    runner.register(Box::new(fcntl2::FcntlSetlkReadTest));
    runner.register(Box::new(fcntl2::FcntlSetlkWriteTest));
    runner.register(Box::new(fcntl2::FcntlGetflTest));
    runner.register(Box::new(fcntl2::FcntlSetflTest));
    runner.register(Box::new(fcntl2::FcntlDupfdCloexecTest));
    runner.register(Box::new(fcntl2::IoctlTiocgpgrpTest));
    runner.register(Box::new(fcntl2::IsattyTest));
    runner.register(Box::new(fcntl2::OpenOsyncTest));
    runner.register(Box::new(fcntl2::OpenOdirectTest));
    runner.register(Box::new(fcntl2::IoctlFioclexTest));
    // io_uring (2)
    runner.register(Box::new(io_uring::IoUringSetupTest));
    runner.register(Box::new(io_uring::IoUringSetupZeroEntriesTest));
    // misc2 (8)
    runner.register(Box::new(misc2::InotifyWatchTest));
    runner.register(Box::new(misc2::InotifyDetectCreateTest));
    runner.register(Box::new(misc2::SyslogReadTest));
    runner.register(Box::new(misc2::PersonalityGetTest));
    runner.register(Box::new(misc2::CapgetTest));
    runner.register(Box::new(misc2::GetcpuTest));
    runner.register(Box::new(misc2::MembarrierTest));
    runner.register(Box::new(misc2::RseqRegisterTest));
        // Directory advanced (8)
    runner.register(Box::new(dirent2::FchdirTest));
    runner.register(Box::new(dirent2::FchmodTest));
    runner.register(Box::new(dirent2::FchmodatTest));
    runner.register(Box::new(dirent2::LchownTest));
    runner.register(Box::new(dirent2::UtimesTest));
    runner.register(Box::new(dirent2::FutimensTest));
    runner.register(Box::new(dirent2::StatvfsTest));
        // Misc (12)
    runner.register(Box::new(misc::SetenvTest));
    runner.register(Box::new(misc::GetrandomTest));
    runner.register(Box::new(misc::GetrandomNonblockTest));
    runner.register(Box::new(misc::DevUrandomTest));
    runner.register(Box::new(misc::IoctlFionreadTest));
    runner.register(Box::new(misc::SyncTest));
    runner.register(Box::new(misc::GetloginTest));
    runner.register(Box::new(misc::GetenvPathTest));
    runner.register(Box::new(misc::MemfdCreateTest));
    runner.register(Box::new(misc::EventfdTest));
    runner.register(Box::new(misc::TimerfdTest));
    runner.register(Box::new(misc::PrctlGetNameTest));
}
