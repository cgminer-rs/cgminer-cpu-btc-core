//! # CPU比特币挖矿核心实现
//!
//! 本模块实现了专门用于CPU比特币挖矿的核心算法和管理功能。
//! 该实现遵循cgminer-core标准接口，提供高性能的CPU挖矿能力。
//!
//! ## 🚀 核心特性
//!
//! ### 挖矿算法
//! - ✅ 真实SHA256双重哈希计算
//! - ✅ 比特币区块头完整结构
//! - ✅ 多设备并行挖矿支持
//! - ✅ 智能设备数量管理 (自动限制为CPU核心数)
//!
//! ### 性能优化
//! - ⚡ CPU亲和性绑定 (可选)
//! - ⚡ 性能优化器集成
//! - ⚡ 批量工作处理
//! - ⚡ 平台特定优化
//!
//! ### 监控功能
//! - 📊 实时统计信息收集
//! - 📊 CGMiner风格结果上报
//! - 📊 健康检查和错误恢复
//! - 📊 详细的设备状态跟踪
//!
//! ## 📦 主要组件
//!
//! - [`SoftwareMiningCore`]: 主要的挖矿核心实现
//! - 设备管理: 支持最多64个虚拟设备
//! - 结果收集: 支持即时上报和批量收集
//! - 配置管理: 支持环境变量和配置文件
//!
//! ## 🎯 设计特点
//!
//! 1. **自适应设备数量**: 自动根据CPU核心数调整设备数量
//! 2. **灵活配置**: 支持多种算力范围和错误率配置
//! 3. **高兼容性**: 完全兼容cgminer-core接口标准
//! 4. **企业级特性**: 完整的生命周期管理和错误处理

use cgminer_core::{
    MiningCore, CoreInfo, CoreCapabilities, CoreConfig, CoreStats, CoreError,
    DeviceInfo, MiningDevice, Work, MiningResult,
    TemperatureCapabilities, VoltageCapabilities, FrequencyCapabilities,
    FanCapabilities, CpuSpecificCapabilities, CpuCacheInfo
};
use crate::device::SoftwareDevice;
use crate::performance::PerformanceOptimizer;
use crate::cpu_affinity::{CpuAffinityManager, CpuAffinityStrategy};
// 平台优化模块
use crate::platform_optimization;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, mpsc};
use tracing::{info, warn, error, debug};

/// 软算法挖矿核心
pub struct SoftwareMiningCore {
    /// 核心信息
    core_info: CoreInfo,
    /// 核心能力
    capabilities: CoreCapabilities,
    /// 核心配置
    config: Option<CoreConfig>,
    /// 设备列表
    devices: Arc<Mutex<HashMap<u32, Box<dyn MiningDevice>>>>,
    /// 核心统计信息
    stats: Arc<RwLock<CoreStats>>,
    /// 是否正在运行
    running: Arc<RwLock<bool>>,
    /// 启动时间
    start_time: Option<SystemTime>,
    /// 性能优化器
    performance_optimizer: Option<PerformanceOptimizer>,
    /// CPU绑定管理器
    cpu_affinity_manager: Option<Arc<RwLock<CpuAffinityManager>>>,
    /// cgminer风格结果通道 - 立即上报
    result_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<MiningResult>>>>,
    result_sender: Option<mpsc::UnboundedSender<MiningResult>>,
    /// 收集到的结果缓存
    collected_results: Arc<Mutex<Vec<MiningResult>>>,
}

