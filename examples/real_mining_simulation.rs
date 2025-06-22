//! 真实挖矿模拟演示
//!
//! 这个示例展示如何模拟真实的比特币挖矿环境
//! 包括矿池连接、工作分配、难度调整、收益计算等

use cgminer_cpu_btc_core::{
    SoftwareDevice,
    SoftwareCoreFactory,
};
use cgminer_core::{
    DeviceInfo, DeviceStatus, DeviceConfig, Work, MiningResult, CoreType,
};
use std::time::{Duration, SystemTime, Instant};
use std::sync::Arc;
use tokio;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 矿池配置
#[derive(Debug, Clone)]
struct PoolConfig {
    name: String,
    url: String,
    port: u16,
    username: String,
    password: String,
    difficulty: f64,
    fee_rate: f64, // 手续费率
}

/// 挖矿统计
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MiningStats {
    total_hashes: u64,
    shares_accepted: u32,
    shares_rejected: u32,
    blocks_found: u32,
    uptime: Duration,
    average_hashrate: f64,
    estimated_earnings: f64, // BTC
    power_consumed: f64,     // kWh
    efficiency: f64,         // H/W
}

/// 真实挖矿模拟器
struct RealMiningSimulator {
    devices: Vec<Arc<SoftwareDevice>>,
    pool_config: PoolConfig,
    mining_stats: MiningStats,
    current_difficulty: f64,
    block_reward: f64, // BTC
    btc_price: f64,    // USD
    electricity_cost: f64, // USD per kWh
    start_time: Instant,
}

impl RealMiningSimulator {
    fn new(pool_config: PoolConfig) -> Self {
        Self {
            devices: Vec::new(),
            pool_config,
            mining_stats: MiningStats {
                total_hashes: 0,
                shares_accepted: 0,
                shares_rejected: 0,
                blocks_found: 0,
                uptime: Duration::from_secs(0),
                average_hashrate: 0.0,
                estimated_earnings: 0.0,
                power_consumed: 0.0,
                efficiency: 0.0,
            },
            current_difficulty: 1.0,
            block_reward: 6.25, // 当前比特币区块奖励
            btc_price: 45000.0, // 假设BTC价格
            electricity_cost: 0.10, // 每kWh 0.10美元
            start_time: Instant::now(),
        }
    }

    /// 添加挖矿设备
    async fn add_device(&mut self, device_config: DeviceConfig, target_hashrate: f64) -> Result<(), Box<dyn std::error::Error>> {
        let device_id = self.devices.len() as u32 + 1;
        let device_info = DeviceInfo::new(
            device_id,
            format!("Mining Rig #{}", device_id),
            "cpu".to_string(),
            0,
        );

        let device = SoftwareDevice::new(
            device_info,
            device_config,
            f64::MAX, // 无算力限制，最大性能
            0.001,    // 0.1% error rate
            100000,   // 超大批次处理
        ).await?;

        self.devices.push(Arc::new(device));
        println!("✅ 添加设备 #{} (算力: 无限制)", device_id);
        Ok(())
    }

    /// 连接到矿池 (模拟)
    async fn connect_to_pool(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔗 连接到矿池: {}", self.pool_config.name);
        println!("  - 地址: {}:{}", self.pool_config.url, self.pool_config.port);
        println!("  - 用户: {}", self.pool_config.username);
        println!("  - 难度: {:.2}", self.pool_config.difficulty);
        println!("  - 手续费: {:.1}%", self.pool_config.fee_rate * 100.0);

        // 模拟连接延迟
        tokio::time::sleep(Duration::from_millis(500)).await;
        println!("✅ 矿池连接成功");
        Ok(())
    }

    /// 获取挖矿工作 (模拟)
    fn get_mining_work(&self) -> Work {
        let mut header = vec![0u8; 80];

        // 模拟真实的区块头数据
        header[0..4].copy_from_slice(&0x20000000u32.to_le_bytes()); // version

        // 随机的前一个区块哈希
        for i in 4..36 {
            header[i] = fastrand::u8(..);
        }

        // 随机的Merkle根
        let merkle_root: Vec<u8> = (0..32).map(|_| fastrand::u8(..)).collect();
        header[36..68].copy_from_slice(&merkle_root);

        // 当前时间戳
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        header[68..72].copy_from_slice(&timestamp.to_le_bytes());

        // 难度目标
        let bits = self.difficulty_to_bits(self.current_difficulty);
        header[72..76].copy_from_slice(&bits.to_le_bytes());

        // Nonce初始为0
        header[76..80].copy_from_slice(&0u32.to_le_bytes());

        // 根据难度计算目标值
        let target = self.bits_to_target(bits);

        Work::new(
            format!("work_{}", fastrand::u64(..)), // job_id
            target,                                 // target
            header,                                 // header
            self.current_difficulty,               // difficulty
        )
    }

