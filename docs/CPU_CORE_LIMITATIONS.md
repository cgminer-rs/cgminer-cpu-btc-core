# CPU挖矿核心实际限制说明

## 概述

本文档详细说明了CPU挖矿核心在实际运行中的限制，帮助用户正确理解和配置CPU挖矿功能。

## 🚫 CPU模式下无法实现的功能

### 1. 温度控制
**限制说明：**
- CPU温度由系统BIOS和操作系统的热管理控制
- 应用程序无法直接调节CPU温度
- 无法控制CPU风扇转速（由主板BIOS控制）

**可以做的：**
- ✅ 监控CPU温度（通过系统传感器）
- ✅ 设置温度阈值告警
- ✅ 温度过高时自动降低工作负载

**无法做的：**
- ❌ 直接控制CPU温度
- ❌ 调节CPU风扇转速
- ❌ 设置目标温度

### 2. 频率调节
**限制说明：**
- CPU频率由操作系统的频率调节器（governor）控制
- 应用程序无法直接设置CPU频率
- 频率调节需要管理员权限和特定的系统配置

**可以做的：**
- ✅ 通过调整工作负载间接影响频率
- ✅ 监控当前CPU频率（如果系统支持）

**无法做的：**
- ❌ 直接设置CPU频率
- ❌ 强制CPU运行在特定频率
- ❌ 绕过系统频率管理

### 3. 功耗控制
**限制说明：**
- CPU功耗由硬件和系统级电源管理控制
- 应用程序无法直接限制CPU功耗
- 功耗控制涉及复杂的硬件机制

**可以做的：**
- ✅ 监控系统功耗（如果系统支持）
- ✅ 通过调整CPU使用率间接影响功耗
- ✅ 设置CPU使用率上限

**无法做的：**
- ❌ 直接设置功耗限制
- ❌ 控制CPU电压
- ❌ 强制功耗预算

### 4. 电压控制
**限制说明：**
- CPU电压完全由硬件控制
- 电压调节需要特殊的硬件支持和驱动
- 错误的电压设置可能损坏硬件

**可以做的：**
- ✅ 监控CPU电压（如果系统支持）

**无法做的：**
- ❌ 调节CPU电压
- ❌ 设置电压范围
- ❌ 电压优化

## ✅ CPU模式下可以实现的功能

### 1. 自适应负载控制
这是CPU模式下唯一可以真正控制的功能：

```toml
[cores.cpu_mining.adaptive_load]
enabled = true                     # 启用自适应负载
cpu_usage_threshold = 90.0         # CPU使用率阈值
temperature_threshold = 80.0       # 温度阈值（触发负载降低）
load_reduction_factor = 0.8        # 负载降低因子
recovery_delay_seconds = 30        # 恢复延迟
min_load_factor = 0.3              # 最小负载因子
```

### 2. CPU绑定和调度
```toml
[cores.cpu_mining.cpu_affinity]
enabled = true                     # 启用CPU绑定
strategy = "intelligent"           # 智能绑定策略
prefer_performance_cores = true    # 优先使用性能核心
avoid_hyperthreading = false       # 利用超线程
load_balancing = true              # 启用负载均衡
```

### 3. 系统监控
```toml
[cores.cpu_mining.monitoring]
enable_system_monitoring = true    # 启用系统监控
enable_temperature_monitoring = true  # 启用温度监控（仅读取）
enable_cpu_usage_monitoring = true    # 启用CPU使用率监控
monitoring_interval = 10          # 监控间隔
log_system_stats = true           # 记录系统统计信息
```

### 4. 算法优化
```toml
[cores.cpu_mining.algorithm]
enable_simd = true                  # 启用SIMD优化
prefer_avx512 = true               # 优先使用AVX-512
prefer_avx2 = true                 # 优先使用AVX2
enable_vectorization = true        # 启用向量化
batch_optimization = true          # 启用批处理优化
cache_optimization = true          # 启用缓存优化
```

## 📋 正确的配置示例

### 高性能配置（专用挖矿机器）
```toml
[cores.cpu_mining]
enabled = true
device_count = 32                   # 根据CPU核心数调整
max_cpu_percent = 95               # 高CPU使用率
batch_size = 50000                 # 大批处理

[cores.cpu_mining.adaptive_load]
enabled = true
cpu_usage_threshold = 95.0
temperature_threshold = 85.0
load_reduction_factor = 0.9
min_load_factor = 0.5
```

### 平衡配置（日常使用电脑）
```toml
[cores.cpu_mining]
enabled = true
device_count = 8                    # 适中的设备数量
max_cpu_percent = 70               # 保留CPU资源给其他应用
batch_size = 15000

[cores.cpu_mining.adaptive_load]
enabled = true
cpu_usage_threshold = 80.0
temperature_threshold = 75.0
load_reduction_factor = 0.7
min_load_factor = 0.3
```

### 低功耗配置（笔记本电脑）
```toml
[cores.cpu_mining]
enabled = true
device_count = 4                    # 少量设备
max_cpu_percent = 50               # 低CPU使用率
batch_size = 5000

[cores.cpu_mining.adaptive_load]
enabled = true
cpu_usage_threshold = 60.0
temperature_threshold = 70.0
load_reduction_factor = 0.6
min_load_factor = 0.2
```

## ⚠️ 重要注意事项

1. **温度监控依赖系统支持**
   - 不是所有系统都提供温度传感器接口
   - macOS和某些Linux发行版可能限制温度访问

2. **CPU绑定需要权限**
   - 某些系统需要管理员权限才能设置CPU亲和性
   - 容器环境可能限制CPU绑定功能

3. **性能监控的准确性**
   - 系统负载会影响监控数据的准确性
   - 其他应用程序会影响CPU使用率统计

4. **自适应负载的响应时间**
   - 温度变化有延迟，自适应调整需要时间
   - 过于频繁的调整可能影响挖矿稳定性

## 🔧 故障排除

### 温度监控不工作
```bash
# Linux: 检查传感器
sensors

# macOS: 检查系统信息
system_profiler SPHardwareDataType
```

### CPU绑定失败
```bash
# 检查权限
id
# 检查CPU拓扑
lscpu  # Linux
sysctl hw.ncpu  # macOS
```

### 性能不佳
1. 检查CPU使用率是否达到预期
2. 确认SIMD指令集支持
3. 调整批处理大小
4. 检查内存使用情况

## 📚 相关文档

- [CPU核心配置指南](./CPU_CORE_CONFIGURATION.md)
- [性能优化建议](./PERFORMANCE_OPTIMIZATION.md)
- [故障排除指南](./TROUBLESHOOTING.md)
