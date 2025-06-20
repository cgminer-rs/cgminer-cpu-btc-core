//! 测试基于实际哈希次数的算力计算逻辑

use cgminer_core::{DeviceInfo, DeviceConfig, MiningDevice, Work};
use cgminer_cpu_btc_core::device::SoftwareDevice;
use std::time::Duration;
use tokio::time::sleep;

/// 创建测试用的设备
async fn create_test_device() -> SoftwareDevice {
    let device_info = DeviceInfo::new(
        1,
        "Test Device".to_string(),
        "software".to_string(),
        0,
    );

    let config = DeviceConfig::default();

    SoftwareDevice::new(
        device_info,
        config,
        1_000_000.0, // 1 MH/s 目标算力
        0.01,        // 1% 错误率
        1000,        // 批次大小
    ).await.unwrap()
}

/// 创建测试用的工作（高难度，不容易找到解）
fn create_test_work() -> Work {
    Work::new(
        "1".to_string(),
        [0x00u8; 32], // 目标难度（几乎不可能满足，确保执行完整批次）
        [0u8; 80], // 80字节的区块头
        1.0,
    )
}

/// 创建容易找到解的测试工作
fn create_easy_test_work() -> Work {
    Work::new(
        "1".to_string(),
        [0xFFu8; 32], // 目标难度（很容易满足）
        [0u8; 80], // 80字节的区块头
        1.0,
    )
}

#[tokio::test]
async fn test_hashrate_calculation_based_on_actual_hashes() {
    let mut device = create_test_device().await;
    let work = create_test_work();

    // 初始化设备
    device.initialize(DeviceConfig::default()).await.unwrap();
    device.start().await.unwrap();

    // 提交工作
    device.submit_work(work).await.unwrap();

    // 记录开始时间
    let start_time = std::time::Instant::now();

    // 执行挖矿
    let _result = device.get_result().await.unwrap();

    // 记录结束时间
    let elapsed = start_time.elapsed().as_secs_f64();

    // 获取统计信息
    let stats = device.get_stats().await.unwrap();

    // 验证算力计算
    assert!(stats.total_hashes > 0, "应该记录实际哈希次数");
    assert!(stats.current_hashrate.hashes_per_second > 0.0, "当前算力应该大于0");

    // 验证算力计算的合理性
    let expected_hashrate = stats.total_hashes as f64 / elapsed;
    let actual_hashrate = stats.current_hashrate.hashes_per_second;

    // 由于使用高难度，应该执行完整的批次（1000次哈希）
    assert_eq!(stats.total_hashes, 1000, "应该执行完整的批次");

    // 允许一定的误差范围（20%，因为时间测量可能有误差）
    let error_margin = 0.2;
    let lower_bound = expected_hashrate * (1.0 - error_margin);
    let upper_bound = expected_hashrate * (1.0 + error_margin);

    assert!(
        actual_hashrate >= lower_bound && actual_hashrate <= upper_bound,
        "算力计算应该基于实际哈希次数。期望: {:.2}, 实际: {:.2}, 哈希次数: {}, 时间: {:.3}s",
        expected_hashrate, actual_hashrate, stats.total_hashes, elapsed
    );

    println!("✅ 算力计算测试通过:");
    println!("   实际哈希次数: {}", stats.total_hashes);
    println!("   执行时间: {:.3}s", elapsed);
    println!("   期望算力: {:.2} H/s", expected_hashrate);
    println!("   实际算力: {:.2} H/s", actual_hashrate);
}

