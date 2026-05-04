# syscall-compat-tests

> A Linux syscall compatibility test suite for the rCore-OS ecosystem (Starry, ArceOS, AxVisor).

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen)]()

## What is this?

`syscall-compat-tests` is a structured, extensible test suite that verifies whether a candidate OS kernel correctly implements Linux syscall semantics — return values, errno codes, and edge-case behavior.

It is designed to run natively on Linux (as a baseline) and be cross-compiled for target kernels like **StarryOS** and **ArceOS** running under QEMU.

## Architecture

```
syscall-compat-tests/
├── src/
│   ├── runner/        # Test runner, SyscallTest trait, TestResult types
│   ├── reporter/      # Console, JSON and Markdown report generation
│   └── testcases/
│       ├── file_io.rs # open, read, write, close, stat, lseek, ...
│       ├── process.rs # getpid, fork, exit, wait, ...
│       ├── memory.rs  # mmap, munmap, mprotect, brk, ...
│       └── time.rs    # clock_gettime, gettimeofday, nanosleep, ...
└── README.md
```

## Quick Start

```bash
# Clone and build
git clone https://github.com/YOUR_USERNAME/syscall-compat-tests
cd syscall-compat-tests
cargo build --release

# Run all tests against Linux (baseline)
./target/release/sct-run --target linux

# Run and generate all report formats
./target/release/sct-run --target linux --format all --output report

# Run only File I/O tests
./target/release/sct-run --target linux --category file-io
```

## Sample Output

```
════════════════════════════════════════════════════════════
  syscall-compat-tests — linux
  2026-05-04 07:54:18
════════════════════════════════════════════════════════════

  [File I/O]
    file_open_basic                          PASS     56μs
    file_open_nonexistent                    PASS      3μs
    file_read_write_basic                    PASS     36μs
    ...

  Total: 32  Pass: 32  Fail: 0  Unimplemented: 0  Error: 0
  Compatibility: 100%
════════════════════════════════════════════════════════════
```

## Compatibility Matrix (example: StarryOS)

| Test | Syscall | Category | StarryOS | Notes |
|------|---------|----------|:---:|-------|
| file_open_basic | `open` | File I/O | ✅ | |
| file_lseek_invalid_whence | `lseek` | File I/O | ❌ | ret exp=-1 got=0, errno exp=22 got=0 |
| proc_exit_code | `exit` | Process | ⚠️ | |

**Legend:** ✅ Compatible · ❌ Incompatible · ⚠️ Not implemented (ENOSYS) · 💥 Error

## Integrating with Your OS (Starry / ArceOS / AxVisor)

1. Cross-compile the test binary for your target architecture:
```bash
cargo build --release --target riscv64gc-unknown-none-elf
```

2. Copy the binary into your OS image and run it under QEMU.

3. Capture the output and compare with the Linux baseline report.

See [docs/integration-guide.md](docs/integration-guide.md) for step-by-step instructions.

## Current Test Coverage

| Category | Tests | Syscalls Covered |
|----------|-------|-----------------|
| File I/O | 16 | open, read, write, close, stat, lseek, unlink, rename, mkdir, rmdir, getcwd, dup, dup2 |
| Process  | 6  | getpid, getppid, getuid, getgid, umask, fork+exit+wait |
| Memory   | 5  | mmap, munmap, mprotect, brk |
| Time     | 5  | clock_gettime (×2), gettimeofday, nanosleep (×2) |
| **Total** | **32** | |

## Adding New Test Cases

```rust
// In src/testcases/your_module.rs
pub struct MyNewTest;
impl SyscallTest for MyNewTest {
    fn name(&self) -> &str { "my_test_name" }
    fn syscall(&self) -> &str { "syscall_name" }
    fn category(&self) -> SyscallCategory { SyscallCategory::FileIO }
    fn description(&self) -> &str { "What this test verifies" }
    fn run(&self) -> TestResult {
        // ... your test logic
    }
}

// Register in src/testcases/mod.rs
runner.register(Box::new(your_module::MyNewTest));
```

## Projects Using This Suite

- [StarryOS](https://github.com/Azure-stars/Starry) — Linux-compatible OS on rCore-OS
- [ArceOS](https://github.com/arceos-org/arceos) — Modular unikernel

## License

MIT
