//! 真实挖矿模拟
//!
//! 模拟真实的比特币挖矿环境和工作流程

use cgminer_cpu_btc_core::{SoftwareDevice, DeviceConfig};
use cgminer_core::{MiningDevice, Work, DeviceInfo};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use rand::Rng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("⛏️  真实挖矿模拟启动");
    println!("🔧 配置挖矿环境...");

    // 创建设备信息
    let device_info = DeviceInfo {
        id: 0,
        name: "Mining-Simulator".to_string(),
        device_type: "CPU".to_string(),
        vendor: "Software".to_string(),
        temperature: 45.0,
        fan_speed: 1500,
        power_usage: 120.0,
    };

    // 创建设备配置
    let config = DeviceConfig {
        name: "Mining-Simulator".to_string(),
        enabled: true,
        threads: num_cpus::get(),
    };

    // 创建设备
    let device = Arc::new(SoftwareDevice::new(device_info, config, 100.0, 150.0, 0).await?);

    // 启动设备
    device.start().await?;
    println!("✅ 挖矿设备已启动");

    // 模拟挖矿循环
    let mut work_id = 1u64;
    let mut interval_timer = interval(Duration::from_secs(30));
    let mut stats_timer = interval(Duration::from_secs(60));

    tokio::select! {
        _ = mining_loop(&device, &mut work_id, &mut interval_timer) => {},
        _ = stats_loop(&device, &mut stats_timer) => {},
        _ = tokio::time::sleep(Duration::from_secs(300)) => {
            println!("⏰ 5分钟挖矿模拟完成");
        }
    }

    // 停止设备
    device.stop().await?;
    println!("🔴 挖矿设备已停止");

    // 显示最终统计
    let final_stats = device.get_stats().await;
    println!("\n📊 最终挖矿统计:");
    println!("   💎 总算力: {:.2} MH/s", final_stats.total_hashrate / 1_000_000.0);
    println!("   ✅ 接受工作: {}", final_stats.accepted_work);
    println!("   ❌ 拒绝工作: {}", final_stats.rejected_work);
    println!("   🔧 硬件错误: {}", final_stats.hardware_errors);

    let total_work = final_stats.accepted_work + final_stats.rejected_work;
    if total_work > 0 {
        let success_rate = (final_stats.accepted_work as f64 / total_work as f64) * 100.0;
        println!("   📈 成功率: {:.1}%", success_rate);
    }

    // 计算预估收益
    let estimated_earnings = calculate_estimated_earnings(final_stats.total_hashrate, 50_000_000_000_000u64);
    println!("   💰 预估日收益: ${:.4} USD", estimated_earnings);

    println!("✨ 真实挖矿模拟完成");
    Ok(())
}

/// 挖矿循环
async fn mining_loop(
    device: &Arc<SoftwareDevice>,
    work_id: &mut u64,
    interval_timer: &mut tokio::time::Interval
) {
    loop {
        interval_timer.tick().await;

        // 生成新的工作
        let work = generate_mining_work(*work_id);
        *work_id += 1;

        // 提交工作
        if let Err(e) = device.submit_work(work).await {
            eprintln!("❌ 提交工作失败: {}", e);
        } else {
            println!("📤 提交新工作 #{}", work_id - 1);
        }

        // 模拟网络延迟
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// 统计循环
async fn stats_loop(device: &Arc<SoftwareDevice>, stats_timer: &mut tokio::time::Interval) {
    loop {
        stats_timer.tick().await;

        let stats = device.get_stats().await;
        let info = device.get_info();

        println!("\n📊 挖矿状态更新:");
        println!("   ⚡ 当前算力: {:.2} MH/s", stats.total_hashrate / 1_000_000.0);
        println!("   🌡️  设备温度: {:.1}°C", info.temperature);
        println!("   💨 风扇转速: {} RPM", info.fan_speed);
        println!("   ⚡ 功耗: {:.1}W", info.power_usage);
        println!("   ✅ 接受: {} | ❌ 拒绝: {} | 🔧 错误: {}",
                 stats.accepted_work, stats.rejected_work, stats.hardware_errors);

        // 模拟温度和功耗变化
        simulate_hardware_changes(&info);
    }
}

/// 生成挖矿工作
fn generate_mining_work(work_id: u64) -> Work {
    let mut rng = rand::thread_rng();

    // 生成随机区块头
    let mut block_header = [0u8; 80];
    rng.fill(&mut block_header);

    // 设置合理的目标难度
    let mut target = [0xFFu8; 32];
    target[0] = 0x00;
    target[1] = 0x00;
    target[2] = 0x00;
    target[3] = 0xFF;

    Work::new(
        format!("job_{}", work_id),
        target,
        block_header,
        1.0,
    )
}

/// 模拟硬件变化
fn simulate_hardware_changes(info: &DeviceInfo) {
    let mut rng = rand::thread_rng();

    // 模拟温度波动 (±5°C)
    let temp_change = rng.gen_range(-5.0..=5.0);
    let new_temp = (info.temperature + temp_change).clamp(40.0, 85.0);

    // 模拟功耗变化 (±20W)
    let power_change = rng.gen_range(-20.0..=20.0);
    let new_power = (info.power_usage + power_change).clamp(80.0, 200.0);

    // 模拟风扇转速调整
    let fan_adjustment = if new_temp > 70.0 { 200 } else if new_temp < 50.0 { -100 } else { 0 };
    let new_fan_speed = (info.fan_speed + fan_adjustment).clamp(1000, 3000);

    if rng.gen_bool(0.1) { // 10%概率显示硬件状态变化
        println!("   🔄 硬件状态: 温度 {:.1}°C → {:.1}°C, 功耗 {:.1}W → {:.1}W, 风扇 {} → {} RPM",
                 info.temperature, new_temp, info.power_usage, new_power, info.fan_speed, new_fan_speed);
    }
}

/// 计算预估收益
fn calculate_estimated_earnings(hashrate: f64, _difficulty: u64) -> f64 {
    // 简化的收益计算（实际计算会更复杂）
    let btc_price = 45000.0; // 假设BTC价格
    let network_hashrate = 400_000_000_000_000_000.0; // 400 EH/s
    let block_reward = 6.25; // BTC
    let blocks_per_day = 144.0; // 平均每天144个区块

    let daily_btc = (hashrate / network_hashrate) * block_reward * blocks_per_day;
    daily_btc * btc_price
}
