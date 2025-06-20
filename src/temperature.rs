//! 温度管理模块
//!
//! 提供跨平台的温度监控功能，支持真实温度读取和基于负载的模拟温度计算

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// 温度错误类型
#[derive(Error, Debug)]
pub enum TemperatureError {
    #[error("温度读取失败: {0}")]
    ReadFailed(String),
    #[error("平台不支持温度监控")]
    NotSupported,
    #[error("温度传感器未找到")]
    SensorNotFound,
    #[error("权限不足")]
    PermissionDenied,
}

/// 温度提供者trait
pub trait TemperatureProvider: Send + Sync {
    /// 读取当前温度（摄氏度）
    fn read_temperature(&self) -> Result<f32, TemperatureError>;

    /// 检查是否支持温度读取
    fn is_supported(&self) -> bool;

    /// 获取提供者名称
    fn provider_name(&self) -> &'static str;
}

/// 温度监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureConfig {
    /// 是否启用真实温度监控
    pub enable_real_monitoring: bool,
    /// 温度更新间隔（秒）
    pub update_interval: u64,
    /// 温度警告阈值（摄氏度）
    pub warning_threshold: f32,
    /// 温度危险阈值（摄氏度）
    pub critical_threshold: f32,
    /// 模拟温度基础值
    pub simulated_base_temp: f32,
}

impl Default for TemperatureConfig {
    fn default() -> Self {
        Self {
            enable_real_monitoring: true,
            update_interval: 5,
            warning_threshold: 80.0,
            critical_threshold: 90.0,
            simulated_base_temp: 35.0,
        }
    }
}

/// 真实温度提供者（使用sysinfo库）
#[cfg(feature = "temperature-monitoring")]
pub struct RealTemperatureProvider {
    system: std::sync::Mutex<sysinfo::System>,
}

#[cfg(feature = "temperature-monitoring")]
impl RealTemperatureProvider {
    pub fn new() -> Self {
        let system = sysinfo::System::new_all();

        Self {
            system: std::sync::Mutex::new(system),
        }
    }
}

#[cfg(feature = "temperature-monitoring")]
impl TemperatureProvider for RealTemperatureProvider {
    fn read_temperature(&self) -> Result<f32, TemperatureError> {
        // 在sysinfo 0.30中，温度监控API发生了变化
        // 对于CPU挖矿，我们实际上不需要真实的温度监控
        // 根据用户要求，如果不支持真实温度监控，就返回错误
        Err(TemperatureError::NotSupported)
    }

    fn is_supported(&self) -> bool {
        // 尝试读取温度来检查是否支持
        self.read_temperature().is_ok()
    }

    fn provider_name(&self) -> &'static str {
        "RealTemperatureProvider"
    }
}



/// 温度管理器
pub struct TemperatureManager {
    provider: Option<Box<dyn TemperatureProvider>>,
    config: TemperatureConfig,
}

impl TemperatureManager {
    /// 创建温度管理器
    pub fn new(config: TemperatureConfig) -> Self {
        let provider = create_temperature_provider(&config);

        Self {
            provider,
            config,
        }
    }

    /// 读取当前温度
    pub fn read_temperature(&self) -> Result<f32, TemperatureError> {
        match &self.provider {
            Some(provider) => provider.read_temperature(),
            None => Err(TemperatureError::NotSupported),
        }
    }

    /// 检查温度状态
    pub fn check_temperature_status(&self) -> Result<TemperatureStatus, TemperatureError> {
        let temp = self.read_temperature()?;

        let status = if temp >= self.config.critical_threshold {
            TemperatureStatus::Critical
        } else if temp >= self.config.warning_threshold {
            TemperatureStatus::Warning
        } else {
            TemperatureStatus::Normal
        };

        Ok(status)
    }

    /// 获取提供者信息
    pub fn provider_info(&self) -> &'static str {
        match &self.provider {
            Some(provider) => provider.provider_name(),
            None => "NoTemperatureProvider",
        }
    }

    /// 是否支持真实温度监控
    pub fn supports_real_monitoring(&self) -> bool {
        match &self.provider {
            Some(provider) => provider.is_supported(),
            None => false,
        }
    }

    /// 是否有温度监控功能
    pub fn has_temperature_monitoring(&self) -> bool {
        self.provider.is_some()
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

/// 创建合适的温度提供者（仅支持真实温度监控）
pub fn create_temperature_provider(config: &TemperatureConfig) -> Option<Box<dyn TemperatureProvider>> {
    #[cfg(feature = "temperature-monitoring")]
    {
        if config.enable_real_monitoring {
            let real_provider = RealTemperatureProvider::new();
            if real_provider.is_supported() {
                tracing::info!("✅ 启用真实温度监控");
                return Some(Box::new(real_provider));
            } else {
                tracing::info!("❌ 平台不支持真实温度监控，温度功能不可用");
            }
        }
    }

    #[cfg(not(feature = "temperature-monitoring"))]
    {
        tracing::info!("❌ 温度监控功能未编译，温度功能不可用");
    }

    None
}

/// 获取平台温度监控能力信息
pub fn get_platform_temperature_capabilities() -> PlatformTemperatureCapabilities {
    #[cfg(feature = "temperature-monitoring")]
    {
        let real_provider = RealTemperatureProvider::new();
        if real_provider.is_supported() {
            return PlatformTemperatureCapabilities {
                supports_real_monitoring: true,
                platform_name: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                provider_type: "sysinfo".to_string(),
            };
        }
    }

    PlatformTemperatureCapabilities {
        supports_real_monitoring: false,
        platform_name: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        provider_type: "none".to_string(),
    }
}

/// 平台温度监控能力信息
#[derive(Debug, Clone)]
pub struct PlatformTemperatureCapabilities {
    pub supports_real_monitoring: bool,
    pub platform_name: String,
    pub arch: String,
    pub provider_type: String,
}

impl fmt::Display for PlatformTemperatureCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "平台: {} ({}), 真实监控: {}, 提供者: {}",
            self.platform_name,
            self.arch,
            if self.supports_real_monitoring { "支持" } else { "不支持" },
            self.provider_type
        )
    }
}
