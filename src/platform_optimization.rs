//! # 平台特定优化模块
//!
//! 本模块提供跨平台的CPU挖矿优化功能，针对不同操作系统和硬件架构
//! 提供特定的优化策略，以最大化CPU挖矿性能。
//!
//! ## 🚀 平台优化特性
//!
//! ### CPU让出策略优化
//! - 🔧 **macOS**: 1000次哈希/让出 (优化响应性)
//! - 🔧 **Linux**: 2000次哈希/让出 (平衡性能与响应)
//! - 🔧 **Windows**: 1500次哈希/让出 (兼容性优先)
//! - 🔧 **其他平台**: 1000次哈希/让出 (保守策略)
//!
//! ### 高性能支持检测
//! - ✅ **x86_64**: 完全支持，包括SIMD优化
//! - ✅ **aarch64**: 完全支持，ARM64优化
//! - ⚠️ **其他架构**: 基础支持，性能可能受限
//!
//! ## 📊 性能调优参数
//!
//! | 平台 | 让出频率 | 优化重点 | 特殊考虑 |
//! |------|----------|----------|----------|
//! | macOS | 1000 | 响应性 | 系统调度优化 |
//! | Linux | 2000 | 吞吐量 | 内核调度配合 |
//! | Windows | 1500 | 兼容性 | 多版本支持 |
//! | 其他 | 1000 | 稳定性 | 保守配置 |
//!
//! ## 🎯 设计目标
//!
//! 1. **跨平台兼容**: 统一API，平台特定实现
//! 2. **性能最优**: 针对每个平台的特性优化
//! 3. **简单易用**: 最小化配置，自动检测
//! 4. **向后兼容**: 支持未知平台的降级处理
//!
//! ## 🔄 使用示例
//!
//! ```rust
//! use cgminer_cpu_btc_core::platform_optimization;
//!
//! // 获取平台信息
//! let platform_info = platform_optimization::get_platform_info();
//! println!("运行平台: {}", platform_info);
//!
//! // 检查高性能支持
//! if platform_optimization::supports_high_performance() {
//!     println!("✅ 当前平台支持高性能优化");
//! }
//!
//! // 在挖矿循环中使用
//! for i in 0..batch_size {
//!     // 执行哈希计算
//!     let hash = calculate_hash(&data);
//!
//!     // 平台特定的CPU让出
//!     if i % platform_optimization::get_platform_yield_frequency() == 0 {
//!         tokio::task::yield_now().await;
//!     }
//! }
//! ```

/// 获取平台特定的CPU让出频率
///
/// 这个函数在挖矿循环中被使用，用于优化CPU让出策略
pub fn get_platform_yield_frequency() -> u64 {
    // 根据平台返回合适的让出频率
    #[cfg(target_os = "macos")]
    {
        1000 // macOS: 每1000次哈希让出一次
    }

    #[cfg(target_os = "linux")]
    {
        2000 // Linux: 每2000次哈希让出一次
    }

    #[cfg(target_os = "windows")]
    {
        1500 // Windows: 每1500次哈希让出一次
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        1000 // 其他平台: 保守的让出频率
    }
}

/// 获取平台信息字符串（用于日志输出）
pub fn get_platform_info() -> String {
    format!("{}-{}",
            std::env::consts::OS,
            std::env::consts::ARCH)
}

/// 检查当前平台是否支持高性能优化
pub fn supports_high_performance() -> bool {
    // 简化的平台检查
    cfg!(any(target_arch = "x86_64", target_arch = "aarch64"))
}
