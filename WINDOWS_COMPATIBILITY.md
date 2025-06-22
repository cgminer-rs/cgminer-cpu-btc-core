# Windows兼容性指南

## 问题描述

如果您在Windows上运行cgminer时遇到 `STATUS_ILLEGAL_INSTRUCTION` 错误，这是因为程序使用了您的CPU不支持的指令集。

## 错误原因

1. **CPU指令集不兼容**: 程序在Mac上编译时使用了`target-cpu=native`，生成了Mac特定的CPU指令
2. **高级优化**: 启用了AVX2、BMI2等现代CPU指令集，但您的CPU可能不支持
3. **SIMD优化**: 某些向量化指令在不同CPU上的支持程度不同

## 解决方案

### 方案1: 使用兼容性构建脚本 (推荐)

#### PowerShell脚本 (Windows 10/11)
```powershell
# 自动检测CPU并选择合适模式
.\build-windows.ps1 auto

# 或手动选择兼容模式
.\build-windows.ps1 compat

# 或手动选择性能模式 (需要现代CPU)
.\build-windows.ps1 perf
```

#### 批处理文件 (所有Windows版本)
```cmd
# 使用最大兼容性模式
.\build-windows-compat.bat
```

### 方案2: 手动编译

#### 兼容性优先 (适用于所有x86_64 CPU)
```bash
# 设置兼容性环境变量
set RUSTFLAGS=-C target-cpu=x86-64 -C target-feature=+sse2,+sse4.1,+sse4.2 -C opt-level=3

# 编译项目
cargo build --release --features=thermal-management,power-management,cpu-affinity

# 编译示例
cargo build --release --examples --features=thermal-management,power-management,cpu-affinity
```

#### 性能优先 (需要现代CPU支持)
```bash
# 设置性能环境变量
set RUSTFLAGS=-C target-cpu=native -C target-feature=+aes,+sha,+sse4.2,+avx2 -C opt-level=3

# 编译项目
cargo build --release --features=simd-optimizations,thermal-management,power-management,cpu-affinity

# 编译示例
cargo build --release --examples --features=simd-optimizations,thermal-management,power-management,cpu-affinity
```

### 方案3: 修改配置文件

如果您想永久修改配置，可以编辑 `.cargo/config.toml` 文件：

```toml
[target.x86_64-pc-windows-msvc]
rustflags = [
    "-C", "target-cpu=x86-64",  # 改为通用架构
    "-C", "target-feature=+sse2,+sse4.1,+sse4.2",  # 只启用基础特性
    "-C", "opt-level=3",
    "-C", "codegen-units=1",
    "-C", "overflow-checks=off",
]
```

## CPU兼容性说明

### 兼容模式支持的CPU
- 所有64位Intel和AMD处理器
- 最低要求: 支持SSE2指令集的CPU (2003年后的CPU)
- 包括: Intel Core 2、AMD Athlon 64及更新的处理器

### 性能模式支持的CPU
- Intel: Core i3/i5/i7/i9 (4代及更新)
- AMD: Ryzen系列、FX系列
- 需要支持: AVX2、AES-NI、SHA指令集

## 验证CPU支持

### 检查CPU型号
```cmd
wmic cpu get name
```

### 检查CPU特性 (需要第三方工具)
- 使用CPU-Z查看支持的指令集
- 查看CPU规格确认AVX2支持

## 构建模式对比

| 模式 | 兼容性 | 性能 | 适用场景 |
|------|--------|------|----------|
| 兼容模式 | 最高 | 中等 | 旧CPU、稳定性优先 |
| 性能模式 | 中等 | 最高 | 现代CPU、性能优先 |
| 自动模式 | 自适应 | 自适应 | 推荐选择 |

## 故障排除

### 如果兼容模式仍然失败
1. **更新Rust工具链**:
   ```bash
   rustup update
   ```

2. **检查Windows版本**: 确保使用Windows 10或更新版本

3. **尝试最小化构建**:
   ```bash
   cargo build --release --no-default-features
   ```

### 如果性能不满意
1. 确认CPU支持现代指令集
2. 尝试性能模式构建
3. 启用更多优化特性:
   ```bash
   cargo build --release --features=simd-optimizations,advanced-math,memory-optimized
   ```

## 联系支持

如果问题仍然存在，请提供以下信息：
- Windows版本
- CPU型号和规格
- 错误信息的完整输出
- 使用的构建命令

## 更新日志

- **v0.2.0**: 添加Windows兼容性支持
- 修复了`STATUS_ILLEGAL_INSTRUCTION`错误
- 添加了自动CPU检测和构建脚本
- 提供了多种兼容性级别选择
