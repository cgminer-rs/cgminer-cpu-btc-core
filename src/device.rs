//! # CPU挖矿设备实现
//!
//! 本模块实现了CPU挖矿的虚拟设备抽象，提供完整的设备生命周期管理和性能监控。
//! 每个虚拟设备代表一个独立的挖矿单元，支持真实的SHA256算法计算。
//!
//! ## 🚀 核心组件
//!
//! ### [`AtomicStats`] - 无锁统计系统
//! - ⚡ 原子操作替代读写锁，消除锁竞争
//! - ⚡ 支持哈希率、接受/拒绝工作、硬件错误统计
//! - ⚡ 实时温度和功耗监控
//! - ⚡ 高精度时间戳记录
//!
//! ### [`BatchStatsUpdater`] - 批量统计更新
//! - 📊 本地缓冲减少原子操作频率
//! - 📊 定时批量提交机制
//! - 📊 显著提升高频统计更新性能
//!
//! ### [`SoftwareDevice`] - 主要设备实现
//! - 🔧 完整的MiningDevice trait实现
//! - 🔧 支持CPU亲和性绑定
//! - 🔧 真实系统温度监控
//! - 🔧 CGMiner风格结果上报
//!
//! ### [`HashrateTracker`] - CGMiner兼容算力跟踪
//! - 📈 指数衰减平均算法 (5s/1m/5m/15m)
//! - 📈 CGMiner标准输出格式
//! - 📈 高性能原子操作实现
//!
//! ## 🎯 性能优化特性
//!
//! 1. **无锁并发**: 使用原子操作替代读写锁
//! 2. **批量处理**: 减少高频操作的系统开销
//! 3. **内存优化**: 位级存储浮点数，节省内存
//! 4. **CPU绑定**: 可选的CPU亲和性管理
//! 5. **智能监控**: 缓存检查结果，避免重复操作
//!
//! ## 📊 监控能力
//!
//! - ✅ 真实系统温度读取 (Linux/macOS)
//! - ✅ 详细的挖矿统计信息
//! - ✅ 健康检查和故障检测
//! - ✅ 设备状态生命周期管理
//! - ✅ 错误恢复和重试机制
//!
//! ## 🔄 工作流程
//!
//! ```text
//! 1. 设备初始化 → 配置CPU绑定和温度监控
//! 2. 启动设备   → 开始接收和处理工作
//! 3. 挖矿循环   → 真实SHA256计算和结果检查
//! 4. 统计更新   → 无锁原子操作更新性能数据
//! 5. 结果上报   → CGMiner风格即时或批量上报
//! ```

use cgminer_core::{
    MiningDevice, DeviceInfo, DeviceConfig, DeviceStatus, DeviceStats,
    Work, MiningResult, DeviceError
};
use crate::cpu_affinity::CpuAffinityManager;
use crate::platform_optimization;
use crate::temperature::{TemperatureManager, TemperatureConfig};
use async_trait::async_trait;
use sha2::Digest;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::{debug, info, warn};

/// 原子统计计数器 - 消除锁竞争
/// 替换 Arc<RwLock<DeviceStats>> 以提高并发性能
#[derive(Debug)]
pub struct AtomicStats {
    // 基础统计
    pub total_hashes: AtomicU64,
    pub accepted_work: AtomicU64,
    pub rejected_work: AtomicU64,
    pub hardware_errors: AtomicU64,

    // 性能指标
    pub last_hashrate: AtomicU64, // 存储为 f64 的位模式
    pub average_hashrate: AtomicU64, // 存储为 f64 的位模式

    // 温度和功耗
    pub temperature: AtomicU32, // 存储为 f32 的位模式
    pub power_consumption: AtomicU32, // 存储为 f32 的位模式

    // 时间戳
    pub start_time_nanos: AtomicU64,
    pub last_update_nanos: AtomicU64,

    // 设备ID
    pub device_id: u32,
}

impl AtomicStats {
    pub fn new(device_id: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        Self {
            total_hashes: AtomicU64::new(0),
            accepted_work: AtomicU64::new(0),
            rejected_work: AtomicU64::new(0),
            hardware_errors: AtomicU64::new(0),
            last_hashrate: AtomicU64::new(0.0f64.to_bits()),
            average_hashrate: AtomicU64::new(0.0f64.to_bits()),
            temperature: AtomicU32::new(0.0f32.to_bits()),
            power_consumption: AtomicU32::new(0.0f32.to_bits()),
            start_time_nanos: AtomicU64::new(now),
            last_update_nanos: AtomicU64::new(now),
            device_id,
        }
    }

