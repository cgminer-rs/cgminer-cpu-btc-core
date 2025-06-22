//! 软算法设备实现

use cgminer_core::{
    MiningDevice, DeviceInfo, DeviceConfig, DeviceStatus, DeviceStats,
    Work, MiningResult, DeviceError, Temperature, Voltage, Frequency
};
use crate::cpu_affinity::CpuAffinityManager;
use crate::platform_optimization;
use crate::temperature::{TemperatureManager, TemperatureConfig};
use async_trait::async_trait;
use sha2::{Sha256, Digest};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, mpsc};
use tokio::time::Instant;
use tracing::{debug, info, warn};

/// 优化的SHA256双重哈希计算 - 使用固定大小数组提高性能
#[inline(always)]
fn optimized_double_sha256(data: &[u8]) -> [u8; 32] {
    let first_hash = sha2::Sha256::digest(data);
    let second_hash = sha2::Sha256::digest(&first_hash);
    second_hash.into()
}

/// 挖矿专用结构体 - 用于异步挖矿任务
#[derive(Clone)]
struct SoftwareDeviceForMining {
    device_id: u32,
    device_info: Arc<RwLock<DeviceInfo>>,
    status: Arc<RwLock<DeviceStatus>>,
    stats: Arc<RwLock<DeviceStats>>,
    current_work: Arc<Mutex<Option<Work>>>,
    result_sender: Option<mpsc::UnboundedSender<MiningResult>>,
    batch_size: u32,
    error_rate: f64,
    last_mining_time: Arc<RwLock<Option<Instant>>>,
}

impl SoftwareDeviceForMining {

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

    /// 持续挖矿 - 立即上报找到的解
    async fn continuous_mining(&self, work: Work) -> Result<(), DeviceError> {
        let device_id = self.device_id;
        let mut total_hashes = 0u64;
        let start_time = Instant::now();
        let mut last_stats_update = start_time;
        let mut last_work_check = start_time;
        let mut active_work = work.clone();
        let mut nonce = fastrand::u32(..);

        debug!("设备 {} 开始持续挖矿", device_id);

        loop {
            // 每10秒检查一次新工作，减少锁竞争
            let now = Instant::now();
            if now.duration_since(last_work_check).as_secs() >= 10 {
                if let Ok(work_guard) = self.current_work.try_lock() {
                    if let Some(new_work) = work_guard.clone() {
                        if new_work.id != active_work.id {
                            active_work = new_work;
                            // 重置nonce以开始新的搜索
                            nonce = fastrand::u32(..);
                        }
                    }
                }
                last_work_check = now;
            }

            // 每次循环做一批哈希
            let batch_size = self.batch_size.min(100_000); // 限制批次大小避免阻塞
            let mut header_data = active_work.header;

            for i in 0..batch_size {
                // 使用递增的nonce，确保覆盖更多可能性
                nonce = nonce.wrapping_add(1);

                // 将nonce写入区块头的最后4个字节
                let nonce_bytes = nonce.to_le_bytes();
                header_data[76..80].copy_from_slice(&nonce_bytes);

                // 执行优化的SHA256双重哈希计算
                let hash = optimized_double_sha256(&header_data);
                total_hashes += 1;

                // 检查是否满足目标难度
                if self.meets_target(&hash, &active_work.target) {
                    let result = MiningResult::new(
                        active_work.id.clone(),
                        device_id,
                        nonce,
                        hash.to_vec(),
                        true,
                    );

                    // 立即上报找到的解
                    if let Some(ref sender) = self.result_sender {
                        if let Err(_) = sender.send(result.clone()) {
                            debug!("设备 {} 结果通道已关闭", device_id);
                            return Ok(());
                        }
                    }

                    debug!("💎 设备 {} 持续挖矿找到解: nonce={:08x}", device_id, nonce);

                    // 更新统计信息
                    {
                        let mut stats = self.stats.write().map_err(|e| {
                            DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
                        })?;
                        stats.accepted_work += 1;
                    }
                }

                // 动态CPU让出策略：根据批次大小调整让出频率
                let yield_frequency = if batch_size > 50_000 { 5_000 } else { 10_000 };
                if i % yield_frequency == 0 {
                    tokio::task::yield_now().await;
                }
            }

            // 优化统计更新频率：每5秒更新一次，减少锁竞争
            let now = Instant::now();
            if now.duration_since(last_stats_update).as_secs() >= 5 {
                let elapsed = now.duration_since(start_time).as_secs_f64();
                if let Ok(mut stats) = self.stats.try_write() {
                    stats.update_hashrate(total_hashes, elapsed);
                    last_stats_update = now;
                }
            }

            // 优化设备状态检查：使用try_read减少阻塞
            if let Ok(status) = self.status.try_read() {
                if !matches!(*status, DeviceStatus::Running) {
                    debug!("设备 {} 停止挖矿", device_id);
                    break;
                }
            }
        }

        Ok(())
    }
}

