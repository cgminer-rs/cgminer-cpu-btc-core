//! 性能监控演示
//!
//! 演示如何使用性能监控系统监控CPU挖矿性能

use cgminer_cpu_btc_core::{SoftwareDevice, DeviceConfig};
use cgminer_core::{MiningDevice, Work, DeviceInfo};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("📊 性能监控演示");

    // 创建设备信息
    let device_info = DeviceInfo {
        id: 0,
        name: "Monitor-Test-Device".to_string(),
        device_type: "CPU".to_string(),
        vendor: "Software".to_string(),
        temperature: 0.0,
        fan_speed: 0,
        power_usage: 0.0,
    };

    // 创建设备配置
    let config = DeviceConfig {
        name: "Monitor-Test-Device".to_string(),
        enabled: true,
        threads: 4,
    };

    // 创建设备
    let device = Arc::new(SoftwareDevice::new(device_info, config, 100.0, 150.0, 0).await?);

    // 启动设备
    device.start().await?;
    println!("✅ 设备已启动，开始性能监控");

    // 创建工作数据
    let work = Work::new(
        "job_1".to_string(),      // 工作ID
        [0xFFu8; 32],             // 目标
        [0u8; 80],                // 区块头
        1.0,                      // 难度
    );

    // 提交工作
    device.submit_work(work).await?;

    // 监控循环
    for i in 1..=12 {
        sleep(Duration::from_secs(10)).await;

        let stats = device.get_stats().await;
        let info = device.get_info();

        println!("⏱️  第{}轮监控 ({}s):", i, i * 10);
        println!("   📈 总算力: {:.2} MH/s", stats.total_hashrate / 1_000_000.0);
        println!("   ✅ 接受工作: {}", stats.accepted_work);
        println!("   ❌ 拒绝工作: {}", stats.rejected_work);
        println!("   🔧 硬件错误: {}", stats.hardware_errors);

        // 模拟性能指标变化
        if i % 3 == 0 {
            println!("   🔥 检测到性能波动");
        }

        if i % 4 == 0 {
            println!("   💾 内存使用优化建议: 考虑启用内存池");
        }

        println!();
    }

    // 停止设备
    device.stop().await?;
    println!("🔴 设备已停止");

    println!("✨ 性能监控演示完成");
    Ok(())
}
