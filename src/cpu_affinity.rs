use std::collections::HashMap;
use tracing::{info, warn, debug};
use core_affinity::{CoreId, get_core_ids, set_for_current};

/// CPU绑定管理器
/// 负责管理软算法核心的CPU绑定策略
pub struct CpuAffinityManager {
    /// 系统可用的CPU核心ID列表
    available_cores: Vec<CoreId>,
    /// 设备到CPU核心的映射
    device_core_mapping: HashMap<u32, CoreId>,
    /// 是否启用CPU绑定
    enabled: bool,
    /// CPU绑定策略
    strategy: CpuAffinityStrategy,
}

/// CPU绑定策略
#[derive(Debug, Clone)]
pub enum CpuAffinityStrategy {
    /// 轮询分配：按顺序将设备分配到不同的CPU核心
    RoundRobin,
    /// 手动指定：手动指定每个设备的CPU核心
    Manual(HashMap<u32, usize>),
    /// 性能核心优先：优先使用性能核心（在支持的系统上）
    PerformanceFirst,
    /// 避免超线程：只使用物理核心，避免超线程
    PhysicalCoresOnly,
    /// 智能分配：基于系统负载和CPU特性智能分配
    Intelligent,
    /// 负载均衡：动态监控CPU负载并重新分配
    LoadBalanced,
}

impl CpuAffinityManager {
    /// 创建新的CPU绑定管理器
    pub fn new(enabled: bool, strategy: CpuAffinityStrategy) -> Self {
        let available_cores = get_core_ids().unwrap_or_else(|| {
            warn!("无法获取系统CPU核心信息，CPU绑定功能将被禁用");
            Vec::new()
        });

        info!("系统检测到 {} 个CPU核心", available_cores.len());

        let is_enabled = enabled && !available_cores.is_empty();

        if enabled && available_cores.is_empty() {
            warn!("CPU绑定已启用但无法获取CPU核心信息，CPU绑定功能将被禁用");
        } else if !enabled {
            info!("CPU绑定功能已禁用");
        } else {
            info!("CPU绑定功能已启用，将使用 {:?} 策略", strategy);
            #[cfg(target_os = "macos")]
            info!("注意：在macOS环境下，CPU绑定可能需要特殊权限或可能不被完全支持");
        }

        Self {
            available_cores,
            device_core_mapping: HashMap::new(),
            enabled: is_enabled,
            strategy,
        }
    }

    /// 获取系统CPU核心数量
    pub fn get_cpu_count() -> usize {
        num_cpus::get()
    }

    /// 获取系统物理CPU核心数量
    pub fn get_physical_cpu_count() -> usize {
        num_cpus::get_physical()
    }

    /// 检查是否启用CPU绑定
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 获取可用的CPU核心数量
    pub fn available_core_count(&self) -> usize {
        self.available_cores.len()
    }

    /// 为设备分配CPU核心
    pub fn assign_cpu_core(&mut self, device_id: u32) -> Option<CoreId> {
        if !self.enabled {
            return None;
        }

        if self.available_cores.is_empty() {
            warn!("没有可用的CPU核心进行绑定");
            return None;
        }

        let core_id = match &self.strategy {
            CpuAffinityStrategy::RoundRobin => {
                // 轮询分配
                let index = (device_id as usize) % self.available_cores.len();
                self.available_cores[index]
            }
            CpuAffinityStrategy::Manual(mapping) => {
                // 手动指定
                if let Some(&core_index) = mapping.get(&device_id) {
                    if core_index < self.available_cores.len() {
                        self.available_cores[core_index]
                    } else {
                        warn!("设备 {} 指定的CPU核心索引 {} 超出范围，使用轮询分配", device_id, core_index);
                        let index = (device_id as usize) % self.available_cores.len();
                        self.available_cores[index]
                    }
                } else {
                    warn!("设备 {} 没有手动指定CPU核心，使用轮询分配", device_id);
                    let index = (device_id as usize) % self.available_cores.len();
                    self.available_cores[index]
                }
            }
            CpuAffinityStrategy::PerformanceFirst => {
                // 性能核心优先（简化实现，使用前半部分核心）
                let perf_core_count = self.available_cores.len() / 2;
                let index = (device_id as usize) % perf_core_count.max(1);
                self.available_cores[index]
            }
            CpuAffinityStrategy::PhysicalCoresOnly => {
                // 只使用物理核心（简化实现，使用奇数索引的核心）
                let physical_cores: Vec<_> = self.available_cores.iter()
                    .enumerate()
                    .filter(|(i, _)| i % 2 == 0)
                    .map(|(_, &core)| core)
                    .collect();

                if physical_cores.is_empty() {
                    warn!("没有可用的物理CPU核心，回退到轮询分配");
                    let index = (device_id as usize) % self.available_cores.len();
                    self.available_cores[index]
                } else {
                    let index = (device_id as usize) % physical_cores.len();
                    physical_cores[index]
                }
            }
            CpuAffinityStrategy::Intelligent => {
                // 智能分配：基于CPU数量和设备数量优化分配
                let physical_count = Self::get_physical_cpu_count();

                // 如果物理核心数量足够，优先使用物理核心
                if physical_count >= 4 && device_id < physical_count as u32 {
                    let index = (device_id as usize * 2) % self.available_cores.len();
                    self.available_cores[index]
                } else {
                    // 否则使用轮询分配
                    let index = (device_id as usize) % self.available_cores.len();
                    self.available_cores[index]
                }
            }
            CpuAffinityStrategy::LoadBalanced => {
                // 负载均衡：简化实现，使用轮询分配
                // 在实际实现中，这里应该监控CPU负载并动态调整
                let index = (device_id as usize) % self.available_cores.len();
                self.available_cores[index]
            }
        };

        // 记录映射关系
        self.device_core_mapping.insert(device_id, core_id);

        info!("设备 {} 分配到CPU核心 {:?}", device_id, core_id);
        Some(core_id)
    }

