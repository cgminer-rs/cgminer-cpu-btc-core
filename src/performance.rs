//! 软算法核心性能优化配置

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// 性能优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// CPU绑定配置
    pub cpu_affinity: CpuAffinityConfig,
    /// 算力优化配置
    pub hashrate_optimization: HashrateOptimizationConfig,
    /// 内存优化配置
    pub memory_optimization: MemoryOptimizationConfig,
    /// 线程优化配置
    pub thread_optimization: ThreadOptimizationConfig,
    /// 批处理优化配置
    pub batch_optimization: BatchOptimizationConfig,
}

/// CPU绑定配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuAffinityConfig {
    /// 是否启用CPU绑定
    pub enabled: bool,
    /// CPU绑定策略
    pub strategy: String,
    /// 手动CPU映射（设备ID -> CPU核心ID）
    pub manual_mapping: HashMap<u32, usize>,
    /// 是否避免超线程
    pub avoid_hyperthreading: bool,
    /// 是否优先使用性能核心
    pub prefer_performance_cores: bool,
}

/// 算力优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashrateOptimizationConfig {
    /// 基础算力 (H/s)
    pub base_hashrate: f64,
    /// 算力范围因子 (0.0-1.0)
    pub hashrate_variance: f64,
    /// 频率-算力比例因子
    pub frequency_hashrate_factor: f64,
    /// 电压-算力比例因子
    pub voltage_hashrate_factor: f64,
    /// 温度对算力的影响因子
    pub temperature_impact_factor: f64,
    /// 自适应算力调整
    pub adaptive_adjustment: bool,
}

/// 内存优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOptimizationConfig {
    /// 工作缓存大小
    pub work_cache_size: usize,
    /// 结果缓存大小
    pub result_cache_size: usize,
    /// 统计数据保留时间（秒）
    pub stats_retention_seconds: u64,
    /// 是否启用内存池
    pub enable_memory_pool: bool,
    /// 预分配内存大小（MB）
    pub preallocated_memory_mb: usize,
}

/// 线程优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadOptimizationConfig {
    /// 每个设备的工作线程数
    pub worker_threads_per_device: usize,
    /// 线程优先级
    pub thread_priority: ThreadPriority,
    /// 线程栈大小（KB）
    pub thread_stack_size_kb: usize,
    /// 是否启用线程池
    pub enable_thread_pool: bool,
}

/// 线程优先级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreadPriority {
    Low,
    Normal,
    High,
    Realtime,
}

/// 批处理优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOptimizationConfig {
    /// 默认批次大小
    pub default_batch_size: u32,
    /// 最小批次大小
    pub min_batch_size: u32,
    /// 最大批次大小
    pub max_batch_size: u32,
    /// 自适应批次大小
    pub adaptive_batch_size: bool,
    /// 批次处理超时（毫秒）
    pub batch_timeout_ms: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cpu_affinity: CpuAffinityConfig::default(),
            hashrate_optimization: HashrateOptimizationConfig::default(),
            memory_optimization: MemoryOptimizationConfig::default(),
            thread_optimization: ThreadOptimizationConfig::default(),
            batch_optimization: BatchOptimizationConfig::default(),
        }
    }
}

impl Default for CpuAffinityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: "intelligent".to_string(),
            manual_mapping: HashMap::new(),
            avoid_hyperthreading: false,
            prefer_performance_cores: true,
        }
    }
}

impl Default for HashrateOptimizationConfig {
    fn default() -> Self {
        Self {
            base_hashrate: 2_000_000_000.0, // 2 GH/s
            hashrate_variance: 0.2, // ±20%
            frequency_hashrate_factor: 1.5,
            voltage_hashrate_factor: 1.2,
            temperature_impact_factor: 0.95,
            adaptive_adjustment: true,
        }
    }
}

impl Default for MemoryOptimizationConfig {
    fn default() -> Self {
        Self {
            work_cache_size: 1000,
            result_cache_size: 10000,
            stats_retention_seconds: 3600, // 1小时
            enable_memory_pool: true,
            preallocated_memory_mb: 64,
        }
    }
}

impl Default for ThreadOptimizationConfig {
    fn default() -> Self {
        Self {
            worker_threads_per_device: 1,
            thread_priority: ThreadPriority::Normal,
            thread_stack_size_kb: 256,
            enable_thread_pool: true,
        }
    }
}

