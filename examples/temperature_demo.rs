//! 温度管理演示
//!
//! 这个示例展示如何监控和管理CPU挖矿过程中的温度
//! 包括温度读取、热保护、动态调节等功能

use cgminer_cpu_btc_core::{
    SoftwareDevice,
    temperature::TemperatureMonitor,
};
use cgminer_core::{
    DeviceInfo, DeviceStatus, DeviceConfig, CoreType,
};
use std::time::{Duration, SystemTime, Instant};
use std::sync::Arc;
use tokio;

/// 温度管理器
struct ThermalManager {
    device: Arc<SoftwareDevice>,
    temperature_history: Vec<(SystemTime, f64)>,
    max_temperature: f64,
    warning_temperature: f64,
    critical_temperature: f64,
    cooling_enabled: bool,
}

impl ThermalManager {
    fn new(device: Arc<SoftwareDevice>) -> Self {
        Self {
            device,
            temperature_history: Vec::new(),
            max_temperature: 85.0,      // 最高安全温度
            warning_temperature: 75.0,  // 警告温度
            critical_temperature: 90.0, // 紧急温度
            cooling_enabled: true,
        }
    }

    /// 读取当前温度
    async fn read_temperature(&mut self) -> Result<f64, Box<dyn std::error::Error>> {
        let status = self.device.get_status().await?;
        let temperature = status.temperature;

        // 记录温度历史
        self.temperature_history.push((SystemTime::now(), temperature));

        // 保持最近100个温度记录
        if self.temperature_history.len() > 100 {
            self.temperature_history.remove(0);
        }

        Ok(temperature)
    }

    /// 检查温度状态
    fn check_temperature_status(&self, temperature: f64) -> ThermalStatus {
        if temperature >= self.critical_temperature {
            ThermalStatus::Critical
        } else if temperature >= self.max_temperature {
            ThermalStatus::Overheating
        } else if temperature >= self.warning_temperature {
            ThermalStatus::Warning
        } else if temperature < 40.0 {
            ThermalStatus::Cold
        } else {
            ThermalStatus::Normal
        }
    }

    /// 获取温度趋势
    fn get_temperature_trend(&self) -> f64 {
        if self.temperature_history.len() < 5 {
            return 0.0;
        }

        let recent_temps: Vec<f64> = self.temperature_history
            .iter()
            .rev()
            .take(5)
            .map(|(_, temp)| *temp)
            .collect();

        let recent_avg = recent_temps.iter().sum::<f64>() / recent_temps.len() as f64;

        let older_temps: Vec<f64> = self.temperature_history
            .iter()
            .rev()
            .skip(5)
            .take(5)
            .map(|(_, temp)| *temp)
            .collect();

        if older_temps.is_empty() {
            return 0.0;
        }

        let older_avg = older_temps.iter().sum::<f64>() / older_temps.len() as f64;
        recent_avg - older_avg
    }

    /// 应用热保护措施
    async fn apply_thermal_protection(&self, status: ThermalStatus) -> Result<(), Box<dyn std::error::Error>> {
        match status {
            ThermalStatus::Critical => {
                println!("🚨 紧急温度保护: 立即停止挖矿!");
                // 在实际实现中，这里会停止挖矿操作
            },
            ThermalStatus::Overheating => {
                println!("🔥 过热保护: 降低挖矿强度");
                // 在实际实现中，这里会降低算力目标
            },
            ThermalStatus::Warning => {
                println!("⚠️  温度警告: 增加散热");
                // 在实际实现中，这里会提高风扇速度
            },
            ThermalStatus::Cold => {
                println!("❄️  温度过低: 可以提高性能");
                // 在实际实现中，这里可以提高算力目标
            },
            ThermalStatus::Normal => {
                // 正常温度，无需特殊处理
            }
        }
        Ok(())
    }

