//! 基准测试演示
//!
//! 演示如何进行CPU挖矿性能基准测试

use cgminer_cpu_btc_core::{SoftwareDevice, DeviceConfig};
use cgminer_core::{MiningDevice, Work, DeviceInfo};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("⚡ CPU挖矿基准测试");
    println!("🔧 准备测试环境...");

    // 预热阶段
    println!("\n🔥 预热阶段 (30秒)");
    warmup_benchmark().await?;

    // 单线程基准测试
    println!("\n📊 单线程基准测试");
    single_thread_benchmark().await?;

    // 多线程基准测试
    println!("\n📊 多线程基准测试");
    multi_thread_benchmark().await?;

    // 负载测试
    println!("\n🔋 负载测试");
    load_test().await?;

    println!("\n✨ 基准测试完成");
    Ok(())
}

/// 预热基准测试
async fn warmup_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    let device_info = DeviceInfo {
        id: 0,
        name: "Warmup-Device".to_string(),
        device_type: "CPU".to_string(),
        vendor: "Software".to_string(),
        temperature: 0.0,
        fan_speed: 0,
        power_usage: 0.0,
    };

    let config = DeviceConfig {
        name: "Warmup-Device".to_string(),
        enabled: true,
        threads: 1,
    };

    let device = Arc::new(SoftwareDevice::new(device_info, config, 100.0, 150.0, 0).await?);
    device.start().await?;

    let work = Work::new(
        "warmup_job".to_string(),
        [0xFFu8; 32],
        [0u8; 80],
        1.0,
    );

    device.submit_work(work).await?;

    // 预热30秒
    for i in 1..=6 {
        sleep(Duration::from_secs(5)).await;
        let stats = device.get_stats().await;
        println!("   预热进度: {}0% - {:.2} MH/s", i, stats.total_hashrate / 1_000_000.0);
    }

    device.stop().await?;
    println!("✅ 预热完成");
    Ok(())
}

/// 单线程基准测试
async fn single_thread_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    let device_info = DeviceInfo {
        id: 1,
        name: "Single-Thread-Device".to_string(),
        device_type: "CPU".to_string(),
        vendor: "Software".to_string(),
        temperature: 0.0,
        fan_speed: 0,
        power_usage: 0.0,
    };

    let config = DeviceConfig {
        name: "Single-Thread-Device".to_string(),
        enabled: true,
        threads: 1,
    };

    let device = Arc::new(SoftwareDevice::new(device_info, config, 100.0, 150.0, 0).await?);
    device.start().await?;

    let work = Work::new(
        "single_thread_job".to_string(),
        [0xFFu8; 32],
        [0u8; 80],
        1.0,
    );

    device.submit_work(work).await?;

    let start_time = Instant::now();
    let initial_stats = device.get_stats().await;

    // 运行60秒
    sleep(Duration::from_secs(60)).await;

    let final_stats = device.get_stats().await;
    let elapsed = start_time.elapsed();

    device.stop().await?;

    // 计算结果
    let hashrate_diff = final_stats.total_hashrate - initial_stats.total_hashrate;
    let time_diff = elapsed.as_secs_f64();
    let avg_hashrate = hashrate_diff / time_diff;

    println!("📈 单线程基准测试结果:");
    println!("   运行时间: {:.1}秒", time_diff);
    println!("   平均算力: {:.2} MH/s", avg_hashrate / 1_000_000.0);
    println!("   总计算量: {:.0} 哈希", hashrate_diff);
    println!("   接受工作: {}", final_stats.accepted_work - initial_stats.accepted_work);

    Ok(())
}

/// 多线程基准测试
async fn multi_thread_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    let cpu_count = num_cpus::get();
    let thread_count = std::cmp::min(cpu_count, 8);

    println!("🔄 使用 {} 个线程进行测试", thread_count);

    let device_info = DeviceInfo {
        id: 2,
        name: "Multi-Thread-Device".to_string(),
        device_type: "CPU".to_string(),
        vendor: "Software".to_string(),
        temperature: 0.0,
        fan_speed: 0,
        power_usage: 0.0,
    };

    let config = DeviceConfig {
        name: "Multi-Thread-Device".to_string(),
        enabled: true,
        threads: thread_count,
    };

    let device = Arc::new(SoftwareDevice::new(device_info, config, 100.0, 150.0, 0).await?);
    device.start().await?;

    let work = Work::new(
        "multi_thread_job".to_string(),
        [0xFFu8; 32],
        [0u8; 80],
        1.0,
    );

    device.submit_work(work).await?;

    let start_time = Instant::now();
    let initial_stats = device.get_stats().await;

    // 运行60秒
    sleep(Duration::from_secs(60)).await;

    let final_stats = device.get_stats().await;
    let elapsed = start_time.elapsed();

    device.stop().await?;

    // 计算结果
    let hashrate_diff = final_stats.total_hashrate - initial_stats.total_hashrate;
    let time_diff = elapsed.as_secs_f64();
    let avg_hashrate = hashrate_diff / time_diff;

    println!("📈 多线程基准测试结果:");
    println!("   线程数量: {}", thread_count);
    println!("   运行时间: {:.1}秒", time_diff);
    println!("   平均算力: {:.2} MH/s", avg_hashrate / 1_000_000.0);
    println!("   每线程算力: {:.2} MH/s", avg_hashrate / thread_count as f64 / 1_000_000.0);
    println!("   总计算量: {:.0} 哈希", hashrate_diff);
    println!("   接受工作: {}", final_stats.accepted_work - initial_stats.accepted_work);

    Ok(())
}

/// 负载测试
async fn load_test() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔋 进行5分钟负载测试...");

    let device_info = DeviceInfo {
        id: 3,
        name: "Load-Test-Device".to_string(),
        device_type: "CPU".to_string(),
        vendor: "Software".to_string(),
        temperature: 0.0,
        fan_speed: 0,
        power_usage: 0.0,
    };

    let config = DeviceConfig {
        name: "Load-Test-Device".to_string(),
        enabled: true,
        threads: num_cpus::get(),
    };

    let device = Arc::new(SoftwareDevice::new(device_info, config, 100.0, 150.0, 0).await?);
    device.start().await?;

    let work = Work::new(
        "load_test_job".to_string(),
        [0xFFu8; 32],
        [0u8; 80],
        1.0,
    );

    device.submit_work(work).await?;

    // 5分钟负载测试，每30秒报告一次
    for minute in 1..=5 {
        sleep(Duration::from_secs(30)).await;
        let stats = device.get_stats().await;

        println!("   第{}分钟: {:.2} MH/s, 接受: {}, 拒绝: {}, 错误: {}",
                 minute, stats.total_hashrate / 1_000_000.0,
                 stats.accepted_work, stats.rejected_work, stats.hardware_errors);
    }

    let final_stats = device.get_stats().await;
    device.stop().await?;

    println!("📈 负载测试结果:");
    println!("   最终算力: {:.2} MH/s", final_stats.total_hashrate / 1_000_000.0);
    println!("   总接受工作: {}", final_stats.accepted_work);
    println!("   总拒绝工作: {}", final_stats.rejected_work);
    println!("   硬件错误: {}", final_stats.hardware_errors);
    println!("   稳定性: {}%",
             if final_stats.accepted_work + final_stats.rejected_work > 0 {
                 (final_stats.accepted_work * 100) / (final_stats.accepted_work + final_stats.rejected_work)
             } else { 0 });

    Ok(())
}