    /// 原子更新哈希率 - 无锁操作
    pub fn update_hashrate(&self, hashes: u64, elapsed_secs: f64) {
        // 原子更新总哈希数
        self.total_hashes.fetch_add(hashes, Ordering::Relaxed);

        // 计算当前哈希率
        let current_hashrate = if elapsed_secs > 0.0 {
            hashes as f64 / elapsed_secs
        } else {
            0.0
        };

        // 原子更新最新哈希率
        self.last_hashrate.store(current_hashrate.to_bits(), Ordering::Relaxed);

        // 计算并更新平均哈希率
        let total = self.total_hashes.load(Ordering::Relaxed);
        let start_time = self.start_time_nanos.load(Ordering::Relaxed);
        let now_nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let total_elapsed = (now_nanos - start_time) as f64 / 1_000_000_000.0;
        let avg_hashrate = if total_elapsed > 0.0 {
            total as f64 / total_elapsed
        } else {
            0.0
        };

        self.average_hashrate.store(avg_hashrate.to_bits(), Ordering::Relaxed);
        self.last_update_nanos.store(now_nanos, Ordering::Relaxed);
    }

    /// 原子增加接受的工作数
    pub fn increment_accepted(&self) {
        self.accepted_work.fetch_add(1, Ordering::Relaxed);
    }

    /// 原子增加拒绝的工作数
    pub fn increment_rejected(&self) {
        self.rejected_work.fetch_add(1, Ordering::Relaxed);
    }

    /// 原子增加硬件错误数
    pub fn increment_hardware_errors(&self) {
        self.hardware_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// 原子更新温度
    pub fn update_temperature(&self, temp: f32) {
        self.temperature.store(temp.to_bits(), Ordering::Relaxed);
    }

    /// 原子更新功耗
    pub fn update_power_consumption(&self, power: f64) {
        self.power_consumption.store(power.to_bits() as u32, Ordering::Relaxed);
    }

    /// 转换为 DeviceStats 结构体 - 用于兼容现有API
    pub fn to_device_stats(&self) -> DeviceStats {
        let mut stats = DeviceStats::new(self.device_id);

        stats.total_hashes = self.total_hashes.load(Ordering::Relaxed);
        stats.accepted_work = self.accepted_work.load(Ordering::Relaxed);
        stats.rejected_work = self.rejected_work.load(Ordering::Relaxed);
        stats.hardware_errors = self.hardware_errors.load(Ordering::Relaxed);

        // 从位模式恢复浮点数 - 映射到正确的字段
        let current_hashrate = f64::from_bits(self.last_hashrate.load(Ordering::Relaxed));
        let average_hashrate = f64::from_bits(self.average_hashrate.load(Ordering::Relaxed));

        stats.current_hashrate = cgminer_core::HashRate::new(current_hashrate);
        stats.average_hashrate = cgminer_core::HashRate::new(average_hashrate);

        let temp_bits = self.temperature.load(Ordering::Relaxed);
        let power_bits = self.power_consumption.load(Ordering::Relaxed);

        if temp_bits != 0 {
            stats.temperature = Some(cgminer_core::Temperature::new(f32::from_bits(temp_bits)));
        }

        if power_bits != 0 {
            stats.power_consumption = Some(f32::from_bits(power_bits) as f64);
        }

        // 更新时间戳
        let _start_nanos = self.start_time_nanos.load(Ordering::Relaxed);
        let update_nanos = self.last_update_nanos.load(Ordering::Relaxed);

        // 注意：DeviceStats中没有start_time字段，只有last_updated
        stats.last_updated = SystemTime::UNIX_EPOCH + Duration::from_nanos(update_nanos);

        stats
    }

    /// 重置所有统计数据
    pub fn reset(&self) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.total_hashes.store(0, Ordering::Relaxed);
        self.accepted_work.store(0, Ordering::Relaxed);
        self.rejected_work.store(0, Ordering::Relaxed);
        self.hardware_errors.store(0, Ordering::Relaxed);
        self.last_hashrate.store(0.0f64.to_bits(), Ordering::Relaxed);
        self.average_hashrate.store(0.0f64.to_bits(), Ordering::Relaxed);
        self.temperature.store(0.0f32.to_bits(), Ordering::Relaxed);
        self.power_consumption.store(0.0f32.to_bits(), Ordering::Relaxed);
        self.start_time_nanos.store(now, Ordering::Relaxed);
        self.last_update_nanos.store(now, Ordering::Relaxed);
    }
}

/// 批量统计更新器 - 减少原子操作频率
#[derive(Debug)]
pub struct BatchStatsUpdater {
    atomic_stats: Arc<AtomicStats>,
    local_hashes: u64,
    local_accepted: u64,
    local_rejected: u64,
    local_errors: u64,
    last_flush: Instant,
    batch_interval: Duration,
}

