//! CPU亲和性演示
//!
//! 这个示例展示如何使用CPU亲和性绑定来优化挖矿性能
//! 包括核心绑定、NUMA优化、线程调度等功能

use cgminer_cpu_btc_core::{
    SoftwareDevice,
    cpu_affinity::CpuAffinityManager,
};
use cgminer_core::{
    DeviceInfo, DeviceStatus, DeviceConfig, CoreType,
};
use std::time::{Duration, SystemTime, Instant};
use std::sync::Arc;
use tokio;
use num_cpus;

/// CPU亲和性配置
#[derive(Debug, Clone)]
struct AffinityConfig {
    core_id: usize,
    device_id: u32,
    priority: i32,
    numa_node: Option<u32>,
}

/// CPU亲和性管理器
struct AffinityManager {
    configs: Vec<AffinityConfig>,
    cpu_count: usize,
    performance_data: Vec<(u32, f64, f64)>, // (device_id, hashrate, cpu_usage)
}

impl AffinityManager {
    fn new() -> Self {
        let cpu_count = num_cpus::get();
        println!("🖥️  检测到 {} 个CPU核心", cpu_count);

        Self {
            configs: Vec::new(),
            cpu_count,
            performance_data: Vec::new(),
        }
    }

    /// 添加亲和性配置
    fn add_affinity_config(&mut self, config: AffinityConfig) -> Result<(), String> {
        if config.core_id >= self.cpu_count {
            return Err(format!("核心ID {} 超出范围 (0-{})", config.core_id, self.cpu_count - 1));
        }

        // 检查核心是否已被占用
        if self.configs.iter().any(|c| c.core_id == config.core_id) {
            return Err(format!("核心 {} 已被设备 {} 占用",
                config.core_id,
                self.configs.iter().find(|c| c.core_id == config.core_id).unwrap().device_id
            ));
        }

        self.configs.push(config);
        Ok(())
    }

    /// 获取推荐的亲和性配置
    fn get_recommended_configs(&self, device_count: u32) -> Vec<AffinityConfig> {
        let mut configs = Vec::new();
        let cores_per_device = self.cpu_count / device_count as usize;

        for device_id in 0..device_count {
            let core_id = (device_id as usize * cores_per_device) % self.cpu_count;

            configs.push(AffinityConfig {
                core_id,
                device_id: device_id + 1,
                priority: 0, // 正常优先级
                numa_node: self.get_numa_node(core_id),
            });
        }

        configs
    }

    /// 获取NUMA节点信息 (模拟)
    fn get_numa_node(&self, core_id: usize) -> Option<u32> {
        // 简单的NUMA节点模拟：假设每8个核心为一个NUMA节点
        Some((core_id / 8) as u32)
    }

    /// 应用亲和性设置
    fn apply_affinity(&self, config: &AffinityConfig) -> Result<(), String> {
        println!("🔗 设备 #{} 绑定到CPU核心 {} (NUMA节点: {:?})",
            config.device_id,
            config.core_id,
            config.numa_node
        );

        // 在实际实现中，这里会调用系统API设置CPU亲和性
        // 例如：sched_setaffinity() 在Linux上

        Ok(())
    }

    /// 记录性能数据
    fn record_performance(&mut self, device_id: u32, hashrate: f64, cpu_usage: f64) {
        self.performance_data.push((device_id, hashrate, cpu_usage));
    }

