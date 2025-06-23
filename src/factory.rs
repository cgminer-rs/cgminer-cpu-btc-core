//! # CPU挖矿核心工厂实现
//!
//! 本模块实现了CPU挖矿核心的工厂模式，负责创建和配置挖矿核心实例。
//! 遵循cgminer-core的CoreFactory标准接口，提供统一的核心创建和管理功能。
//!
//! ## 🏭 工厂模式特性
//!
//! ### [`SoftwareCoreFactory`] - 主要工厂类
//! - 🔧 实现标准CoreFactory trait
//! - 🔧 提供核心信息和能力描述
//! - 🔧 支持异步核心创建
//! - 🔧 完整的配置验证机制
//!
//! ## 🎯 核心功能
//!
//! ### 核心创建流程
//! ```text
//! 1. 配置验证 → 检查所有参数的有效性
//! 2. 核心实例化 → 创建SoftwareMiningCore对象
//! 3. 初始化配置 → 应用用户配置参数
//! 4. 返回实例 → 提供可用的挖矿核心
//! ```
//!
//! ### 配置验证功能
//! - ✅ 设备数量范围检查 (1-100个设备)
//! - ✅ 算力参数验证 (最小/最大算力合理性)
//! - ✅ 错误率范围验证 (0.0-1.0)
//! - ✅ 设备配置完整性检查
//! - ✅ 自定义参数类型验证
//!
//! ### 默认配置提供
//! - 📋 4个虚拟设备的标准配置
//! - 📋 1-5 GH/s的算力范围
//! - 📋 1%的默认错误率
//! - 📋 1000的批处理大小
//! - 📋 5秒的工作超时时间
//!
//! ## 🔄 使用示例
//!
//! ```rust
//! use cgminer_cpu_btc_core::SoftwareCoreFactory;
//! use cgminer_core::{CoreFactory, CoreConfig};
//!
//! // 创建工厂实例
//! let factory = SoftwareCoreFactory::new();
//!
//! // 获取默认配置
//! let config = factory.default_config();
//!
//! // 创建挖矿核心
//! let core = factory.create_core(config).await?;
//! ```
//!
//! ## ⚙️ 配置参数说明
//!
//! | 参数名 | 类型 | 默认值 | 说明 |
//! |--------|------|--------|------|
//! | `device_count` | u64 | 4 | 虚拟设备数量 |
//! | `min_hashrate` | f64 | 1e9 | 最小算力 (H/s) |
//! | `max_hashrate` | f64 | 5e9 | 最大算力 (H/s) |
//! | `error_rate` | f64 | 0.01 | 模拟错误率 |
//! | `batch_size` | u64 | 1000 | 批处理大小 |
//! | `work_timeout_ms` | u64 | 5000 | 工作超时 (ms) |

use crate::core::SoftwareMiningCore;
use cgminer_core::{
    CoreFactory, CoreType, CoreInfo, CoreConfig, MiningCore, CoreError
};
use async_trait::async_trait;
use tracing::{error, info, debug};

/// 软算法核心工厂
pub struct SoftwareCoreFactory {
    /// 核心信息
    core_info: CoreInfo,
}

impl SoftwareCoreFactory {
    /// 创建新的软算法核心工厂
    pub fn new() -> Self {
        let core_info = CoreInfo::new(
            "Software Mining Core".to_string(),
            CoreType::Custom("software".to_string()),
            crate::VERSION.to_string(),
            "软算法挖矿核心，使用真实的SHA256算法进行CPU挖矿计算。产生真实可用的挖矿数据，适用于测试、开发和低功耗挖矿场景。".to_string(),
            "CGMiner Rust Team".to_string(),
            vec!["software".to_string(), "cpu".to_string()],
        );

        Self { core_info }
    }
}

impl Default for SoftwareCoreFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CoreFactory for SoftwareCoreFactory {
    /// 获取核心类型
    fn core_type(&self) -> CoreType {
        CoreType::Custom("software".to_string())
    }

    /// 获取核心信息
    fn core_info(&self) -> CoreInfo {
        self.core_info.clone()
    }

    /// 创建核心实例
    async fn create_core(&self, config: CoreConfig) -> Result<Box<dyn MiningCore>, CoreError> {
        info!("🏭 创建软算法挖矿核心实例: {}", config.name);
        debug!("📋 配置参数: {:?}", config.custom_params);

        debug!("🔧 创建软算法核心对象...");
        let mut core = SoftwareMiningCore::new(config.name.clone());
        debug!("✅ 软算法核心对象创建成功");

        debug!("🚀 开始初始化软算法核心...");
        match core.initialize(config).await {
            Ok(()) => {
                info!("🎉 软算法核心初始化成功");
            }
            Err(e) => {
                error!("💥 软算法核心初始化失败: {}", e);
                return Err(e);
            }
        }

        debug!("📦 返回软算法核心实例");
        Ok(Box::new(core))
    }

