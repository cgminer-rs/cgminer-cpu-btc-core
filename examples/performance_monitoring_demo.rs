//! 性能监控演示
//!
//! 这个示例展示如何监控和分析CPU挖矿的性能指标
//! 包括算力统计、温度监控、功耗分析、效率计算等

use cgminer_cpu_btc_core::{
    SoftwareDevice,
    performance::PerformanceMonitor,
};
use cgminer_core::{
    DeviceInfo, DeviceStatus, DeviceConfig, Work, CoreType,
};
use std::time::{Duration, SystemTime, Instant};
use std::sync::Arc;
use tokio;
use serde::{Serialize, Deserialize};

/// 性能统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceStats {
    timestamp: SystemTime,
    hashrate: f64,
    temperature: f64,
    power_consumption: f64,
    efficiency: f64, // H/W
    cpu_usage: f64,
    memory_usage: f64,
}

/// 性能分析器
struct PerformanceAnalyzer {
    stats_history: Vec<PerformanceStats>,
    start_time: Instant,
}

impl PerformanceAnalyzer {
    fn new() -> Self {
        Self {
            stats_history: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// 记录性能数据
    fn record_stats(&mut self, stats: PerformanceStats) {
        self.stats_history.push(stats);
    }

    /// 计算平均性能
    fn calculate_averages(&self) -> Option<PerformanceStats> {
        if self.stats_history.is_empty() {
            return None;
        }

        let count = self.stats_history.len() as f64;
        let sum_hashrate: f64 = self.stats_history.iter().map(|s| s.hashrate).sum();
        let sum_temp: f64 = self.stats_history.iter().map(|s| s.temperature).sum();
        let sum_power: f64 = self.stats_history.iter().map(|s| s.power_consumption).sum();
        let sum_efficiency: f64 = self.stats_history.iter().map(|s| s.efficiency).sum();
        let sum_cpu: f64 = self.stats_history.iter().map(|s| s.cpu_usage).sum();
        let sum_memory: f64 = self.stats_history.iter().map(|s| s.memory_usage).sum();

        Some(PerformanceStats {
            timestamp: SystemTime::now(),
            hashrate: sum_hashrate / count,
            temperature: sum_temp / count,
            power_consumption: sum_power / count,
            efficiency: sum_efficiency / count,
            cpu_usage: sum_cpu / count,
            memory_usage: sum_memory / count,
        })
    }

    /// 获取性能趋势
    fn get_trends(&self) -> (f64, f64, f64) {
        if self.stats_history.len() < 2 {
            return (0.0, 0.0, 0.0);
        }

        let recent_count = std::cmp::min(5, self.stats_history.len());
        let recent_stats = &self.stats_history[self.stats_history.len() - recent_count..];
        let older_stats = &self.stats_history[0..recent_count];

        let recent_avg_hashrate: f64 = recent_stats.iter().map(|s| s.hashrate).sum::<f64>() / recent_count as f64;
        let older_avg_hashrate: f64 = older_stats.iter().map(|s| s.hashrate).sum::<f64>() / recent_count as f64;

        let recent_avg_temp: f64 = recent_stats.iter().map(|s| s.temperature).sum::<f64>() / recent_count as f64;
        let older_avg_temp: f64 = older_stats.iter().map(|s| s.temperature).sum::<f64>() / recent_count as f64;

        let recent_avg_power: f64 = recent_stats.iter().map(|s| s.power_consumption).sum::<f64>() / recent_count as f64;
        let older_avg_power: f64 = older_stats.iter().map(|s| s.power_consumption).sum::<f64>() / recent_count as f64;

        let hashrate_trend = if older_avg_hashrate > 0.0 {
            (recent_avg_hashrate - older_avg_hashrate) / older_avg_hashrate * 100.0
        } else { 0.0 };

        let temp_trend = recent_avg_temp - older_avg_temp;
        let power_trend = recent_avg_power - older_avg_power;

        (hashrate_trend, temp_trend, power_trend)
    }

    /// 生成性能报告
    fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("📊 性能监控报告\n");
        report.push_str("================\n\n");

        if let Some(avg_stats) = self.calculate_averages() {
            report.push_str(&format!("📈 平均性能指标:\n"));
            report.push_str(&format!("  - 算力: {:.2} MH/s\n", avg_stats.hashrate / 1_000_000.0));
            report.push_str(&format!("  - 温度: {:.1}°C\n", avg_stats.temperature));
            report.push_str(&format!("  - 功耗: {:.1}W\n", avg_stats.power_consumption));
            report.push_str(&format!("  - 效率: {:.0} H/W\n", avg_stats.efficiency));
            report.push_str(&format!("  - CPU使用率: {:.1}%\n", avg_stats.cpu_usage));
            report.push_str(&format!("  - 内存使用: {:.1}MB\n", avg_stats.memory_usage));
        }

        let (hashrate_trend, temp_trend, power_trend) = self.get_trends();
        report.push_str(&format!("\n📊 性能趋势:\n"));
        report.push_str(&format!("  - 算力变化: {:+.1}%\n", hashrate_trend));
        report.push_str(&format!("  - 温度变化: {:+.1}°C\n", temp_trend));
        report.push_str(&format!("  - 功耗变化: {:+.1}W\n", power_trend));

        report.push_str(&format!("\n⏱️  监控时长: {:.1}秒\n", self.start_time.elapsed().as_secs_f64()));
        report.push_str(&format!("📊 数据点数: {}\n", self.stats_history.len()));

        report
    }
}

/// 创建测试设备
async fn create_test_device() -> Result<Arc<SoftwareDevice>, Box<dyn std::error::Error>> {
    let device_info = DeviceInfo::new(
        1,
        "Performance Test Device".to_string(),
        "cpu".to_string(),
        0,
    );

    let config = DeviceConfig {
        frequency: 2400,
        voltage: 1250,
        fan_speed: Some(70),
        power_limit: Some(200),
        temperature_limit: Some(85),
        auto_fan: true,
        auto_frequency: true,
        auto_voltage: false,
    };

    let device = SoftwareDevice::new(
        device_info,
        config,
        f64::MAX,  // 无算力限制，最大性能
        0.001,     // 0.1% error rate
        100000,    // 超大批次处理
    ).await?;

    Ok(Arc::new(device))
}

/// 模拟系统资源使用情况
fn get_system_stats() -> (f64, f64) {
    // 模拟CPU和内存使用率
    let cpu_usage = 60.0 + fastrand::f64() * 30.0; // 60-90%
    let memory_usage = 1024.0 + fastrand::f64() * 512.0; // 1-1.5GB
    (cpu_usage, memory_usage)
}

/// 实时性能监控
async fn real_time_monitoring(device: Arc<SoftwareDevice>, duration: Duration) -> Result<PerformanceAnalyzer, Box<dyn std::error::Error>> {
    println!("🔍 开始实时性能监控 ({}秒)", duration.as_secs());
    println!("监控间隔: 1秒");
    println!("----------------------------------------");

    let mut analyzer = PerformanceAnalyzer::new();
    let start_time = Instant::now();
    let mut sample_count = 0;

    while start_time.elapsed() < duration {
        sample_count += 1;

        // 获取设备状态
        let status = device.get_status().await?;
        let (cpu_usage, memory_usage) = get_system_stats();

        // 计算效率
        let efficiency = if status.power_consumption > 0.0 {
            status.hashrate / status.power_consumption
        } else {
            0.0
        };

        // 创建性能统计
        let stats = PerformanceStats {
            timestamp: SystemTime::now(),
            hashrate: status.hashrate,
            temperature: status.temperature,
            power_consumption: status.power_consumption,
            efficiency,
            cpu_usage,
            memory_usage,
        };

        // 记录统计数据
        analyzer.record_stats(stats.clone());

        // 显示实时数据
        println!("#{:02} | {:.2}MH/s | {:.1}°C | {:.1}W | {:.0}H/W | CPU:{:.1}% | MEM:{:.0}MB",
            sample_count,
            stats.hashrate / 1_000_000.0,
            stats.temperature,
            stats.power_consumption,
            stats.efficiency,
            stats.cpu_usage,
            stats.memory_usage
        );

        // 检查异常情况
        if stats.temperature > 80.0 {
            println!("  ⚠️  警告: 温度过高!");
        }
        if stats.power_consumption > 180.0 {
            println!("  ⚠️  警告: 功耗过高!");
        }
        if stats.hashrate < 1_000_000.0 {
            println!("  ⚠️  警告: 算力过低!");
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(analyzer)
}

/// 性能压力测试
async fn stress_test(device: Arc<SoftwareDevice>) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔥 开始性能压力测试");
    println!("====================");

    let test_phases = vec![
        ("轻负载", 1_000_000.0, 30), // 1 MH/s, 30秒
        ("中负载", 2_500_000.0, 30), // 2.5 MH/s, 30秒
        ("重负载", 4_000_000.0, 30), // 4 MH/s, 30秒
        ("极限负载", 6_000_000.0, 20), // 6 MH/s, 20秒
    ];

    for (phase_name, target_hashrate, duration) in test_phases {
        println!("\n📊 {} 阶段 (目标: {:.1} MH/s, 时长: {}秒)",
            phase_name, target_hashrate / 1_000_000.0, duration);

        let start_time = Instant::now();
        let mut max_temp = 0.0;
        let mut max_power = 0.0;
        let mut min_hashrate = f64::MAX;
        let mut max_hashrate = 0.0;

        while start_time.elapsed().as_secs() < duration {
            let status = device.get_status().await?;

            max_temp = max_temp.max(status.temperature);
            max_power = max_power.max(status.power_consumption);
            min_hashrate = min_hashrate.min(status.hashrate);
            max_hashrate = max_hashrate.max(status.hashrate);

            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        println!("  结果:");
        println!("    - 算力范围: {:.2} - {:.2} MH/s",
            min_hashrate / 1_000_000.0, max_hashrate / 1_000_000.0);
        println!("    - 最高温度: {:.1}°C", max_temp);
        println!("    - 最大功耗: {:.1}W", max_power);

        // 评估性能
        let hashrate_stability = (max_hashrate - min_hashrate) / max_hashrate * 100.0;
        if hashrate_stability < 10.0 {
            println!("    - 算力稳定性: ✅ 优秀 ({:.1}%变化)", hashrate_stability);
        } else if hashrate_stability < 20.0 {
            println!("    - 算力稳定性: ⚠️  一般 ({:.1}%变化)", hashrate_stability);
        } else {
            println!("    - 算力稳定性: ❌ 较差 ({:.1}%变化)", hashrate_stability);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CGMiner CPU BTC Core - 性能监控演示");
    println!("====================================");

    // 创建测试设备
    println!("📱 创建性能测试设备...");
    let device = create_test_device().await?;
    println!("✅ 设备创建成功");

    // 实时性能监控
    let analyzer = real_time_monitoring(Arc::clone(&device), Duration::from_secs(30)).await?;

    // 显示监控报告
    println!("\n{}", analyzer.generate_report());

    // 性能压力测试
    stress_test(device).await?;

    println!("\n🎉 性能监控演示完成！");
    println!("\n💡 性能优化建议:");
    println!("  - 监控温度，避免过热影响性能");
    println!("  - 关注功耗效率，寻找最佳工作点");
    println!("  - 定期检查算力稳定性");
    println!("  - 根据系统负载调整挖矿强度");

    Ok(())
}
