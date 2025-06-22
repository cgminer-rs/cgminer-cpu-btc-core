//! CGMiner CPU BTC Core 基准测试演示
//!
//! 这个示例展示如何使用 cgminer-cpu-btc-core 进行基本的性能测试
//! 包括SHA256哈希计算、设备创建等核心功能

use cgminer_cpu_btc_core::{
    SoftwareDevice,
    SoftwareCoreFactory,
};
use cgminer_core::{
    DeviceInfo, DeviceStatus, DeviceConfig, Work, MiningResult, CoreType, CoreFactory,
};
use std::time::{Duration, SystemTime, Instant};
use std::sync::Arc;
use tokio;
use sha2::{Sha256, Digest};

/// 创建测试用的设备信息
fn create_test_device_info() -> DeviceInfo {
    DeviceInfo::new(
        1,                                    // device_id
        "Software CPU Miner".to_string(),     // name
        CoreType::Cpu,                        // core_type
        "v1.0.0".to_string(),                // version
        "CPU软件挖矿设备".to_string(),         // description
    )
}

/// 创建测试用的设备配置
fn create_test_device_config() -> DeviceConfig {
    DeviceConfig {
        frequency: 1000,      // MHz
        voltage: 1200,        // mV
        fan_speed: Some(50),  // %
        power_limit: Some(100), // W
        temperature_limit: Some(80), // °C
        auto_fan: true,
        auto_frequency: false,
        auto_voltage: false,
    }
}

/// 创建测试用的工作数据
fn create_demo_work() -> Work {
    // 创建一个80字节的区块头
    let mut header = vec![0u8; 80];

    // 版本 (4字节)
    header[0..4].copy_from_slice(&1u32.to_le_bytes());

    // 前一个区块哈希 (32字节) - 使用简单的测试数据
    header[4..36].copy_from_slice(&[0u8; 32]);

    // Merkle根 (32字节) - 使用简单的测试数据
    let merkle_root = vec![1u8; 32];
    header[36..68].copy_from_slice(&merkle_root);

    // 时间戳 (4字节)
    header[68..72].copy_from_slice(&1231006505u32.to_le_bytes());

    // 难度目标 (4字节)
    header[72..76].copy_from_slice(&0x1d00ffffu32.to_le_bytes());

    // Nonce (4字节) - 初始为0
    header[76..80].copy_from_slice(&0u32.to_le_bytes());

    // 创建目标难度 (32字节)
    let mut target = vec![0xFFu8; 32];
    target[0..4].copy_from_slice(&[0x00, 0x00, 0x0F, 0xFF]);

    Work {
        id: 1,
        work_id: "demo_job_001".to_string(),
        header,
        merkle_root: merkle_root,
        midstate: vec![0u8; 32], // 中间状态
        target,
        difficulty: 1.0,
        height: Some(1),
        timestamp: SystemTime::now(),
    }
}

/// 演示设备创建和基本操作
async fn demo_device_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 演示设备创建和基本操作");

    // 创建设备信息和配置
    let device_info = create_test_device_info();
    let config = create_test_device_config();

    // 创建设备
    let start_time = Instant::now();
    let device = SoftwareDevice::new(
        device_info.clone(),
        config.clone(),
        1000000.0, // target_hashrate: 1 MH/s
        0.01,      // error_rate: 1%
        1000,      // batch_size
    ).await?;
    let creation_time = start_time.elapsed();

    println!("  ✅ 设备创建耗时: {:?}", creation_time);
    println!("  📊 设备信息: {}", device_info.name);
    println!("  📈 设备ID: {}", device.device_id());

    Ok(())
}

/// 演示核心工厂
async fn demo_core_factory() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏭 演示核心工厂");

    // 创建工厂
    let start_time = Instant::now();
    let factory = SoftwareCoreFactory::new();
    let factory_time = start_time.elapsed();

    println!("  ✅ 工厂创建耗时: {:?}", factory_time);

    // 获取核心信息
    let core_info = factory.get_core_info();
    println!("  🔍 核心信息:");
    println!("    - 名称: {}", core_info.name);
    println!("    - 版本: {}", core_info.version);
    println!("    - 描述: {}", core_info.description);

    Ok(())
}



/// 简单的性能基准测试
async fn simple_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚀 简单性能基准测试");

    use sha2::{Sha256, Digest};

    // SHA256 双重哈希基准测试
    let test_data = [0u8; 80]; // 模拟区块头
    let iterations = 10000;

    println!("  🔄 执行 {} 次 SHA256 双重哈希", iterations);

    let start_time = Instant::now();

    for i in 0..iterations {
        let mut data = test_data;
        // 修改nonce
        data[76..80].copy_from_slice(&(i as u32).to_le_bytes());

        // 第一次SHA256
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash1 = hasher.finalize();

        // 第二次SHA256
        let mut hasher = Sha256::new();
        hasher.update(&hash1);
        let _hash2 = hasher.finalize();
    }

    let total_time = start_time.elapsed();
    let avg_time = total_time / iterations;
    let hashrate = 1_000_000_000.0 / avg_time.as_nanos() as f64; // H/s

    println!("  ✅ 基准测试完成");
    println!("  📊 总耗时: {:?}", total_time);
    println!("  ⚡ 平均每次: {:?}", avg_time);
    println!("  🔥 估算算力: {:.2} MH/s", hashrate / 1_000_000.0);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CGMiner CPU BTC Core 基准测试演示");
    println!("=====================================");

    // 运行各种演示
    demo_device_operations().await?;
    demo_core_factory().await?;
    simple_benchmark().await?;

    println!("\n🎉 演示完成！");
    println!("\n💡 提示:");
    println!("  - 运行完整基准测试: ./run_benchmarks.sh --all");
    println!("  - 快速基准测试: ./run_benchmarks.sh --quick");
    println!("  - 查看帮助: ./run_benchmarks.sh --help");

    Ok(())
}
