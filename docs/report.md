# syscall-compat-tests 技术总结报告

## 一、项目概述

本项目是面向 rCore-OS 生态的 Linux syscall 兼容性测试套件。

项目地址：https://github.com/irinaparchina-art/syscall-compat-tests

tgoskits PR：https://github.com/rcore-os/tgoskits/pull/995

## 二、项目背景

rCore-OS 生态中的 StarryOS、ArceOS 等内核正在实现 Linux syscall 兼容性，以支持在其上运行 ROS2、FFmpeg、OpenCV 等 Linux 软件。然而目前缺少一个系统性工具来回答：

- 当前内核兼容了多少 Linux syscall？
- 哪些 syscall 的返回值或 errno 语义有偏差？
- 边界条件（如非法参数）是否按 Linux 标准处理？

本项目填补了这一空白，为 rCore-OS 生态提供可量化的兼容性测试基础设施。

## 三、技术实现

### 3.1 整体架构

```
syscall-compat-tests/
├── src/
│   ├── runner/        # 测试框架核心
│   │   └── mod.rs     # SyscallTest trait、TestResult、TestRunner
│   ├── reporter/      # 报告生成模块
│   │   └── mod.rs     # Console、JSON、Markdown 三种输出格式
│   └── testcases/     # 35 个测试模块，360 个测试用例
│       ├── file_io.rs       # 基础文件 I/O（16 个测试）
│       ├── file_io2.rs      # 高级文件 I/O（12 个测试）
│       ├── memory.rs        # 内存管理基础（5 个测试）
│       ├── memory2.rs       # 高级内存操作（6 个测试）
│       ├── memory3.rs       # mmap 边界测试（10 个测试）
│       ├── network.rs       # 网络基础（9 个测试）
│       ├── network3.rs      # 网络高级选项（9 个测试）
│       ├── network4.rs      # 网络边界测试（8 个测试）
│       ├── signal.rs        # 信号基础（10 个测试）
│       ├── signal2.rs       # 信号高级（8 个测试）
│       ├── process.rs       # 进程基础（10 个测试）
│       ├── process2.rs      # 进程高级（10 个测试）
│       ├── process3.rs      # /proc 文件系统（10 个测试）
│       ├── process4.rs      # 环境变量、alarm（12 个测试）
│       ├── thread.rs        # 线程与 futex（7 个测试）
│       ├── ipc.rs           # 进程间通信（7 个测试）
│       ├── time.rs          # 时间基础（5 个测试）
│       ├── time2.rs         # 时间高级（7 个测试）
│       ├── time3.rs         # timerfd、clock 变体（8 个测试）
│       ├── dirent.rs        # 目录遍历（8 个测试）
│       ├── dirent2.rs       # 目录高级操作（7 个测试）
│       ├── pipe.rs          # 管道基础（6 个测试）
│       ├── pipe2.rs         # 管道高级（6 个测试）
│       ├── poll_select.rs   # I/O 多路复用
│       ├── io_advanced.rs   # readv/writev/splice（8 个测试）
│       ├── io_uring.rs      # io_uring 接口（2 个测试）
│       ├── fcntl2.rs        # fcntl 高级（10 个测试）
│       ├── filestat.rs      # 文件属性（10 个测试）
│       ├── sysinfo.rs       # 系统信息
│       ├── sysinfo2.rs      # /proc 系统信息（10 个测试）
│       ├── resource.rs      # 资源限制与调度
│       ├── misc.rs          # 杂项 syscall（12 个测试）
│       ├── misc2.rs         # inotify、capability（8 个测试）
│       ├── errno_edge.rs    # errno 边界测试（20 个测试）
│       ├── errno_edge2.rs   # 更多边界测试（10 个测试）
│       ├── compat_check.rs  # 兼容性核心验证（9 个测试）
│       ├── final_tests.rs   # 综合测试（12 个测试）
│       ├── epoll.rs         # epoll I/O 多路复用（10 个测试）
│       ├── sched.rs         # 调度器相关（6 个测试）
│       └── fileops_ext.rs   # 扩展文件操作（12 个测试）
├── docs/                    # 设计文档与技术报告
└── .github/workflows/
    └── ci.yml               # CI：Linux x86_64 + RISC-V 交叉编译
```

### 3.2 核心抽象

每个测试用例实现统一的 `SyscallTest` trait：

```rust
pub trait SyscallTest: Send + Sync {
    fn name(&self) -> &str;                // 唯一标识符
    fn syscall(&self) -> &str;             // 被测 syscall 名称
    fn category(&self) -> SyscallCategory; // 分类
    fn description(&self) -> &str;         // 人类可读描述
    fn run(&self) -> TestResult;           // 执行测试
}
```

测试结果精确区分四种状态：

```rust
pub enum TestStatus {
    Pass,                              // 行为完全符合 Linux 语义
    Fail { expected_ret, actual_ret,
           expected_errno, actual_errno }, // 返回值或 errno 不匹配
    Unimplemented,                     // 返回 ENOSYS，syscall 未实现
    Error(String),                     // 测试本身执行出错
}
```

### 3.3 测试覆盖