    /// 难度转换为bits格式
    fn difficulty_to_bits(&self, difficulty: f64) -> u32 {
        // 简化的难度转换
        let target_max = 0x1d00ffffu32;
        (target_max as f64 / difficulty) as u32
    }

    /// bits转换为目标值
    fn bits_to_target(&self, bits: u32) -> [u8; 32] {
        let mut target = [0u8; 32];
        // 简化的目标值计算，设置更容易的目标
        target[24..32].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00]);
        target
    }

    /// 开始挖矿
    async fn start_mining(&mut self, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n⛏️  开始真实挖矿模拟 ({}分钟)", duration.as_secs() / 60);
        println!("========================================");

        let start_time = Instant::now();
        let mut work_count = 0;
        let mut last_stats_time = Instant::now();

        while start_time.elapsed() < duration {
            work_count += 1;

            // 获取新的挖矿工作
            let work = self.get_mining_work();
            println!("\n📋 工作 #{}: {} (难度: {:.2})", work_count, work.work_id, work.difficulty);

            // 分配工作给所有设备
            let mut device_tasks = Vec::new();
            for (i, device) in self.devices.iter().enumerate() {
                let device_clone = Arc::clone(device);
                let work_clone = work.clone();
                let device_id = i + 1;

                let task = tokio::spawn(async move {
                    Self::device_mining_task(device_clone, work_clone, device_id as u32).await
                });
                device_tasks.push(task);
            }

            // 等待设备完成工作或超时
            let work_timeout = Duration::from_secs(30);
            let work_start = Instant::now();

            while work_start.elapsed() < work_timeout {
                // 检查是否有设备找到解
                let mut found_solution = false;
                for task in &device_tasks {
                    if task.is_finished() {
                        found_solution = true;
                        break;
                    }
                }

                if found_solution {
                    println!("  🎉 找到解决方案!");
                    self.mining_stats.shares_accepted += 1;

                    // 模拟找到区块的概率
                    if fastrand::f64() < 0.001 { // 0.1% 概率
                        self.mining_stats.blocks_found += 1;
                        println!("  🏆 找到新区块! 总区块数: {}", self.mining_stats.blocks_found);
                    }
                    break;
                } else {
                    // 模拟拒绝的概率
                    if fastrand::f64() < 0.05 { // 5% 概率
                        self.mining_stats.shares_rejected += 1;
                        println!("  ❌ 工作被拒绝");
                        break;
                    }
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            // 更新统计信息
            self.update_mining_stats().await?;

            // 每30秒显示一次统计
            if last_stats_time.elapsed() >= Duration::from_secs(30) {
                self.display_mining_stats();
                last_stats_time = Instant::now();
            }

            // 模拟难度调整
            if work_count % 10 == 0 {
                self.adjust_difficulty();
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        Ok(())
    }

    /// 设备挖矿任务
    async fn device_mining_task(
        device: Arc<SoftwareDevice>,
        work: Work,
        device_id: u32
    ) -> Result<MiningResult, Box<dyn std::error::Error>> {
        // 模拟挖矿过程
        let mining_time = Duration::from_millis(fastrand::u64(5000..15000));
        tokio::time::sleep(mining_time).await;

        let status = device.get_status().await?;

        Ok(MiningResult {
            device_id,
            work_id: work.work_id,
            nonce: Some(fastrand::u32(..)),
            hash: vec![0u8; 32],
            target_met: fastrand::f64() < 0.1, // 10% 成功率
            hashrate: status.hashrate,
            timestamp: SystemTime::now(),
            shares_accepted: 1,
            shares_rejected: 0,
            hardware_errors: 0,
        })
    }

    /// 更新挖矿统计
    async fn update_mining_stats(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut total_hashrate = 0.0;
        let mut total_power = 0.0;

        for device in &self.devices {
            let status = device.get_status().await?;
            total_hashrate += status.hashrate;
            total_power += status.power_consumption;
        }

        self.mining_stats.uptime = self.start_time.elapsed();
        self.mining_stats.average_hashrate = total_hashrate;
        self.mining_stats.total_hashes += (total_hashrate * 2.0) as u64; // 2秒间隔
        self.mining_stats.power_consumed += total_power / 1000.0 / 3600.0 * 2.0; // kWh
        self.mining_stats.efficiency = if total_power > 0.0 { total_hashrate / total_power } else { 0.0 };

        // 计算预估收益
        self.calculate_estimated_earnings();

        Ok(())
    }

    /// 计算预估收益
    fn calculate_estimated_earnings(&mut self) {
        let network_hashrate = 200_000_000_000_000_000.0; // 200 EH/s
        let blocks_per_day = 144.0; // 平均每天144个区块

        let my_share = self.mining_stats.average_hashrate / network_hashrate;
        let daily_blocks = my_share * blocks_per_day;
        let daily_btc = daily_blocks * self.block_reward * (1.0 - self.pool_config.fee_rate);

        let hours_elapsed = self.mining_stats.uptime.as_secs_f64() / 3600.0;
        self.mining_stats.estimated_earnings = daily_btc * hours_elapsed / 24.0;
    }

    /// 显示挖矿统计
    fn display_mining_stats(&self) {
        println!("\n📊 挖矿统计报告");
        println!("================");
        println!("⏱️  运行时间: {:.1} 小时", self.mining_stats.uptime.as_secs_f64() / 3600.0);
        println!("⚡ 平均算力: {:.2} MH/s", self.mining_stats.average_hashrate / 1_000_000.0);
        println!("🔢 总哈希数: {:.2} M", self.mining_stats.total_hashes as f64 / 1_000_000.0);
        println!("✅ 接受份额: {}", self.mining_stats.shares_accepted);
        println!("❌ 拒绝份额: {}", self.mining_stats.shares_rejected);
        println!("🏆 找到区块: {}", self.mining_stats.blocks_found);
        println!("🔌 功耗效率: {:.0} H/W", self.mining_stats.efficiency);
        println!("⚡ 总耗电量: {:.3} kWh", self.mining_stats.power_consumed);

        // 收益分析
        let electricity_cost_total = self.mining_stats.power_consumed * self.electricity_cost;
        let gross_income = self.mining_stats.estimated_earnings * self.btc_price;
        let net_profit = gross_income - electricity_cost_total;

        println!("\n💰 收益分析:");
        println!("  - 预估BTC收益: {:.8} BTC", self.mining_stats.estimated_earnings);
        println!("  - 毛收入: ${:.2}", gross_income);
        println!("  - 电费成本: ${:.2}", electricity_cost_total);
        println!("  - 净利润: ${:.2}", net_profit);

        if net_profit > 0.0 {
            println!("  - 状态: ✅ 盈利");
        } else {
            println!("  - 状态: ❌ 亏损");
        }
    }

    /// 调整难度
    fn adjust_difficulty(&mut self) {
        let old_difficulty = self.current_difficulty;

        // 根据接受率调整难度
        let total_shares = self.mining_stats.shares_accepted + self.mining_stats.shares_rejected;
        if total_shares > 0 {
            let accept_rate = self.mining_stats.shares_accepted as f64 / total_shares as f64;

            if accept_rate > 0.95 {
                self.current_difficulty *= 1.1; // 提高难度
            } else if accept_rate < 0.85 {
                self.current_difficulty *= 0.9; // 降低难度
            }
        }

        if (self.current_difficulty - old_difficulty).abs() > 0.01 {
            println!("🎯 难度调整: {:.2} -> {:.2}", old_difficulty, self.current_difficulty);
        }
    }
}

/// 创建矿池配置
fn create_pool_config() -> PoolConfig {
    PoolConfig {
        name: "Demo Mining Pool".to_string(),
        url: "stratum+tcp://demo.pool.com".to_string(),
        port: 4444,
        username: "demo_miner.worker1".to_string(),
        password: "x".to_string(),
        difficulty: 1.0,
        fee_rate: 0.01, // 1% 手续费
    }
}

/// 创建设备配置
fn create_device_configs() -> Vec<DeviceConfig> {
    vec![
        DeviceConfig {
            frequency: 2400,
            voltage: 1250,
            fan_speed: Some(70),
            power_limit: Some(180),
            temperature_limit: Some(85),
            auto_fan: true,
            auto_frequency: true,
            auto_voltage: false,
        },
        DeviceConfig {
            frequency: 2200,
            voltage: 1200,
            fan_speed: Some(65),
            power_limit: Some(160),
            temperature_limit: Some(80),
            auto_fan: true,
            auto_frequency: true,
            auto_voltage: false,
        },
    ]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CGMiner CPU BTC Core - 真实挖矿模拟");
    println!("===================================");

    // 创建矿池配置
    let pool_config = create_pool_config();
    let mut simulator = RealMiningSimulator::new(pool_config);

    // 添加挖矿设备
    println!("📱 配置挖矿设备:");
    let device_configs = create_device_configs();

    for (i, config) in device_configs.into_iter().enumerate() {
        simulator.add_device(config, f64::MAX).await?; // 无算力限制
    }

    // 连接矿池
    simulator.connect_to_pool().await?;

    // 开始挖矿模拟
    let mining_duration = Duration::from_secs(300); // 5分钟模拟
    simulator.start_mining(mining_duration).await?;

    // 最终统计报告
    println!("\n🏁 挖矿模拟完成");
    simulator.display_mining_stats();

    println!("\n💡 真实挖矿要点:");
    println!("  - 选择稳定可靠的矿池");
    println!("  - 监控设备温度和功耗");
    println!("  - 计算电费成本和盈利能力");
    println!("  - 定期检查硬件状态");
    println!("  - 关注比特币价格和网络难度变化");

    Ok(())
}