    /// 生成温度报告
    fn generate_temperature_report(&self) -> String {
        let mut report = String::new();
        report.push_str("🌡️  温度监控报告\n");
        report.push_str("================\n\n");

        if !self.temperature_history.is_empty() {
            let current_temp = self.temperature_history.last().unwrap().1;
            let max_temp = self.temperature_history.iter().map(|(_, t)| *t).fold(0.0, f64::max);
            let min_temp = self.temperature_history.iter().map(|(_, t)| *t).fold(100.0, f64::min);
            let avg_temp = self.temperature_history.iter().map(|(_, t)| *t).sum::<f64>() / self.temperature_history.len() as f64;

            report.push_str(&format!("📊 温度统计:\n"));
            report.push_str(&format!("  - 当前温度: {:.1}°C\n", current_temp));
            report.push_str(&format!("  - 最高温度: {:.1}°C\n", max_temp));
            report.push_str(&format!("  - 最低温度: {:.1}°C\n", min_temp));
            report.push_str(&format!("  - 平均温度: {:.1}°C\n", avg_temp));
            report.push_str(&format!("  - 温度趋势: {:+.1}°C\n", self.get_temperature_trend()));

            report.push_str(&format!("\n🎯 温度阈值:\n"));
            report.push_str(&format!("  - 警告温度: {:.1}°C\n", self.warning_temperature));
            report.push_str(&format!("  - 最高温度: {:.1}°C\n", self.max_temperature));
            report.push_str(&format!("  - 紧急温度: {:.1}°C\n", self.critical_temperature));

            report.push_str(&format!("\n📈 监控数据:\n"));
            report.push_str(&format!("  - 数据点数: {}\n", self.temperature_history.len()));
            report.push_str(&format!("  - 监控时长: {:.1}分钟\n",
                self.temperature_history.len() as f64 / 60.0));
        }

        report
    }
}

/// 温度状态枚举
#[derive(Debug, Clone, PartialEq)]
enum ThermalStatus {
    Cold,       // 温度过低
    Normal,     // 正常温度
    Warning,    // 警告温度
    Overheating, // 过热
    Critical,   // 紧急过热
}

impl ThermalStatus {
    fn to_emoji(&self) -> &str {
        match self {
            ThermalStatus::Cold => "❄️",
            ThermalStatus::Normal => "✅",
            ThermalStatus::Warning => "⚠️",
            ThermalStatus::Overheating => "🔥",
            ThermalStatus::Critical => "🚨",
        }
    }

    fn to_description(&self) -> &str {
        match self {
            ThermalStatus::Cold => "温度过低",
            ThermalStatus::Normal => "温度正常",
            ThermalStatus::Warning => "温度警告",
            ThermalStatus::Overheating => "设备过热",
            ThermalStatus::Critical => "紧急过热",
        }
    }
}

/// 创建测试设备
async fn create_test_device() -> Result<Arc<SoftwareDevice>, Box<dyn std::error::Error>> {
    let device_info = DeviceInfo::new(
        1,
        "Temperature Test Device".to_string(),
        "cpu".to_string(),
        0,
    );

    let config = DeviceConfig {
        frequency: 2200,
        voltage: 1200,
        fan_speed: Some(50),
        power_limit: Some(150),
        temperature_limit: Some(85),
        auto_fan: true,
        auto_frequency: true,
        auto_voltage: false,
    };

    let device = SoftwareDevice::new(
        device_info,
        config,
        f64::MAX,  // 无算力限制，最大性能
        0.001,     // 0.1% error rate
        50000,     // 大批次处理
    ).await?;

    Ok(Arc::new(device))
}

