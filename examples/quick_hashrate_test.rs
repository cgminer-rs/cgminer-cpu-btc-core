use cgminer_cpu_btc_core::*;
use cgminer_core::{Work, MiningCore, CoreConfig, CoreFactory};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 快速算力测试");

    // 创建核心
    let factory = SoftwareCoreFactory::new();
    let mut custom_params = std::collections::HashMap::new();
    custom_params.insert("device_count".to_string(), serde_json::Value::Number(serde_json::Number::from(4)));

    let config = CoreConfig {
        name: "Quick Test".to_string(),
        enabled: true,
        custom_params,
        devices: vec![],
    };

    let mut core = factory.create_core(config).await?;

    println!("✅ 核心创建完成");

    // 启动核心
    core.start().await?;
    println!("✅ 核心启动完成");

    // 提交工作
    let work = Work::new(
        "test".to_string(),
        [0xff; 32], // 超级容易的目标
        [0u8; 80],
        1.0,
    );

    core.submit_work(work).await?;
    println!("✅ 工作提交完成");

    // 等待2秒
    sleep(Duration::from_secs(2)).await;

    // 检查统计信息
    match core.get_stats().await {
        Ok(stats) => {
            println!("📊 统计信息:");
            println!("   总算力: {:.2} MH/s", stats.total_hashrate / 1_000_000.0);
            println!("   设备数: {}", stats.device_count);
            println!("   活跃设备: {}", stats.active_devices);
            println!("   接受的工作: {}", stats.accepted_work);

            if stats.total_hashrate > 0.0 {
                println!("✅ 算力正常: {:.2} MH/s", stats.total_hashrate / 1_000_000.0);
            } else {
                println!("❌ 算力为0！");
            }
        }
        Err(e) => {
            println!("❌ 获取统计失败: {}", e);
        }
    }

    // 停止核心
    core.stop().await?;
    println!("✅ 测试完成");

    Ok(())
}
