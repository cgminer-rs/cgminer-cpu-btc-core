//! # 性能优化配置模块
//!
//! 本模块提供简化而高效的性能优化配置，专注于CPU挖矿的核心性能调优。
//! 经过简化后，移除了过度复杂的优化系统，保留最有效的优化策略。
//!
//! ## 🚀 优化策略
//!
//! ### [`PerformanceConfig`] - 性能配置
//! - ⚡ CPU亲和性绑定配置
//! - ⚡ 基础算力参数设置
//! - ⚡ 批处理大小优化
//! - ⚡ 全局优化开关控制
//!
//! ### [`PerformanceOptimizer`] - 性能优化器
//! - 🔧 系统自适应优化
//! - 🔧 设备配置微调
//! - 🔧 批次大小智能调整
//! - 🔧 CPU绑定自动启用
//!
//! ## 🎯 自动优化规则
//!
//! ### CPU数量自适应
//! ```text
//! CPU核心数量 → 批次大小优化:
//! ├── >= 8核心  → 2000 (高性能)
//! ├── >= 4核心  → 1500 (中等性能)
//! └── < 4核心   → 1000 (保守配置)
//! ```
//!
//! ### CPU绑定启用条件
//! - ✅ 物理核心数 >= 4: 自动启用CPU绑定
//! - ❌ 物理核心数 < 4: 保持禁用状态
//!
//! ## 📊 性能参数说明
//!
//! | 参数 | 默认值 | 说明 | 影响 |
//! |------|--------|------|------|
//! | `base_hashrate` | 2 GH/s | 基础算力 | 设备算力基准 |
//! | `batch_size` | 1000 | 批次大小 | CPU使用效率 |
//! | `enable_optimizations` | true | 优化开关 | 整体性能 |
//! | `cpu_affinity` | 默认配置 | CPU绑定 | 缓存命中率 |
//!
//! ## 🔄 使用示例
//!
//! ### 基本使用
//! ```rust
//! use cgminer_cpu_btc_core::performance::{PerformanceOptimizer, PerformanceConfig};
//!
//! // 创建默认配置
//! let config = PerformanceConfig::default();
//! let mut optimizer = PerformanceOptimizer::new(config);
//!
//! // 自动优化
//! optimizer.optimize_for_system();
//!
//! // 应用到设备配置
//! optimizer.apply_to_device_config(&mut device_config, device_id);
//! ```
//!
//! ### 自定义配置
//! ```rust
//! let config = PerformanceConfig {
//!     base_hashrate: 3_000_000_000.0, // 3 GH/s
//!     batch_size: 1500,
//!     enable_optimizations: true,
//!     cpu_affinity: CpuAffinityConfig::round_robin(),
//! };
//! ```
//!
//! ## ⚙️ 设计原则
//!
//! 1. **简单有效**: 移除复杂配置，专注核心优化
//! 2. **自动调优**: 基于系统特性自动调整参数
//! 3. **渐进优化**: 小幅度调整，避免过度优化
//! 4. **兼容性**: 确保在各种硬件上稳定运行
//! 5. **可观测**: 提供清晰的优化日志输出

use crate::cpu_affinity::CpuAffinityConfig;


/// 简化的性能配置
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// CPU绑定配置
    pub cpu_affinity: CpuAffinityConfig,
    /// 基础算力配置
    pub base_hashrate: f64,
    /// 批次大小
    pub batch_size: u32,
    /// 是否启用优化
    pub enable_optimizations: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cpu_affinity: CpuAffinityConfig::default(),
            base_hashrate: 2_000_000_000.0, // 2 GH/s
            batch_size: 1000,
            enable_optimizations: true,
        }
    }
}

/// 简化的性能优化器
pub struct PerformanceOptimizer {
    config: PerformanceConfig,
}

impl PerformanceOptimizer {
    /// 创建性能优化器
    pub fn new(config: PerformanceConfig) -> Self {
        Self { config }
    }

    /// 针对系统进行优化
    pub fn optimize_for_system(&mut self) {
        // 根据系统CPU数量调整配置
        let cpu_count = num_cpus::get();
        let physical_cpu_count = num_cpus::get_physical();

        // 简化的系统优化
        if cpu_count >= 8 {
            self.config.enable_optimizations = true;
            self.config.batch_size = 2000;
        } else if cpu_count >= 4 {
            self.config.batch_size = 1500;
        } else {
            self.config.batch_size = 1000;
        }

        // 如果有足够的物理核心，启用CPU绑定
        if physical_cpu_count >= 4 {
            self.config.cpu_affinity.enabled = true;
        }

        tracing::info!("🔧 性能优化完成: CPU核心数={}, 物理核心数={}, 批次大小={}",
                      cpu_count, physical_cpu_count, self.config.batch_size);
    }

    /// 获取配置
    pub fn get_config(&self) -> &PerformanceConfig {
        &self.config
    }

    /// 应用优化到设备配置
    pub fn apply_to_device_config(&self, device_config: &mut cgminer_core::DeviceConfig, device_id: u32) {
        if self.config.enable_optimizations {
            // 简单的频率优化
            device_config.frequency += (device_id % 4) * 25; // 小幅度调整频率

            // 简单的电压优化
            device_config.voltage += (device_id % 3) * 10; // 小幅度调整电压
        }
    }
}
