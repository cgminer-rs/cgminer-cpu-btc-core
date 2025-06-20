//! 软算法核心集成测试
//!
//! 这些测试验证软算法核心的各个组件是否正确工作，包括：
//! - CPU绑定管理器
//! - 软算法设备
//! - 软算法核心
//! - SHA256挖矿算法

use cgminer_core::{DeviceInfo, DeviceConfig, MiningDevice, MiningCore, Work};
use cgminer_s_btc_core::{
    SoftwareMiningCore, SoftwareDevice,
    cpu_affinity::{CpuAffinityManager, CpuAffinityStrategy}
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

/// 创建测试用的设备信息
fn create_test_device_info(id: u32, name: &str) -> DeviceInfo {
    DeviceInfo::new(id, name.to_string(), "software".to_string(), 0)
}

/// 创建测试用的工作
fn create_test_work(id: u64) -> Work {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // 创建一个简单的区块头（80字节）
    let mut header = [0u8; 80];

    // 版本号 (4字节)
    header[0..4].copy_from_slice(&1u32.to_le_bytes());

    // 前一个区块哈希 (32字节) - 使用测试数据
    for i in 4..36 {
        header[i] = (i % 256) as u8;
    }

    // Merkle根 (32字节) - 使用测试数据
    for i in 36..68 {
        header[i] = ((i * 2) % 256) as u8;
    }

    // 时间戳 (4字节)
    header[68..72].copy_from_slice(&(timestamp as u32).to_le_bytes());

    // 难度目标 (4字节) - 设置较低的难度便于测试
    header[72..76].copy_from_slice(&0x207fffffu32.to_le_bytes());

    // Nonce (4字节) - 初始为0，挖矿时会修改
    header[76..80].copy_from_slice(&0u32.to_le_bytes());

    // 创建目标值 - 设置较低的难度
    let mut target = [0xffu8; 32];
    target[0] = 0x00;
    target[1] = 0x00;
    target[2] = 0x7f;

    Work::new(
        format!("test_job_{}", id),
        target,
        header,
        1.0,
    )
}

#[tokio::test]
async fn test_cpu_affinity_manager() {
    // 测试CPU绑定管理器的基本功能
    let mut cpu_manager = CpuAffinityManager::new(true, CpuAffinityStrategy::RoundRobin);

    // 检查是否正确初始化
    assert!(cpu_manager.available_core_count() > 0, "应该检测到可用的CPU核心");

    // 测试设备CPU绑定分配
    for device_id in 0..4 {
        cpu_manager.assign_cpu_core(device_id);
        let core_id = cpu_manager.get_device_core(device_id);
        assert!(core_id.is_some(), "设备 {} 应该被分配到CPU核心", device_id);
    }

    // 测试CPU绑定统计信息
    let stats = cpu_manager.get_affinity_stats();
    assert!(stats.total_cpu_cores > 0, "总CPU核心数应该大于0");
    assert!(stats.available_cores > 0, "可用核心数应该大于0");
    assert_eq!(stats.bound_devices, 4, "应该有4个设备被绑定");
    assert!(stats.enabled, "CPU绑定应该被启用");
}

#[tokio::test]
async fn test_cpu_affinity_manager_disabled() {
    // 测试禁用CPU绑定的情况
    let cpu_manager = CpuAffinityManager::new(false, CpuAffinityStrategy::RoundRobin);

    // 检查是否正确禁用
    assert!(!cpu_manager.is_enabled(), "CPU绑定应该被禁用");

    // 测试绑定当前线程（应该成功但不执行实际绑定）
    let result = cpu_manager.bind_current_thread(0);
    assert!(result.is_ok(), "禁用状态下绑定线程应该成功（但不执行实际绑定）");
}

#[tokio::test]
async fn test_software_device_creation() {
    // 测试软算法设备的创建
    let device_info = create_test_device_info(0, "测试设备");
    let config = DeviceConfig::default();

    let device = SoftwareDevice::new(
        device_info.clone(),
        config,
        1_000_000_000.0, // 1 GH/s
        0.01,            // 1% 错误率
        1000,            // 批次大小
    ).await;

    assert!(device.is_ok(), "软算法设备创建应该成功");

    let device = device.unwrap();

    // 验证设备ID
    assert_eq!(device.device_id(), 0, "设备ID应该匹配");

    // 获取设备信息
    let info = device.get_info().await;
    assert!(info.is_ok(), "获取设备信息应该成功");

    let info = info.unwrap();
    assert_eq!(info.id, 0, "设备信息ID应该匹配");
    assert_eq!(info.name, "测试设备", "设备名称应该匹配");
    assert_eq!(info.device_type, "software", "设备类型应该是software");
}

#[tokio::test]
async fn test_software_device_lifecycle() {
    // 测试软算法设备的生命周期
    let device_info = create_test_device_info(1, "生命周期测试设备");
    let config = DeviceConfig::default();

    let mut device = SoftwareDevice::new(
        device_info,
        config.clone(),
        500_000_000.0, // 500 MH/s
        0.05,          // 5% 错误率
        500,           // 批次大小
    ).await.expect("设备创建应该成功");

    // 初始化设备
    let init_result = device.initialize(config).await;
    assert!(init_result.is_ok(), "设备初始化应该成功");

    // 启动设备
    let start_result = device.start().await;
    assert!(start_result.is_ok(), "设备启动应该成功");

    // 检查设备状态
    let status = device.get_status().await;
    assert!(status.is_ok(), "获取设备状态应该成功");

    // 获取设备统计信息
    let stats = device.get_stats().await;
    assert!(stats.is_ok(), "获取设备统计信息应该成功");

    let stats = stats.unwrap();
    assert_eq!(stats.device_id, 1, "统计信息中的设备ID应该匹配");

    // 停止设备
    let stop_result = device.stop().await;
    assert!(stop_result.is_ok(), "设备停止应该成功");
}

#[tokio::test]
async fn test_software_device_mining() {
    // 测试软算法设备的挖矿功能
    let device_info = create_test_device_info(2, "挖矿测试设备");
    let config = DeviceConfig::default();

    let mut device = SoftwareDevice::new(
        device_info,
        config.clone(),
        100_000_000.0, // 100 MH/s - 较低的算力便于测试
        0.1,           // 10% 错误率
        100,           // 较小的批次大小
    ).await.expect("设备创建应该成功");

    // 初始化并启动设备
    device.initialize(config).await.expect("设备初始化应该成功");
    device.start().await.expect("设备启动应该成功");

    // 创建测试工作
    let work = create_test_work(1);

    // 提交工作
    let submit_result = device.submit_work(work).await;
    assert!(submit_result.is_ok(), "提交工作应该成功");

    // 等待一段时间让设备处理工作
    sleep(Duration::from_millis(100)).await;

    // 尝试获取结果（可能没有结果，因为难度设置）
    let result = device.get_result().await;
    assert!(result.is_ok(), "获取挖矿结果应该成功（即使没有找到有效结果）");

    // 获取更新后的统计信息
    let stats = device.get_stats().await.expect("获取统计信息应该成功");

    // 验证统计信息已更新
    assert!(stats.current_hashrate.hashes_per_second >= 0.0, "当前算力应该是非负数");
    assert!(stats.average_hashrate.hashes_per_second >= 0.0, "平均算力应该是非负数");

    // 停止设备
    device.stop().await.expect("设备停止应该成功");
}

#[tokio::test]
async fn test_software_mining_core() {
    // 测试软算法挖矿核心
    let mut core = SoftwareMiningCore::new("测试核心".to_string());

    // 获取核心信息
    let core_info = core.get_info();
    assert_eq!(core_info.name, "测试核心", "核心名称应该匹配");
    assert_eq!(core_info.version, "0.1.0", "核心版本应该匹配");

    // 获取核心能力
    let capabilities = core.get_capabilities();
    assert!(capabilities.supports_temperature_monitoring, "应该支持温度监控");
    assert!(capabilities.supports_frequency_control, "应该支持频率控制");
    assert!(capabilities.supports_multiple_chains, "应该支持多链");
    assert!(capabilities.supported_algorithms.contains(&"SHA256".to_string()), "应该支持SHA256算法");

    // 启动核心
    let start_result = core.start().await;
    assert!(start_result.is_ok(), "核心启动应该成功");

    // 扫描设备
    let devices = core.scan_devices().await;
    assert!(devices.is_ok(), "扫描设备应该成功");

    // 停止核心
    let stop_result = core.stop().await;
    assert!(stop_result.is_ok(), "核心停止应该成功");
}

#[tokio::test]
async fn test_software_core_device_creation() {
    // 测试软算法核心的设备创建功能
    let core = SoftwareMiningCore::new("设备创建测试核心".to_string());

    // 创建测试设备信息
    let device_info = create_test_device_info(3, "核心创建的设备");

    // 通过核心创建设备
    let device_result = core.create_device(device_info.clone()).await;
    assert!(device_result.is_ok(), "通过核心创建设备应该成功");

    let device = device_result.unwrap();
    assert_eq!(device.device_id(), 3, "设备ID应该匹配");

    // 验证设备信息
    let info = device.get_info().await.expect("获取设备信息应该成功");
    assert_eq!(info.id, 3, "设备信息ID应该匹配");
    assert_eq!(info.name, "核心创建的设备", "设备名称应该匹配");
}

#[tokio::test]
async fn test_cpu_affinity_strategies() {
    // 测试不同的CPU绑定策略

    // 测试轮询策略
    let mut round_robin_manager = CpuAffinityManager::new(true, CpuAffinityStrategy::RoundRobin);
    round_robin_manager.assign_cpu_core(0);
    round_robin_manager.assign_cpu_core(1);

    let core0 = round_robin_manager.get_device_core(0);
    let core1 = round_robin_manager.get_device_core(1);

    assert!(core0.is_some(), "设备0应该被分配到CPU核心");
    assert!(core1.is_some(), "设备1应该被分配到CPU核心");

    // 在多核系统上，设备应该被分配到不同的核心
    if round_robin_manager.available_core_count() > 1 {
        assert_ne!(core0, core1, "在多核系统上，不同设备应该被分配到不同的核心");
    }

    // 测试性能优先策略
    let mut perf_manager = CpuAffinityManager::new(true, CpuAffinityStrategy::PerformanceFirst);
    perf_manager.assign_cpu_core(0);
    let perf_core = perf_manager.get_device_core(0);
    assert!(perf_core.is_some(), "性能优先策略应该分配CPU核心");

    // 测试物理核心策略
    let mut physical_manager = CpuAffinityManager::new(true, CpuAffinityStrategy::PhysicalCoresOnly);
    physical_manager.assign_cpu_core(0);
    let physical_core = physical_manager.get_device_core(0);
    assert!(physical_core.is_some(), "物理核心策略应该分配CPU核心");
}
