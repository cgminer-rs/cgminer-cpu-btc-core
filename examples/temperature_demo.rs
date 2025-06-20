//! 温度监控功能演示
//!
//! 这个示例展示了cgminer-cpu-btc-core中的温度监控功能

use cgminer_cpu_btc_core::temperature::{
    TemperatureConfig, TemperatureManager, get_platform_temperature_capabilities
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🌡️  cgminer-cpu-btc-core 温度监控演示");
    println!("=====================================");

    // 1. 显示平台温度监控能力
    println!("\n📊 平台温度监控能力:");
    let capabilities = get_platform_temperature_capabilities();
    println!("{}", capabilities);

    // 2. 创建温度管理器
    println!("\n🔧 创建温度管理器...");
    let config = TemperatureConfig::default();
    let temp_manager = TemperatureManager::new(config);

    println!("温度提供者: {}", temp_manager.provider_info());
    println!("支持真实监控: {}", temp_manager.supports_real_monitoring());
    println!("有温度监控功能: {}", temp_manager.has_temperature_monitoring());

    // 3. 尝试读取温度
    println!("\n🌡️  尝试读取温度...");
    match temp_manager.read_temperature() {
        Ok(temp) => {
            println!("✅ 成功读取温度: {:.1}°C", temp);
            
            // 检查温度状态
            match temp_manager.check_temperature_status() {
                Ok(status) => println!("温度状态: {}", status),
                Err(e) => println!("温度状态检查失败: {}", e),
            }
        }
        Err(e) => {
            println!("❌ 温度读取失败: {}", e);
            println!("这是预期的行为，因为CPU核心不支持模拟温度");
        }
    }

    // 4. 演示配置选项
    println!("\n⚙️  温度配置选项:");
    let config = TemperatureConfig::default();
    println!("启用真实监控: {}", config.enable_real_monitoring);
    println!("更新间隔: {}秒", config.update_interval);
    println!("警告阈值: {:.1}°C", config.warning_threshold);
    println!("危险阈值: {:.1}°C", config.critical_threshold);
    println!("模拟基础温度: {:.1}°C", config.simulated_base_temp);

    // 5. 演示不同配置的行为
    println!("\n🔄 测试不同配置...");
    
    // 禁用真实监控的配置
    let mut disabled_config = TemperatureConfig::default();
    disabled_config.enable_real_monitoring = false;
    let disabled_manager = TemperatureManager::new(disabled_config);
    
    println!("禁用真实监控的管理器:");
    println!("  提供者: {}", disabled_manager.provider_info());
    println!("  支持真实监控: {}", disabled_manager.supports_real_monitoring());
    println!("  有温度监控功能: {}", disabled_manager.has_temperature_monitoring());

    match disabled_manager.read_temperature() {
        Ok(temp) => println!("  温度: {:.1}°C", temp),
        Err(e) => println!("  温度读取失败: {} (预期行为)", e),
    }

    println!("\n✅ 演示完成!");
    println!("\n📝 总结:");
    println!("- CPU核心不提供模拟温度功能");
    println!("- 如果平台不支持真实温度监控，温度功能将不可用");
    println!("- 主程序cgminer应该检查has_temperature_monitoring()来决定是否显示温度信息");
    println!("- 这确保了CPU核心的纯净性，避免了虚假的温度数据");

    Ok(())
}