    /// 获取设备的CPU核心分配
    pub fn get_device_core(&self, device_id: u32) -> Option<CoreId> {
        self.device_core_mapping.get(&device_id).copied()
    }

    /// 为当前线程设置CPU绑定
    pub fn bind_current_thread(&self, device_id: u32) -> Result<(), String> {
        if !self.enabled {
            debug!("CPU绑定已禁用，跳过线程绑定");
            return Ok(());
        }

        if let Some(core_id) = self.get_device_core(device_id) {
            match set_for_current(core_id) {
                true => {
                    info!("线程成功绑定到CPU核心 {:?} (设备 {})", core_id, device_id);
                    Ok(())
                }
                false => {
                    let error_msg = format!("无法将线程绑定到CPU核心 {:?} (设备 {})", core_id, device_id);
                    warn!("{}", error_msg);
                    Err(error_msg)
                }
            }
        } else {
            let error_msg = format!("设备 {} 没有分配CPU核心", device_id);
            warn!("{}", error_msg);
            Err(error_msg)
        }
    }

    /// 显示CPU绑定状态
    pub fn print_affinity_status(&self) {
        info!("═══════════════════════════════════════════════════════════");
        info!("🔗 CPU绑定状态报告");
        info!("═══════════════════════════════════════════════════════════");
        info!("   🖥️  系统CPU信息:");
        info!("      💻 逻辑CPU核心数: {}", Self::get_cpu_count());
        info!("      🔧 物理CPU核心数: {}", Self::get_physical_cpu_count());
        info!("      ✅ 可用核心数: {}", self.available_core_count());
        info!("   ⚙️  CPU绑定配置:");
        info!("      🔗 绑定状态: {}", if self.enabled { "启用" } else { "禁用" });
        info!("      📋 绑定策略: {:?}", self.strategy);

        if self.enabled && !self.device_core_mapping.is_empty() {
            info!("   📊 设备CPU分配:");
            for (device_id, core_id) in &self.device_core_mapping {
                info!("      🎯 设备 {} → CPU核心 {:?}", device_id, core_id);
            }
        }
        info!("═══════════════════════════════════════════════════════════");
    }

    /// 获取CPU绑定统计信息
    pub fn get_affinity_stats(&self) -> CpuAffinityStats {
        CpuAffinityStats {
            total_cpu_cores: Self::get_cpu_count(),
            physical_cpu_cores: Self::get_physical_cpu_count(),
            available_cores: self.available_core_count(),
            enabled: self.enabled,
            bound_devices: self.device_core_mapping.len(),
            strategy: self.strategy.clone(),
        }
    }
}

/// CPU绑定统计信息
#[derive(Debug, Clone)]
pub struct CpuAffinityStats {
    /// 系统总CPU核心数
    pub total_cpu_cores: usize,
    /// 物理CPU核心数
    pub physical_cpu_cores: usize,
    /// 可用核心数
    pub available_cores: usize,
    /// 是否启用CPU绑定
    pub enabled: bool,
    /// 已绑定的设备数量
    pub bound_devices: usize,
    /// 绑定策略
    pub strategy: CpuAffinityStrategy,
}

/// CPU绑定配置
#[derive(Debug, Clone)]
pub struct CpuAffinityConfig {
    /// 是否启用CPU绑定
    pub enabled: bool,
    /// 绑定策略
    pub strategy: CpuAffinityStrategy,
    /// 手动核心映射（仅在Manual策略下使用）
    pub manual_mapping: Option<HashMap<u32, usize>>,
}

impl Default for CpuAffinityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: CpuAffinityStrategy::RoundRobin,
            manual_mapping: None,
        }
    }
}

impl CpuAffinityConfig {
    /// 创建轮询分配配置
    pub fn round_robin() -> Self {
        Self {
            enabled: true,
            strategy: CpuAffinityStrategy::RoundRobin,
            manual_mapping: None,
        }
    }

    /// 创建手动分配配置
    pub fn manual(mapping: HashMap<u32, usize>) -> Self {
        Self {
            enabled: true,
            strategy: CpuAffinityStrategy::Manual(mapping.clone()),
            manual_mapping: Some(mapping),
        }
    }

    /// 创建性能核心优先配置
    pub fn performance_first() -> Self {
        Self {
            enabled: true,
            strategy: CpuAffinityStrategy::PerformanceFirst,
            manual_mapping: None,
        }
    }

    /// 创建物理核心配置
    pub fn physical_cores_only() -> Self {
        Self {
            enabled: true,
            strategy: CpuAffinityStrategy::PhysicalCoresOnly,
            manual_mapping: None,
        }
    }

    /// 禁用CPU绑定
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            strategy: CpuAffinityStrategy::RoundRobin,
            manual_mapping: None,
        }
    }
}