impl SoftwareMiningCore {
    /// 创建新的软算法挖矿核心
    pub fn new(name: String) -> Self {
        let core_info = CoreInfo::new(
            name.clone(),
            cgminer_core::CoreType::Custom("optimized_cpu".to_string()),
            crate::VERSION.to_string(),
            "优化CPU挖矿核心，支持SIMD加速、智能温度管理和动态负载均衡".to_string(),
            "CGMiner Rust Team".to_string(),
            vec!["optimized_cpu".to_string(), "simd".to_string(), "cpu".to_string()],
        );

        let capabilities = CoreCapabilities {
            supports_auto_tuning: false,
            temperature_capabilities: TemperatureCapabilities {
                supports_monitoring: true,  // CPU可以监控温度
                supports_control: false,    // CPU无法直接控制温度
                supports_threshold_alerts: true,
                monitoring_precision: Some(1.0), // 1度精度
            },
            voltage_capabilities: VoltageCapabilities {
                supports_monitoring: false, // CPU软算法无法监控电压
                supports_control: false,    // CPU软算法无法控制电压
                control_range: None,
            },
            frequency_capabilities: FrequencyCapabilities {
                supports_monitoring: false, // CPU软算法无法监控频率
                supports_control: false,    // CPU软算法无法控制频率
                control_range: None,
            },
            fan_capabilities: FanCapabilities {
                supports_monitoring: false, // CPU软算法无法监控风扇
                supports_control: false,    // CPU软算法无法控制风扇
                fan_count: None,
            },
            supports_multiple_chains: true,
            max_devices: Some(64), // 软算法核心支持最多64个设备
            supported_algorithms: vec!["SHA256".to_string(), "SHA256d".to_string()],
            cpu_capabilities: Some(CpuSpecificCapabilities {
                simd_support: vec!["SSE".to_string(), "AVX".to_string(), "AVX2".to_string(), "SHA".to_string()], // 优化SIMD支持
                supports_cpu_affinity: true,  // 支持CPU绑定
                supports_numa_awareness: true, // 优化版本支持NUMA
                physical_cores: num_cpus::get_physical() as u32,
                logical_cores: num_cpus::get() as u32,
                cache_info: Some(CpuCacheInfo {
                    l1_data_kb: 32,
                    l1_instruction_kb: 32,
                    l2_kb: 256,
                    l3_kb: 8192,
                }),
            }),
            core_type: cgminer_core::CoreType::Custom("optimized_cpu".to_string()),
        };

        let stats = CoreStats::new(name);

        // 创建cgminer风格的结果通道
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            core_info,
            capabilities,
            config: None,
            devices: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(RwLock::new(stats)),
            running: Arc::new(RwLock::new(false)),
            start_time: None,
            performance_optimizer: None,
            cpu_affinity_manager: None,
            result_receiver: Arc::new(Mutex::new(Some(receiver))),
            result_sender: Some(sender),
            collected_results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 创建软算法设备
    async fn create_software_devices(&self, config: &CoreConfig) -> Result<Vec<Box<dyn MiningDevice>>, CoreError> {
        let mut devices = Vec::new();

        // 从配置中获取设备数量（支持环境变量覆盖）
        let requested_device_count = self.get_device_count_from_config_with_params(config);

        // CPU挖矿优化：限制设备数量为CPU核心数，避免不必要的开销
        let cpu_cores = num_cpus::get() as u32;
        let device_count = if requested_device_count > cpu_cores {
            info!("⚠️  请求的设备数量 {} 超过CPU核心数 {}，自动限制为CPU核心数以获得最佳性能",
                  requested_device_count, cpu_cores);
            cpu_cores
        } else {
            requested_device_count
        };

        info!("实际设备数量: {} (CPU核心数: {})", device_count, cpu_cores);
        debug!("完整配置参数: {:?}", config.custom_params);

        // 获取算力范围 - 提高到您期望的35MH/s水平
        let min_hashrate = config.custom_params
            .get("min_hashrate")
            .and_then(|v| v.as_f64())
            .unwrap_or(30_000_000.0); // 30 MH/s

        let max_hashrate = config.custom_params
            .get("max_hashrate")
            .and_then(|v| v.as_f64())
            .unwrap_or(40_000_000.0); // 40 MH/s

        let error_rate = config.custom_params
            .get("error_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.01); // 1%

        let batch_size = config.custom_params
            .get("batch_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1_000_000) as u32; // 增加批次大小到100万，提高实际算力

        info!("🔥 创建 {} 个优化CPU设备 (CPU核心数: {})，算力范围: {:.2} - {:.2} MH/s",
              device_count,
              cpu_cores,
              min_hashrate / 1_000_000.0,
              max_hashrate / 1_000_000.0);

        for i in 0..device_count {
            // 为每个设备分配不同的算力
            let device_hashrate = min_hashrate +
                (max_hashrate - min_hashrate) * (i as f64 / device_count.max(1) as f64);

            let mut device_config = if (i as usize) < config.devices.len() {
                config.devices[i as usize].clone()
            } else {
                cgminer_core::DeviceConfig {
                    chain_id: i as u8,
                    enabled: true,
                    frequency: 600 + (i * 50), // 递增频率
                    voltage: 900 + (i * 20),   // 递增电压
                    auto_tune: false,
                    chip_count: 64,
                    temperature_limit: 80.0,
                    fan_speed: Some(50 + i * 5),
                }
            };

            // 应用性能优化
            if let Some(optimizer) = &self.performance_optimizer {
                optimizer.apply_to_device_config(&mut device_config, 1000 + i);
            }

            let device_info = DeviceInfo::new(
                1000 + i,
                format!("Software Device {}", i),
                "software".to_string(),
                i as u8,
            );

            let mut device = if let Some(cpu_affinity) = &self.cpu_affinity_manager {
                // 为CPU绑定管理器分配设备
                {
                    let mut affinity_manager = cpu_affinity.write().map_err(|e| {
                        CoreError::runtime(format!("Failed to acquire write lock: {}", e))
                    })?;
                    affinity_manager.assign_cpu_core(1000 + i);
                }

                SoftwareDevice::new_with_cpu_affinity(
                    device_info,
                    device_config,
                    device_hashrate,
                    error_rate,
                    batch_size,
                    cpu_affinity.clone(),
                ).await?
            } else {
                SoftwareDevice::new(
                    device_info,
                    device_config,
                    device_hashrate,
                    error_rate,
                    batch_size,
                ).await?
            };

            // 设置cgminer风格的结果发送通道
            if let Some(ref sender) = self.result_sender {
                device.set_result_sender(sender.clone());
            }

            devices.push(Box::new(device) as Box<dyn MiningDevice>);
        }

        Ok(devices)
    }

    /// 更新核心统计信息 - 核心层负责算力计算
    async fn update_stats(&self) -> Result<(), CoreError> {
        let devices = self.devices.lock().await;
        let mut total_hashrate = 0.0;
        let mut total_accepted = 0;
        let mut total_rejected = 0;
        let mut total_errors = 0;
        let mut active_devices = 0;
        let mut total_hashes = 0u64;

        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        for device in devices.values() {
            // 获取设备的原始统计数据
            if let Ok(device_stats) = device.get_stats().await {
                total_accepted += device_stats.accepted_work;
                total_rejected += device_stats.rejected_work;
                total_errors += device_stats.hardware_errors;
                total_hashes += device_stats.total_hashes;
                active_devices += 1;

                // 如果设备支持原始数据获取，计算设备算力
                // 注意：这里需要设备提供原始数据接口，暂时使用现有数据
                let device_hashrate = device_stats.current_hashrate.hashes_per_second;
                total_hashrate += device_hashrate;
            }
        }

        // 计算核心级别的算力
        let core_start_time = self.start_time.map(|t|
            t.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
        ).unwrap_or(current_time);

        let total_elapsed_secs = (current_time - core_start_time) as f64 / 1_000_000_000.0;
        let core_average_hashrate = if total_elapsed_secs > 0.0 {
            total_hashes as f64 / total_elapsed_secs
        } else {
            0.0
        };

        let mut stats = self.stats.write().map_err(|e| {
            CoreError::runtime(format!("Failed to acquire write lock: {}", e))
        })?;

        stats.device_count = devices.len() as u32;
        stats.active_devices = active_devices;
        stats.total_hashrate = total_hashrate; // 当前算力（所有设备当前算力之和）
        stats.average_hashrate = core_average_hashrate; // 核心平均算力（基于总哈希数计算）
        stats.accepted_work = total_accepted;
        stats.rejected_work = total_rejected;
        stats.hardware_errors = total_errors;

        if let Some(start_time) = self.start_time {
            stats.uptime = SystemTime::now()
                .duration_since(start_time)
                .unwrap_or(Duration::from_secs(0));
        }

        stats.last_updated = SystemTime::now();

        debug!("核心统计更新: 设备数={}, 活跃={}, 当前算力={:.2} H/s, 平均算力={:.2} H/s",
               stats.device_count, stats.active_devices, stats.total_hashrate, stats.average_hashrate);

        Ok(())
    }

    /// 从配置获取设备数量（带配置参数）
    fn get_device_count_from_config_with_params(&self, config: &CoreConfig) -> u32 {
        // 优先级：环境变量 > 配置文件 > 默认值

        // 1. 检查环境变量
        if let Ok(count_str) = std::env::var("CGMINER_SOFTWARE_DEVICE_COUNT") {
            if let Ok(count) = count_str.parse::<u32>() {
                if count > 0 && count <= 1000 {
                    info!("从环境变量读取优化CPU设备数量: {}", count);
                    return count;
                } else {
                    warn!("环境变量中的设备数量 {} 超出范围，使用配置文件值", count);
                }
            }
        }

        // 2. 从传入的配置参数读取
        debug!("传入的配置参数: {:?}", config.custom_params);
        if let Some(device_count) = config.custom_params.get("device_count") {
            debug!("找到device_count参数: {:?}", device_count);
            if let Some(count) = device_count.as_u64() {
                let count = count as u32;
                debug!("解析device_count为: {}", count);
                if count > 0 && count <= 1000 {
                    info!("从配置文件读取优化CPU设备数量: {}", count);
                    return count;
                } else {
                    warn!("配置文件中的设备数量 {} 超出范围，使用默认值", count);
                }
            } else {
                warn!("device_count参数无法解析为数字: {:?}", device_count);
            }
        } else {
            warn!("未找到device_count参数，可用参数: {:?}", config.custom_params.keys().collect::<Vec<_>>());
        }

        // 3. 使用默认值
        info!("使用默认优化CPU设备数量: 4");
        4u32
    }

    /// 从配置获取设备数量
    fn get_device_count_from_config(&self) -> u32 {
        // 优先级：环境变量 > 配置文件 > 默认值

        // 1. 检查环境变量
        if let Ok(count_str) = std::env::var("CGMINER_SOFTWARE_DEVICE_COUNT") {
            if let Ok(count) = count_str.parse::<u32>() {
                if count > 0 && count <= 1000 {
                    info!("从环境变量读取优化CPU设备数量: {}", count);
                    return count;
                } else {
                    warn!("环境变量中的设备数量 {} 超出范围，使用配置文件值", count);
                }
            }
        }

        // 2. 从配置文件读取
        if let Some(config) = &self.config {
            if let Some(device_count) = config.custom_params.get("device_count") {
                if let Some(count) = device_count.as_u64() {
                    let count = count as u32;
                    if count > 0 && count <= 1000 {
                        info!("从配置文件读取优化CPU设备数量: {}", count);
                        return count;
                    } else {
                        warn!("配置文件中的设备数量 {} 超出范围，使用默认值", count);
                    }
                }
            }
        }

        // 3. 使用默认值
        info!("使用默认优化CPU设备数量: 4");
        4u32
    }

    /// 启动立即上报的结果收集任务
    async fn start_result_collection(&self) -> Result<(), CoreError> {
        let receiver = {
            let mut receiver_guard = self.result_receiver.lock().await;
            receiver_guard.take()
        };

        if let Some(mut receiver) = receiver {
            let collected_results = self.collected_results.clone();
            let stats = self.stats.clone();

            tokio::spawn(async move {
                while let Some(result) = receiver.recv().await {
                    // 立即处理收到的结果
                    debug!("💎 设备 {} 找到解: nonce={:08x}",
                          result.device_id, result.nonce);

                    // 更新统计信息
                    {
                        let mut stats_guard = stats.write().unwrap();
                        stats_guard.accepted_work += 1;
                    }

                    // 缓存结果供collect_results使用
                    {
                        let mut results_guard = collected_results.lock().await;
                        results_guard.push(result);
                    }
                }
            });

            info!("立即上报结果收集任务已启动");
        }

        Ok(())
    }
}

#[async_trait]
impl MiningCore for SoftwareMiningCore {
    /// 获取核心信息
    fn get_info(&self) -> &CoreInfo {
        &self.core_info
    }

    /// 获取核心能力
    fn get_capabilities(&self) -> &CoreCapabilities {
        &self.capabilities
    }

    /// 初始化核心
    async fn initialize(&mut self, config: CoreConfig) -> Result<(), CoreError> {
        info!("开始初始化优化CPU挖矿核心: {}", config.name);
        debug!("配置参数: {:?}", config.custom_params);

        // 打印平台信息
        info!("🚀 平台信息: {}", platform_optimization::get_platform_info());
        if platform_optimization::supports_high_performance() {
            info!("✅ 当前平台支持高性能优化");
        } else {
            info!("⚠️  当前平台性能优化有限");
        }

        // 验证配置
        debug!("验证配置...");
        self.validate_config(&config)?;
        debug!("配置验证通过");

        // 初始化性能优化器
        let mut perf_config = crate::performance::PerformanceConfig::default();
        let mut optimizer = PerformanceOptimizer::new(perf_config.clone());
        optimizer.optimize_for_system();
        perf_config = optimizer.get_config().clone();
        self.performance_optimizer = Some(optimizer);

        // 初始化CPU绑定管理器
        if perf_config.cpu_affinity.enabled {
            let strategy = CpuAffinityStrategy::Intelligent; // 简化为固定策略

            let cpu_manager = CpuAffinityManager::new(true, strategy);
            self.cpu_affinity_manager = Some(Arc::new(RwLock::new(cpu_manager)));
            info!("✅ CPU绑定管理器已启用，策略: 智能分配");
        }

        // 创建设备
        debug!("开始创建优化CPU设备...");
        let devices = self.create_software_devices(&config).await?;
        info!("优化CPU设备创建完成，共创建 {} 个设备", devices.len());

        // 存储设备
        {
            let mut device_map = self.devices.lock().await;
            for device in devices {
                let device_id = device.device_id();
                device_map.insert(device_id, device);
            }
        }

        // 初始化所有设备
        {
            let mut device_map = self.devices.lock().await;
            for (device_id, device) in device_map.iter_mut() {
                let device_config = config.devices
                    .iter()
                    .find(|dc| dc.chain_id == (*device_id - 1000) as u8)
                    .cloned()
                    .unwrap_or_default();

                if let Err(e) = device.initialize(device_config).await {
                    error!("初始化设备 {} 失败: {}", device_id, e);
                    return Err(CoreError::Device(e));
                }
            }
        }

        self.config = Some(config);

        // 检查设备数量
        let device_count = {
            let devices = self.devices.lock().await;
            devices.len()
        };
        debug!("最终设备数量: {}", device_count);

        info!("优化CPU挖矿核心初始化完成");
        Ok(())
    }

    /// 启动核心
    async fn start(&mut self) -> Result<(), CoreError> {
        info!("启动优化CPU挖矿核心");

        {
            let mut running = self.running.write().map_err(|e| {
                CoreError::runtime(format!("Failed to acquire write lock: {}", e))
            })?;

            if *running {
                return Err(CoreError::runtime("核心已经在运行中"));
            }
            *running = true;
        }

        // 启动立即上报的结果收集任务
        self.start_result_collection().await?;

        // 启动所有设备
        {
            let mut devices = self.devices.lock().await;
            for (device_id, device) in devices.iter_mut() {
                if let Err(e) = device.start().await {
                    error!("启动设备 {} 失败: {}", device_id, e);
                    // 继续启动其他设备，不因为一个设备失败而停止
                }
            }
        }

        self.start_time = Some(SystemTime::now());
        info!("优化CPU挖矿核心启动完成 - 立即上报已启用");
        Ok(())
    }

    /// 停止核心
    async fn stop(&mut self) -> Result<(), CoreError> {
        info!("停止优化CPU挖矿核心");

        {
            let mut running = self.running.write().map_err(|e| {
                CoreError::runtime(format!("Failed to acquire write lock: {}", e))
            })?;
            *running = false;
        }

        // 停止所有设备
        {
            let mut devices = self.devices.lock().await;
            for (device_id, device) in devices.iter_mut() {
                if let Err(e) = device.stop().await {
                    error!("停止设备 {} 失败: {}", device_id, e);
                }
            }
        }

        info!("优化CPU挖矿核心已停止");
        Ok(())
    }

    /// 重启核心
    async fn restart(&mut self) -> Result<(), CoreError> {
        info!("重启优化CPU挖矿核心");
        self.stop().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.start().await?;
        Ok(())
    }

    /// 扫描设备
    async fn scan_devices(&self) -> Result<Vec<DeviceInfo>, CoreError> {
        debug!("扫描优化CPU设备");

        // 如果设备已经创建，返回现有设备信息
        let devices = self.devices.lock().await;
        if !devices.is_empty() {
            let mut device_infos = Vec::new();
            for device in devices.values() {
                match device.get_info().await {
                    Ok(info) => device_infos.push(info),
                    Err(e) => warn!("获取设备信息失败: {}", e),
                }
            }
            return Ok(device_infos);
        }
        drop(devices);

        // 如果设备未创建，根据配置生成应该创建的设备信息
        let requested_device_count = self.get_device_count_from_config();

        // CPU挖矿优化：限制设备数量为CPU核心数
        let cpu_cores = num_cpus::get() as u32;
        let device_count = if requested_device_count > cpu_cores {
            info!("⚠️  环境变量设置的设备数量 {} 超过CPU核心数 {}，自动限制为CPU核心数",
                  requested_device_count, cpu_cores);
            cpu_cores
        } else {
            requested_device_count
        };

        info!("扫描到 {} 个软算法设备 (CPU核心数: {})", device_count, cpu_cores);

        let mut device_infos = Vec::new();
        for i in 0..device_count {
            let device_info = DeviceInfo::new(
                1000 + i, // 软算法设备ID范围: 1000-1999
                format!("Software Device {}", i),
                "software".to_string(),
                i as u8,
            );
            device_infos.push(device_info);
        }

        Ok(device_infos)
    }

    /// 创建设备
    async fn create_device(&self, device_info: DeviceInfo) -> Result<Box<dyn MiningDevice>, CoreError> {
        info!("创建软算法设备: {}", device_info.name);

        let device_config = cgminer_core::DeviceConfig::default();

        // 从配置中获取参数，如果没有配置则使用合理的默认值
        let default_config = CoreConfig::default();
        let config = self.config.as_ref().unwrap_or(&default_config);

        let target_hashrate = config.custom_params
            .get("max_hashrate")
            .and_then(|v| v.as_f64())
            .unwrap_or(2_000_000_000.0); // 2 GH/s 默认算力

        let error_rate = config.custom_params
            .get("error_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.01); // 1% 错误率

        let batch_size = config.custom_params
            .get("batch_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000) as u32; // 批次大小

        let device = SoftwareDevice::new(
            device_info,
            device_config,
            target_hashrate,
            error_rate,
            batch_size,
        ).await?;

        Ok(Box::new(device))
    }

    /// 获取所有设备
    async fn get_devices(&self) -> Result<Vec<Box<dyn MiningDevice>>, CoreError> {
        Err(CoreError::runtime("get_devices 方法暂未实现"))
    }

    /// 获取设备数量
    async fn device_count(&self) -> Result<u32, CoreError> {
        let devices = self.devices.lock().await;
        Ok(devices.len() as u32)
    }

    /// 提交工作到所有设备
    async fn submit_work(&mut self, work: Work) -> Result<(), CoreError> {
        let mut devices = self.devices.lock().await;
        let device_count = devices.len();
        let mut success_count = 0;
        let mut failed_devices = Vec::new();

        for (device_id, device) in devices.iter_mut() {
            match device.submit_work(work.clone()).await {
                Ok(()) => {
                    success_count += 1;
                }
                Err(e) => {
                    warn!("向设备 {} 提交工作失败: {}", device_id, e);
                    failed_devices.push(*device_id);
                }
            }
        }

        // 只在有失败或者成功率不是100%时才记录详细信息
        if failed_devices.is_empty() {
            debug!("工作 {} 成功分发到所有 {} 个设备", work.id, device_count);
        } else {
            warn!("工作 {} 分发完成: 成功 {}/{} 个设备，失败设备: {:?}",
                  work.id, success_count, device_count, failed_devices);
        }

        Ok(())
    }

    /// 收集所有设备的挖矿结果 - 从缓存获取立即上报的结果
    async fn collect_results(&mut self) -> Result<Vec<MiningResult>, CoreError> {
        // 从缓存中获取已经立即上报的结果
        let mut results_guard = self.collected_results.lock().await;
        let results = results_guard.drain(..).collect::<Vec<_>>();

        if !results.is_empty() {
            debug!("🎯 从缓存收集到 {} 个结果", results.len());
        }

        Ok(results)
    }

    /// 获取核心统计信息
    async fn get_stats(&self) -> Result<CoreStats, CoreError> {
        self.update_stats().await?;
        let stats = self.stats.read().map_err(|e| {
            CoreError::runtime(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(stats.clone())
    }

    /// 健康检查
    async fn health_check(&self) -> Result<bool, CoreError> {
        let devices = self.devices.lock().await;
        let mut healthy_devices = 0;

        for device in devices.values() {
            match device.health_check().await {
                Ok(true) => healthy_devices += 1,
                Ok(false) => {},
                Err(e) => warn!("设备健康检查失败: {}", e),
            }
        }

        // 如果至少有一半设备健康，则认为核心健康
        let health_threshold = (devices.len() + 1) / 2;
        Ok(healthy_devices >= health_threshold)
    }

    /// 验证配置
    fn validate_config(&self, config: &CoreConfig) -> Result<(), CoreError> {
        if config.name.is_empty() {
            return Err(CoreError::config("核心名称不能为空"));
        }

        // 验证设备数量
        if let Some(device_count) = config.custom_params.get("device_count") {
            if let Some(count) = device_count.as_u64() {
                if count == 0 {
                    return Err(CoreError::config("软算法设备数量不能为0"));
                }
                if count > 1000 {
                    return Err(CoreError::config("软算法设备数量不能超过1000"));
                }

                // 对于大量设备的警告
                if count > 32 {
                    warn!("配置了 {} 个软算法设备，这可能会消耗大量系统资源", count);
                }

                // 检查系统资源
                if count > 64 {
                    let cpu_count = num_cpus::get();
                    if count as usize > cpu_count * 4 {
                        warn!("设备数量 ({}) 远超CPU核心数 ({})，可能影响性能", count, cpu_count);
                    }
                }
            }
        }

        // 验证算力配置
        if let Some(min_hashrate) = config.custom_params.get("min_hashrate") {
            if let Some(max_hashrate) = config.custom_params.get("max_hashrate") {
                if let (Some(min), Some(max)) = (min_hashrate.as_f64(), max_hashrate.as_f64()) {
                    if min >= max {
                        return Err(CoreError::config("最小算力不能大于等于最大算力"));
                    }
                    if min <= 0.0 || max <= 0.0 {
                        return Err(CoreError::config("算力值必须大于0"));
                    }
                }
            }
        }

        // 验证错误率
        if let Some(error_rate) = config.custom_params.get("error_rate") {
            if let Some(rate) = error_rate.as_f64() {
                if rate < 0.0 || rate > 1.0 {
                    return Err(CoreError::config("错误率必须在0.0到1.0之间"));
                }
            }
        }

        Ok(())
    }

    /// 获取默认配置
    fn default_config(&self) -> CoreConfig {
        use std::collections::HashMap;

        let mut custom_params = HashMap::new();
        custom_params.insert("device_count".to_string(), serde_json::Value::Number(serde_json::Number::from(4)));
        custom_params.insert("min_hashrate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(1_000_000_000.0).unwrap()));
        custom_params.insert("max_hashrate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(5_000_000_000.0).unwrap()));
        custom_params.insert("error_rate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.01).unwrap()));
        custom_params.insert("batch_size".to_string(), serde_json::Value::Number(serde_json::Number::from(1000)));

        CoreConfig {
            name: "software-core".to_string(),
            enabled: true,
            devices: vec![cgminer_core::DeviceConfig::default(); 4],
            custom_params,
        }
    }

    /// 关闭核心
    async fn shutdown(&mut self) -> Result<(), CoreError> {
        info!("关闭软算法挖矿核心");
        self.stop().await?;

        // 清空设备列表
        {
            let mut devices = self.devices.lock().await;
            devices.clear();
        }

        info!("软算法挖矿核心已关闭");
        Ok(())
    }
}
