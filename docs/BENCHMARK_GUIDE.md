# CGMiner CPU BTC Core 基准测试指南

## 概述

本指南介绍如何使用 cgminer-cpu-btc-core 的完整基准测试套件来评估CPU Bitcoin挖矿性能。

## 快速开始

### 1. 运行完整基准测试

```bash
# 运行所有基准测试
./run_benchmarks.sh --all

# 快速测试模式
./run_benchmarks.sh --quick

# 详细测试模式
./run_benchmarks.sh --detailed
```

### 2. 运行特定测试

```bash
# SHA256 哈希性能测试
./run_benchmarks.sh --sha256

# 设备创建和管理测试
./run_benchmarks.sh --device

# 工作处理性能测试
./run_benchmarks.sh --work

# 并发性能测试
./run_benchmarks.sh --concurrency
```

### 3. 查看结果

```bash
# 生成报告
./run_benchmarks.sh --report

# 打开HTML报告
./run_benchmarks.sh --open-report
```

## 基准测试套件

### 1. SHA256 双重哈希测试 (`bench_double_sha256`)

测试Bitcoin挖矿核心算法的性能：

- **单次哈希**: 测试单个区块头的SHA256双重哈希性能
- **批量哈希**: 测试批量处理的吞吐量（100, 1000, 10000个）
- **内存效率**: 比较预分配vs动态分配的性能差异

**关键指标**:
- 平均哈希时间 (纳秒)
- 每秒哈希数 (H/s)
- 吞吐量 (元素/秒)

### 2. 设备创建测试 (`bench_device_creation`)

测试设备生命周期管理性能：

- **设备创建**: 软件设备实例化时间
- **设备初始化**: 异步初始化过程性能
- **配置应用**: 不同配置参数的影响

**关键指标**:
- 创建时间 (微秒)
- 初始化时间 (毫秒)
- 内存使用量

### 3. 工作处理测试 (`bench_work_processing`)

测试挖矿工作提交和处理性能：

- **单个工作提交**: 基本工作处理延迟
- **批量工作处理**: 批量提交的吞吐量（10, 50, 100个）
- **工作队列管理**: 队列操作性能

**关键指标**:
- 工作提交延迟 (毫秒)
- 批量处理吞吐量 (工作/秒)
- 队列操作时间

### 4. 核心工厂测试 (`bench_core_factory`)

测试核心组件工厂模式性能：

- **工厂创建**: 工厂实例化时间
- **核心创建**: 通过工厂创建核心的性能
- **能力查询**: 获取核心能力信息的开销

**关键指标**:
- 工厂创建时间 (微秒)
- 核心创建时间 (毫秒)
- 查询响应时间

### 5. 性能监控测试 (`bench_performance_monitoring`)

测试性能监控系统的开销：

- **监控器创建**: 性能监控器初始化
- **算力记录**: 算力数据记录性能
- **批量记录**: 大量性能数据的处理能力

**关键指标**:
- 记录延迟 (纳秒)
- 批量处理能力 (记录/秒)
- 内存开销

### 6. 温度监控测试 (`bench_temperature_monitoring`)

测试温度监控功能性能：

- **温度读取**: 单次温度读取时间
- **批量读取**: 连续温度监控的性能影响

**关键指标**:
- 读取延迟 (微秒)
- 监控频率 (读取/秒)

### 7. 内存效率测试 (`bench_memory_efficiency`)

测试内存使用模式的性能影响：

- **预分配向量**: 预先分配内存的性能
- **动态向量**: 动态分配内存的性能
- **内存访问模式**: 不同访问模式的效率

**关键指标**:
- 分配时间 (微秒)
- 内存使用量 (字节)
- 访问延迟

### 8. 并发性能测试 (`bench_concurrency`)

测试多设备并发挖矿性能：

- **并发设备**: 多个设备同时工作的性能
- **异步处理**: 异步任务调度效率
- **资源竞争**: 共享资源访问的影响

**关键指标**:
- 并发吞吐量
- 任务调度延迟
- 资源利用率

## 运行示例

### 基本演示

```bash
# 运行基准测试演示
cargo run --example benchmark_demo

# 编译并运行演示（优化模式）
cargo run --release --example benchmark_demo
```

### 自定义基准测试

```bash
# 使用自定义参数运行
cargo bench --bench cpu_btc_core_benchmark -- --sample-size 1000

# 保存基线
./run_benchmarks.sh --save-baseline my_baseline

# 与基线比较
./run_benchmarks.sh --compare my_baseline
```

## 结果解读

### 性能指标

1. **延迟 (Latency)**: 单次操作的时间
2. **吞吐量 (Throughput)**: 单位时间内处理的操作数
3. **置信区间**: 结果的统计可信度范围
4. **标准差**: 性能稳定性指标

### HTML报告

基准测试会生成详细的HTML报告，包含：

- 性能趋势图表
- 统计分析结果
- 性能回归检测
- 详细的测量数据

### 性能优化建议

基于基准测试结果的优化方向：

1. **SHA256优化**: 利用硬件加速指令
2. **内存优化**: 减少分配和提高缓存命中率
3. **并发优化**: 优化线程调度和资源共享
4. **算法优化**: 改进核心挖矿算法

## 故障排除

### 常见问题

1. **编译错误**: 确保所有依赖都已安装
2. **权限问题**: 确保有写入target目录的权限
3. **内存不足**: 减少批量测试的大小
4. **时间过长**: 使用快速模式进行初步测试

### 调试技巧

```bash
# 启用详细日志
RUST_LOG=debug ./run_benchmarks.sh --sha256

# 检查依赖
cargo check --benches

# 清理并重新构建
cargo clean && cargo build --release
```

## 配置文件

基准测试可以通过 `benchmark_config.toml` 文件进行配置：

- 测试参数调整
- 输出格式设置
- 系统信息收集
- 优化选项配置

## 最佳实践

1. **环境一致性**: 在相同环境下进行比较测试
2. **多次运行**: 运行多次以获得稳定结果
3. **基线管理**: 定期保存和更新性能基线
4. **结果分析**: 结合系统监控分析性能瓶颈
5. **持续监控**: 将基准测试集成到CI/CD流程

## 扩展基准测试

### 添加新的测试

1. 在 `benches/cpu_btc_core_benchmark.rs` 中添加新函数
2. 使用 `criterion_group!` 宏注册测试
3. 更新 `run_benchmarks.sh` 脚本支持新测试

### 自定义指标

```rust
// 添加自定义测量
group.bench_function("custom_test", |b| {
    b.iter_custom(|iters| {
        let start = Instant::now();
        for _ in 0..iters {
            // 自定义测试逻辑
        }
        start.elapsed()
    })
});
```

---

更多信息请参考 [Criterion.rs 文档](https://docs.rs/criterion/) 和项目源码。
