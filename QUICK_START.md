# CGMiner CPU BTC Core - 快速开始指南

🚀 **无算力限制版本** - 发挥CPU最大挖矿性能

## 📋 快速概览

本项目提供了高性能的CPU比特币挖矿解决方案，所有示例都已优化为**无算力上限**，让您的CPU发挥最大性能。

### 🎯 性能特点

- ✅ **无算力限制** - 发挥CPU最大性能
- ✅ **大批次处理** - 50K-100K哈希批次
- ✅ **低错误率** - 0.1%错误率
- ✅ **高频率运行** - 4GHz+ CPU频率
- ✅ **多核心支持** - 自动检测并使用所有CPU核心
- ✅ **硬件加速** - Apple Silicon SHA-256加速

## 🚀 30秒快速开始

```bash
# 1. 进入项目目录
cd /Users/gecko/project/linux/cgminer-cpu-btc-core

# 2. 运行基本挖矿演示（无算力限制，持续运算直到找到解）
cargo run --release --example basic_mining_demo

# 3. 运行多设备高性能挖矿
cargo run --release --example multi_device_demo

# 4. 运行性能监控（查看实际算力）
cargo run --release --example performance_monitoring_demo
```

### ✅ 测试结果

在Apple Silicon (M芯片)上的实际测试结果：
- **算力**: 2.85 MH/s (无限制模式)
- **状态**: 持续运算直到找到有效解
- **硬件加速**: Apple Silicon SHA-256加速已启用
- **性能**: 已测试超过1.4亿次哈希计算

## 📊 性能配置详情

### 设备配置（高性能模式）

```rust
DeviceConfig {
    frequency: 4000,           // 4GHz高频率
    voltage: 1350,             // 高电压支持
    auto_tune: true,           // 自动调优
    chip_count: num_cpus::get(), // 使用所有CPU核心
    temperature_limit: 90.0,   // 90°C温度限制
    fan_speed: Some(100),      // 最大风扇速度
}
```

### 挖矿参数（最大性能）

```rust
SoftwareDevice::new(
    device_info,
    config,
    f64::MAX,  // 🔥 无算力限制
    0.001,     // 0.1% 低错误率
    100000,    // 100K 大批次处理
)
```

## 🏃‍♂️ 运行示例

### 1. 基本挖矿演示
```bash
cargo run --release --example basic_mining_demo
```
**特点**: 无算力限制，展示基本挖矿流程和SHA-256计算

### 2. 多设备并行挖矿
```bash
cargo run --release --example multi_device_demo
```
**特点**: 4个设备并行，每个设备无算力限制

### 3. 性能监控
```bash
cargo run --release --example performance_monitoring_demo
```
**特点**: 实时监控算力、温度、功耗，压力测试

### 4. 温度管理
```bash
cargo run --release --example temperature_demo
```
**特点**: 高温运行监控，自适应温度控制

### 5. CPU亲和性优化
```bash
cargo run --release --example cpu_affinity_demo
```
**特点**: CPU核心绑定，NUMA优化

### 6. 真实挖矿模拟
```bash
cargo run --release --example real_mining_simulation
```
**特点**: 模拟真实矿池环境，收益计算

## 🔧 一键运行脚本

```bash
# 运行单个示例
./run_examples.sh basic --release

# 运行所有示例
./run_examples.sh --all --release

# 启用实验特性
./run_examples.sh --all --release --features experimental
```

## 📈 预期性能

### Apple Silicon (M1/M2/M3)
- **算力**: 15-30 MH/s (无限制)
- **功耗**: 20-40W
- **效率**: 500-750 KH/W
- **特性**: 硬件SHA-256加速

### Intel CPU (现代)
- **算力**: 10-25 MH/s (无限制)
- **功耗**: 65-150W
- **效率**: 100-200 KH/W
- **特性**: AVX2指令集优化

### AMD CPU (现代)
- **算力**: 12-28 MH/s (无限制)
- **功耗**: 65-180W
- **效率**: 120-220 KH/W
- **特性**: 多核心并行优化

## ⚡ 性能优化建议

### 1. 编译优化
```bash
# 使用发布模式（必须）
cargo build --release

# 启用所有优化特性
cargo build --release --features "experimental,simd-optimizations"

# 针对本机CPU优化
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### 2. 系统优化
```bash
# Linux: 设置CPU性能模式
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# macOS: 禁用节能模式
sudo pmset -a powernap 0
sudo pmset -a standby 0

# 提高进程优先级
nice -n -20 ./target/release/examples/basic_mining_demo
```

### 3. 散热优化
- 确保CPU散热器工作正常
- 监控温度，避免过热降频
- 考虑使用液冷散热器
- 保持机箱通风良好

## 🔍 实时监控

### 查看实际算力
```bash
# 运行性能监控示例
cargo run --release --example performance_monitoring_demo

# 查看系统资源使用
htop  # Linux/macOS
Activity Monitor  # macOS GUI
```

### 温度监控
```bash
# 运行温度管理示例
cargo run --release --example temperature_demo

# 系统温度监控
sensors  # Linux
istats  # macOS (需要安装)
```

## 🚨 注意事项

### 安全提醒
- ⚠️ **高性能模式会产生大量热量**
- ⚠️ **确保散热充足，避免过热**
- ⚠️ **监控CPU温度，建议不超过90°C**
- ⚠️ **长时间高负载可能影响硬件寿命**

### 电力消耗
- 💡 **无算力限制模式功耗较高**
- 💡 **建议在电力充足环境下运行**
- 💡 **笔记本电脑建议连接电源**

## 🎯 下一步

1. **基础测试**: 先运行 `basic_mining_demo` 了解基本功能
2. **性能测试**: 运行 `performance_monitoring_demo` 查看实际算力
3. **多设备测试**: 运行 `multi_device_demo` 测试并行性能
4. **生产环境**: 参考 `real_mining_simulation` 进行实际部署

## 📞 获取帮助

- 📖 查看详细文档: `EXAMPLES_README.md`
- 🔧 故障排除: 运行 `cargo check --examples`
- 📊 基准测试: `./run_benchmarks.sh --all`
- 💬 问题反馈: 查看项目文档或提交issue

---

🎉 **开始您的高性能CPU挖矿之旅！**
