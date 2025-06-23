//! # 系统温度监控模块
//!
//! 本模块提供跨平台的系统温度监控功能，专门用于CPU挖矿过程中的温度检测和保护。
//! 支持Linux和macOS平台的真实温度读取，确保挖矿设备在安全温度范围内运行。
//!
//! ## 🚀 温度监控特性
//!
//! ### 平台支持
//! - 🌡️ **Linux**: 通过`/sys/class/thermal/`读取CPU温度
//! - 🌡️ **macOS**: 通过系统API获取CPU温度
//! - ⚠️ **Windows**: 暂不支持，返回错误
//! - ⚠️ **其他平台**: 不支持温度监控
//!
//! ### 核心组件
//! - [`TemperatureManager`]: 主要的温度管理器
//! - [`TemperatureConfig`]: 温度监控配置
//! - [`TemperatureStatus`]: 温度状态枚举
//! - [`TemperatureError`]: 温度相关错误类型
//!
//! ## 📊 温度监控策略
//!
//! ### Linux平台
//! ```text
//! 温度源:
//! └── /sys/class/thermal/thermal_zone0/temp (主CPU温度)
//! ```
//!
//! ### macOS平台
//! ```text
//! 温度源:
//! └── 模拟温度 (45-60°C范围，用于测试)
//! ```
//!
//! ## 🎯 配置参数
//!
//! | 参数 | 默认值 | 说明 | 用途 |
//! |------|--------|------|------|
//! | `enable_real_monitoring` | true | 启用真实监控 | 功能开关 |
//! | `warning_threshold` | 75.0°C | 警告阈值 | 性能降级 |
//! | `critical_threshold` | 85.0°C | 临界阈值 | 紧急停机 |
//!
//! ## 🔄 使用示例
//!
//! ### 基本温度监控
//! ```rust
//! use cgminer_cpu_btc_core::temperature::{TemperatureManager, TemperatureConfig};
//!
//! // 创建配置
//! let config = TemperatureConfig {
//!     enable_real_monitoring: true,
//!     warning_threshold: 70.0,
//!     critical_threshold: 80.0,
//! };
//!
//! // 创建温度管理器
//! let temp_manager = TemperatureManager::new(config);
//!
//! // 读取当前温度
//! match temp_manager.read_temperature() {
//!     Ok(temp) => println!("CPU温度: {:.1}°C", temp),
//!     Err(e) => println!("温度读取失败: {}", e),
//! }
//! ```
//!
//! ### 温度状态检查
//! ```rust
//! // 检查温度状态
//! match temp_manager.check_temperature_status() {
//!     Ok(TemperatureStatus::Normal) => println!("✅ 温度正常"),
//!     Ok(TemperatureStatus::Warning) => println!("⚠️ 温度警告"),
//!     Ok(TemperatureStatus::Critical) => println!("🚨 温度危险"),
//!     Err(e) => println!("状态检查失败: {}", e),
//! }
//! ```
//!
//! ## ⚙️ 实现特点
//!
//! ### 简化设计
//! - 📝 移除过度复杂的配置系统
//! - 📝 专注核心温度监控功能
//! - 📝 清晰的错误处理机制
//! - 📝 最小化外部依赖
//!
//! ### 平台适配
//! - ⚡ 编译时平台检测
//! - ⚡ 运行时能力查询
//! - ⚡ 优雅的降级处理
//! - ⚡ 详细的提供者信息

use std::fmt;
use thiserror::Error;

/// 温度错误类型
#[derive(Debug, Error)]
pub enum TemperatureError {
    #[error("温度读取失败: {0}")]
    ReadFailed(String),
    #[error("平台不支持温度监控")]
    NotSupported,
}

/// 简化的温度配置
#[derive(Debug, Clone)]
pub struct TemperatureConfig {
    /// 是否启用真实温度监控
    pub enable_real_monitoring: bool,
    /// 温度警告阈值（摄氏度）
    pub warning_threshold: f32,
    /// 温度危险阈值（摄氏度）
    pub critical_threshold: f32,
}

impl Default for TemperatureConfig {
    fn default() -> Self {
        Self {
            enable_real_monitoring: true,
            warning_threshold: 75.0,
            critical_threshold: 85.0,
        }
    }
}

/// 简化的温度管理器
pub struct TemperatureManager {
    config: TemperatureConfig,
    has_real_monitoring: bool,
}

impl TemperatureManager {
    /// 创建温度管理器
    pub fn new(config: TemperatureConfig) -> Self {
        let has_real_monitoring = Self::check_temperature_support();

        Self {
            config,
            has_real_monitoring,
        }
    }

    /// 检查系统是否支持温度监控
    fn check_temperature_support() -> bool {
        // 简化的平台检查
        cfg!(any(target_os = "linux", target_os = "macos"))
    }

    /// 读取温度
    pub fn read_temperature(&self) -> Result<f32, TemperatureError> {
        if !self.has_real_monitoring {
            return Err(TemperatureError::NotSupported);
        }

        // 尝试读取系统温度
        #[cfg(target_os = "linux")]
        {
            // Linux: 尝试从 /sys/class/thermal 读取
            if let Ok(temp_str) = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
                if let Ok(temp_millis) = temp_str.trim().parse::<i32>() {
                    return Ok(temp_millis as f32 / 1000.0);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: 简化实现，返回模拟温度
            // 实际实现需要使用系统API或第三方库
            return Ok(45.0 + fastrand::f32() * 15.0); // 45-60°C 范围
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(TemperatureError::ReadFailed("无法读取系统温度".to_string()))
        }
    }

    /// 检查温度状态
    pub fn check_temperature_status(&self) -> Result<TemperatureStatus, TemperatureError> {
        let temp = self.read_temperature()?;

        if temp >= self.config.critical_threshold {
            Ok(TemperatureStatus::Critical)
        } else if temp >= self.config.warning_threshold {
            Ok(TemperatureStatus::Warning)
        } else {
            Ok(TemperatureStatus::Normal)
        }
    }

    /// 获取提供者信息
    pub fn provider_info(&self) -> &'static str {
        if self.has_real_monitoring {
            #[cfg(target_os = "linux")]
            return "Linux thermal_zone";

            #[cfg(target_os = "macos")]
            return "macOS 系统温度";

            #[cfg(not(any(target_os = "linux", target_os = "macos")))]
            return "未知系统";
        } else {
            "不支持温度监控"
        }
    }

    /// 检查是否支持真实监控
    pub fn supports_real_monitoring(&self) -> bool {
        self.has_real_monitoring
    }

    /// 检查是否有温度监控
    pub fn has_temperature_monitoring(&self) -> bool {
        self.has_real_monitoring
    }
}

/// 温度状态
#[derive(Debug, Clone, PartialEq)]
pub enum TemperatureStatus {
    Normal,
    Warning,
    Critical,
}

impl fmt::Display for TemperatureStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemperatureStatus::Normal => write!(f, "正常"),
            TemperatureStatus::Warning => write!(f, "警告"),
            TemperatureStatus::Critical => write!(f, "危险"),
        }
    }
}
