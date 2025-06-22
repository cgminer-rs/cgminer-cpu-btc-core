//! CGMiner-CPU-BTC-Core - 高性能CPU比特币挖矿核心
//!
//! 这是一个专门用于CPU比特币挖矿的核心库，使用真实的SHA256算法进行软件挖矿计算。
//! 该库采用静态编译方式，专注于单一核心类型的高性能实现。
//!
//! ## 核心特性
//!
//! ### 真实算法挖矿
//! - 使用真实的SHA256双重哈希算法
//! - 产生真实可用的挖矿数据
//! - 支持多线程并行计算
//!
//! ### 高性能优化
//! - 平台特定的SIMD优化
//! - CPU亲和性绑定
//! - 智能批处理和内存对齐
//! - 实时性能监控
//!
//! ### 企业级特性
//! - 完整的设备模拟
//! - 详细的统计和监控
//! - 可配置的算力和设备数量
//! - 适用于测试、开发和生产环境

// 核心库模块
pub mod core;
pub mod device;
pub mod factory;
pub mod cpu_affinity;
pub mod performance;
pub mod platform_optimization;
pub mod temperature;

#[cfg(test)]
mod temperature_test;

// 重新导出主要类型
pub use factory::SoftwareCoreFactory;
pub use core::SoftwareMiningCore;
pub use device::SoftwareDevice;

use cgminer_core::{CoreType, CoreInfo};

/// 库版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 获取优化CPU核心信息
pub fn get_core_info() -> CoreInfo {
    CoreInfo::new(
        "Optimized CPU Mining Core".to_string(),
        CoreType::Custom("optimized_cpu".to_string()),
        VERSION.to_string(),
        "优化CPU挖矿核心，支持SIMD加速、智能线程调度和动态负载均衡".to_string(),
        "CGMiner Rust Team".to_string(),
        vec!["optimized_cpu".to_string(), "simd".to_string(), "cpu".to_string()],
    )
}

/// 创建优化CPU核心工厂
pub fn create_factory() -> Box<dyn cgminer_core::CoreFactory> {
    Box::new(SoftwareCoreFactory::new())
}
