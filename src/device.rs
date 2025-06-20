//! 软算法设备实现

use cgminer_core::{
    MiningDevice, DeviceInfo, DeviceConfig, DeviceStatus, DeviceStats,
    Work, MiningResult, DeviceError, Temperature, Voltage, Frequency
};
use crate::cpu_affinity::CpuAffinityManager;
use crate::platform_optimization;
use async_trait::async_trait;
use sha2::{Sha256, Digest};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{debug, info, warn};

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
        })
    }

    /// 执行SHA256双重哈希
    fn double_sha256(&self, data: &[u8]) -> Vec<u8> {
        let first_hash = Sha256::digest(data);
        let second_hash = Sha256::digest(&first_hash);
        second_hash.to_vec()
    }

    /// 检查哈希是否满足目标难度
    fn meets_target(&self, hash: &[u8], target: &[u8]) -> bool {
        if hash.len() != target.len() {
            return false;
        }

        for (h, t) in hash.iter().zip(target.iter()) {
            if h < t {
                return true;
            } else if h > t {
                return false;
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

            // 执行真实的SHA256双重哈希计算
            let hash = self.double_sha256(&header_data);
            hashes_done += 1;

            // 检查是否满足目标难度
            let meets_target = self.meets_target(&hash, &work.target);

            // 模拟错误率
            let has_error = fastrand::f64() < self.error_rate;

            if meets_target && !has_error {
                debug!("设备 {} 找到有效解: nonce={:08x}", device_id, nonce);
                found_solution = Some(MiningResult::new(
                    work.id,
                    device_id,
                    nonce,
                    hash,
                    true,
                ));
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

    /// 更新设备温度（基于频率和电压模拟）
    fn update_temperature(&self) -> Result<(), DeviceError> {
        let config = self.config.read().map_err(|e| {
            DeviceError::hardware_error(format!("Failed to acquire read lock: {}", e))
        })?;

        // 基于频率和电压计算模拟温度
        let base_temp = 35.0; // 基础温度
        let freq_factor = config.frequency as f32 / 600.0; // 基准频率600MHz
        let voltage_factor = config.voltage as f32 / 900.0; // 基准电压900mV

        let temp_increase = (freq_factor - 1.0) * 15.0 + (voltage_factor - 1.0) * 10.0;
        let temperature = base_temp + temp_increase + fastrand::f32() * 5.0; // 添加随机波动

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
        info!("初始化软算法设备 {}", self.device_id());

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

    /// 提交工作
    async fn submit_work(&mut self, work: Work) -> Result<(), DeviceError> {
        {
            let mut current_work = self.current_work.lock().await;
            *current_work = Some(work);
        }

        Ok(())
    }

    /// 获取挖矿结果
    async fn get_result(&mut self) -> Result<Option<MiningResult>, DeviceError> {
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
        info!("设置软算法设备 {} 频率为 {} MHz", self.device_id(), frequency);

        {
            let mut config = self.config.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;
            config.frequency = frequency;
        }

        // 更新设备信息
        {
            let mut info = self.device_info.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;
            info.update_frequency(frequency);
        }

        // 根据频率调整目标算力
        let freq_factor = frequency as f64 / 600.0; // 基准频率600MHz
        self.target_hashrate = self.target_hashrate * freq_factor;

        // 更新温度
        self.update_temperature()?;

        Ok(())
    }

    /// 设置电压
    async fn set_voltage(&mut self, voltage: u32) -> Result<(), DeviceError> {
        info!("设置软算法设备 {} 电压为 {} mV", self.device_id(), voltage);

        {
            let mut config = self.config.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;
            config.voltage = voltage;
        }

        // 更新设备信息
        {
            let mut info = self.device_info.write().map_err(|e| {
                DeviceError::hardware_error(format!("Failed to acquire write lock: {}", e))
            })?;
            info.update_voltage(voltage);
        }

        // 更新温度
        self.update_temperature()?;

        Ok(())
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
