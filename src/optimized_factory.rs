//! 优化CPU核心工厂实现

use crate::optimized_core::OptimizedCpuMiningCore;
use cgminer_core::{
    CoreFactory, CoreType, CoreInfo, CoreConfig, MiningCore, CoreError
};
use async_trait::async_trait;
use tracing::{error, info, debug};
use std::collections::HashMap;

/// 优化CPU核心工厂
pub struct OptimizedCpuCoreFactory {
    /// 核心信息
    core_info: CoreInfo,
}

impl OptimizedCpuCoreFactory {
    /// 创建新的优化CPU核心工厂
    pub fn new() -> Self {
        let core_info = CoreInfo::new(
            "Optimized CPU Mining Core".to_string(),
            CoreType::Custom("optimized_cpu".to_string()),
            crate::VERSION.to_string(),
            "优化CPU挖矿核心，支持SIMD加速、智能温度管理和动态负载均衡。专门针对现代CPU架构优化，提供最佳的挖矿性能和效率。".to_string(),
            "CGMiner Rust Team".to_string(),
            vec!["cpu".to_string(), "simd".to_string(), "optimized".to_string()],
        );

        Self { core_info }
    }
}

impl Default for OptimizedCpuCoreFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CoreFactory for OptimizedCpuCoreFactory {
    /// 获取核心类型
    fn core_type(&self) -> CoreType {
        CoreType::Custom("optimized_cpu".to_string())
    }

    /// 获取核心信息
    fn core_info(&self) -> CoreInfo {
        self.core_info.clone()
    }

