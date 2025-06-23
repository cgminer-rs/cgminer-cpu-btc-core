//! 温度管理功能测试

#[cfg(test)]
mod tests {
    use super::super::temperature::{
        TemperatureConfig, TemperatureManager, get_platform_temperature_capabilities
    };
    use cgminer_core::MiningDevice;

    #[test]
    fn test_temperature_config_default() {
        let config = TemperatureConfig::default();
        assert_eq!(config.enable_real_monitoring, true);
        assert_eq!(config.update_interval, 5);
        assert_eq!(config.warning_threshold, 80.0);
        assert_eq!(config.critical_threshold, 90.0);
        assert_eq!(config.simulated_base_temp, 35.0);
    }

    #[test]
    fn test_temperature_manager_creation() {
        let config = TemperatureConfig::default();
        let manager = TemperatureManager::new(config);

        // 检查是否有温度监控功能
        println!("温度监控功能: {}", manager.has_temperature_monitoring());
        println!("提供者信息: {}", manager.provider_info());

        // 尝试读取温度
        match manager.read_temperature() {
            Ok(temp) => println!("读取到温度: {:.1}°C", temp),
            Err(e) => println!("温度读取失败: {}", e),
        }
    }

    #[test]
    fn test_platform_capabilities() {
        let capabilities = get_platform_temperature_capabilities();
        println!("平台温度监控能力: {}", capabilities);

        assert!(!capabilities.platform_name.is_empty());
        assert!(!capabilities.arch.is_empty());
        assert!(!capabilities.provider_type.is_empty());
    }

    #[tokio::test]
    async fn test_device_temperature_integration() {
        use crate::device::SoftwareDevice;
        use cgminer_core::{DeviceInfo, DeviceConfig};

        let device_info = DeviceInfo::new(
            1,
            "Test Device".to_string(),
            "CPU".to_string(),
            1, // chain_id as u8
        );

        let device_config = DeviceConfig {
            frequency: 600,
            voltage: 900,
            fan_speed: Some(50),
            ..Default::default()
        };

        let mut device = SoftwareDevice::new(
            device_info,
            device_config.clone(),
            1000000.0, // 1MH/s
            0.01,      // 1% error rate
            1000,      // batch size
        ).await.expect("Failed to create device");

        // 初始化设备
        device.initialize(device_config).await.expect("Failed to initialize device");

        // 获取统计信息
        let stats = device.get_stats().await.expect("Failed to get stats");

        if let Some(temp) = stats.temperature {
            println!("设备温度: {:.1}°C", temp.celsius);
        } else {
            println!("设备不支持温度监控");
        }

        // 检查健康状态
        let health = device.health_check().await.expect("Failed to check health");
        println!("设备健康状态: {}", if health { "正常" } else { "异常" });
    }
}
