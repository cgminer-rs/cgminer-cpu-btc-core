# CGMiner CPU BTC Core - 运行示例指南

这个文档提供了 cgminer-cpu-btc-core 项目的完整运行示例和使用指南。

## 📋 目录

- [快速开始](#快速开始)
- [示例列表](#示例列表)
- [运行方法](#运行方法)
- [基准测试](#基准测试)
- [故障排除](#故障排除)

## 🚀 快速开始

### 环境要求

- Rust 1.70+ 
- Cargo
- 支持的操作系统：Linux, macOS, Windows

### 安装依赖

```bash
# 克隆项目
cd /Users/gecko/project/linux/cgminer-cpu-btc-core

# 检查依赖
cargo check

# 构建项目
cargo build --release
```

## 📚 示例列表

### 1. 基本挖矿演示 (`basic_mining_demo.rs`)

**功能**: 展示基本的比特币挖矿流程
- 设备创建和配置
- 工作数据生成
- SHA256双重哈希计算
- 有效解的寻找
- 设备状态监控

**运行命令**:
```bash
cargo run --example basic_mining_demo
```

**预期输出**:
```
🚀 开始基本挖矿演示
==================
📱 步骤1: 创建挖矿设备
  ✅ 设备创建成功
  📊 设备名称: CPU Bitcoin Miner
  🆔 设备ID: 1
  🎯 目标算力: 2.0 MH/s
...
```

### 2. 多设备挖矿演示 (`multi_device_demo.rs`)

**功能**: 展示多设备并行挖矿
- 设备管理器
- 并行挖矿任务
- 负载均衡
- 统计汇总

**运行命令**:
```bash
cargo run --example multi_device_demo
```

### 3. 性能监控演示 (`performance_monitoring_demo.rs`)

**功能**: 展示性能监控和分析
- 实时性能监控
- 性能趋势分析
- 压力测试
- 性能报告生成

**运行命令**:
```bash
cargo run --example performance_monitoring_demo
```

### 4. 温度管理演示 (`temperature_demo.rs`)

**功能**: 展示温度监控和热管理
- 温度实时监控
- 热保护机制
- 温度趋势分析
- 自适应温度控制

**运行命令**:
```bash
cargo run --example temperature_demo
```

### 5. CPU亲和性演示 (`cpu_affinity_demo.rs`)

**功能**: 展示CPU亲和性优化
- CPU核心绑定
- NUMA优化
- 性能对比测试
- 亲和性配置推荐

**运行命令**:
```bash
cargo run --example cpu_affinity_demo
```

### 6. 真实挖矿模拟 (`real_mining_simulation.rs`)

**功能**: 模拟真实挖矿环境
- 矿池连接模拟
- 工作分配
- 难度调整
- 收益计算

**运行命令**:
```bash
cargo run --example real_mining_simulation
```

## 🏃 运行方法

### 单个示例运行

```bash
# 运行基本挖矿演示
cargo run --example basic_mining_demo

# 运行多设备演示
cargo run --example multi_device_demo

# 运行性能监控演示
cargo run --example performance_monitoring_demo
```

### 批量运行所有示例

```bash
# 创建运行脚本
cat > run_all_examples.sh << 'EOF'
#!/bin/bash
echo "🚀 运行所有CGMiner CPU BTC Core示例"
echo "=================================="

examples=(
    "basic_mining_demo"
    "multi_device_demo" 
    "performance_monitoring_demo"
    "temperature_demo"
    "cpu_affinity_demo"
    "real_mining_simulation"
)

for example in "${examples[@]}"; do
    echo ""
    echo "📋 运行示例: $example"
    echo "----------------------------------------"
    cargo run --example "$example"
    echo ""
    echo "✅ $example 完成"
    echo "========================================"
done

echo "🎉 所有示例运行完成!"
EOF

chmod +x run_all_examples.sh
./run_all_examples.sh
```

## 🏁 基准测试

### 运行基准测试

```bash
# 快速基准测试
./run_benchmarks.sh --quick

# 完整基准测试
./run_benchmarks.sh --all

# 特定测试
./run_benchmarks.sh --sha256
./run_benchmarks.sh --device
./run_benchmarks.sh --performance

# 生成报告
./run_benchmarks.sh --all --report --open-report
```

### 基准测试结果

基准测试结果将保存在以下位置：
- HTML报告: `target/criterion/report/index.html`
- JSON数据: `target/criterion/benchmark_results.json`
- 自定义报告: `benchmark_report_YYYYMMDD_HHMMSS.md`

## 🔧 配置选项

### 环境变量

```bash
# 设置日志级别
export RUST_LOG=info

# 启用性能分析
export RUST_PROFILE=1

# 设置CPU亲和性
export CPU_AFFINITY=1
```

### 编译选项

```bash
# 发布模式编译（最高性能）
cargo build --release

# 启用所有优化特性
cargo build --release --features "experimental"

# 启用特定特性
cargo build --release --features "simd-optimizations,thermal-management"
```

## 🐛 故障排除

### 常见问题

1. **编译错误**
   ```bash
   # 更新依赖
   cargo update
   
   # 清理构建缓存
   cargo clean
   
   # 重新构建
   cargo build --release
   ```

2. **运行时错误**
   ```bash
   # 检查系统要求
   rustc --version  # 需要 1.70+
   
   # 检查依赖
   cargo check
   ```

3. **性能问题**
   ```bash
   # 确保使用发布模式
   cargo run --release --example basic_mining_demo
   
   # 启用优化特性
   cargo run --release --features "experimental" --example performance_monitoring_demo
   ```

### 调试模式

```bash
# 启用详细日志
RUST_LOG=debug cargo run --example basic_mining_demo

# 启用性能分析
cargo run --release --features "profiling" --example performance_monitoring_demo
```

## 📊 性能参考

### 典型性能指标

在现代CPU上的预期性能：

| CPU类型 | 算力 (MH/s) | 功耗 (W) | 效率 (H/W) |
|---------|-------------|----------|------------|
| Intel i7-12700K | 15-25 | 150-200 | 100-150 |
| AMD Ryzen 7 5800X | 12-20 | 120-180 | 100-140 |
| Apple M1 Pro | 8-15 | 80-120 | 100-125 |
| Intel i5-11400 | 8-12 | 100-150 | 80-120 |

*注意：实际性能取决于具体的硬件配置、散热条件和系统负载*

## 💡 使用建议

1. **首次使用**: 从 `basic_mining_demo` 开始
2. **性能测试**: 使用 `performance_monitoring_demo`
3. **生产环境**: 参考 `real_mining_simulation`
4. **优化调试**: 使用基准测试工具

## 📞 支持

如果遇到问题：

1. 查看 [故障排除](#故障排除) 部分
2. 检查项目文档: `docs/` 目录
3. 运行诊断: `cargo check --examples`
4. 查看日志: `RUST_LOG=debug cargo run --example <name>`

## 🔗 相关链接

- [项目主页](https://github.com/your-org/cgminer-rs)
- [API文档](https://docs.rs/cgminer-cpu-btc-core)
- [基准测试指南](docs/BENCHMARK_GUIDE.md)
- [CPU核心限制说明](docs/CPU_CORE_LIMITATIONS.md)