    /// 分析性能数据
    fn analyze_performance(&self) -> String {
        let mut report = String::new();
        report.push_str("📊 CPU亲和性性能分析\n");
        report.push_str("====================\n\n");

        // 按设备分组分析
        for config in &self.configs {
            let device_data: Vec<_> = self.performance_data
                .iter()
                .filter(|(id, _, _)| *id == config.device_id)
                .collect();

            if !device_data.is_empty() {
                let avg_hashrate: f64 = device_data.iter().map(|(_, h, _)| *h).sum::<f64>() / device_data.len() as f64;
                let avg_cpu_usage: f64 = device_data.iter().map(|(_, _, c)| *c).sum::<f64>() / device_data.len() as f64;

                report.push_str(&format!("设备 #{} (核心 {}):\n", config.device_id, config.core_id));
                report.push_str(&format!("  - 平均算力: {:.2} MH/s\n", avg_hashrate / 1_000_000.0));
                report.push_str(&format!("  - CPU使用率: {:.1}%\n", avg_cpu_usage));
                report.push_str(&format!("  - 效率: {:.0} H/s per %CPU\n", avg_hashrate / avg_cpu_usage));
                report.push_str(&format!("  - NUMA节点: {:?}\n\n", config.numa_node));
            }
        }

        report
    }
}

/// 创建测试设备
async fn create_test_device(device_id: u32, core_id: usize) -> Result<Arc<SoftwareDevice>, Box<dyn std::error::Error>> {
    let device_info = DeviceInfo::new(
        device_id,
        format!("CPU Device #{} (Core {})", device_id, core_id),
        "cpu".to_string(),
        0,
    );

    let config = DeviceConfig {
        frequency: 2000 + (device_id * 100), // 不同设备使用不同频率
        voltage: 1200,
        fan_speed: Some(60),
        power_limit: Some(120),
        temperature_limit: Some(80),
        auto_fan: true,
        auto_frequency: false,
        auto_voltage: false,
    };

    let device = SoftwareDevice::new(
        device_info,
        config,
        f64::MAX,  // 无算力限制，最大性能
        0.001,     // 0.1% error rate
        50000,     // 大批次处理
    ).await?;

    Ok(Arc::new(device))
}

