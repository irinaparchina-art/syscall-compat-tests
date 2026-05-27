# syscall-compat-tests 技术总结报告

## 一、项目概述

本项目是面向 rCore-OS 生态的 Linux syscall 兼容性测试套件。

项目地址：https://github.com/irinaparchina-art/syscall-compat-tests

tgoskits PR：https://github.com/rcore-os/tgoskits/pull/995

## 二、项目背景

rCore-OS 生态中的 StarryOS、ArceOS 等内核正在实现 Linux syscall 兼容性，本项目提供系统性测试工具回答：当前内核兼容了多少 syscall？哪些语义有偏差？

## 三、技术实现

### 测试覆盖

| 模块 | 测试数 |
|------|--------|
| File I/O | 48 |
| Memory | 27 |
| Network | 34 |
| Signal | 18 |
| Process | 42 |
| Thread | 7 |
| IPC | 7 |
| Time | 20 |
| 其他 | 129 |
| 合计 | 332 |

### 核心特性

- errno 语义精确验证
- 边界条件测试
- POSIX 语义保证验证

## 四、RISC-V 交叉编译

支持交叉编译到 riscv64gc-unknown-linux-gnu，可在 StarryOS/ArceOS 下直接运行。

## 五、CI/CD

GitHub Actions 全自动测试，每次 push 运行 332 个测试，当前 331 Pass / 1 Unimplemented / 0 Fail。

## 六、对 tgoskits 的贡献

PR #995：在 test-suit/starryos/syscall/ 添加 syscall_compat_test.c。

## 七、项目数据

- 代码行数：10520 行
- 测试用例：332 个
- 模块数量：32 个
- 时间：2026年5月
