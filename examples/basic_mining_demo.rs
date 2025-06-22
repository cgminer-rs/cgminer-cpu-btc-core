//! 基本挖矿演示
//!
//! 这个示例展示如何使用 cgminer-cpu-btc-core 进行基本的比特币挖矿操作
//! 包括设备创建、工作分配、挖矿执行等核心功能

use cgminer_cpu_btc_core::{
    SoftwareDevice,
};
use cgminer_core::{
    DeviceInfo, DeviceConfig, Work, MiningDevice,
};
use std::time::{Duration, SystemTime, Instant};
use tokio;
use sha2::{Sha256, Digest};
use num_cpus;

/// 创建测试用的设备信息
fn create_device_info() -> DeviceInfo {
    DeviceInfo::new(
        1,                                    // device_id
        "CPU Bitcoin Miner".to_string(),      // name
        "cpu".to_string(),                    // device_type
        0,                                    // chain_id
    )
}

/// 创建设备配置
fn create_device_config() -> DeviceConfig {
    DeviceConfig {
        chain_id: 0,
        enabled: true,
        frequency: 4000,      // MHz - 高频率，最大性能
        voltage: 1350,        // mV - 高电压支持高频率
        auto_tune: true,      // 启用自动调优
        chip_count: num_cpus::get() as u32, // 使用所有CPU核心
        temperature_limit: 90.0, // °C - 更高温度限制
        fan_speed: Some(100), // % - 最大风扇速度
    }
}

/// 创建真实的比特币工作数据
fn create_bitcoin_work() -> Work {
    // 创建一个80字节的区块头
    let mut header = [0u8; 80];

    // 版本 (4字节) - Bitcoin版本1
    header[0..4].copy_from_slice(&1u32.to_le_bytes());

    // 前一个区块哈希 (32字节) - 创世区块的哈希
    let prev_hash = hex::decode("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f")
        .unwrap_or_else(|_| vec![0u8; 32]);
    if prev_hash.len() >= 32 {
        header[4..36].copy_from_slice(&prev_hash[0..32]);
    }

    // Merkle根 (32字节) - 简化的Merkle根
    let merkle_root_vec = hex::decode("4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b")
        .unwrap_or_else(|_| vec![1u8; 32]);
    let mut merkle_root = [0u8; 32];
    if merkle_root_vec.len() >= 32 {
        merkle_root.copy_from_slice(&merkle_root_vec[0..32]);
        header[36..68].copy_from_slice(&merkle_root);
    }

    // 时间戳 (4字节) - 当前时间
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    header[68..72].copy_from_slice(&timestamp.to_le_bytes());

    // 难度目标 (4字节) - 简化的难度
    header[72..76].copy_from_slice(&0x1d00ffffu32.to_le_bytes());

    // Nonce (4字节) - 初始为0，挖矿时会修改
    header[76..80].copy_from_slice(&0u32.to_le_bytes());

    // 创建目标难度 (32字节) - 相对简单的目标，更容易找到解
    let mut target = [0x00u8; 32];
    // 设置更高的目标值，使难度更低，更容易找到解
    target[24..32].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00]);

    Work::new(
        "bitcoin_work_001".to_string(), // job_id
        target,                         // target
        header,                         // header
        1.0,                           // difficulty
    )
}