    /// 创建核心实例
    async fn create_core(&self, config: CoreConfig) -> Result<Box<dyn MiningCore>, CoreError> {
        info!("🏭 创建优化CPU挖矿核心实例: {}", config.name);
        debug!("📋 配置参数: {:?}", config.custom_params);

        debug!("🔧 创建优化CPU核心对象...");
        let mut core = OptimizedCpuMiningCore::new(config.name.clone());
        debug!("✅ 优化CPU核心对象创建成功");

        debug!("🚀 开始初始化优化CPU核心...");
        match core.initialize(config).await {
            Ok(()) => {
                info!("🎉 优化CPU核心初始化成功");
            }
            Err(e) => {
                error!("💥 优化CPU核心初始化失败: {}", e);
                return Err(e);
            }
        }

        debug!("📦 返回优化CPU核心实例");
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

        // 验证优化CPU特有参数
        if let Some(device_count) = config.custom_params.get("device_count") {
            if let Some(count) = device_count.as_u64() {
                if count == 0 {
                    return Err(CoreError::config("CPU设备数量不能为0"));
                }
                if count > 256 {
                    return Err(CoreError::config("CPU设备数量不能超过256"));
                }
            } else {
                return Err(CoreError::config("device_count 必须是正整数"));
            }
        }

        // 验证算力配置
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

        // 验证批处理大小
        if let Some(batch_size) = config.custom_params.get("batch_size") {
            if let Some(size) = batch_size.as_u64() {
                if size == 0 {
                    return Err(CoreError::config("批处理大小不能为0"));
                }
                if size > 100000 {
                    return Err(CoreError::config("批处理大小不能超过100000"));
                }
            } else {
                return Err(CoreError::config("batch_size 必须是正整数"));
            }
        }

        // 验证SIMD配置
        if let Some(simd_config) = config.custom_params.get("simd") {
            if let Some(simd_obj) = simd_config.as_object() {
                if let Some(enabled) = simd_obj.get("enabled") {
                    if !enabled.is_boolean() {
                        return Err(CoreError::config("SIMD enabled 必须是布尔值"));
                    }
                }
            }
        }

        // 验证温度管理配置
        if let Some(thermal_config) = config.custom_params.get("thermal") {
            if let Some(thermal_obj) = thermal_config.as_object() {
                if let Some(target_temp) = thermal_obj.get("target_temperature") {
                    if let Some(temp) = target_temp.as_f64() {
                        if temp <= 0.0 || temp > 100.0 {
                            return Err(CoreError::config("目标温度必须在0-100°C之间"));
                        }
                    }
                }
                if let Some(max_temp) = thermal_obj.get("max_temperature") {
                    if let Some(temp) = max_temp.as_f64() {
                        if temp <= 0.0 || temp > 120.0 {
                            return Err(CoreError::config("最大温度必须在0-120°C之间"));
                        }
                    }
                }
            }
        }

        // 验证功耗管理配置
        if let Some(power_config) = config.custom_params.get("power") {
            if let Some(power_obj) = power_config.as_object() {
                if let Some(power_budget) = power_obj.get("power_budget_watts") {
                    if let Some(budget) = power_budget.as_f64() {
                        if budget <= 0.0 {
                            return Err(CoreError::config("功耗预算必须大于0"));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 获取默认配置
    fn default_config(&self) -> CoreConfig {
        let mut custom_params = HashMap::new();
        
        // 基础配置
        custom_params.insert("device_count".to_string(), serde_json::Value::Number(serde_json::Number::from(8)));
        custom_params.insert("min_hashrate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(5000000000.0).unwrap())); // 5 GH/s
        custom_params.insert("max_hashrate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(20000000000.0).unwrap())); // 20 GH/s
        custom_params.insert("error_rate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.001).unwrap())); // 0.1% 错误率
        custom_params.insert("batch_size".to_string(), serde_json::Value::Number(serde_json::Number::from(20000)));
        custom_params.insert("work_timeout_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(1000)));

        // SIMD配置
        let mut simd_config = serde_json::Map::new();
        simd_config.insert("enabled".to_string(), serde_json::Value::Bool(true));
        simd_config.insert("prefer_avx512".to_string(), serde_json::Value::Bool(true));
        simd_config.insert("prefer_avx2".to_string(), serde_json::Value::Bool(true));
        custom_params.insert("simd".to_string(), serde_json::Value::Object(simd_config));

        // CPU绑定配置
        let mut cpu_affinity_config = serde_json::Map::new();
        cpu_affinity_config.insert("enabled".to_string(), serde_json::Value::Bool(true));
        cpu_affinity_config.insert("strategy".to_string(), serde_json::Value::String("intelligent".to_string()));
        cpu_affinity_config.insert("prefer_performance_cores".to_string(), serde_json::Value::Bool(true));
        custom_params.insert("cpu_affinity".to_string(), serde_json::Value::Object(cpu_affinity_config));

        // 温度管理配置
        let mut thermal_config = serde_json::Map::new();
        thermal_config.insert("enabled".to_string(), serde_json::Value::Bool(true));
        thermal_config.insert("target_temperature".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(75.0).unwrap()));
        thermal_config.insert("max_temperature".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(85.0).unwrap()));
        thermal_config.insert("cooling_strategy".to_string(), serde_json::Value::String("adaptive".to_string()));
        custom_params.insert("thermal".to_string(), serde_json::Value::Object(thermal_config));

        // 功耗管理配置
        let mut power_config = serde_json::Map::new();
        power_config.insert("enabled".to_string(), serde_json::Value::Bool(true));
        power_config.insert("power_budget_watts".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(150.0).unwrap()));
        power_config.insert("frequency_scaling".to_string(), serde_json::Value::Bool(true));
        power_config.insert("efficiency_mode".to_string(), serde_json::Value::String("balanced".to_string()));
        custom_params.insert("power".to_string(), serde_json::Value::Object(power_config));

        // 性能优化配置
        custom_params.insert("enable_performance_optimization".to_string(), serde_json::Value::Bool(true));

        CoreConfig {
            name: "optimized-cpu-core".to_string(),
            enabled: true,
            devices: vec![
                cgminer_core::DeviceConfig {
                    chain_id: 0,
                    enabled: true,
                    frequency: 800,  // 更高的频率
                    voltage: 1000,   // 适中的电压
                    auto_tune: true, // 启用自动调优
                    chip_count: 1,   // CPU核心通常为1
                    temperature_limit: 85.0,
                    fan_speed: Some(70),
                },
            ],
            custom_params,
        }
    }
}