impl BatchStatsUpdater {
    pub fn new(atomic_stats: Arc<AtomicStats>, batch_interval_ms: u64) -> Self {
        Self {
            atomic_stats,
            local_hashes: 0,
            local_accepted: 0,
            local_rejected: 0,
            local_errors: 0,
            last_flush: Instant::now(),
            batch_interval: Duration::from_millis(batch_interval_ms),
        }
    }

    /// 本地累积哈希数
    pub fn add_hashes(&mut self, count: u64) {
        self.local_hashes += count;
        self.try_flush();
    }

    /// 本地累积接受数
    pub fn add_accepted(&mut self, count: u64) {
        self.local_accepted += count;
        self.try_flush();
    }

    /// 本地累积拒绝数
    pub fn add_rejected(&mut self, count: u64) {
        self.local_rejected += count;
        self.try_flush();
    }

    /// 本地累积错误数
    pub fn add_errors(&mut self, count: u64) {
        self.local_errors += count;
        self.try_flush();
    }

    /// 尝试批量提交统计数据
    fn try_flush(&mut self) {
        if self.last_flush.elapsed() >= self.batch_interval {
            self.force_flush();
        }
    }

    /// 强制批量提交统计数据
    pub fn force_flush(&mut self) {
        if self.local_hashes > 0 {
            let elapsed = self.last_flush.elapsed().as_secs_f64();
            self.atomic_stats.update_hashrate(self.local_hashes, elapsed);
            self.local_hashes = 0;
        }

        if self.local_accepted > 0 {
            for _ in 0..self.local_accepted {
                self.atomic_stats.increment_accepted();
            }
            self.local_accepted = 0;
        }

        if self.local_rejected > 0 {
            for _ in 0..self.local_rejected {
                self.atomic_stats.increment_rejected();
            }
            self.local_rejected = 0;
        }

        if self.local_errors > 0 {
            for _ in 0..self.local_errors {
                self.atomic_stats.increment_hardware_errors();
            }
            self.local_errors = 0;
        }

        self.last_flush = Instant::now();
    }
}

/// 优化的SHA256双重哈希计算 - 使用固定大小数组提高性能
#[inline(always)]
fn optimized_double_sha256(data: &[u8]) -> [u8; 32] {
    let first_hash = sha2::Sha256::digest(data);
    let second_hash = sha2::Sha256::digest(&first_hash);
    second_hash.into()
}

/// 软算法设备（阶段2优化版本）
pub struct SoftwareDevice {
    /// 设备信息
    device_info: Arc<RwLock<DeviceInfo>>,
    /// 设备配置
    config: Arc<RwLock<DeviceConfig>>,
    /// 设备状态
    status: Arc<RwLock<DeviceStatus>>,
    /// 原子统计信息 - 替换RwLock<DeviceStats>消除锁竞争
    atomic_stats: Arc<AtomicStats>,
    /// 无锁工作队列 - 替换Mutex<Option<Work>>
    work_queue: Arc<crate::concurrent_optimization::LockFreeWorkQueue>,
    /// 目标算力 (hashes per second)
    target_hashrate: f64,
    /// 错误率
    error_rate: f64,
    /// 批次大小
    batch_size: u32,
    /// 启动时间
    start_time: Option<Instant>,
    /// 最后一次挖矿时间
    last_mining_time: Arc<RwLock<Option<Instant>>>,
    /// CPU绑定管理器
    cpu_affinity: Option<Arc<RwLock<CpuAffinityManager>>>,
    /// 温度管理器
    temperature_manager: Option<TemperatureManager>,
    /// 缓存温度监控能力检查结果，避免重复检查和日志输出
    temperature_capability_checked: Arc<AtomicBool>,
    temperature_capability_supported: Arc<AtomicBool>,
    /// cgminer风格结果发送通道 - 立即上报
    result_sender: Option<mpsc::UnboundedSender<MiningResult>>,

    /// 批量统计更新器
    batch_stats_updater: Arc<std::sync::Mutex<BatchStatsUpdater>>,
}

