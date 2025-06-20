//! 平台特定的性能优化
//! 
//! 针对不同操作系统和CPU架构提供优化的配置参数

use tracing::{info, debug};

/// 平台优化配置
#[derive(Debug, Clone)]
pub struct PlatformOptimization {
    /// CPU让出频率
    pub yield_frequency: u64,
    /// 推荐的批处理大小
    pub recommended_batch_size: u32,
    /// 推荐的设备数量倍数（相对于CPU核心数）
    pub device_count_multiplier: f32,
    /// 是否启用CPU绑定
    pub enable_cpu_affinity: bool,
    /// 平台名称
    pub platform_name: String,
}

impl PlatformOptimization {
    /// 获取当前平台的优化配置
    pub fn get_current_platform_config() -> Self {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            info!("🍎 检测到 Mac M4 (Apple Silicon) 平台，应用专用优化");
            Self {
                yield_frequency: 50000,        // 大幅减少CPU让出频率
                recommended_batch_size: 12000, // 大批处理提高效率
                device_count_multiplier: 8.0,  // M4性能强劲，支持更多设备
                enable_cpu_affinity: false,    // macOS限制CPU绑定
                platform_name: "Mac M4 (Apple Silicon)".to_string(),
            }
        }
        
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            info!("🍎 检测到 Intel Mac 平台，应用优化配置");
            Self {
                yield_frequency: 10000,
                recommended_batch_size: 6000,
                device_count_multiplier: 4.0,
                enable_cpu_affinity: false,    // macOS限制CPU绑定
                platform_name: "Intel Mac".to_string(),
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            info!("🐧 检测到 Linux 平台，应用优化配置");
            Self {
                yield_frequency: 5000,
                recommended_batch_size: 4000,
                device_count_multiplier: 3.0,
                enable_cpu_affinity: true,     // Linux支持CPU绑定
                platform_name: "Linux".to_string(),
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            info!("🪟 检测到 Windows 平台，应用优化配置");
            Self {
                yield_frequency: 2000,
                recommended_batch_size: 3000,
                device_count_multiplier: 2.5,
                enable_cpu_affinity: true,     // Windows支持CPU绑定
                platform_name: "Windows".to_string(),
            }
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            info!("❓ 检测到未知平台，使用默认配置");
            Self {
                yield_frequency: 1000,
                recommended_batch_size: 2000,
                device_count_multiplier: 2.0,
                enable_cpu_affinity: false,
                platform_name: "Unknown".to_string(),
            }
        }
    }
    
    /// 根据CPU核心数计算推荐的设备数量
    pub fn calculate_recommended_device_count(&self, cpu_cores: usize) -> u32 {
        let recommended = (cpu_cores as f32 * self.device_count_multiplier) as u32;
        let min_devices = cpu_cores as u32;
        let max_devices = (cpu_cores as u32) * 16; // 最多16倍核心数
        
        recommended.clamp(min_devices, max_devices)
    }
    
    /// 获取平台特定的性能提示
    pub fn get_performance_tips(&self) -> Vec<String> {
        let mut tips = Vec::new();
        
        match self.platform_name.as_str() {
            "Mac M4 (Apple Silicon)" => {
                tips.push("🚀 M4芯片性能强劲，建议使用大批处理和高设备数量".to_string());
                tips.push("⚡ 禁用CPU绑定，让macOS系统调度器优化性能".to_string());
                tips.push("🔥 监控温度，M4在高负载下可能需要散热".to_string());
                tips.push("💡 建议设备数量为CPU核心数的6-10倍".to_string());
            }
            "Intel Mac" => {
                tips.push("💻 Intel Mac建议适中的配置参数".to_string());
                tips.push("🌡️ 注意温度控制，Intel芯片发热较大".to_string());
                tips.push("⚖️ 平衡性能和稳定性".to_string());
            }
            "Linux" => {
                tips.push("🐧 Linux平台支持完整的CPU绑定功能".to_string());
                tips.push("🔧 可以精确控制CPU使用率".to_string());
                tips.push("📊 建议启用详细的性能监控".to_string());
            }
            "Windows" => {
                tips.push("🪟 Windows平台建议保守的配置".to_string());
                tips.push("🛡️ 注意防病毒软件的影响".to_string());
                tips.push("⚡ 可能需要管理员权限获得最佳性能".to_string());
            }
            _ => {
                tips.push("❓ 未知平台，建议谨慎调整参数".to_string());
            }
        }
        
        tips
    }
    
    /// 打印平台优化信息
    pub fn print_optimization_info(&self) {
        info!("🎯 平台优化配置:");
        info!("   平台: {}", self.platform_name);
        info!("   CPU让出频率: 每{}次哈希", self.yield_frequency);
        info!("   推荐批处理大小: {}", self.recommended_batch_size);
        info!("   设备数量倍数: {:.1}x", self.device_count_multiplier);
        info!("   CPU绑定: {}", if self.enable_cpu_affinity { "启用" } else { "禁用" });
        
        debug!("💡 性能提示:");
        for tip in self.get_performance_tips() {
            debug!("   {}", tip);
        }
    }
}

/// 获取平台特定的CPU让出频率
pub fn get_platform_yield_frequency() -> u64 {
    PlatformOptimization::get_current_platform_config().yield_frequency
}

/// 获取平台特定的推荐批处理大小
pub fn get_platform_batch_size() -> u32 {
    PlatformOptimization::get_current_platform_config().recommended_batch_size
}

/// 检查当前平台是否为Apple Silicon
pub fn is_apple_silicon() -> bool {
    cfg!(all(target_os = "macos", target_arch = "aarch64"))
}

/// 检查当前平台是否支持CPU绑定
pub fn supports_cpu_affinity() -> bool {
    PlatformOptimization::get_current_platform_config().enable_cpu_affinity
}
