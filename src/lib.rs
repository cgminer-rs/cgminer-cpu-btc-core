//! CGMiner Software Core - 软算法挖矿核心
//!
//! 这个库提供基于CPU的软件算法挖矿实现，使用真实的SHA256算法进行计算。
//! 软算法核心产生真实可用的挖矿数据，适用于测试、开发和低功耗挖矿场景。

pub mod core;
pub mod device;
pub mod factory;
pub mod cpu_affinity;
pub mod performance;
pub mod platform_optimization;

// 重新导出主要类型
pub use core::SoftwareMiningCore;
pub use device::SoftwareDevice;
pub use factory::SoftwareCoreFactory;

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
