use cgminer_cpu_btc_core::SoftwareMiningCore;
use cgminer_core::{MiningCore, CoreConfig, Work};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("🌡️  温度监控演示启动");

    // 创建挖矿核心
    let mut core = SoftwareMiningCore::new("TemperatureDemo".to_string());

    // 配置温度敏感的设备
    let mut custom_params = HashMap::new();
    custom_params.insert("device_count".to_string(), serde_json::Value::Number(4.into()));
    custom_params.insert("min_hashrate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(1_000_000_000.0).unwrap()));
    custom_params.insert("max_hashrate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(6_000_000_000.0).unwrap()));
    custom_params.insert("temperature_limit".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(75.0).unwrap()));

    let config = CoreConfig {
        name: "TemperatureDemo".to_string(),
        enabled: true,
        devices: vec![],
        custom_params,
    };

    // 显示温度监控能力
    let capabilities = core.get_capabilities();
    info!("🔍 温度监控能力:");
    info!("  - 支持温度监控: {}", capabilities.temperature_capabilities.supports_monitoring);
    info!("  - 支持温度控制: {}", capabilities.temperature_capabilities.supports_control);
    info!("  - 支持阈值报警: {}", capabilities.temperature_capabilities.supports_threshold_alerts);
    if let Some(precision) = capabilities.temperature_capabilities.monitoring_precision {
        info!("  - 监控精度: {:.1}°C", precision);
    }

    // 初始化和启动
    core.initialize(config).await?;
    core.start().await?;

    // 创建测试工作
    let header = [0u8; 80]; // 80字节区块头
    let target = [0x00, 0x00, 0x00, 0x1d, 0x00, 0xff, 0xff, 0x00,
                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let work = Work::new("temp_test_job".to_string(), target, header, 1.0);

    core.submit_work(work).await?;

    // 温度监控循环
    info!("🔥 开始温度监控（持续45秒）");

    for cycle in 0..9 {
        sleep(Duration::from_secs(5)).await;

        // 模拟温度检查（实际环境中应该读取真实的传感器数据）
        info!("🌡️  第{}轮温度检查:", cycle + 1);

        for device_id in 0..4 {
            // 模拟温度读取（实际实现中会从系统获取）
            let simulated_temp = 45.0 + (cycle as f64 * 2.0) + (rand::random::<f64>() * 10.0);

            // 温度状态判断
            let temp_status = if simulated_temp > 80.0 {
                "🔴 危险"
            } else if simulated_temp > 70.0 {
                "🟡 警告"
            } else if simulated_temp > 60.0 {
                "🟠 注意"
            } else {
                "🟢 正常"
            };

            info!("  设备 {}: {:.1}°C {}",
                  device_id,
                  simulated_temp,
                  temp_status);

            // 温度警告
            if simulated_temp > 75.0 {
                warn!("⚠️  设备 {} 温度过高: {:.1}°C，建议降低负载",
                      device_id,
                      simulated_temp);
            }
        }

        // 获取核心统计信息
        match core.get_stats().await {
            Ok(stats) => {
                info!("📊 当前算力: {:.2} MH/s | 接受: {} | 拒绝: {}",
                      stats.total_hashrate / 1_000_000.0,
                      stats.accepted_work,
                      stats.rejected_work);
            }
            Err(e) => error!("获取统计信息失败: {}", e),
        }

        // 模拟温度保护机制
        if cycle > 5 {
            warn!("🛡️  模拟温度保护触发，建议在实际使用中实现自动降频");
        }
    }

    // CPU核心模式的温度监控限制说明
    info!("ℹ️  CPU核心模式温度监控说明:");
    info!("  - ✅ 可以监控系统温度传感器");
    info!("  - ❌ 无法直接控制CPU温度");
    info!("  - ✅ 可以通过调整工作负载间接影响温度");
    info!("  - ✅ 可以监控温度阈值并发出警报");
    info!("  - ❌ 无法控制风扇转速");
    info!("  - ❌ 无法调节CPU电压");

    // 关闭挖矿
    core.stop().await?;
    info!("✅ 温度监控演示完成");

    Ok(())
}
