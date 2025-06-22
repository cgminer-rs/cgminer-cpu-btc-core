//! 多设备挖矿演示
//!
//! 这个示例展示如何同时使用多个CPU设备进行并行挖矿
//! 包括设备管理、负载均衡、统计汇总等功能

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
use futures::future::join_all;
use sha2::{Sha256, Digest};

/// 设备管理器
struct DeviceManager {
    devices: Vec<Arc<SoftwareDevice>>,
    device_configs: Vec<DeviceConfig>,
}

impl DeviceManager {
    /// 创建新的设备管理器
    fn new() -> Self {
        Self {
            devices: Vec::new(),
            device_configs: Vec::new(),
        }
    }

    /// 添加设备
    async fn add_device(
        &mut self,
        device_id: u32,
        name: String,
        target_hashrate: f64,
        config: DeviceConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let device_info = DeviceInfo::new(
            device_id,
            name,
            "cpu".to_string(),
            0,
        );

        let device = SoftwareDevice::new(
            device_info,
            config.clone(),
            f64::MAX, // 无算力限制，发挥最大性能
            0.001,    // 0.1% error rate
            50000,    // 大批次处理
        ).await?;

        self.devices.push(Arc::new(device));
        self.device_configs.push(config);

        println!("  ✅ 设备 #{} 创建成功 (算力: 无限制)", device_id);
        Ok(())
    }

    /// 获取设备数量
    fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// 获取所有设备状态
    async fn get_all_status(&self) -> Result<Vec<DeviceStatus>, Box<dyn std::error::Error>> {
        let mut statuses = Vec::new();

        for device in &self.devices {
            let status = device.get_status().await?;
            statuses.push(status);
        }

        Ok(statuses)
    }

    /// 启动所有设备的挖矿任务（持续运算直到找到解）
    async fn start_mining_until_solution(&self, work: Work) -> Result<Vec<MiningResult>, Box<dyn std::error::Error>> {
        println!("🚀 启动 {} 个设备进行并行挖矿（持续运算直到找到解）", self.devices.len());
        println!("💡 提示: 设备将持续运算直到找到有效解，请耐心等待");

        let mut tasks = Vec::new();

        for (i, device) in self.devices.iter().enumerate() {
            let device_clone = Arc::clone(device);
            let work_clone = work.clone();
            let device_id = i + 1;

            let task = tokio::spawn(async move {
                println!("  🔄 设备 #{} 开始挖矿", device_id);

                // 真实挖矿过程 - 持续运算直到找到解
                let start_time = Instant::now();
                let mut total_hashes = 0u64;
                let mut nonce_start = (device_id as u32 - 1) * 1000000; // 每个设备从不同nonce开始
                let mut nonce = nonce_start;
                let mut solutions_found = 0u32;

                loop {
                    // 修改区块头中的nonce
                    let mut test_header = work_clone.header;
                    test_header[76..80].copy_from_slice(&nonce.to_le_bytes());

                    // 计算双重SHA256哈希
                    let hash = Self::calculate_double_sha256(&test_header);
                    total_hashes += 1;

                    // 检查是否满足难度要求
                    if Self::is_valid_hash(&hash, &work_clone.target) {
                        solutions_found += 1;
                        println!("  🎉 设备 #{} 找到解! Nonce: {} (总解数: {})", device_id, nonce, solutions_found);
                        break; // 找到解后退出
                    }

                    // 每100000次尝试显示进度
                    if total_hashes % 100000 == 0 {
                        let elapsed = start_time.elapsed();
                        let hashrate = total_hashes as f64 / elapsed.as_secs_f64();
                        println!("  📊 设备 #{}: {} 次 | {:.2} MH/s",
                            device_id, total_hashes, hashrate / 1_000_000.0);
                    }

                    nonce += 1;

                    // 防止nonce溢出
                    if nonce == u32::MAX {
                        nonce = nonce_start;
                    }
                }

                let actual_time = start_time.elapsed();
                let hashrate = total_hashes as f64 / actual_time.as_secs_f64();

                println!("  ✅ 设备 #{} 挖矿完成", device_id);
                println!("    - 总哈希数: {}", total_hashes);
                println!("    - 找到解数: {}", solutions_found);
                println!("    - 平均算力: {:.2} MH/s", hashrate / 1_000_000.0);
                println!("    - 用时: {:.2}秒", actual_time.as_secs_f64());

                MiningResult {
                    device_id: device_id as u32,
                    work_id: work_clone.work_id,
                    nonce: Some(nonce),
                    hash: vec![0u8; 32],
                    target_met: true,
                    hashrate,
                    timestamp: SystemTime::now(),
                    shares_accepted: solutions_found,
                    shares_rejected: 0,
                    hardware_errors: 0,
                }
            });

            tasks.push(task);
        }

        // 等待第一个设备找到解
        let (result, _index, _remaining) = futures::future::select_all(tasks).await;

        match result {
            Ok(mining_result) => {
                println!("🏆 有设备找到解！停止其他设备...");
                Ok(vec![mining_result])
            },
            Err(e) => {
                println!("❌ 挖矿任务失败: {}", e);
                Err(Box::new(e))
            }
        }
    }