/// 软算法设备
pub struct SoftwareDevice {
    /// 设备信息
    device_info: Arc<RwLock<DeviceInfo>>,
    /// 设备配置
    config: Arc<RwLock<DeviceConfig>>,
    /// 设备状态
    status: Arc<RwLock<DeviceStatus>>,
    /// 设备统计信息
    stats: Arc<RwLock<DeviceStats>>,
    /// 当前工作
    current_work: Arc<Mutex<Option<Work>>>,
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
    /// 挖矿任务句柄
    mining_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl SoftwareDevice {
    /// 创建新的软算法设备
    pub async fn new(
        device_info: DeviceInfo,
        config: DeviceConfig,
        target_hashrate: f64,
        error_rate: f64,
        batch_size: u32,
    ) -> Result<Self, DeviceError> {
        let device_id = device_info.id;
        let stats = DeviceStats::new(device_id);

        // 创建温度管理器（仅在支持真实温度监控时）
        let temp_config = TemperatureConfig::default();
        let temperature_manager = Some(TemperatureManager::new(temp_config));

        Ok(Self {
            device_info: Arc::new(RwLock::new(device_info)),
            config: Arc::new(RwLock::new(config)),
            status: Arc::new(RwLock::new(DeviceStatus::Uninitialized)),
            stats: Arc::new(RwLock::new(stats)),
            current_work: Arc::new(Mutex::new(None)),
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
            mining_handle: Arc::new(Mutex::new(None)),
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
        let stats = DeviceStats::new(device_id);

        // 创建温度管理器
        let temp_config = TemperatureConfig::default();
        let temperature_manager = Some(TemperatureManager::new(temp_config));

        Ok(Self {
            device_info: Arc::new(RwLock::new(device_info)),
            config: Arc::new(RwLock::new(config)),
            status: Arc::new(RwLock::new(DeviceStatus::Uninitialized)),
            stats: Arc::new(RwLock::new(stats)),
            current_work: Arc::new(Mutex::new(None)),
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
            mining_handle: Arc::new(Mutex::new(None)),
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

        // 执行实际的哈希计算循环
        for _ in 0..self.batch_size {
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
        {
            let mut stats = self.stats.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;

            // 更新工作统计
            if found_solution.is_some() {
                stats.accepted_work += 1;
            }

            // 基于实际哈希次数更新算力统计
            stats.update_hashrate(hashes_done, elapsed);
        }

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
        let config = self.config.read().map_err(|e| {
            DeviceError::hardware_error(format!("Failed to acquire read lock: {}", e))
        })?;

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

                        // 更新统计信息中的温度
                        {
                            let mut stats = self.stats.write().map_err(|e| {
                                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
                            })?;
                            stats.temperature = Some(Temperature::new(temperature));
                            stats.voltage = Some(Voltage::new(config.voltage));
                            stats.frequency = Some(Frequency::new(config.frequency));
                            stats.fan_speed = config.fan_speed;
                        }
                    }
                    Err(e) => {
                        debug!("设备 {} 温度读取失败: {}", self.device_id(), e);
                        // 不设置温度信息，让上层知道温度不可用
                        {
                            let mut stats = self.stats.write().map_err(|e| {
                                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
                            })?;
                            stats.temperature = None; // 明确设置为None
                            stats.voltage = Some(Voltage::new(config.voltage));
                            stats.frequency = Some(Frequency::new(config.frequency));
                            stats.fan_speed = config.fan_speed;
                        }
                    }
                }
            } else {
                // 只在第一次检查时输出日志，避免重复日志
                if !self.temperature_capability_checked.load(Ordering::Relaxed) {
                    debug!("设备 {} 不支持温度监控", self.device_id());
                    self.temperature_capability_checked.store(true, Ordering::Relaxed);
                    self.temperature_capability_supported.store(false, Ordering::Relaxed);
                }
                // 不设置温度信息
                {
                    let mut stats = self.stats.write().map_err(|e| {
                        DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
                    })?;
                    stats.temperature = None; // 明确设置为None
                    stats.voltage = Some(Voltage::new(config.voltage));
                    stats.frequency = Some(Frequency::new(config.frequency));
                    stats.fan_speed = config.fan_speed;
                }
            }
        } else {
            // 只在第一次检查时输出日志，避免重复日志
            if !self.temperature_capability_checked.load(Ordering::Relaxed) {
                debug!("设备 {} 没有温度管理器", self.device_id());
                self.temperature_capability_checked.store(true, Ordering::Relaxed);
                self.temperature_capability_supported.store(false, Ordering::Relaxed);
            }
            // 不设置温度信息
            {
                let mut stats = self.stats.write().map_err(|e| {
                    DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
                })?;
                stats.temperature = None; // 明确设置为None
                stats.voltage = Some(Voltage::new(config.voltage));
                stats.frequency = Some(Frequency::new(config.frequency));
                stats.fan_speed = config.fan_speed;
            }
        }

        Ok(())
    }

    /// 为挖矿任务克隆必要的数据
    async fn clone_for_mining(&self) -> Result<SoftwareDeviceForMining, DeviceError> {
        Ok(SoftwareDeviceForMining {
            device_id: self.device_id(),
            device_info: self.device_info.clone(),
            status: self.status.clone(),
            stats: self.stats.clone(),
            current_work: self.current_work.clone(),
            result_sender: self.result_sender.clone(),
            batch_size: self.batch_size,
            error_rate: self.error_rate,
            last_mining_time: self.last_mining_time.clone(),
        })
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

        // 清除当前工作
        {
            let mut work = self.current_work.lock().await;
            *work = None;
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

    /// 提交工作 - 启动持续挖矿
    async fn submit_work(&mut self, work: Work) -> Result<(), DeviceError> {
        let device_id = self.device_id();

        // 更新当前工作
        {
            let mut current_work = self.current_work.lock().await;
            *current_work = Some(work.clone());
        }

        // 如果有结果发送通道，启动持续挖矿
        if self.result_sender.is_some() {
            let mut handle_guard = self.mining_handle.lock().await;
            if handle_guard.is_none() {
                let device_clone = self.clone_for_mining().await?;
                let work_clone = work.clone();

                let handle = tokio::spawn(async move {
                    if let Err(e) = device_clone.continuous_mining(work_clone).await {
                        debug!("设备 {} 挖矿任务结束: {}", device_id, e);
                    }
                });

                *handle_guard = Some(handle);
                debug!("设备 {} 启动持续挖矿任务", device_id);
            }
        }

        Ok(())
    }

    /// 获取挖矿结果
    async fn get_result(&mut self) -> Result<Option<MiningResult>, DeviceError> {
        // 立即上报模式：如果有结果通道，结果通过通道立即上报，这里返回None
        if self.result_sender.is_some() {
            return Ok(None);
        }

        // 传统模式：执行挖矿并返回结果
        let work = {
            let current_work = self.current_work.lock().await;
            current_work.clone()
        };

        if let Some(work) = work {
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

    /// 获取设备统计信息
    async fn get_stats(&self) -> Result<DeviceStats, DeviceError> {
        // 更新运行时间
        if let Some(start_time) = self.start_time {
            let mut stats = self.stats.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;
            stats.uptime = start_time.elapsed();
        }

        let stats = self.stats.read().map_err(|e| {
            DeviceError::hardware_error(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(stats.clone())
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

    /// 重置设备
    async fn reset(&mut self) -> Result<(), DeviceError> {
        info!("重置软算法设备 {}", self.device_id());

        // 重置统计信息
        {
            let mut stats = self.stats.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;
            *stats = DeviceStats::new(self.device_id());
        }

        // 清除当前工作
        {
            let mut work = self.current_work.lock().await;
            *work = None;
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