impl SoftwareDevice {
    /// 创建新的软算法设备（阶段2优化版本）
    pub async fn new(
        device_info: DeviceInfo,
        config: DeviceConfig,
        target_hashrate: f64,
        error_rate: f64,
        batch_size: u32,
    ) -> Result<Self, DeviceError> {
        let device_id = device_info.id;

        // 创建原子统计 - 替换RwLock<DeviceStats>
        let atomic_stats = Arc::new(AtomicStats::new(device_id));

        // 创建无锁工作队列 - 替换Mutex<Option<Work>>
        let work_queue = Arc::new(crate::concurrent_optimization::LockFreeWorkQueue::new(1000));

        // 创建批量统计更新器
        let batch_stats_updater = Arc::new(std::sync::Mutex::new(
            BatchStatsUpdater::new(atomic_stats.clone(), 100) // 每100ms批量更新
        ));

        // 创建温度管理器（仅在支持真实温度监控时）
        let temp_config = TemperatureConfig::default();
        let temperature_manager = Some(TemperatureManager::new(temp_config));

        Ok(Self {
            device_info: Arc::new(RwLock::new(device_info)),
            config: Arc::new(RwLock::new(config)),
            status: Arc::new(RwLock::new(DeviceStatus::Uninitialized)),
            atomic_stats,
            work_queue,
            target_hashrate,
            error_rate,
            batch_size,
            start_time: None,
            last_mining_time: Arc::new(RwLock::new(None)),
            cpu_affinity: None,
            temperature_manager,
            temperature_capability_checked: Arc::new(AtomicBool::new(false)),
            temperature_capability_supported: Arc::new(AtomicBool::new(false)),
            result_sender: None,
            batch_stats_updater,
        })
    }

    /// 创建带CPU绑定的软算法设备
    pub async fn new_with_cpu_affinity(
        device_info: DeviceInfo,
        config: DeviceConfig,
        target_hashrate: f64,
        error_rate: f64,
        batch_size: u32,
        cpu_affinity: Arc<RwLock<CpuAffinityManager>>,
    ) -> Result<Self, DeviceError> {
        let device_id = device_info.id;

        // 创建原子统计
        let atomic_stats = Arc::new(AtomicStats::new(device_id));

        // 创建无锁工作队列
        let work_queue = Arc::new(crate::concurrent_optimization::LockFreeWorkQueue::new(1000));

        // 创建批量统计更新器
        let batch_stats_updater = Arc::new(std::sync::Mutex::new(
            BatchStatsUpdater::new(atomic_stats.clone(), 100)
        ));

        // 创建温度管理器
        let temp_config = TemperatureConfig::default();
        let temperature_manager = Some(TemperatureManager::new(temp_config));

        Ok(Self {
            device_info: Arc::new(RwLock::new(device_info)),
            config: Arc::new(RwLock::new(config)),
            status: Arc::new(RwLock::new(DeviceStatus::Uninitialized)),
            atomic_stats,
            work_queue,
            target_hashrate,
            error_rate,
            batch_size,
            start_time: None,
            last_mining_time: Arc::new(RwLock::new(None)),
            cpu_affinity: Some(cpu_affinity),
            temperature_manager,
            temperature_capability_checked: Arc::new(AtomicBool::new(false)),
            temperature_capability_supported: Arc::new(AtomicBool::new(false)),
            result_sender: None,
            batch_stats_updater,
        })
    }

    /// 设置结果发送通道 - 立即上报
    pub fn set_result_sender(&mut self, sender: mpsc::UnboundedSender<MiningResult>) {
        self.result_sender = Some(sender);
    }

    /// 检查哈希是否满足目标难度
    fn meets_target(&self, hash: &[u8], target: &[u8]) -> bool {
        debug_assert_eq!(hash.len(), 32);
        debug_assert_eq!(target.len(), 32);
        unsafe {
            let hash_u64 = &*(hash.as_ptr() as *const [u64; 4]);
            let target_u64 = &*(target.as_ptr() as *const [u64; 4]);
            for i in 0..4 {
                let h = u64::from_be(hash_u64[i]);
                let t = u64::from_be(target_u64[i]);
                if h < t { return true; }
                if h > t { return false; }
            }
        }
        false
    }