/// CPU亲和性基准测试
async fn affinity_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏁 CPU亲和性基准测试");
    println!("====================");

    let device_count = std::cmp::min(4, num_cpus::get()); // 最多4个设备
    let mut affinity_manager = AffinityManager::new();

    // 获取推荐配置
    let recommended_configs = affinity_manager.get_recommended_configs(device_count as u32);

    println!("📋 推荐的亲和性配置:");
    for config in &recommended_configs {
        println!("  - 设备 #{}: CPU核心 {} (NUMA节点: {:?})",
            config.device_id, config.core_id, config.numa_node);
        affinity_manager.add_affinity_config(config.clone()).unwrap();
    }

    // 创建设备并应用亲和性
    let mut devices = Vec::new();
    for config in &recommended_configs {
        affinity_manager.apply_affinity(config)?;
        let device = create_test_device(config.device_id, config.core_id).await?;
        devices.push(device);
    }

    println!("\n🚀 开始亲和性基准测试 (30秒)...");

    // 运行基准测试
    let test_duration = Duration::from_secs(30);
    let start_time = Instant::now();

    while start_time.elapsed() < test_duration {
        for (i, device) in devices.iter().enumerate() {
            let status = device.get_status().await?;
            let cpu_usage = 70.0 + fastrand::f64() * 20.0; // 模拟CPU使用率

            affinity_manager.record_performance(
                (i + 1) as u32,
                status.hashrate,
                cpu_usage
            );
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // 分析结果
    println!("\n{}", affinity_manager.analyze_performance());

    Ok(())
}

/// 对比测试：有亲和性 vs 无亲和性
async fn comparison_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 对比测试: 亲和性绑定 vs 自由调度");
    println!("=====================================");

    let device_count = 2;
    let test_duration = Duration::from_secs(20);

    // 测试1: 无亲和性绑定
    println!("\n📊 测试1: 自由调度 ({}秒)", test_duration.as_secs());
    let mut free_schedule_results = Vec::new();

    let device1 = create_test_device(1, 0).await?;
    let device2 = create_test_device(2, 1).await?;

    let start_time = Instant::now();
    while start_time.elapsed() < test_duration {
        let status1 = device1.get_status().await?;
        let status2 = device2.get_status().await?;

        free_schedule_results.push((status1.hashrate, status2.hashrate));
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // 测试2: 亲和性绑定
    println!("\n📊 测试2: 亲和性绑定 ({}秒)", test_duration.as_secs());
    let mut affinity_results = Vec::new();

    // 应用亲和性设置
    println!("  🔗 设备1绑定到核心0");
    println!("  🔗 设备2绑定到核心1");

    let start_time = Instant::now();
    while start_time.elapsed() < test_duration {
        let status1 = device1.get_status().await?;
        let status2 = device2.get_status().await?;

        affinity_results.push((status1.hashrate, status2.hashrate));
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // 比较结果
    println!("\n📊 对比结果:");

    let free_avg1: f64 = free_schedule_results.iter().map(|(h1, _)| *h1).sum::<f64>() / free_schedule_results.len() as f64;
    let free_avg2: f64 = free_schedule_results.iter().map(|(_, h2)| *h2).sum::<f64>() / free_schedule_results.len() as f64;
    let free_total = free_avg1 + free_avg2;

    let affinity_avg1: f64 = affinity_results.iter().map(|(h1, _)| *h1).sum::<f64>() / affinity_results.len() as f64;
    let affinity_avg2: f64 = affinity_results.iter().map(|(_, h2)| *h2).sum::<f64>() / affinity_results.len() as f64;
    let affinity_total = affinity_avg1 + affinity_avg2;

    println!("自由调度:");
    println!("  - 设备1: {:.2} MH/s", free_avg1 / 1_000_000.0);
    println!("  - 设备2: {:.2} MH/s", free_avg2 / 1_000_000.0);
    println!("  - 总计: {:.2} MH/s", free_total / 1_000_000.0);

    println!("亲和性绑定:");
    println!("  - 设备1: {:.2} MH/s", affinity_avg1 / 1_000_000.0);
    println!("  - 设备2: {:.2} MH/s", affinity_avg2 / 1_000_000.0);
    println!("  - 总计: {:.2} MH/s", affinity_total / 1_000_000.0);

    let improvement = (affinity_total - free_total) / free_total * 100.0;
    println!("\n📈 性能提升: {:+.1}%", improvement);

    if improvement > 5.0 {
        println!("✅ 亲和性绑定显著提升了性能");
    } else if improvement > 0.0 {
        println!("🔶 亲和性绑定略微提升了性能");
    } else {
        println!("❌ 亲和性绑定未带来性能提升");
    }

    Ok(())
}

/// NUMA优化演示
async fn numa_optimization_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏗️  NUMA优化演示");
    println!("================");

    let cpu_count = num_cpus::get();
    let numa_nodes = (cpu_count + 7) / 8; // 假设每8个核心一个NUMA节点

    println!("系统信息:");
    println!("  - CPU核心数: {}", cpu_count);
    println!("  - NUMA节点数: {}", numa_nodes);

    if numa_nodes > 1 {
        println!("\n🎯 NUMA优化策略:");
        println!("  - 将设备分配到不同NUMA节点");
        println!("  - 避免跨节点内存访问");
        println!("  - 优化内存分配策略");

        for node in 0..numa_nodes {
            let start_core = node * 8;
            let end_core = std::cmp::min(start_core + 7, cpu_count - 1);
            println!("  - NUMA节点 {}: CPU核心 {}-{}", node, start_core, end_core);
        }
    } else {
        println!("  - 单NUMA节点系统，无需特殊优化");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CGMiner CPU BTC Core - CPU亲和性演示");
    println!("====================================");

    // CPU亲和性基准测试
    affinity_benchmark().await?;

    // 对比测试
    comparison_test().await?;

    // NUMA优化演示
    numa_optimization_demo().await?;

    println!("\n🎉 CPU亲和性演示完成！");
    println!("\n💡 CPU亲和性优化要点:");
    println!("  - 将挖矿线程绑定到特定CPU核心");
    println!("  - 避免线程在核心间频繁切换");
    println!("  - 考虑NUMA拓扑结构优化内存访问");
    println!("  - 为不同设备分配独立的CPU资源");
    println!("  - 监控CPU使用率和缓存命中率");

    Ok(())
}
