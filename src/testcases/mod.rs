pub mod file_io;
pub mod filestat;
pub mod process;
pub mod memory;
pub mod time;
pub mod signal;
pub mod pipe;
pub mod network;

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

    // Process (6)
    runner.register(Box::new(process::GetpidTest));
    runner.register(Box::new(process::GetppidTest));
    runner.register(Box::new(process::GetuidTest));
    runner.register(Box::new(process::GetgidTest));
    runner.register(Box::new(process::UmaskTest));
    runner.register(Box::new(process::ExitCodeTest));

    // Memory (5)
    runner.register(Box::new(memory::BrkBasicTest));
    runner.register(Box::new(memory::MmapAnonTest));
    runner.register(Box::new(memory::MmapReadWriteTest));
    runner.register(Box::new(memory::MunmapTest));
    runner.register(Box::new(memory::MprotectTest));

    // Time (5)
    runner.register(Box::new(time::ClockGettimeRealtimeTest));
    runner.register(Box::new(time::ClockGettimeMonotonicTest));
    runner.register(Box::new(time::GettimeofdayTest));
    runner.register(Box::new(time::NanosleepBasicTest));
    runner.register(Box::new(time::NanosleepInvalidTest));

    // Signal (6)
    runner.register(Box::new(signal::KillSelfTest));
    runner.register(Box::new(signal::KillInvalidPidTest));
    runner.register(Box::new(signal::SigactionBasicTest));
    runner.register(Box::new(signal::SigprocmaskBlockTest));
    runner.register(Box::new(signal::SigpendingTest));
    runner.register(Box::new(signal::RaiseTest));

    // Pipe / fcntl (6)
    runner.register(Box::new(pipe::PipeBasicTest));
    runner.register(Box::new(pipe::PipeReadEofTest));
    runner.register(Box::new(pipe::Pipe2CloseOnExecTest));
    runner.register(Box::new(pipe::FcntlGetFdTest));
    runner.register(Box::new(pipe::FcntlSetNonblockTest));
    runner.register(Box::new(pipe::ReadNonblockTest));

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
}