    /// 执行真实的挖矿过程（基于实际哈希次数）
    async fn mine_work(&self, work: &Work) -> Result<Option<MiningResult>, DeviceError> {
        let device_id = self.device_id();

        let start_time = Instant::now();
        let mut hashes_done = 0u64;
        let mut found_solution = None;

        // 根据目标算力计算批次大小
        let target_hashes_per_second = self.target_hashrate;
        let adjusted_batch_size = if target_hashes_per_second > 0.0 {
            (target_hashes_per_second / 10.0).max(self.batch_size as f64).min(self.batch_size as f64 * 2.0) as u32
        } else {
            self.batch_size
        };

        // 执行实际的哈希计算循环
        for _ in 0..adjusted_batch_size {
            // 生成随机nonce
            let nonce = fastrand::u32(..);

            // 构建区块头数据
            let mut header_data = work.header.clone();
            if header_data.len() >= 4 {
                // 将nonce写入区块头的最后4个字节
                let nonce_bytes = nonce.to_le_bytes();
                let start_idx = header_data.len() - 4;
                header_data[start_idx..].copy_from_slice(&nonce_bytes);
            }

            // 执行优化的SHA256双重哈希计算
            let hash = optimized_double_sha256(&header_data);
            hashes_done += 1;

            // 检查是否满足目标难度
            let meets_target = self.meets_target(&hash, &work.target);

            // 模拟错误率
            let has_error = fastrand::f64() < self.error_rate;

            if meets_target && !has_error {
                let result = MiningResult::new(
                    work.id,
                    device_id,
                    nonce,
                    hash.to_vec(),
                    true,
                );

                // 立即上报找到的解
                if let Some(ref sender) = self.result_sender {
                    if let Err(_) = sender.send(result.clone()) {
                        debug!("设备 {} 结果通道已关闭", device_id);
                        return Ok(None);
                    }
                    debug!("💎 设备 {} 立即上报解: nonce={:08x}", device_id, nonce);
                } else {
                    // 如果没有通道，保持原有行为
                    debug!("设备 {} 找到有效解: nonce={:08x}", device_id, nonce);
                    found_solution = Some(result);
                }
                break; // 找到解后退出循环
            }

            // 使用平台特定的CPU让出策略优化
            if hashes_done % platform_optimization::get_platform_yield_frequency() == 0 {
                tokio::task::yield_now().await;
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();

        // 更新统计信息
        // 使用原子统计更新 - 无锁操作
        if found_solution.is_some() {
            self.atomic_stats.increment_accepted();
        }

        // 基于实际哈希次数更新算力统计
        self.atomic_stats.update_hashrate(hashes_done, elapsed);

        // 更新最后挖矿时间
        {
            let mut last_time = self.last_mining_time.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;
            *last_time = Some(Instant::now());
        }

        Ok(found_solution)
    }

    /// 更新设备温度（仅支持真实温度读取）
    fn update_temperature(&self) -> Result<(), DeviceError> {

        // 尝试从温度管理器读取真实温度
        if let Some(ref temp_manager) = self.temperature_manager {
            if temp_manager.has_temperature_monitoring() {
                match temp_manager.read_temperature() {
                    Ok(temperature) => {
                        debug!("设备 {} 读取到真实温度: {:.1}°C", self.device_id(), temperature);

                        // 更新设备信息中的温度
                        {
                            let mut info = self.device_info.write().map_err(|e| {
                                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
                            })?;
                            info.update_temperature(temperature);
                        }

                        // 更新统计信息中的温度 - 使用原子操作
                        self.atomic_stats.update_temperature(temperature);
                    }
                    Err(e) => {
                        debug!("设备 {} 温度读取失败: {}", self.device_id(), e);
                        // 不设置温度信息，让上层知道温度不可用
                        // 对于原子统计，温度读取失败时保持默认值
                    }
                }
            } else {
                // 只在第一次检查时输出日志，避免重复日志
                if !self.temperature_capability_checked.load(Ordering::Relaxed) {
                    debug!("设备 {} 不支持温度监控", self.device_id());
                    self.temperature_capability_checked.store(true, Ordering::Relaxed);
                    self.temperature_capability_supported.store(false, Ordering::Relaxed);
                }
                // 对于原子统计，不支持温度监控时保持默认值
            }
        } else {
            // 只在第一次检查时输出日志，避免重复日志
            if !self.temperature_capability_checked.load(Ordering::Relaxed) {
                debug!("设备 {} 没有温度管理器", self.device_id());
                self.temperature_capability_checked.store(true, Ordering::Relaxed);
                self.temperature_capability_supported.store(false, Ordering::Relaxed);
            }
            // 对于原子统计，没有温度管理器时保持默认值
        }

        Ok(())
    }
}

#[async_trait]
impl MiningDevice for SoftwareDevice {
    /// 获取设备ID
    fn device_id(&self) -> u32 {
        // 直接读取设备ID，避免在测试环境中使用block_in_place
        self.device_info.read().unwrap().id
    }

    /// 获取设备信息
    async fn get_info(&self) -> Result<DeviceInfo, DeviceError> {
        let info = self.device_info.read().map_err(|e| {
            DeviceError::hardware_error(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(info.clone())
    }

    /// 初始化设备
    async fn initialize(&mut self, config: DeviceConfig) -> Result<(), DeviceError> {
        debug!("初始化软算法设备 {}", self.device_id());

        // 更新配置
        {
            let mut device_config = self.config.write().map_err(|e| {
                DeviceError::initialization_failed(format!("Failed to acquire write lock: {}", e))
            })?;
            *device_config = config;
        }

        // 更新状态
        {
            let mut status = self.status.write().map_err(|e| {
                DeviceError::initialization_failed(format!("Failed to acquire write lock: {}", e))
            })?;
            *status = DeviceStatus::Idle;
        }

        // 更新温度
        self.update_temperature()?;

        // 显示温度监控信息（只在初始化时显示一次）
        if let Some(ref temp_manager) = self.temperature_manager {
            if temp_manager.has_temperature_monitoring() {
                info!("设备 {} 温度监控: ✅ 真实监控 ({})",
                    self.device_id(),
                    temp_manager.provider_info()
                );
                self.temperature_capability_checked.store(true, Ordering::Relaxed);
                self.temperature_capability_supported.store(true, Ordering::Relaxed);
            } else {
                info!("设备 {} 温度监控: ❌ 不支持 ({})",
                    self.device_id(),
                    temp_manager.provider_info()
                );
                self.temperature_capability_checked.store(true, Ordering::Relaxed);
                self.temperature_capability_supported.store(false, Ordering::Relaxed);
            }
        } else {
            info!("设备 {} 温度监控: ❌ 未配置", self.device_id());
            self.temperature_capability_checked.store(true, Ordering::Relaxed);
            self.temperature_capability_supported.store(false, Ordering::Relaxed);
        }

        info!("软算法设备 {} 初始化完成", self.device_id());
        Ok(())
    }

    /// 启动设备
    async fn start(&mut self) -> Result<(), DeviceError> {
        let device_id = self.device_id();
        info!("启动软算法设备 {}", device_id);

        // 如果启用了CPU绑定，为当前线程设置CPU绑定
        if let Some(cpu_affinity) = &self.cpu_affinity {
            let affinity_manager = cpu_affinity.read().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire read lock: {}", e))
            })?;

            if let Err(e) = affinity_manager.bind_current_thread(device_id) {
                warn!("设备 {} CPU绑定失败: {}", device_id, e);
                // CPU绑定失败不应该阻止设备启动，只是记录警告
            } else {
                info!("✅ 设备 {} 已绑定到指定CPU核心", device_id);
            }
        }

        {
            let mut status = self.status.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;
            *status = DeviceStatus::Running;
        }

        self.start_time = Some(Instant::now());
        info!("软算法设备 {} 启动完成", device_id);
        Ok(())
    }

    /// 停止设备
    async fn stop(&mut self) -> Result<(), DeviceError> {
        info!("停止软算法设备 {}", self.device_id());

        {
            let mut status = self.status.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;
            *status = DeviceStatus::Idle;
        }

        // 清除工作队列中的旧工作
        let cleared_count = self.work_queue.clear_stale_work(0); // 清除所有旧工作
        if cleared_count > 0 {
            debug!("设备 {} 停止时清除了 {} 个旧工作", self.device_id(), cleared_count);
        }

        info!("软算法设备 {} 已停止", self.device_id());
        Ok(())
    }

    /// 重启设备
    async fn restart(&mut self) -> Result<(), DeviceError> {
        info!("重启软算法设备 {}", self.device_id());
        self.stop().await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.start().await?;
        Ok(())
    }

    /// 提交工作到设备（简化版本 - 移除复杂的任务管理）
    async fn submit_work(&mut self, work: Work) -> Result<(), DeviceError> {
        let device_id = self.device_id();

        // 使用无锁工作队列提交工作
        match self.work_queue.enqueue_work(work.clone()) {
            Ok(()) => {
                debug!("设备 {} 成功提交工作到队列: {}", device_id, work.id);
                Ok(())
            }
            Err(rejected_work) => {
                warn!("设备 {} 工作队列已满，丢弃工作: {}", device_id, rejected_work.id);
                // 队列满了不算错误，只是警告
                Ok(())
            }
        }
    }

    /// 获取挖矿结果
    async fn get_result(&mut self) -> Result<Option<MiningResult>, DeviceError> {
        // 立即上报模式：如果有结果通道，结果通过通道立即上报，这里返回None
        if self.result_sender.is_some() {
            return Ok(None);
        }

        // 传统模式：从工作队列获取工作并返回结果
        if let Some(work) = self.work_queue.dequeue_work() {
            // 更新温度
            self.update_temperature()?;

            // 执行挖矿
            let result = self.mine_work(&work).await?;

            Ok(result)
        } else {
            // 没有工作 - 这是正常的
            Ok(None)
        }
    }

    /// 获取设备状态
    async fn get_status(&self) -> Result<DeviceStatus, DeviceError> {
        let status = self.status.read().map_err(|e| {
            DeviceError::hardware_error(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(status.clone())
    }

    /// 获取设备统计信息（阶段2优化 - 使用原子统计）
    async fn get_stats(&self) -> Result<DeviceStats, DeviceError> {
        // 强制刷新批量统计更新器
        if let Ok(mut updater) = self.batch_stats_updater.try_lock() {
            updater.force_flush();
        }

        // 使用原子统计 - 无锁操作，消除读写锁竞争
        let mut stats = self.atomic_stats.to_device_stats();

        // 更新运行时间
        if let Some(start_time) = self.start_time {
            stats.uptime = start_time.elapsed();
        }

        // 获取工作队列统计信息
        let queue_stats = self.work_queue.get_stats();
        debug!(
            "设备 {} 队列统计: 待处理={}, 活跃={}, 已完成={}",
            self.device_id(),
            queue_stats.pending_count,
            queue_stats.active_count,
            queue_stats.completed_count
        );

        Ok(stats)
    }

    /// 设置频率
    async fn set_frequency(&mut self, frequency: u32) -> Result<(), DeviceError> {
        // 软算法核心不支持硬件级别的频率设置
        warn!("软算法设备 {} 不支持频率设置 (请求: {} MHz)，CPU挖矿无法调整硬件频率",
              self.device_id(), frequency);

        Err(DeviceError::hardware_error(
            "软算法核心不支持频率设置，CPU挖矿无法调整硬件频率".to_string()
        ))
    }

    /// 设置电压
    async fn set_voltage(&mut self, voltage: u32) -> Result<(), DeviceError> {
        // 软算法核心不支持硬件级别的电压设置
        warn!("软算法设备 {} 不支持电压设置 (请求: {} mV)，CPU挖矿无法调整硬件电压",
              self.device_id(), voltage);

        Err(DeviceError::hardware_error(
            "软算法核心不支持电压设置，CPU挖矿无法调整硬件电压".to_string()
        ))
    }

    /// 设置风扇速度
    async fn set_fan_speed(&mut self, speed: u32) -> Result<(), DeviceError> {
        info!("设置软算法设备 {} 风扇速度为 {}%", self.device_id(), speed);

        {
            let mut config = self.config.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;
            config.fan_speed = Some(speed);
        }

        // 更新设备信息
        {
            let mut info = self.device_info.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;
            info.fan_speed = Some(speed);
            info.updated_at = SystemTime::now();
        }

        Ok(())
    }

    /// 重置设备（阶段2优化 - 使用原子统计）
    async fn reset(&mut self) -> Result<(), DeviceError> {
        info!("重置软算法设备 {}", self.device_id());

        // 重置原子统计信息 - 无锁操作
        self.atomic_stats.reset();

        // 重置批量统计更新器
        if let Ok(mut updater) = self.batch_stats_updater.try_lock() {
            updater.force_flush();
        }

        // 清空工作队列中的过期工作
        let new_version = self.work_queue.update_work_version();
        let cleared_count = self.work_queue.clear_stale_work(new_version);
        if cleared_count > 0 {
            info!("设备 {} 重置时清理了 {} 个过期工作", self.device_id(), cleared_count);
        }

        // 重置时间
        self.start_time = Some(Instant::now());

        info!("软算法设备 {} 重置完成", self.device_id());
        Ok(())
    }

    /// 获取设备健康状态
    async fn health_check(&self) -> Result<bool, DeviceError> {
        let status = self.get_status().await?;
        let stats = self.get_stats().await?;

        // 检查设备状态
        let status_ok = matches!(status, DeviceStatus::Running | DeviceStatus::Idle);

        // 检查温度
        let temp_ok = if let Some(temp) = stats.temperature {
            temp.celsius < 90.0 // 温度不超过90度
        } else {
            true
        };

        // 检查错误率
        let error_rate_ok = stats.error_rate() < 0.1; // 错误率不超过10%

        Ok(status_ok && temp_ok && error_rate_ok)
    }
}

// 🔧 CGMiner风格的算力追踪器
#[derive(Debug)]
pub struct HashrateTracker {
    total_hashes: AtomicU64,
    start_time: std::time::Instant,
    last_update_time: AtomicU64, // 纳秒时间戳

    // 指数衰减平均算力 (哈希/秒)
    avg_5s: AtomicU64,   // f64 as u64 bits
    avg_1m: AtomicU64,
    avg_5m: AtomicU64,
    avg_15m: AtomicU64,

    // 统计数据
    accepted_shares: AtomicU64,
    rejected_shares: AtomicU64,
    hardware_errors: AtomicU64,
}

impl HashrateTracker {
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            total_hashes: AtomicU64::new(0),
            start_time: now,
            last_update_time: AtomicU64::new(now.elapsed().as_nanos() as u64),
            avg_5s: AtomicU64::new(0),
            avg_1m: AtomicU64::new(0),
            avg_5m: AtomicU64::new(0),
            avg_15m: AtomicU64::new(0),
            accepted_shares: AtomicU64::new(0),
            rejected_shares: AtomicU64::new(0),
            hardware_errors: AtomicU64::new(0),
        }
    }

    /// 添加哈希数 - 挖矿线程调用，最小开销
    pub fn add_hashes(&self, hashes: u64) {
        self.total_hashes.fetch_add(hashes, Ordering::Relaxed);
    }

    /// 更新指数衰减平均算力 - 统计线程调用
    pub fn update_averages(&self) {
        let now_nanos = self.start_time.elapsed().as_nanos() as u64;
        let last_update = self.last_update_time.load(Ordering::Relaxed);

        if now_nanos <= last_update {
            return; // 避免时间倒流
        }

        let elapsed_secs = (now_nanos - last_update) as f64 / 1_000_000_000.0;
        if elapsed_secs < 0.1 {
            return; // 更新太频繁，跳过
        }

        let total_hashes = self.total_hashes.load(Ordering::Relaxed);
        let total_elapsed = self.start_time.elapsed().as_secs_f64();

        if total_elapsed <= 0.0 {
            return;
        }

        // 当前瞬时算力
        let current_hashrate = total_hashes as f64 / total_elapsed;

        // 指数衰减因子 (基于cgminer的实现)
        let alpha_5s = 1.0 - (-elapsed_secs / 5.0).exp();
        let alpha_1m = 1.0 - (-elapsed_secs / 60.0).exp();
        let alpha_5m = 1.0 - (-elapsed_secs / 300.0).exp();
        let alpha_15m = 1.0 - (-elapsed_secs / 900.0).exp();

        // 更新指数衰减平均值
        self.update_ema(&self.avg_5s, current_hashrate, alpha_5s);
        self.update_ema(&self.avg_1m, current_hashrate, alpha_1m);
        self.update_ema(&self.avg_5m, current_hashrate, alpha_5m);
        self.update_ema(&self.avg_15m, current_hashrate, alpha_15m);

        // 更新时间戳
        self.last_update_time.store(now_nanos, Ordering::Relaxed);
    }

    fn update_ema(&self, atomic_avg: &AtomicU64, current_value: f64, alpha: f64) {
        let old_bits = atomic_avg.load(Ordering::Relaxed);
        let old_value = if old_bits == 0 {
            current_value // 初始值
        } else {
            f64::from_bits(old_bits)
        };

        let new_value = old_value + alpha * (current_value - old_value);
        atomic_avg.store(new_value.to_bits(), Ordering::Relaxed);
    }

    /// 获取CGMiner风格的算力字符串
    pub fn get_cgminer_hashrate_string(&self) -> String {
        let avg_5s = f64::from_bits(self.avg_5s.load(Ordering::Relaxed));
        let avg_1m = f64::from_bits(self.avg_1m.load(Ordering::Relaxed));
        let avg_5m = f64::from_bits(self.avg_5m.load(Ordering::Relaxed));
        let avg_15m = f64::from_bits(self.avg_15m.load(Ordering::Relaxed));

        let total_hashes = self.total_hashes.load(Ordering::Relaxed);
        let total_elapsed = self.start_time.elapsed().as_secs_f64();
        let avg_total = if total_elapsed > 0.0 {
            total_hashes as f64 / total_elapsed
        } else {
            0.0
        };

        let accepted = self.accepted_shares.load(Ordering::Relaxed);
        let rejected = self.rejected_shares.load(Ordering::Relaxed);
        let hw_errors = self.hardware_errors.load(Ordering::Relaxed);

        format!(
            "(5s):{:.2}M (1m):{:.2}M (5m):{:.2}M (15m):{:.2}M (avg):{:.2}Mh/s A:{} R:{} HW:{}",
            avg_5s / 1_000_000.0,
            avg_1m / 1_000_000.0,
            avg_5m / 1_000_000.0,
            avg_15m / 1_000_000.0,
            avg_total / 1_000_000.0,
            accepted,
            rejected,
            hw_errors
        )
    }

    pub fn increment_accepted(&self) {
        self.accepted_shares.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_rejected(&self) {
        self.rejected_shares.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_hardware_error(&self) {
        self.hardware_errors.fetch_add(1, Ordering::Relaxed);
    }
}