/// 温度监控演示
async fn temperature_monitoring_demo(device: Arc<SoftwareDevice>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌡️  开始温度监控演示");
    println!("====================");

    let mut thermal_manager = ThermalManager::new(device);
    let monitoring_duration = Duration::from_secs(60); // 监控1分钟
    let start_time = Instant::now();
    let mut sample_count = 0;

    println!("监控时长: {}秒", monitoring_duration.as_secs());
    println!("采样间隔: 2秒");
    println!("----------------------------------------");

    while start_time.elapsed() < monitoring_duration {
        sample_count += 1;

        // 读取温度
        let temperature = thermal_manager.read_temperature().await?;
        let status = thermal_manager.check_temperature_status(temperature);
        let trend = thermal_manager.get_temperature_trend();

        // 显示温度信息
        println!("#{:02} | {:.1}°C | {} {} | 趋势: {:+.1}°C",
            sample_count,
            temperature,
            status.to_emoji(),
            status.to_description(),
            trend
        );

        // 应用热保护
        thermal_manager.apply_thermal_protection(status).await?;

        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // 生成报告
    println!("\n{}", thermal_manager.generate_temperature_report());

    Ok(())
}

/// 热压力测试
async fn thermal_stress_test(device: Arc<SoftwareDevice>) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔥 热压力测试");
    println!("==============");

    let test_phases = vec![
        ("预热阶段", 30, 1_000_000.0),   // 30秒，1 MH/s
        ("加热阶段", 45, 3_000_000.0),   // 45秒，3 MH/s
        ("高温阶段", 30, 5_000_000.0),   // 30秒，5 MH/s
        ("冷却阶段", 60, 500_000.0),     // 60秒，0.5 MH/s
    ];

    for (phase_name, duration, target_hashrate) in test_phases {
        println!("\n📊 {} ({}秒, {:.1} MH/s)",
            phase_name, duration, target_hashrate / 1_000_000.0);

        let phase_start = Instant::now();
        let mut temp_readings = Vec::new();

        while phase_start.elapsed().as_secs() < duration {
            let status = device.get_status().await?;
            temp_readings.push(status.temperature);

            print!("  {:.1}°C", status.temperature);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            tokio::time::sleep(Duration::from_secs(3)).await;
        }

        println!(); // 换行

        // 分析这个阶段的温度变化
        if !temp_readings.is_empty() {
            let max_temp = temp_readings.iter().fold(0.0, |a, &b| a.max(b));
            let min_temp = temp_readings.iter().fold(100.0, |a, &b| a.min(b));
            let avg_temp = temp_readings.iter().sum::<f64>() / temp_readings.len() as f64;
            let temp_rise = temp_readings.last().unwrap() - temp_readings.first().unwrap();

            println!("  结果: 最高{:.1}°C, 最低{:.1}°C, 平均{:.1}°C, 变化{:+.1}°C",
                max_temp, min_temp, avg_temp, temp_rise);
        }
    }

    Ok(())
}

/// 自适应温度控制演示
async fn adaptive_thermal_control_demo(device: Arc<SoftwareDevice>) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎛️  自适应温度控制演示");
    println!("======================");

    let target_temperature = 70.0; // 目标温度
    let mut current_hashrate = 2_000_000.0; // 初始算力
    let control_duration = Duration::from_secs(120); // 控制2分钟
    let start_time = Instant::now();

    println!("目标温度: {:.1}°C", target_temperature);
    println!("初始算力: {:.1} MH/s", current_hashrate / 1_000_000.0);
    println!("控制时长: {}秒", control_duration.as_secs());
    println!("----------------------------------------");

    while start_time.elapsed() < control_duration {
        let status = device.get_status().await?;
        let current_temp = status.temperature;
        let temp_error = current_temp - target_temperature;

        // 简单的PID控制算法
        let adjustment_factor = -temp_error * 0.1; // 比例控制
        let hashrate_adjustment = current_hashrate * adjustment_factor;
        current_hashrate = (current_hashrate + hashrate_adjustment).max(500_000.0).min(5_000_000.0);

        println!("温度: {:.1}°C | 误差: {:+.1}°C | 算力: {:.2} MH/s | 调整: {:+.1}%",
            current_temp,
            temp_error,
            current_hashrate / 1_000_000.0,
            adjustment_factor * 100.0
        );

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    println!("\n✅ 自适应控制演示完成");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CGMiner CPU BTC Core - 温度管理演示");
    println!("===================================");

    // 创建测试设备
    println!("📱 创建温度测试设备...");
    let device = create_test_device().await?;
    println!("✅ 设备创建成功");

    // 温度监控演示
    temperature_monitoring_demo(Arc::clone(&device)).await?;

    // 热压力测试
    thermal_stress_test(Arc::clone(&device)).await?;

    // 自适应温度控制
    adaptive_thermal_control_demo(device).await?;

    println!("\n🎉 温度管理演示完成！");
    println!("\n💡 温度管理要点:");
    println!("  - 持续监控温度，避免过热损坏硬件");
    println!("  - 设置合理的温度阈值和保护措施");
    println!("  - 使用自适应控制维持最佳工作温度");
    println!("  - 定期清理散热器，保持良好散热");

    Ok(())
}
