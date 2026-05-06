# syscall-compat-tests 设计文档

## 项目背景

rCore-OS 生态中的 StarryOS、ArceOS 等候选内核正在努力实现 Linux syscall 兼容性。
本项目提供一套系统性的测试工具，帮助量化"兼容了多少"、"哪些 syscall 语义有偏差"。

## 架构设计

核心目录结构：
- src/runner/        测试框架核心：SyscallTest trait、TestResult 类型
- src/reporter/      报告生成：Console、JSON、Markdown 三种格式
- src/testcases/     测试用例：按 syscall 类别分 32 个模块
- .github/workflows/ CI：Linux x86_64 测试 + RISC-V 交叉编译

## 如何接入新的目标内核

1. 交叉编译：cargo build --release --target riscv64gc-unknown-linux-gnu
2. 将二进制复制到 QEMU 镜像
3. 在目标内核下运行：./sct-run --target starry --format all --output report
4. 对比 Linux baseline 报告找出差异

## 已发现的兼容性问题示例

- lseek 对非法 whence 参数的 errno 处理
- nanosleep 对 tv_nsec >= 1e9 的 EINVAL 验证
- mmap 零长度映射的 EINVAL 行为
- pipe2 对无效 flags 的拒绝处理
