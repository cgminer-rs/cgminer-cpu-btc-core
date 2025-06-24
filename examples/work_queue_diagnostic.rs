use cgminer_cpu_btc_core::{
    SoftwareDevice,
    concurrent_optimization::LockFreeWorkQueue
};
use cgminer_core::{Work, DeviceInfo, DeviceConfig, MiningDevice};
use std::sync::Arc;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🔍 工作队列任务获取诊断工具");
    println!("========================================");

    // 创建设备
    let device_info = DeviceInfo::new(1, "CPU-Test".to_string(), "软件设备".to_string(), 0);
    let device_config = DeviceConfig::default();

    let mut device = SoftwareDevice::new(
        device_info,
        device_config,
        1_000_000.0, // 1 MH/s
        0.0,         // 无错误率
        1000         // 批次大小
    ).await?;

    // 测试场景1：无结果通道的情况
    println!("\n📋 测试场景1：无结果通道 (传统工作队列模式)");
    println!("----------------------------------------");

    // 初始化设备
    device.initialize(DeviceConfig::default()).await?;

    // 提交一些工作
    let work1 = Arc::new(Work::new("test_job_1".to_string(), [0u8; 32], [0u8; 80], 1.0));
    let work2 = Arc::new(Work::new("test_job_2".to_string(), [0u8; 32], [0u8; 80], 1.0));

    device.submit_work(work1.clone()).await?;
    device.submit_work(work2.clone()).await?;

    println!("✅ 已提交2个工作任务");

    // 尝试获取结果
    for i in 1..=3 {
        match device.get_result().await? {
            Some(result) => {
                println!("✅ 第{}次获取结果成功: work_id={}", i, result.work_id);
            }
            None => {
                println!("❌ 第{}次获取结果失败: 返回None", i);
            }
        }
    }

    // 测试场景2：有结果通道的情况
    println!("\n📋 测试场景2：有结果通道 (立即上报模式)");
    println!("----------------------------------------");

    // 创建新设备并设置结果通道
    let mut device_with_channel = SoftwareDevice::new(
        DeviceInfo::new(2, "CPU-Test-2".to_string(), "软件设备".to_string(), 0),
        DeviceConfig::default(),
        1_000_000.0,
        0.0,
        1000
    ).await?;

    // 设置结果通道
    let (tx, mut rx) = mpsc::unbounded_channel();
    device_with_channel.set_result_sender(tx);

    device_with_channel.initialize(DeviceConfig::default()).await?;

    // 提交工作
    let work3 = Arc::new(Work::new("test_job_3".to_string(), [0u8; 32], [0u8; 80], 1.0));
    device_with_channel.submit_work(work3.clone()).await?;

    println!("✅ 已提交1个工作任务到有通道的设备");

    // 尝试获取结果 (应该返回None)
    match device_with_channel.get_result().await? {
        Some(result) => {
            println!("🚨 意外：有通道设备的get_result()返回了结果: work_id={}", result.work_id);
        }
        None => {
            println!("✅ 预期行为：有通道设备的get_result()返回None");
        }
    }

    // 检查通道是否接收到结果
    match rx.try_recv() {
        Ok(result) => {
            println!("✅ 通道接收到结果: work_id={}", result.work_id);
        }
        Err(_) => {
            println!("❌ 通道未接收到结果");
        }
    }

    // 测试场景3：直接测试工作队列
    println!("\n📋 测试场景3：直接测试无锁工作队列");
    println!("----------------------------------------");

    let queue = LockFreeWorkQueue::new(10);

    let work4 = Arc::new(Work::new("direct_test".to_string(), [0u8; 32], [0u8; 80], 1.0));

    // 入队
    match queue.enqueue_work(work4.clone()) {
        Ok(()) => println!("✅ 工作入队成功"),
        Err(_) => println!("❌ 工作入队失败"),
    }

    // 出队
    match queue.dequeue_work() {
        Some(work) => println!("✅ 工作出队成功: id={}", work.id),
        None => println!("❌ 工作出队失败: 队列为空"),
    }

    // 统计信息
    let stats = queue.get_stats();
    println!("📊 队列统计: 入队={}, 出队={}, 待处理={}, 活跃={}",
             stats.total_enqueued, stats.total_dequeued,
             stats.pending_count, stats.active_count);

    println!("\n🎯 诊断完成！");
    println!("如果看到'预期行为：有通道设备的get_result()返回None'，");
    println!("说明问题就在于：设置了结果通道后，get_result()不再从工作队列获取任务。");

    Ok(())
}
