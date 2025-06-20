//! CGMiner-RS - 高性能比特币挖矿程序
//!
//! CGMiner-RS是一个用Rust编写的高性能比特币挖矿程序，支持多种挖矿核心：
//! - ASIC挖矿核心（硬件挖矿）
//! - CPU软算法核心（软件挖矿）
//! - GPU挖矿核心（计划中）
//!
//! ## 架构特点
//!
//! ### 动态核心加载
//! - 支持运行时加载不同的挖矿核心
//! - 每个核心作为独立的库存在
//! - 统一的核心接口和管理
//!
//! ### 高性能设计
//! - 异步I/O和并发处理
//! - 内存池和对象复用
//! - 智能负载均衡
//! - 实时性能监控
//!
//! ### 企业级特性
//! - 完整的API接口
//! - Web管理界面
//! - 详细的日志和监控
//! - 安全的远程管理

// 核心库模块
pub mod core;
pub mod device;
pub mod factory;
// 暂时禁用优化模块，直到接口问题解决
// pub mod optimized_core;
// pub mod optimized_device;
// pub mod optimized_factory;
pub mod cpu_affinity;
pub mod platform_optimization;
pub mod performance;

// 重新导出主要类型
pub use factory::SoftwareCoreFactory;
// 暂时禁用优化工厂
// pub use optimized_factory::OptimizedCpuCoreFactory;

use cgminer_core::{CoreType, CoreInfo};

/// 库版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 获取软算法核心信息
pub fn get_core_info() -> CoreInfo {
    CoreInfo::new(
        "Software Mining Core".to_string(),
        CoreType::Custom("software".to_string()),
        VERSION.to_string(),
        "软算法挖矿核心，使用真实的SHA256算法进行CPU挖矿计算".to_string(),
        "CGMiner Rust Team".to_string(),
        vec!["software".to_string(), "cpu".to_string()],
    )
}

/// 创建软算法核心工厂
pub fn create_factory() -> Box<dyn cgminer_core::CoreFactory> {
    Box::new(SoftwareCoreFactory::new())
}

/// 创建优化CPU核心工厂（推荐）- 暂时禁用
// pub fn create_optimized_factory() -> Box<dyn cgminer_core::CoreFactory> {
//     Box::new(OptimizedCpuCoreFactory::new())
// }

/// 获取优化CPU核心信息
pub fn get_optimized_core_info() -> CoreInfo {
    CoreInfo::new(
        "Optimized CPU Mining Core".to_string(),
        CoreType::Custom("optimized_cpu".to_string()),
        VERSION.to_string(),
        "优化CPU挖矿核心，支持SIMD加速、智能温度管理和动态负载均衡".to_string(),
        "CGMiner Rust Team".to_string(),
        vec!["optimized_cpu".to_string(), "simd".to_string(), "cpu".to_string()],
    )
}

// C FFI 导出函数，用于动态加载
#[no_mangle]
pub extern "C" fn cgminer_s_btc_core_info() -> *const std::os::raw::c_char {
    use std::ffi::CString;

    let info = get_core_info();
    let json = serde_json::to_string(&info).unwrap_or_default();
    let c_string = CString::new(json).unwrap_or_default();

    // 注意：这里返回的指针需要调用者负责释放
    c_string.into_raw()
}

#[no_mangle]
pub extern "C" fn cgminer_s_btc_create_factory() -> *mut std::os::raw::c_void {
    let factory = create_factory();
    Box::into_raw(Box::new(factory)) as *mut std::os::raw::c_void
}

#[no_mangle]
pub extern "C" fn cgminer_s_btc_free_string(ptr: *mut std::os::raw::c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(ptr);
        }
    }
}

// 优化CPU核心的C FFI导出函数
#[no_mangle]
pub extern "C" fn cgminer_optimized_cpu_core_info() -> *const std::os::raw::c_char {
    use std::ffi::CString;

    let info = get_optimized_core_info();
    let json = serde_json::to_string(&info).unwrap_or_default();
    let c_string = CString::new(json).unwrap_or_default();

    // 注意：这里返回的指针需要调用者负责释放
    c_string.into_raw()
}

// 暂时禁用优化CPU核心的C FFI导出函数
// #[no_mangle]
// pub extern "C" fn cgminer_optimized_cpu_create_factory() -> *mut std::os::raw::c_void {
//     let factory = create_optimized_factory();
//     Box::into_raw(Box::new(factory)) as *mut std::os::raw::c_void
// }
