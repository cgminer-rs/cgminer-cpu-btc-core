//! 基本挖矿演示 - CGMiner风格优化版本
//!
//! 这个示例展示CGMiner风格的算力上报机制：
//! - 指数衰减平均算法 (5s/1m/5m/15m)
//! - 优化的挖矿循环 (大批次，减少中断)
//! - 时间驱动的统计更新
//! - 真实的CGMiner输出格式

// 移除未使用的导入
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, Duration};
use tokio::time::sleep;
use tracing::info;
use sha2::{Sha256, Digest}; // 添加真实的SHA256库

/// CGMiner风格算力跟踪器 - 基于指数衰减平均
#[derive(Debug)]
struct CGMinerHashrateTracker {
    // 原子计数器 - 挖矿线程只更新这些
    total_hashes: AtomicU64,
    start_time: Instant,
    last_update_time: AtomicU64, // 微秒时间戳

    // 指数衰减平均值 (存储为 f64 的位表示)
    avg_5s: AtomicU64,
    avg_1m: AtomicU64,
    avg_5m: AtomicU64,
    avg_15m: AtomicU64,

    // 工作统计
    accepted_shares: AtomicU64,
    rejected_shares: AtomicU64,
    hardware_errors: AtomicU64,
}

impl CGMinerHashrateTracker {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            total_hashes: AtomicU64::new(0),
            start_time: now,
            last_update_time: AtomicU64::new(0),
            avg_5s: AtomicU64::new(0),
            avg_1m: AtomicU64::new(0),
            avg_5m: AtomicU64::new(0),
            avg_15m: AtomicU64::new(0),
            accepted_shares: AtomicU64::new(0),
            rejected_shares: AtomicU64::new(0),
            hardware_errors: AtomicU64::new(0),
        }
    }

    /// 挖矿线程调用 - 仅累加哈希数
    fn add_hashes(&self, hash_count: u64) {
        self.total_hashes.fetch_add(hash_count, Ordering::Relaxed);
    }

    /// 添加工作结果
    fn add_work_result(&self, accepted: bool) {
        if accepted {
            self.accepted_shares.fetch_add(1, Ordering::Relaxed);
        } else {
            self.rejected_shares.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 统计线程调用 - 计算指数衰减平均
    fn update_averages(&self) {
        let now_micros = self.start_time.elapsed().as_micros() as u64;
        let last_update_micros = self.last_update_time.load(Ordering::Relaxed);

        if last_update_micros == 0 {
            // 首次更新
            self.last_update_time.store(now_micros, Ordering::Relaxed);
            return;
        }

        let elapsed_secs = (now_micros - last_update_micros) as f64 / 1_000_000.0;
        if elapsed_secs < 0.1 {
            return; // 避免过于频繁的更新
        }

        // 计算当前瞬时算力
        let total_hashes = self.total_hashes.load(Ordering::Relaxed) as f64;
        let total_elapsed = self.start_time.elapsed().as_secs_f64();
        let current_hashrate = if total_elapsed > 0.0 {
            total_hashes / total_elapsed
        } else {
            0.0
        };

        // CGMiner的指数衰减算法
        // alpha = 1.0 - exp(-elapsed_secs / window_secs)
        self.update_exponential_average(&self.avg_5s, current_hashrate, elapsed_secs, 5.0);
        self.update_exponential_average(&self.avg_1m, current_hashrate, elapsed_secs, 60.0);
        self.update_exponential_average(&self.avg_5m, current_hashrate, elapsed_secs, 300.0);
        self.update_exponential_average(&self.avg_15m, current_hashrate, elapsed_secs, 900.0);

        self.last_update_time.store(now_micros, Ordering::Relaxed);
    }

    fn update_exponential_average(&self, avg_atomic: &AtomicU64, current_value: f64, elapsed_secs: f64, window_secs: f64) {
        let old_bits = avg_atomic.load(Ordering::Relaxed);
        let old_value = if old_bits == 0 {
            current_value // 首次设置
        } else {
            f64::from_bits(old_bits)
        };

        // CGMiner的指数衰减公式
        let alpha = 1.0 - (-elapsed_secs / window_secs).exp();
        let new_value = old_value + alpha * (current_value - old_value);

        avg_atomic.store(new_value.to_bits(), Ordering::Relaxed);
    }

    /// 获取CGMiner风格的格式化输出
    fn format_cgminer_output(&self, device_count: u32) -> String {
        let avg_5s = f64::from_bits(self.avg_5s.load(Ordering::Relaxed)) / 1_000_000.0;
        let avg_1m = f64::from_bits(self.avg_1m.load(Ordering::Relaxed)) / 1_000_000.0;
        let avg_5m = f64::from_bits(self.avg_5m.load(Ordering::Relaxed)) / 1_000_000.0;
        let avg_15m = f64::from_bits(self.avg_15m.load(Ordering::Relaxed)) / 1_000_000.0;

        let accepted = self.accepted_shares.load(Ordering::Relaxed);
        let rejected = self.rejected_shares.load(Ordering::Relaxed);
        let hw_errors = self.hardware_errors.load(Ordering::Relaxed);

        // CGMiner标准格式: (5s):X.XXXMh/s (1m):X.XXXMh/s (5m):X.XXXMh/s (15m):X.XXXMh/s A:XX R:XX HW:XX [XDEV]
        format!("(5s):{:.3}Mh/s (1m):{:.3}Mh/s (5m):{:.3}Mh/s (15m):{:.3}Mh/s A:{} R:{} HW:{} [{}DEV]",
                avg_5s, avg_1m, avg_5m, avg_15m, accepted, rejected, hw_errors, device_count)
    }

    /// 获取总算力
    fn get_total_hashrate(&self) -> f64 {
        let total_hashes = self.total_hashes.load(Ordering::Relaxed) as f64;
        let total_elapsed = self.start_time.elapsed().as_secs_f64();
        if total_elapsed > 0.0 {
            total_hashes / total_elapsed
        } else {
            0.0
        }
    }
}

/// 真实的比特币区块头结构
#[derive(Debug, Clone)]
struct BlockHeader {
    version: u32,
    prev_block_hash: [u8; 32],
    merkle_root: [u8; 32],
    timestamp: u32,
    bits: u32,
    nonce: u32,
}

impl BlockHeader {
    /// 创建用于测试的区块头
    fn new_test_header() -> Self {
        Self {
            version: 1,
            prev_block_hash: [0u8; 32], // 简化的前一个区块哈希
            merkle_root: [0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf, 0x10,
                         0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20], // 示例Merkle根
            timestamp: 1640995200, // 2022-01-01 00:00:00 UTC
            bits: 0x1d00ffff,      // 简化的难度目标
            nonce: 0,
        }
    }

    /// 将区块头序列化为80字节的数组
    fn to_bytes(&self) -> [u8; 80] {
        let mut bytes = [0u8; 80];

        // 版本号 (4字节，小端)
        bytes[0..4].copy_from_slice(&self.version.to_le_bytes());

        // 前一个区块哈希 (32字节)
        bytes[4..36].copy_from_slice(&self.prev_block_hash);

        // Merkle根 (32字节)
        bytes[36..68].copy_from_slice(&self.merkle_root);

        // 时间戳 (4字节，小端)
        bytes[68..72].copy_from_slice(&self.timestamp.to_le_bytes());

        // 难度目标 (4字节，小端)
        bytes[72..76].copy_from_slice(&self.bits.to_le_bytes());

        // Nonce (4字节，小端)
        bytes[76..80].copy_from_slice(&self.nonce.to_le_bytes());

        bytes
    }

    /// 计算比特币双重SHA256哈希
    fn calculate_hash(&self) -> [u8; 32] {
        let header_bytes = self.to_bytes();

        // 第一次SHA256
        let mut hasher = Sha256::new();
        hasher.update(&header_bytes);
        let first_hash = hasher.finalize();

        // 第二次SHA256
        let mut hasher = Sha256::new();
        hasher.update(&first_hash);
        let second_hash = hasher.finalize();

        let mut result = [0u8; 32];
        result.copy_from_slice(&second_hash);
        result
    }

    /// 检查哈希是否满足难度目标
    fn check_target(&self, target_leading_zeros: u8) -> bool {
        let hash = self.calculate_hash();

        // 检查前导零的数量
        let mut leading_zeros = 0u8;
        for byte in hash.iter() {
            if *byte == 0 {
                leading_zeros += 8;
            } else {
                leading_zeros += byte.leading_zeros() as u8;
                break;
            }
        }

        leading_zeros >= target_leading_zeros
    }
}

/// 优化的挖矿模拟器 - 大批次计算，最小化统计中断
struct OptimizedMiningSimulator {
    tracker: Arc<CGMinerHashrateTracker>,
    device_count: u32,
    target_hashrate_per_device: f64, // 每个设备的目标算力 (H/s)
}

impl OptimizedMiningSimulator {
    fn new(tracker: Arc<CGMinerHashrateTracker>, device_count: u32, target_hashrate_per_device: f64) -> Self {
        Self {
            tracker,
            device_count,
            target_hashrate_per_device,
        }
    }

    /// 启动优化的挖矿循环
    async fn start_mining(&self, duration_secs: u64) {
        let end_time = Instant::now() + Duration::from_secs(duration_secs);

        // 为每个设备启动一个任务
        let mut handles = Vec::new();

        for device_id in 0..self.device_count {
            let tracker = self.tracker.clone();
            let target_hashrate = self.target_hashrate_per_device;

            let handle = tokio::spawn(async move {
                Self::device_mining_loop(device_id, tracker, target_hashrate, end_time).await;
            });

            handles.push(handle);
        }

        // 等待所有设备完成
        for handle in handles {
            let _ = handle.await;
        }

        info!("✅ 所有设备挖矿循环完成");
    }

    /// 单个设备的挖矿循环 - 使用真实的SHA256哈希
    async fn device_mining_loop(
        device_id: u32,
        tracker: Arc<CGMinerHashrateTracker>,
        target_hashrate: f64,
        end_time: Instant,
    ) {
        const BATCH_SIZE: u64 = 100_000; // 大批次，减少统计开销
        const TARGET_DIFFICULTY: u8 = 20; // 目标难度：20个前导零位 (大约1/2^20的概率)

        info!("📱 设备 {} 开始挖矿，目标算力: {:.2} MH/s, 难度: {} 前导零位",
              device_id, target_hashrate / 1_000_000.0, TARGET_DIFFICULTY);

        let mut total_hashes = 0u64;
        let start_time = Instant::now();
        let mut base_header = BlockHeader::new_test_header();

        // 为每个设备设置不同的时间戳，避免重复工作
        base_header.timestamp = base_header.timestamp.wrapping_add(device_id);

        while Instant::now() < end_time {
            let batch_start = Instant::now();

            // 🔥 真实的比特币挖矿过程 🔥
            for i in 0..BATCH_SIZE {
                // 设置当前nonce
                base_header.nonce = i as u32;

                // 计算真实的SHA256双重哈希
                let _hash = base_header.calculate_hash();

                // 检查是否满足难度目标
                if base_header.check_target(TARGET_DIFFICULTY) {
                    // 找到有效的哈希！
                    // info!("💎 设备 {} 找到有效哈希！Nonce: {}, 哈希: {}",
                    //       device_id, base_header.nonce, hex::encode(&hash));
                    tracker.add_work_result(true);
                }
            }

            // 如果需要控制算力，可以取消注释以下代码
            // let target_batch_duration = Duration::from_secs_f64(BATCH_SIZE as f64 / target_hashrate);
            // let actual_duration = batch_start.elapsed();
            // if actual_duration < target_batch_duration {
            //     sleep(target_batch_duration - actual_duration).await;
            // }

            // 批次完成后，原子性地更新统计
            tracker.add_hashes(BATCH_SIZE);
            total_hashes += BATCH_SIZE;

            // 更新区块头时间戳，模拟新的工作
            base_header.timestamp = base_header.timestamp.wrapping_add(1);

            let batch_duration = batch_start.elapsed();
            let actual_hashrate = BATCH_SIZE as f64 / batch_duration.as_secs_f64();

            // 每1000个批次输出一次设备状态 (避免日志污染)
            if total_hashes % (BATCH_SIZE * 1000) == 0 {
                let avg_hashrate = total_hashes as f64 / start_time.elapsed().as_secs_f64();
                info!("⚡ 设备 {} 状态: 总哈希={:.1}M, 平均算力={:.2}MH/s, 瞬时算力={:.2}MH/s",
                      device_id,
                      total_hashes as f64 / 1_000_000.0,
                      avg_hashrate / 1_000_000.0,
                      actual_hashrate / 1_000_000.0);
            }

            // 💡 适当让出CPU控制权
            if total_hashes % 10000 == 0 {
                tokio::task::yield_now().await;
            }
        }

        let final_hashrate = total_hashes as f64 / start_time.elapsed().as_secs_f64();
        info!("🏁 设备 {} 完成挖矿: 总哈希={:.1}M, 平均算力={:.2}MH/s",
              device_id,
              total_hashes as f64 / 1_000_000.0,
              final_hashrate / 1_000_000.0);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 CGMiner风格真实SHA256挖矿演示");
    info!("📊 特性: 真实双重SHA256、指数衰减平均、大批次计算、时间驱动统计");
    info!("⚠️  注意: 这是真实的SHA256计算，会消耗更多CPU资源");

    // 配置参数
    let device_count = 16; // 使用16个设备，如用户要求
    let target_hashrate_per_device = 2_500_000.0; // 2.5 MH/s per device
    let total_target_hashrate = target_hashrate_per_device * device_count as f64;
    let mining_duration_secs = 120; // 2分钟演示

    info!("💻 配置: {} 个设备, 目标总算力: {:.1} MH/s",
          device_count, total_target_hashrate / 1_000_000.0);
    info!("🔍 挖矿难度: 20个前导零位 (约1/1048576的概率找到有效哈希)");
    info!("📝 使用真实的比特币区块头结构和SHA256双重哈希");

    // 创建CGMiner风格算力跟踪器
    let tracker = Arc::new(CGMinerHashrateTracker::new());

    // 创建挖矿模拟器
    let simulator = OptimizedMiningSimulator::new(
        tracker.clone(),
        device_count,
        target_hashrate_per_device,
    );

    // 启动统计更新线程 - 每1秒更新一次指数衰减平均
    let stats_tracker = tracker.clone();
    let stats_handle = tokio::spawn(async move {
        let mut last_cgminer_output = Instant::now();

        loop {
            // 更新指数衰减平均
            stats_tracker.update_averages();

            // 每5秒输出CGMiner风格的算力报告
            if last_cgminer_output.elapsed().as_secs() >= 5 {
                println!("{}", stats_tracker.format_cgminer_output(device_count));
                last_cgminer_output = Instant::now();
            }

            sleep(Duration::from_secs(1)).await;
        }
    });

    info!("📈 CGMiner风格输出格式:");
    info!("    (5s):X.XXXMh/s (1m):X.XXXMh/s (5m):X.XXXMh/s (15m):X.XXXMh/s A:XX R:XX HW:XX [XDEV]");
    info!("⛏️  开始真实SHA256挖矿...");

    // 等待1秒让统计线程启动
    sleep(Duration::from_secs(1)).await;

    // 启动挖矿模拟
    let mining_start = Instant::now();
    simulator.start_mining(mining_duration_secs).await;
    let mining_duration = mining_start.elapsed();

    // 停止统计线程
    stats_handle.abort();

    // 最终统计
    let final_hashrate = tracker.get_total_hashrate();
    let total_hashes = tracker.total_hashes.load(Ordering::Relaxed);
    let accepted = tracker.accepted_shares.load(Ordering::Relaxed);

    info!("🏁 真实SHA256挖矿完成！");
    info!("📊 最终统计:");
    info!("    ⏱️  运行时间: {:.1} 秒", mining_duration.as_secs_f64());
    info!("    🔢 总哈希数: {:.1} M", total_hashes as f64 / 1_000_000.0);
    info!("    ⚡ 平均算力: {:.2} MH/s", final_hashrate / 1_000_000.0);
    info!("    💎 找到有效哈希: {}", accepted);
    info!("    📈 目标算力: {:.1} MH/s", total_target_hashrate / 1_000_000.0);
    info!("    📊 算力达成率: {:.1}%", (final_hashrate / total_target_hashrate) * 100.0);

    if accepted > 0 {
        info!("    🎉 成功率: {:.6}% (找到{}/{}哈希)",
              accepted as f64 / total_hashes as f64 * 100.0,
              accepted,
              total_hashes);
    } else {
        info!("    ℹ️  本次演示未找到有效哈希（这很正常，因为难度较高）");
    }

    // 最后一次CGMiner输出
    println!("\n🎯 最终CGMiner输出:");
    println!("{}", tracker.format_cgminer_output(device_count));

    info!("💡 提示: 真实挖矿会根据网络难度调整，目标是约10分钟出一个区块");

    Ok(())
}