    /// 计算双重SHA256哈希
    fn calculate_double_sha256(data: &[u8]) -> Vec<u8> {
        use sha2::{Sha256, Digest};

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
}

/// 创建工作数据
fn create_work() -> Work {
    let mut header = [0u8; 80];

    // 填充基本的区块头数据
    header[0..4].copy_from_slice(&1u32.to_le_bytes()); // version
    header[68..72].copy_from_slice(&(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as u32).to_le_bytes()); // timestamp
    header[72..76].copy_from_slice(&0x1d00ffffu32.to_le_bytes()); // bits

    // 创建更容易的目标，使演示更快完成
    let mut target = [0x00u8; 32];
    target[24..32].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00]);

    Work::new(
        "multi_device_work_001".to_string(), // job_id
        target,                              // target
        header,                              // header
        1.0,                                // difficulty
    )
}

/// 创建设备配置
fn create_device_config(device_id: u32) -> DeviceConfig {
    DeviceConfig {
        frequency: 1800 + (device_id * 100), // 不同设备使用不同频率
        voltage: 1200,
        fan_speed: Some(50 + (device_id * 5)), // 不同的风扇速度
        power_limit: Some(100 + (device_id * 10)), // 不同的功耗限制
        temperature_limit: Some(80),
        auto_fan: true,
        auto_frequency: false,
        auto_voltage: false,
    }
}

/// 显示统计汇总
fn display_summary(results: &[MiningResult]) {
    println!("\n📊 多设备挖矿统计汇总");
    println!("====================");

    let total_hashrate: f64 = results.iter().map(|r| r.hashrate).sum();
    let total_shares: u32 = results.iter().map(|r| r.shares_accepted).sum();
    let total_devices = results.len();

    println!("📈 总体统计:");
    println!("  - 参与设备数: {}", total_devices);
    println!("  - 总算力: {:.2} MH/s", total_hashrate / 1_000_000.0);
    println!("  - 平均算力: {:.2} KH/s", total_hashrate / total_devices as f64 / 1000.0);
    println!("  - 总找到解数: {}", total_shares);

    println!("\n📋 各设备详情:");
    for result in results {
        println!("  设备 #{}:", result.device_id);
        println!("    - 算力: {:.2} KH/s", result.hashrate / 1000.0);
        println!("    - 找到解: {}", result.shares_accepted);
        println!("    - 效率: {:.1}%", if total_hashrate > 0.0 { result.hashrate / total_hashrate * 100.0 } else { 0.0 });
    }
}

/// 演示设备状态监控
async fn monitor_devices(manager: &DeviceManager, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 开始设备状态监控 ({}秒)", duration.as_secs());

    let start_time = Instant::now();
    let mut monitor_count = 0;

    while start_time.elapsed() < duration {
        monitor_count += 1;
        println!("\n📊 监控周期 #{}", monitor_count);

        let statuses = manager.get_all_status().await?;

        for (i, status) in statuses.iter().enumerate() {
            println!("  设备 #{}: 温度={:.1}°C, 算力={:.2}KH/s, 功耗={:.1}W",
                i + 1,
                status.temperature,
                status.hashrate / 1000.0,
                status.power_consumption
            );
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CGMiner CPU BTC Core - 多设备挖矿演示");
    println!("=======================================");

    // 创建设备管理器
    let mut manager = DeviceManager::new();

    // 添加多个设备
    println!("\n📱 创建多个挖矿设备:");
    for i in 1..=4 {
        let config = create_device_config(i);

        manager.add_device(
            i,
            format!("CPU Miner #{}", i),
            f64::MAX, // 无算力限制
            config,
        ).await?;
    }

    println!("\n✅ 成功创建 {} 个设备", manager.device_count());

    // 创建工作
    let work = create_work();
    println!("\n⚒️  创建挖矿工作: {}", work.work_id);

    // 启动并行监控任务
    let manager_arc = Arc::new(manager);
    let monitor_manager = Arc::clone(&manager_arc);

    let monitor_task = tokio::spawn(async move {
        if let Err(e) = monitor_devices(&monitor_manager, Duration::from_secs(10)).await {
            println!("❌ 监控任务失败: {}", e);
        }
    });

    // 启动挖矿（持续运算直到找到解）
    println!("\n⛏️  开始多设备并行挖矿（持续运算直到找到解）...");
    let mining_results = manager_arc.start_mining_until_solution(work).await?;

    // 等待监控任务完成
    let _ = monitor_task.await;

    // 显示结果
    display_summary(&mining_results);

    println!("\n🎉 多设备挖矿演示完成！");
    println!("\n💡 提示:");
    println!("  - 实际使用时可以根据CPU核心数调整设备数量");
    println!("  - 可以为不同设备设置不同的算力目标");
    println!("  - 支持动态添加和移除设备");

    Ok(())
}
