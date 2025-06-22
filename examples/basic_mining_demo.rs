//! 基本挖矿演示 - CGMiner风格
//!
//! 这个示例展示如何使用 cgminer-cpu-btc-core 进行比特币挖矿
//! 包括CGMiner风格的实时算力显示和立即上报机制

use cgminer_cpu_btc_core::SoftwareCoreFactory;
use cgminer_core::{CoreConfig, Work, CoreFactory};
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

/// CGMiner风格算力跟踪器
struct HashrateTracker {
    samples: Vec<f64>,
}

impl HashrateTracker {
    fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    fn add_sample(&mut self, hashrate: f64) {
        self.samples.push(hashrate);
        // 保留最近15分钟的样本（假设每5秒一个样本）
        if self.samples.len() > 180 {
            self.samples.remove(0);
        }
    }

    fn format_cgminer_output(&self, current_mhs: f64, device_count: u32, accepted: u64) -> String {
        let avg_1m = self.get_average_hashrate(12); // 最近1分钟
        let avg_5m = self.get_average_hashrate(60); // 最近5分钟
        let avg_15m = self.get_average_hashrate(180); // 最近15分钟

        format!("{:.1}/{:.1}/{:.1}/{:.1}Mh/s A:{} R:0 HW:0 [{}DEV]",
                current_mhs, avg_1m, avg_5m, avg_15m, accepted, device_count)
    }

    fn get_average_hashrate(&self, samples: usize) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }

        let start_idx = if self.samples.len() > samples {
            self.samples.len() - samples
        } else {
            0
        };

        let sum: f64 = self.samples[start_idx..].iter().sum();
        let count = self.samples.len() - start_idx;

        if count > 0 {
            (sum / count as f64) / 1_000_000.0 // 转换为MH/s
        } else {
            0.0
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 基本挖矿演示 - CGMiner风格");

    // 获取CPU核心数
    let cpu_cores = num_cpus::get();
    info!("💻 检测到 {} 个CPU核心", cpu_cores);

    // 创建核心工厂
    let factory = SoftwareCoreFactory::new();

    // 创建配置 - 使用CPU核心数
    let mut custom_params = std::collections::HashMap::new();
    custom_params.insert("device_count".to_string(), serde_json::Value::Number(serde_json::Number::from(cpu_cores)));
    custom_params.insert("max_hashrate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(2_000_000.0).unwrap()));
    custom_params.insert("error_rate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()));
    custom_params.insert("batch_size".to_string(), serde_json::Value::Number(serde_json::Number::from(100000)));

    let config = CoreConfig {
        name: "Basic Mining Demo".to_string(),
        enabled: true,
        custom_params,
        devices: vec![],
    };

    info!("🏗️  创建CPU核心...");
    let creation_start = Instant::now();

    // 创建核心
    let mut core = factory.create_core(config.clone()).await?;

    let creation_time = creation_start.elapsed();
    info!("✅ 核心创建完成 ({:.2}s)", creation_time.as_secs_f64());

    // 获取实际设备数量
    let device_count = core.device_count().await?;
    info!("📊 实际设备数量: {}", device_count);

    // 启动核心
    info!("🚀 启动核心...");
    core.start().await?;
    info!("✅ 核心启动完成");

    // 创建测试工作
    let work = Work::new(
        "test_work_1".to_string(), // job_id
        [0x00, 0x00, 0x0f, 0xff].repeat(8).try_into().unwrap(), // target
        [0u8; 80], // header
        1.0, // difficulty
    );

    info!("📋 提交工作到所有设备...");
    core.submit_work(work).await?;

    // CGMiner风格算力跟踪
    let mut hashrate_tracker = HashrateTracker::new();
    let mut last_log_time = Instant::now();
    let mut total_accepted = 0u64;

    info!("⛏️  开始CGMiner风格挖矿...");
    info!("📊 CGMiner风格输出格式: [当前/1分钟/5分钟/15分钟]Mh/s A:[接受] R:[拒绝] HW:[硬件错误] [设备数]");

    // 挖矿循环 - 简单的30轮演示
    for _round in 1..=30 {
        // 收集结果 - 核心负责挖矿和上报，示例负责统计
        let results = core.collect_results().await?;

        // 统计新找到的解
        if !results.is_empty() {
            total_accepted += results.len() as u64;
            info!("💎 本轮找到 {} 个解，总计: {}", results.len(), total_accepted);
        }

        // 每5秒输出CGMiner风格的算力日志
        let now = Instant::now();
        if now.duration_since(last_log_time).as_secs() >= 5 {
            match core.get_stats().await {
                Ok(stats) => {
                    let current_hashrate_mhs = stats.total_hashrate / 1_000_000.0;

                    // 添加当前算力样本
                    hashrate_tracker.add_sample(stats.total_hashrate);

                    // 输出CGMiner风格的日志
                    println!("{}", hashrate_tracker.format_cgminer_output(current_hashrate_mhs, device_count, total_accepted));
                }
                Err(e) => {
                    warn!("获取统计信息失败: {}", e);
                }
            }
            last_log_time = now;
        }

        // 短暂等待
        sleep(Duration::from_millis(500)).await;
    }

    // 停止核心
    info!("🛑 停止核心...");
    core.stop().await?;
    info!("✅ 演示完成！总共找到 {} 个解", total_accepted);

    Ok(())
}
