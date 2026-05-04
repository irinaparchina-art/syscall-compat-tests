pub mod dirent;
pub mod file_io;
pub mod filestat;
pub mod io_advanced;
pub mod ipc;
pub mod memory;
pub mod memory2;
pub mod network;
pub mod pipe;
pub mod poll_select;
pub mod process;
pub mod process2;
pub mod signal;
pub mod time;

use crate::runner::TestRunner;

pub fn register_all(runner: &mut TestRunner) {
    // File I/O (16)
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

    // Signal (6)
    runner.register(Box::new(signal::KillSelfTest));
    runner.register(Box::new(signal::KillInvalidPidTest));
    runner.register(Box::new(signal::SigactionBasicTest));
    runner.register(Box::new(signal::SigprocmaskBlockTest));
    runner.register(Box::new(signal::SigpendingTest));
    runner.register(Box::new(signal::RaiseTest));

    // Time (5)
    runner.register(Box::new(time::ClockGettimeRealtimeTest));
    runner.register(Box::new(time::ClockGettimeMonotonicTest));
    runner.register(Box::new(time::GettimeofdayTest));
    runner.register(Box::new(time::NanosleepBasicTest));
    runner.register(Box::new(time::NanosleepInvalidTest));

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
}