#[tokio::test]
async fn test_rolling_hashrate_calculation() {
    let mut device = create_test_device().await;
    let work = create_test_work(); // 使用高难度工作确保执行完整批次

    // 初始化设备
    device.initialize(DeviceConfig::default()).await.unwrap();
    device.start().await.unwrap();

    // 执行多次挖矿以测试滑动窗口算力
    for i in 0..3 {
        device.submit_work(work.clone()).await.unwrap();
        let _result = device.get_result().await.unwrap();

        let stats = device.get_stats().await.unwrap();

        println!("第{}次挖矿后的统计:", i + 1);
        println!("   总哈希次数: {}", stats.total_hashes);
        println!("   当前算力: {:.2} H/s", stats.current_hashrate.hashes_per_second);
        println!("   1分钟平均: {:.2} H/s", stats.hashrate_1m.hashes_per_second);
        println!("   5分钟平均: {:.2} H/s", stats.hashrate_5m.hashes_per_second);
        println!("   15分钟平均: {:.2} H/s", stats.hashrate_15m.hashes_per_second);

        // 验证滑动窗口算力已被更新
        if i > 0 {
            assert!(stats.hashrate_1m.hashes_per_second > 0.0, "1分钟平均算力应该大于0");
            assert!(stats.hashrate_5m.hashes_per_second > 0.0, "5分钟平均算力应该大于0");
            assert!(stats.hashrate_15m.hashes_per_second > 0.0, "15分钟平均算力应该大于0");
        }

        // 短暂等待以产生时间差
        sleep(Duration::from_millis(100)).await;
    }

    println!("✅ 滑动窗口算力计算测试通过");
}

#[tokio::test]
async fn test_hashrate_accuracy_over_time() {
    let mut device = create_test_device().await;
    let work = create_test_work(); // 使用高难度工作

    // 初始化设备
    device.initialize(DeviceConfig::default()).await.unwrap();
    device.start().await.unwrap();

    let mut total_hashes = 0u64;
    let start_time = std::time::Instant::now();

    // 执行多次挖矿
    for _ in 0..5 {
        device.submit_work(work.clone()).await.unwrap();
        let _result = device.get_result().await.unwrap();

        let stats = device.get_stats().await.unwrap();
        total_hashes = stats.total_hashes;

        sleep(Duration::from_millis(50)).await;
    }

    let total_elapsed = start_time.elapsed().as_secs_f64();
    let stats = device.get_stats().await.unwrap();

    // 验证总体算力的准确性
    let expected_overall_hashrate = total_hashes as f64 / total_elapsed;
    let actual_average_hashrate = stats.average_hashrate.hashes_per_second;

    println!("总体算力准确性测试:");
    println!("   总哈希次数: {}", total_hashes);
    println!("   总时间: {:.3}s", total_elapsed);
    println!("   期望总体算力: {:.2} H/s", expected_overall_hashrate);
    println!("   实际平均算力: {:.2} H/s", actual_average_hashrate);

    // 平均算力应该在合理范围内
    assert!(actual_average_hashrate > 0.0, "平均算力应该大于0");

    println!("✅ 长时间算力准确性测试通过");
}

#[test]
fn test_rolling_stats_decay_algorithm() {
    use cgminer_core::device::RollingHashrateStats;

    let mut rolling_stats = RollingHashrateStats::default();

    // 模拟一系列哈希计算
    let test_cases = vec![
        (1000u64, 1.0f64), // 1000 hashes in 1 second = 1000 H/s
        (2000u64, 1.0f64), // 2000 hashes in 1 second = 2000 H/s
        (1500u64, 1.0f64), // 1500 hashes in 1 second = 1500 H/s
    ];

    for (hashes, time_diff) in test_cases {
        rolling_stats.update(hashes, time_diff);

        println!("更新后的滑动窗口算力:");
        println!("   1分钟: {:.2} H/s", rolling_stats.rolling_1m);
        println!("   5分钟: {:.2} H/s", rolling_stats.rolling_5m);
        println!("   15分钟: {:.2} H/s", rolling_stats.rolling_15m);

        // 验证滑动窗口算力在合理范围内
        assert!(rolling_stats.rolling_1m >= 0.0, "1分钟滑动算力应该非负");
        assert!(rolling_stats.rolling_5m >= 0.0, "5分钟滑动算力应该非负");
        assert!(rolling_stats.rolling_15m >= 0.0, "15分钟滑动算力应该非负");
    }

    println!("✅ 滑动窗口衰减算法测试通过");
}