| 模块 | 测试数 | 覆盖的主要 syscall |
|------|--------|-------------------|
| File I/O | 48 | open/read/write/stat/lseek/dup/fcntl |
| Memory | 27 | mmap/munmap/mprotect/madvise/mremap |
| Network | 34 | socket/bind/connect/send/recv |
| Signal | 18 | kill/sigaction/sigprocmask/signalfd |
| Process | 42 | fork/exec/wait/clone/getpid/setsid |
| Thread | 7 | pthread/futex/gettid/mutex/cond |
| IPC | 7 | shmget/msgget/semget/msgsnd |
| Time | 20 | clock_gettime/timerfd/nanosleep |
| Directory | 23 | opendir/readdir/chdir/fchmod |
| Epoll | 10 | epoll_create1/epoll_ctl/epoll_wait |
| Scheduler | 6 | sched_yield/sched_getaffinity/sched_setaffinity |
| File Ops Ext | 12 | umask/dup3/chown/fchown/faccessat/fstatat |
| Misc | 106 | inotify/getrandom/memfd/prctl/io_uring |
| **合计** | **360** | **120+ syscall** |

### 3.4 核心特性

**errno 语义精确验证：** 不仅验证返回值，还验证错误码。例如：
- `open()` 打开不存在路径必须返回 `ENOENT`
- `lseek()` 使用非法 whence 必须返回 `EINVAL`
- `write()` 写只读 fd 必须返回 `EBADF`
- `open()` 以 `O_WRONLY` 打开目录必须返回 `EISDIR`
- `chown()` 操作不存在文件必须返回 `ENOENT`
- `dup3()` 相同 fd 必须返回 `EINVAL`
- `epoll_ctl()` 重复添加 fd 必须返回 `EEXIST`

**边界条件测试：** 专门测试边界行为：
- `mmap()` 零长度映射的 EINVAL 处理
- `pipe2()` 非法 flags 的拒绝
- 符号链接循环的 ELOOP 检测
- 超长路径名的 ENAMETOOLONG 处理
- `sched_getaffinity()` 对不存在 PID 返回 ESRCH
- `epoll_ctl()` 对无效 fd 返回 EBADF

**POSIX 语义保证验证：**
- 匿名 mmap 页面必须零初始化
- fork() 子进程具有独立内存副本（COW 语义）
- CLOCK_MONOTONIC 单调不减
- dup() 后两个 fd 共享文件偏移量
- umask() 设置后可正确 roundtrip
- epoll_wait 超时返回 0
- sched_yield() 成功返回 0

## 四、RISC-V 交叉编译

项目支持交叉编译到 RISC-V，可在 StarryOS/ArceOS 下直接运行：

```bash
# 第一步：安装工具链
rustup target add riscv64gc-unknown-linux-gnu
sudo apt install gcc-riscv64-linux-gnu

# 第二步：交叉编译
cargo build --release --target riscv64gc-unknown-linux-gnu

# 第三步：将二进制复制到 QEMU 镜像并运行
./sct-run --target starry --format all --output /tmp/report

# 第四步：查看兼容性报告
cat /tmp/report.md
```

编译产物为静态链接的 ELF 二进制，可直接在 StarryOS/ArceOS 的 RISC-V 环境中运行，无需额外依赖。

## 五、CI/CD

使用 GitHub Actions 实现全自动化测试，每次代码提交自动触发：

- **Test on Linux x86_64**：运行全部 360 个测试，当前 359 Pass / 1 Unimplemented / 0 Fail
- **Cross-compile riscv64**：验证可成功交叉编译到 riscv64gc-unknown-linux-gnu 目标

CI 配置文件：`.github/workflows/ci.yml`

## 六、对 tgoskits 的贡献

向 rcore-os/tgoskits 提交 PR #995，在 `test-suit/starryos/normal/qemu-smp1/syscall/test-compat/c/` 目录下添加 syscall 兼容性测试。

PR 严格遵循项目规范：
- 文件放置在正确的 `normal/qemu-smp1/syscall/` 路径下，可被 xtask runner 自动发现
- 包含 `c/CMakeLists.txt` 编译配置，遵循项目统一安装到 `usr/bin/starry-test-suit`
- 使用项目统一的 `test_framework.h`（TEST_START/CHECK/CHECK_ERR/CHECK_RET/TEST_DONE 宏）
- 已通过 mai-team-app bot 审核（Approved）

测试内容覆盖两个测试组：

**File I/O 组（6个测试）：**
- open 不存在文件返回 ENOENT
- openat O_CREAT 创建写入文件成功
- write 返回实际写入字节数
- stat 返回正确文件大小
- unlink 删除文件成功
- open 目录以 O_WRONLY 返回 EISDIR

**Process 组（4个测试）：**
- getpid 返回正值
- getppid 返回正值
- pid == ppid 时输出 INFO 日志（软断言，兼容 init 进程场景）
- getuid/getgid 返回非负值

## 七、项目数据

| 指标 | 数值 |
|------|------|
| 代码行数 | 11,655 行 |
| 测试用例 | 360 个 |
| 测试模块 | 35 个 |
| CI 通过 | 359 Pass / 1 Unimplemented / 0 Fail |
| tgoskits PR | #995（已通过 bot 审核） |

## 八、总结

syscall-compat-tests 为 rCore-OS 生态提供了一套系统性、可量化的 Linux syscall 兼容性测试工具。通过在 Linux 上建立 baseline，再交叉编译到 RISC-V 目标内核运行，可以精确定位兼容性差异，为 StarryOS、ArceOS 等内核的 Linux 兼容性改进提供数据支撑。

项目覆盖 120+ Linux syscall，横跨文件 I/O、内存管理、网络、信号、进程、线程、IPC、时间、目录操作、epoll、调度器、扩展文件操作等 13 个大类，是 rCore-OS 生态中目前最全面的 syscall 兼容性测试工具。

时间：2026年5月
