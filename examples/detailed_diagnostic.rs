use cgminer_cpu_btc_core::SoftwareDevice;
use cgminer_core::{Work, DeviceInfo, DeviceConfig, MiningDevice, meets_target};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🔬 详细挖矿过程诊断工具");
    println!("========================================");

    // 创建设备
    let device_info = DeviceInfo::new(1, "CPU-Test".to_string(), "软件设备".to_string(), 0);
    let device_config = DeviceConfig::default();

    let mut device = SoftwareDevice::new(
        device_info,
        device_config,
        100_000.0, // 降低目标算力到100KH/s以便调试
        0.0,       // 无错误率
        100        // 较小的批次大小便于观察
    ).await?;

    // 初始化设备
    device.initialize(DeviceConfig::default()).await?;

    // 创建一个非常简单的工作，目标难度较低
    let mut target = [0xFFu8; 32]; // 很低的难度目标
    target[0] = 0x0F; // 只需要前4位为0

    let work = Arc::new(Work::new(
        "test_easy_job".to_string(),
        target,
        [0u8; 80], // 简单的区块头
        1.0
    ));

    println!("📋 创建的工作:");
    println!("  工作ID: {}", work.id);
    println!("  目标: {:02x}{:02x}{:02x}{:02x}...", target[0], target[1], target[2], target[3]);

    // 测试目标难度检查
    let test_hash = [0x05u8; 32]; // 应该满足目标
    let meets = meets_target(&test_hash, &target);
    println!("  测试哈希 0x05... 是否满足目标: {}", meets);

    let test_hash2 = [0x10u8; 32]; // 不应该满足目标
    let meets2 = meets_target(&test_hash2, &target);
    println!("  测试哈希 0x10... 是否满足目标: {}", meets2);

    // 提交工作
    println!("\n📤 提交工作到设备...");
    device.submit_work(work.clone()).await?;

    // 获取设备统计信息
    let stats_before = device.get_stats().await?;
    println!("📊 挖矿前统计: 总哈希={}, 接受={}",
             stats_before.total_hashes, stats_before.accepted_work);

    // 尝试获取结果
    println!("\n🔍 尝试获取挖矿结果...");
    for attempt in 1..=5 {
        println!("  第{}次尝试...", attempt);

        match device.get_result().await? {
            Some(result) => {
                println!("✅ 获取到结果!");
                println!("  工作ID: {}", result.work_id);
                println!("  设备ID: {}", result.device_id);
                println!("  nonce: 0x{:08x}", result.nonce);
                println!("  哈希: {:02x}{:02x}{:02x}{:02x}...",
                         result.hash[0], result.hash[1], result.hash[2], result.hash[3]);
                println!("  是否有效: {}", result.meets_target);
                break;
            }
            None => {
                println!("❌ 未获取到结果");

                // 检查设备统计
                let stats = device.get_stats().await?;
                println!("    当前统计: 总哈希={}, 接受={}",
                         stats.total_hashes, stats.accepted_work);
            }
        }

        // 短暂等待
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // 最终统计
    println!("\n📊 最终统计:");
    let final_stats = device.get_stats().await?;
    println!("  总哈希数: {}", final_stats.total_hashes);
    println!("  接受工作: {}", final_stats.accepted_work);
    println!("  拒绝工作: {}", final_stats.rejected_work);
    println!("  硬件错误: {}", final_stats.hardware_errors);
    println!("  当前算力: {:.2} H/s", final_stats.current_hashrate.hashes_per_second);

    println!("\n🎯 诊断完成！");

    Ok(())
}