    /// 验证配置
    fn validate_config(&self, config: &CoreConfig) -> Result<(), CoreError> {
        if config.name.is_empty() {
            return Err(CoreError::config("核心名称不能为空"));
        }

        // 验证设备配置
        for (i, device_config) in config.devices.iter().enumerate() {
            if device_config.frequency == 0 {
                return Err(CoreError::config(format!(
                    "设备 {} 的频率不能为0", i
                )));
            }

            if device_config.voltage == 0 {
                return Err(CoreError::config(format!(
                    "设备 {} 的电压不能为0", i
                )));
            }

            if device_config.temperature_limit <= 0.0 {
                return Err(CoreError::config(format!(
                    "设备 {} 的温度限制必须大于0", i
                )));
            }

            if device_config.chip_count == 0 {
                return Err(CoreError::config(format!(
                    "设备 {} 的芯片数量不能为0", i
                )));
            }
        }

        // 验证自定义参数
        if let Some(device_count) = config.custom_params.get("device_count") {
            if let Some(count) = device_count.as_u64() {
                if count == 0 {
                    return Err(CoreError::config("软算法设备数量不能为0"));
                }
                if count > 100 {
                    return Err(CoreError::config("软算法设备数量不能超过100"));
                }
            } else {
                return Err(CoreError::config("device_count 必须是正整数"));
            }
        }

        if let Some(min_hashrate) = config.custom_params.get("min_hashrate") {
            if let Some(hashrate) = min_hashrate.as_f64() {
                if hashrate <= 0.0 {
                    return Err(CoreError::config("最小算力必须大于0"));
                }
            } else {
                return Err(CoreError::config("min_hashrate 必须是正数"));
            }
        }

        if let Some(max_hashrate) = config.custom_params.get("max_hashrate") {
            if let Some(hashrate) = max_hashrate.as_f64() {
                if hashrate <= 0.0 {
                    return Err(CoreError::config("最大算力必须大于0"));
                }
            } else {
                return Err(CoreError::config("max_hashrate 必须是正数"));
            }
        }

        // 验证最小和最大算力的关系
        if let (Some(min_val), Some(max_val)) = (
            config.custom_params.get("min_hashrate").and_then(|v| v.as_f64()),
            config.custom_params.get("max_hashrate").and_then(|v| v.as_f64()),
        ) {
            if min_val >= max_val {
                return Err(CoreError::config("最小算力必须小于最大算力"));
            }
        }

        if let Some(error_rate) = config.custom_params.get("error_rate") {
            if let Some(rate) = error_rate.as_f64() {
                if rate < 0.0 || rate > 1.0 {
                    return Err(CoreError::config("错误率必须在0.0到1.0之间"));
                }
            } else {
                return Err(CoreError::config("error_rate 必须是0.0到1.0之间的数值"));
            }
        }

        Ok(())
    }

    /// 获取默认配置
    fn default_config(&self) -> CoreConfig {
        use std::collections::HashMap;

        let mut custom_params = HashMap::new();
        custom_params.insert("device_count".to_string(), serde_json::Value::Number(serde_json::Number::from(4)));
        custom_params.insert("min_hashrate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(1000000000.0).unwrap())); // 1 GH/s
        custom_params.insert("max_hashrate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(5000000000.0).unwrap())); // 5 GH/s
        custom_params.insert("error_rate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.01).unwrap())); // 1% 错误率
        custom_params.insert("batch_size".to_string(), serde_json::Value::Number(serde_json::Number::from(1000)));
        custom_params.insert("work_timeout_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(5000)));

        CoreConfig {
            name: "software-core".to_string(),
            enabled: true,
            devices: vec![
                cgminer_core::DeviceConfig {
                    chain_id: 0,
                    enabled: true,
                    frequency: 600,
                    voltage: 900,
                    auto_tune: false,
                    chip_count: 64,
                    temperature_limit: 80.0,
                    fan_speed: Some(50),
                },
                cgminer_core::DeviceConfig {
                    chain_id: 1,
                    enabled: true,
                    frequency: 650,
                    voltage: 920,
                    auto_tune: false,
                    chip_count: 64,
                    temperature_limit: 80.0,
                    fan_speed: Some(55),
                },
                cgminer_core::DeviceConfig {
                    chain_id: 2,
                    enabled: true,
                    frequency: 700,
                    voltage: 950,
                    auto_tune: false,
                    chip_count: 64,
                    temperature_limit: 80.0,
                    fan_speed: Some(60),
                },
                cgminer_core::DeviceConfig {
                    chain_id: 3,
                    enabled: true,
                    frequency: 750,
                    voltage: 980,
                    auto_tune: false,
                    chip_count: 64,
                    temperature_limit: 80.0,
                    fan_speed: Some(65),
                },
            ],
            custom_params,
        }
    }
}