/// 演示基本挖矿流程
async fn demo_basic_mining() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 开始基本挖矿演示");
    println!("==================");

    // 1. 创建设备
    println!("\n📱 步骤1: 创建挖矿设备");
    let device_info = create_device_info();
    let config = create_device_config();

    let device = SoftwareDevice::new(
        device_info.clone(),
        config.clone(),
        f64::MAX,  // target_hashrate: 无限制，发挥最大性能
        0.001,     // error_rate: 0.1% 更低错误率
        50000,     // batch_size: 更大批次提高效率
    ).await?;

    println!("  ✅ 设备创建成功");
    println!("  📊 设备名称: {}", device_info.name);
    println!("  🆔 设备ID: {}", device.device_id());
    println!("  🎯 目标算力: 无限制 (最大性能)");

    // 2. 创建工作
    println!("\n⚒️  步骤2: 创建挖矿工作");
    let work = create_bitcoin_work();
    println!("  ✅ 工作创建成功");
    println!("  🆔 工作ID: {}", work.work_id);
    println!("  📏 区块头大小: {} 字节", work.header.len());
    println!("  🎯 难度: {:.2}", work.difficulty);

    // 3. 开始挖矿
    println!("\n⛏️  步骤3: 开始挖矿");
    let start_time = Instant::now();

    // 持续挖矿直到找到有效解
    let mut attempts = 0u64;
    let mut found_solution = false;
    let mut nonce = 0u32;

    println!("  🔄 开始寻找有效nonce（持续运算直到找到解）...");
    println!("  💡 提示: 这可能需要一些时间，请耐心等待");

    loop {
        attempts += 1;

        // 修改区块头中的nonce
        let mut test_header = work.header;
        test_header[76..80].copy_from_slice(&nonce.to_le_bytes());

        // 计算双重SHA256哈希
        let hash = calculate_double_sha256(&test_header);

        // 检查是否满足难度要求
        if is_valid_hash(&hash, &work.target) {
            found_solution = true;
            println!("  🎉 找到有效解!");
            println!("  🔢 Nonce: {}", nonce);
            println!("  🔐 哈希: {}", hex::encode(&hash));
            println!("  🎯 目标: {}", hex::encode(&work.target));
            break;
        }

        // 每100000次尝试显示进度和当前算力
        if attempts % 100000 == 0 {
            let elapsed = start_time.elapsed();
            let hashrate = attempts as f64 / elapsed.as_secs_f64();
            println!("  📊 已尝试: {} 次 | 算力: {:.2} MH/s | 用时: {:.1}秒",
                attempts, hashrate / 1_000_000.0, elapsed.as_secs_f64());
        }

        // 防止nonce溢出，如果达到最大值就重新开始
        if nonce == u32::MAX {
            println!("  🔄 Nonce达到最大值，重新开始...");
            nonce = 0;
        } else {
            nonce += 1;
        }
    }

    let mining_time = start_time.elapsed();

    // 4. 显示结果
    println!("\n📊 步骤4: 挖矿结果");
    let mining_time = start_time.elapsed();
    println!("  ⏱️  挖矿时间: {:.2}秒", mining_time.as_secs_f64());
    println!("  🔢 总尝试次数: {}", attempts);

    if found_solution {
        println!("  ✅ 状态: 成功找到有效解!");
        let hashrate = attempts as f64 / mining_time.as_secs_f64();
        println!("  ⚡ 平均算力: {:.2} MH/s", hashrate / 1_000_000.0);
        println!("  🏆 挖矿成功! 这就是真实的比特币挖矿过程");
    } else {
        println!("  ❌ 状态: 未找到有效解（不应该发生）");
    }

    Ok(())
}

/// 计算双重SHA256哈希
fn calculate_double_sha256(data: &[u8]) -> Vec<u8> {
    // 第一次SHA256
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash1 = hasher.finalize();

    // 第二次SHA256
    let mut hasher = Sha256::new();
    hasher.update(&hash1);
    let hash2 = hasher.finalize();

    hash2.to_vec()
}

/// 检查哈希是否满足难度要求
fn is_valid_hash(hash: &[u8], target: &[u8]) -> bool {
    // 比较哈希值是否小于目标值
    for i in 0..32 {
        if hash[i] < target[i] {
            return true;
        } else if hash[i] > target[i] {
            return false;
        }
    }
    false
}

/// 演示设备状态监控
async fn demo_device_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 设备状态监控演示");
    println!("==================");

    let device_info = create_device_info();
    let config = create_device_config();

    let device = SoftwareDevice::new(
        device_info,
        config,
        f64::MAX,  // 无算力限制
        0.001,     // 0.1% error rate
        50000,     // 大批次处理
    ).await?;

    // 监控设备状态
    for i in 1..=5 {
        println!("\n📊 监控周期 {}/5", i);

        // 获取设备状态、信息和统计
        let status = device.get_status().await?;
        let info = device.get_info().await?;
        let stats = device.get_stats().await?;

        println!("  🔋 状态: {:?}", status);
        println!("  🌡️  温度: {:.1}°C", info.temperature.unwrap_or(45.0));
        println!("  ⚡ 算力: {:.2} MH/s", stats.current_hashrate.hashes_per_second / 1_000_000.0);
        println!("  🔌 功耗: {:.1}W", stats.power_consumption.unwrap_or(100.0));

        let total_shares = stats.accepted_work + stats.rejected_work;
        if total_shares > 0 {
            println!("  🎯 接受率: {:.1}%", stats.accepted_work as f64 / total_shares as f64 * 100.0);
        } else {
            println!("  🎯 接受率: N/A");
        }

        // 等待1秒
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("\n✅ 监控演示完成");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CGMiner CPU BTC Core - 基本挖矿演示");
    println!("=====================================");
    println!("这个演示将展示如何使用CPU进行比特币挖矿的基本流程");

    // 运行基本挖矿演示
    demo_basic_mining().await?;

    // 运行设备监控演示
    demo_device_monitoring().await?;

    println!("\n🎉 演示完成！");
    println!("\n💡 更多示例:");
    println!("  - 多设备挖矿: cargo run --example multi_device_demo");
    println!("  - 性能监控: cargo run --example performance_monitoring_demo");
    println!("  - 温度管理: cargo run --example temperature_demo");
    println!("  - CPU亲和性: cargo run --example cpu_affinity_demo");
    println!("  - 真实挖矿模拟: cargo run --example real_mining_simulation");

    Ok(())
}
