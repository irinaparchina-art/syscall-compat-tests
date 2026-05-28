# syscall-compat-tests 设计文档

## 项目背景

rCore-OS 生态中的 StarryOS、ArceOS 等候选内核正在努力实现 Linux syscall 兼容性。
但目前缺少一个系统性的测试工具来量化：
- 当前内核兼容了多少 Linux syscall？
- 哪些 syscall 的返回值或 errno 语义有偏差？
- 边界条件（如非法参数）是否按 Linux 标准处理？

本项目 syscall-compat-tests 填补了这一空白。

## 架构设计

核心目录结构：
- src/runner/        测试框架核心：SyscallTest trait、TestResult、TestRunner
- src/reporter/      报告生成：Console、JSON、Markdown 三种输出格式
- src/testcases/     测试用例，35 个模块，360 个用例
- .github/workflows/ CI：Linux x86_64 测试 + RISC-V 交叉编译

## 核心抽象

每个测试用例实现 SyscallTest trait：

    pub trait SyscallTest: Send + Sync {
        fn name(&self) -> &str;
        fn syscall(&self) -> &str;
        fn category(&self) -> SyscallCategory;
        fn description(&self) -> &str;
        fn run(&self) -> TestResult;
    }

测试结果类型：

    pub enum TestStatus {
        Pass,
        Fail { expected_ret, actual_ret, expected_errno, actual_errno },
        Unimplemented,
        Error(String),
    }

## 使用方法

在 Linux 上运行 baseline：

    cargo build --release
    cargo run --release --bin sct-run -- --target linux

交叉编译到 RISC-V：

    rustup target add riscv64gc-unknown-linux-gnu
    sudo apt install gcc-riscv64-linux-gnu
    cargo build --release --target riscv64gc-unknown-linux-gnu

只测试特定类别：

    cargo run --bin sct-run -- --target linux --category file-io
    cargo run --bin sct-run -- --target linux --category memory

生成全格式报告：

    cargo run --bin sct-run -- --target linux --format all --output report

## 如何添加新测试用例

1. 在 src/testcases/ 下找到合适的模块文件
2. 实现 SyscallTest trait
3. 在 src/testcases/mod.rs 的 register_all() 中注册

## 已发现的兼容性问题示例

- lseek 对非法 whence 参数未返回 EINVAL
- nanosleep 对 tv_nsec >= 1e9 未返回 EINVAL
- mmap 零长度映射未返回 EINVAL
- pipe2 对无效 flags 未返回 EINVAL
- write 写只读 fd 未返回 EBADF
- open 以 O_WRONLY 打开目录未返回 EISDIR

## CI 集成

- Test on Linux x86_64：每次 push 自动运行全部 360 个测试
- Cross-compile riscv64：验证可成功交叉编译到 RISC-V 目标
