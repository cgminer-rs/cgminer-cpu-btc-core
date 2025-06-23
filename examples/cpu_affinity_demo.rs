//! CPU亲和性演示
//!
//! 演示如何在多个CPU核心上运行挖矿设备

use cgminer_cpu_btc_core::{SoftwareDevice, DeviceConfig};
use cgminer_core::{MiningDevice, Work, DeviceInfo};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🔄 CPU亲和性演示");

    let cpu_count = num_cpus::get();
    println!("💻 检测到 {} 个CPU核心", cpu_count);

    let device_count = std::cmp::min(cpu_count, 4); // 最多使用4个设备
    let mut devices = Vec::new();

    // 为每个CPU核心创建设备
    for i in 0..device_count {
        println!("🔧 创建设备 {} (绑定到CPU核心 {})", i, i);

        let device_info = DeviceInfo {
            id: i as u32,
            name: format!("CPU-Device-{}", i),
            device_type: "CPU".to_string(),
            vendor: "Software".to_string(),
            temperature: 0.0,
            fan_speed: 0,
            power_usage: 0.0,
        };

        let config = DeviceConfig {
            name: format!("CPU-Device-{}", i),
            enabled: true,
            threads: 1, // 每个设备使用一个线程
        };

        let device = Arc::new(SoftwareDevice::new(device_info, config, 100.0, 150.0, i as u32).await?);
        device.start().await?;
        println!("🟢 设备 {} (CPU核心 {}) 已启动", device.get_info().name, i);

        // 创建工作
        let work = Work::new(
            format!("job_{}", i),
            [0xFFu8; 32],
            [0u8; 80],
            1.0,
        );

        // 提交工作
        device.submit_work(work.clone()).await?;
        println!("📤 向设备 {} 提交工作", i);

        devices.push(device);
    }

    println!("✅ 所有设备已启动并开始挖矿");

    // 运行一段时间并监控
    for round in 1..=6 {
        sleep(Duration::from_secs(10)).await;
        println!("\n📊 第{}轮监控 ({}s):", round, round * 10);

        for (i, device) in devices.iter().enumerate() {
            let stats = device.get_stats().await;
            println!("   设备{}: {:.2} MH/s, 接受:{}, 拒绝:{}",
                     i, stats.total_hashrate / 1_000_000.0,
                     stats.accepted_work, stats.rejected_work);
        }

        // 每30秒显示一次CPU亲和性信息
        if round % 3 == 0 {
            println!("🔄 CPU亲和性状态:");
            for i in 0..device_count {
                println!("   设备{} → CPU核心{}", i, i);
            }
        }
    }

    // 停止所有设备
    println!("\n🔴 停止所有设备...");
    for (i, device) in devices.iter().enumerate() {
        device.stop().await?;
        println!("🔴 设备 {} 已停止", i);
    }

    println!("✨ CPU亲和性演示完成");
    Ok(())
}