impl Default for BatchOptimizationConfig {
    fn default() -> Self {
        Self {
            default_batch_size: 1000,
            min_batch_size: 100,
            max_batch_size: 10000,
            adaptive_batch_size: true,
            batch_timeout_ms: 1000,
        }
    }
}

/// 性能优化器
pub struct PerformanceOptimizer {
    config: PerformanceConfig,
}

impl PerformanceOptimizer {
    /// 创建新的性能优化器
    pub fn new(config: PerformanceConfig) -> Self {
        Self { config }
    }

    /// 根据系统特性优化配置
    pub fn optimize_for_system(&mut self) {
        info!("🔧 开始系统性能优化...");

        // 检测CPU特性
        let cpu_count = num_cpus::get();
        let physical_cpu_count = num_cpus::get_physical();

        info!("💻 检测到 {} 个逻辑CPU核心，{} 个物理CPU核心", cpu_count, physical_cpu_count);

        // 优化CPU绑定策略
        if cpu_count >= 8 {
            self.config.cpu_affinity.strategy = "intelligent".to_string();
            self.config.cpu_affinity.prefer_performance_cores = true;
            info!("✅ 多核心系统，启用智能CPU绑定策略");
        } else if cpu_count >= 4 {
            self.config.cpu_affinity.strategy = "round_robin".to_string();
            info!("✅ 中等核心数系统，使用轮询CPU绑定策略");
        } else {
            self.config.cpu_affinity.enabled = false;
            warn!("⚠️  少核心系统，禁用CPU绑定以避免性能损失");
        }

        // 优化线程配置
        if cpu_count >= 16 {
            self.config.thread_optimization.worker_threads_per_device = 2;
            info!("✅ 高核心数系统，每设备使用2个工作线程");
        } else {
            self.config.thread_optimization.worker_threads_per_device = 1;
            info!("✅ 标准配置，每设备使用1个工作线程");
        }

        // 优化内存配置
        let available_memory_gb = Self::get_available_memory_gb();
        if available_memory_gb >= 8.0 {
            self.config.memory_optimization.preallocated_memory_mb = 128;
            self.config.memory_optimization.work_cache_size = 2000;
            self.config.memory_optimization.result_cache_size = 20000;
            info!("✅ 充足内存，增大缓存配置");
        } else if available_memory_gb >= 4.0 {
            self.config.memory_optimization.preallocated_memory_mb = 64;
            info!("✅ 中等内存，使用标准缓存配置");
        } else {
            self.config.memory_optimization.preallocated_memory_mb = 32;
            self.config.memory_optimization.work_cache_size = 500;
            self.config.memory_optimization.result_cache_size = 5000;
            warn!("⚠️  内存较少，减小缓存配置");
        }

        // 优化批处理配置
        if cpu_count >= 8 {
            self.config.batch_optimization.default_batch_size = 2000;
            self.config.batch_optimization.max_batch_size = 20000;
            info!("✅ 多核心系统，增大批处理大小");
        }

        info!("🎯 系统性能优化完成");
    }

    /// 获取可用内存（GB）
    fn get_available_memory_gb() -> f64 {
        // 简化实现，实际应该通过系统API获取
        // 这里假设有足够的内存
        8.0
    }

    /// 获取优化后的配置
    pub fn get_config(&self) -> &PerformanceConfig {
        &self.config
    }

