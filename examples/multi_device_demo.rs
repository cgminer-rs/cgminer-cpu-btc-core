use cgminer_cpu_btc_core::SoftwareMiningCore;
use cgminer_core::{MiningCore, CoreConfig, Work};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("🚀 多设备CPU挖矿演示");

    // 创建挖矿核心
    let mut core = SoftwareMiningCore::new("MultiDeviceDemo".to_string());

    // 配置多个设备
    let mut custom_params = HashMap::new();
    custom_params.insert("device_count".to_string(), serde_json::Value::Number(4.into()));
    custom_params.insert("min_hashrate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(2_000_000_000.0).unwrap()));
    custom_params.insert("max_hashrate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(8_000_000_000.0).unwrap()));

    let config = CoreConfig {
        name: "MultiDeviceDemo".to_string(),
        enabled: true,
        devices: vec![],
        custom_params,
    };

    // 初始化核心
    core.initialize(config).await?;

    // 启动挖矿
    core.start().await?;

    // 模拟工作提交
    let header = [0u8; 80]; // 80字节区块头
    let target = [0x00, 0x00, 0x00, 0x1d, 0x00, 0xff, 0xff, 0x00,
                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let work = Work::new("test_job_1".to_string(), target, header, 1.0);

    core.submit_work(work).await?;

    // 运行30秒并监控统计信息
    for i in 0..6 {
        sleep(Duration::from_secs(5)).await;

        match core.get_stats().await {
            Ok(stats) => {
                info!("⏰ 第{}次统计 - 总算力: {:.2} MH/s, 接受: {}, 拒绝: {}",
                      i + 1,
                      stats.total_hashrate / 1_000_000.0,
                      stats.accepted_work,
                      stats.rejected_work);
            }
            Err(e) => error!("获取统计信息失败: {}", e),
        }

        // 收集结果
        match core.collect_results().await {
            Ok(results) => {
                if !results.is_empty() {
                    info!("📊 收集到 {} 个挖矿结果", results.len());
                }
            }
            Err(e) => error!("收集结果失败: {}", e),
        }
    }

    // 关闭挖矿
    core.stop().await?;
    info!("✅ 多设备演示完成");

    Ok(())
}
