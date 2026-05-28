# syscall-compat-tests

> A Linux syscall compatibility test suite for the rCore-OS ecosystem (Starry, ArceOS, AxVisor).

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![build passing](https://github.com/irinaparchina-art/syscall-compat-tests/actions/workflows/ci.yml/badge.svg)](https://github.com/irinaparchina-art/syscall-compat-tests/actions)

## What is this?

`syscall-compat-tests` verifies whether a candidate OS kernel correctly implements Linux syscall semantics — return values, errno codes, and edge-case behavior. It is designed to run natively on Linux (as a baseline) and be cross-compiled for target kernels like **StarryOS** and **ArceOS** running under QEMU.

## Quick Start

```bash
git clone https://github.com/irinaparchina-art/syscall-compat-tests
cd syscall-compat-tests
cargo build --release
cargo run --release --bin sct-run -- --target linux --format all --output report
```

## Coverage (360 tests, 11655 lines)

| Module | Tests | Key Syscalls |
|--------|-------|-------------|
| File I/O | 48 | open/read/write/stat/lseek/dup/mmap/... |
| Memory | 27 | mmap/munmap/mprotect/madvise/mremap/... |
| Network | 34 | socket/bind/connect/send/recv/epoll/... |
| Signal | 18 | kill/sigaction/sigprocmask/signalfd/... |
| Process | 42 | fork/exec/wait/clone/getpid/setsid/... |
| Thread | 7 | pthread/futex/gettid/mutex/cond/... |
| IPC | 7 | shmget/msgget/semget/msgsnd/... |
| Time | 20 | clock_gettime/timerfd/nanosleep/... |
| Dir/FS | 23 | opendir/readdir/chdir/fchmod/... |
| Epoll | 10 | epoll_create1/epoll_ctl/epoll_wait/... |
| Scheduler | 6 | sched_yield/sched_getaffinity/sched_setaffinity/... |
| File Ops Ext | 12 | umask/dup3/chown/fchown/faccessat/fstatat/... |
| Epoll | 10 | epoll_create1/epoll_ctl/epoll_wait/... |
| Scheduler | 6 | sched_yield/sched_getaffinity/... |
| File Ops Ext | 12 | umask/dup3/chown/faccessat/fstatat/... |
| Misc | 106 | inotify/getrandom/memfd/prctl/... |

## Cross-compile to RISC-V

```bash
cargo build --release --target riscv64gc-unknown-linux-gnu
# Copy binary to your Starry/ArceOS QEMU image and run
```

## Output Formats

```bash
# Console output
cargo run --bin sct-run -- --target starry

# JSON + Markdown reports
cargo run --bin sct-run -- --target starry --format all --output report
```

## Compatibility Matrix Example

| Test | Syscall | Linux | Starry | ArceOS |
|------|---------|:-----:|:------:|:------:|
| file_open_basic | open | ✅ | ? | ? |
| mem_mmap_anon | mmap | ✅ | ? | ? |
| net_socket_tcp | socket | ✅ | ? | ? |
| signal_kill_self | kill | ✅ | ? | ? |

**Legend:** ✅ Pass · ❌ Fail · ⚠️ ENOSYS · ? Not tested yet

## License

MIT