    /// 应用性能优化到设备配置
    pub fn apply_to_device_config(&self, device_config: &mut cgminer_core::DeviceConfig, device_id: u32) {
        // 根据性能配置调整设备参数

        // 调整频率（基于算力优化配置）
        let base_frequency = device_config.frequency;
        let optimized_frequency = (base_frequency as f64 * self.config.hashrate_optimization.frequency_hashrate_factor) as u32;
        device_config.frequency = optimized_frequency.min(1000).max(400); // 限制在合理范围内

        // 调整电压（基于算力优化配置）
        let base_voltage = device_config.voltage;
        let optimized_voltage = (base_voltage as f64 * self.config.hashrate_optimization.voltage_hashrate_factor) as u32;
        device_config.voltage = optimized_voltage.min(1200).max(800); // 限制在合理范围内

        info!("⚡ 设备 {} 性能优化: 频率 {}MHz -> {}MHz, 电压 {}mV -> {}mV",
              device_id, base_frequency, device_config.frequency, base_voltage, device_config.voltage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();

        assert!(config.cpu_affinity.enabled);
        assert_eq!(config.cpu_affinity.strategy, "intelligent");
        assert!(config.cpu_affinity.prefer_performance_cores);

        assert_eq!(config.hashrate_optimization.base_hashrate, 2_000_000_000.0);
        assert_eq!(config.hashrate_optimization.hashrate_variance, 0.2);
        assert!(config.hashrate_optimization.adaptive_adjustment);

        assert_eq!(config.memory_optimization.work_cache_size, 1000);
        assert_eq!(config.memory_optimization.result_cache_size, 10000);
        assert!(config.memory_optimization.enable_memory_pool);

        assert_eq!(config.thread_optimization.worker_threads_per_device, 1);
        assert!(matches!(config.thread_optimization.thread_priority, ThreadPriority::Normal));

        assert_eq!(config.batch_optimization.default_batch_size, 1000);
        assert!(config.batch_optimization.adaptive_batch_size);
    }

    #[test]
    fn test_performance_optimizer_system_optimization() {
        let config = PerformanceConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config);

        optimizer.optimize_for_system();
        let optimized_config = optimizer.get_config();

        // 验证系统优化后的配置
        assert!(optimized_config.cpu_affinity.enabled);
        assert!(optimized_config.memory_optimization.enable_memory_pool);
        assert!(optimized_config.batch_optimization.adaptive_batch_size);
    }

    #[test]
    fn test_device_config_optimization() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);

        let mut device_config = cgminer_core::DeviceConfig {
            chain_id: 0,
            enabled: true,
            frequency: 600,
            voltage: 900,
            auto_tune: false,
            chip_count: 64,
            temperature_limit: 80.0,
            fan_speed: Some(50),
        };

        let original_frequency = device_config.frequency;
        let original_voltage = device_config.voltage;

        optimizer.apply_to_device_config(&mut device_config, 1000);

        // 验证设备配置被优化
        assert!(device_config.frequency >= original_frequency);
        assert!(device_config.voltage >= original_voltage);
        assert!(device_config.frequency <= 1000); // 最大限制
        assert!(device_config.voltage <= 1200);   // 最大限制
    }

    #[test]
    fn test_cpu_affinity_config_variants() {
        let round_robin = CpuAffinityConfig {
            enabled: true,
            strategy: "round_robin".to_string(),
            manual_mapping: HashMap::new(),
            avoid_hyperthreading: false,
            prefer_performance_cores: false,
        };

        let intelligent = CpuAffinityConfig {
            enabled: true,
            strategy: "intelligent".to_string(),
            manual_mapping: HashMap::new(),
            avoid_hyperthreading: false,
            prefer_performance_cores: true,
        };

        assert_eq!(round_robin.strategy, "round_robin");
        assert_eq!(intelligent.strategy, "intelligent");
        assert!(intelligent.prefer_performance_cores);
        assert!(!round_robin.prefer_performance_cores);
    }

    #[test]
    fn test_thread_priority_variants() {
        let priorities = vec![
            ThreadPriority::Low,
            ThreadPriority::Normal,
            ThreadPriority::High,
            ThreadPriority::Realtime,
        ];

        for priority in priorities {
            let config = ThreadOptimizationConfig {
                worker_threads_per_device: 1,
                thread_priority: priority,
                thread_stack_size_kb: 256,
                enable_thread_pool: true,
            };

            assert_eq!(config.worker_threads_per_device, 1);
            assert!(config.enable_thread_pool);
        }
    }

    #[test]
    fn test_batch_optimization_bounds() {
        let config = BatchOptimizationConfig {
            default_batch_size: 1000,
            min_batch_size: 100,
            max_batch_size: 10000,
            adaptive_batch_size: true,
            batch_timeout_ms: 1000,
        };

        assert!(config.min_batch_size <= config.default_batch_size);
        assert!(config.default_batch_size <= config.max_batch_size);
        assert!(config.adaptive_batch_size);
        assert!(config.batch_timeout_ms > 0);
    }
}
