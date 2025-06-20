//! 平台优化模块
//!
//! 提供针对不同平台和CPU架构的优化配置和策略

use serde::{Deserialize, Serialize};

/// 平台优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformOptimization {
    /// 平台名称
    pub platform_name: String,
    /// CPU架构
    pub cpu_arch: String,
    /// 优化级别
    pub optimization_level: OptimizationLevel,
    /// CPU让出频率
    pub yield_frequency: u64,
    /// 批处理大小优化
    pub batch_size_multiplier: f64,
    /// 线程数优化
    pub thread_count_multiplier: f64,
    /// 内存对齐优化
    pub memory_alignment: usize,
    /// SIMD优化配置
    pub simd_config: SimdConfig,
}

/// 优化级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// 基础优化
    Basic,
    /// 标准优化
    Standard,
    /// 高级优化
    Advanced,
    /// 极致优化
    Extreme,
}

/// SIMD优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimdConfig {
    /// 是否启用SIMD
    pub enabled: bool,
    /// 支持的指令集
    pub instruction_sets: Vec<String>,
    /// 向量宽度
    pub vector_width: usize,
    /// 并行度
    pub parallelism: usize,
}

impl PlatformOptimization {
    /// 获取当前平台的优化配置
    pub fn get_current_platform_config() -> Self {
        let platform_name = std::env::consts::OS.to_string();
        let cpu_arch = std::env::consts::ARCH.to_string();

        match (platform_name.as_str(), cpu_arch.as_str()) {
            ("macos", "aarch64") => Self::apple_silicon_config(),
            ("macos", "x86_64") => Self::intel_mac_config(),
            ("linux", "x86_64") => Self::linux_x86_64_config(),
            ("linux", "aarch64") => Self::linux_arm64_config(),
            ("windows", "x86_64") => Self::windows_x86_64_config(),
            _ => Self::default_config(),
        }
    }

    /// Apple Silicon (M1/M2/M3/M4) 优化配置
    fn apple_silicon_config() -> Self {
        Self {
            platform_name: "macOS".to_string(),
            cpu_arch: "aarch64".to_string(),
            optimization_level: OptimizationLevel::Extreme,
            yield_frequency: 10000, // Apple Silicon 高效核心，较少让出
            batch_size_multiplier: 1.5,
            thread_count_multiplier: 0.8, // 80% CPU使用率
            memory_alignment: 64, // Apple Silicon 缓存行大小
            simd_config: SimdConfig {
                enabled: true,
                instruction_sets: vec![
                    "neon".to_string(),
                    "crypto".to_string(),
                    "sha2".to_string(),
                    "aes".to_string(),
                ],
                vector_width: 128,
                parallelism: 4,
            },
        }
    }

    /// Intel Mac 优化配置
    fn intel_mac_config() -> Self {
        Self {
            platform_name: "macOS".to_string(),
            cpu_arch: "x86_64".to_string(),
            optimization_level: OptimizationLevel::Advanced,
            yield_frequency: 8000,
            batch_size_multiplier: 1.2,
            thread_count_multiplier: 0.75,
            memory_alignment: 64,
            simd_config: SimdConfig {
                enabled: true,
                instruction_sets: vec![
                    "avx2".to_string(),
                    "sha".to_string(),
                    "aes".to_string(),
                    "sse4.2".to_string(),
                ],
                vector_width: 256,
                parallelism: 8,
            },
        }
    }

    /// Linux x86_64 优化配置
    fn linux_x86_64_config() -> Self {
        Self {
            platform_name: "Linux".to_string(),
            cpu_arch: "x86_64".to_string(),
            optimization_level: OptimizationLevel::Advanced,
            yield_frequency: 5000,
            batch_size_multiplier: 1.3,
            thread_count_multiplier: 0.85,
            memory_alignment: 64,
            simd_config: SimdConfig {
                enabled: true,
                instruction_sets: vec![
                    "avx2".to_string(),
                    "avx512".to_string(),
                    "sha".to_string(),
                    "aes".to_string(),
                ],
                vector_width: 512,
                parallelism: 16,
            },
        }
    }

    /// Linux ARM64 优化配置
    fn linux_arm64_config() -> Self {
        Self {
            platform_name: "Linux".to_string(),
            cpu_arch: "aarch64".to_string(),
            optimization_level: OptimizationLevel::Standard,
            yield_frequency: 6000,
            batch_size_multiplier: 1.1,
            thread_count_multiplier: 0.8,
            memory_alignment: 64,
            simd_config: SimdConfig {
                enabled: true,
                instruction_sets: vec![
                    "neon".to_string(),
                    "crypto".to_string(),
                ],
                vector_width: 128,
                parallelism: 4,
            },
        }
    }

    /// Windows x86_64 优化配置
    fn windows_x86_64_config() -> Self {
        Self {
            platform_name: "Windows".to_string(),
            cpu_arch: "x86_64".to_string(),
            optimization_level: OptimizationLevel::Standard,
            yield_frequency: 7000,
            batch_size_multiplier: 1.0,
            thread_count_multiplier: 0.75,
            memory_alignment: 64,
            simd_config: SimdConfig {
                enabled: true,
                instruction_sets: vec![
                    "avx2".to_string(),
                    "sha".to_string(),
                    "aes".to_string(),
                ],
                vector_width: 256,
                parallelism: 8,
            },
        }
    }

    /// 默认优化配置
    fn default_config() -> Self {
        Self {
            platform_name: "Unknown".to_string(),
            cpu_arch: "Unknown".to_string(),
            optimization_level: OptimizationLevel::Basic,
            yield_frequency: 10000,
            batch_size_multiplier: 1.0,
            thread_count_multiplier: 0.5,
            memory_alignment: 32,
            simd_config: SimdConfig {
                enabled: false,
                instruction_sets: vec![],
                vector_width: 128,
                parallelism: 1,
            },
        }
    }

    /// 打印优化信息
    pub fn print_optimization_info(&self) {
        println!("🚀 平台优化配置:");
        println!("   平台: {} ({})", self.platform_name, self.cpu_arch);
        println!("   优化级别: {:?}", self.optimization_level);
        println!("   CPU让出频率: {}", self.yield_frequency);
        println!("   批处理倍数: {:.2}", self.batch_size_multiplier);
        println!("   线程数倍数: {:.2}", self.thread_count_multiplier);
        println!("   内存对齐: {} 字节", self.memory_alignment);

        if self.simd_config.enabled {
            println!("   SIMD优化: 启用");
            println!("   指令集: {:?}", self.simd_config.instruction_sets);
            println!("   向量宽度: {} 位", self.simd_config.vector_width);
            println!("   并行度: {}", self.simd_config.parallelism);
        } else {
            println!("   SIMD优化: 禁用");
        }
    }
}

/// 获取平台特定的CPU让出频率
pub fn get_platform_yield_frequency() -> u64 {
    let config = PlatformOptimization::get_current_platform_config();
    config.yield_frequency
}

/// 获取平台特定的批处理大小倍数
pub fn get_platform_batch_size_multiplier() -> f64 {
    let config = PlatformOptimization::get_current_platform_config();
    config.batch_size_multiplier
}

/// 获取平台特定的线程数倍数
pub fn get_platform_thread_count_multiplier() -> f64 {
    let config = PlatformOptimization::get_current_platform_config();
    config.thread_count_multiplier
}

/// 获取平台特定的内存对齐大小
pub fn get_platform_memory_alignment() -> usize {
    let config = PlatformOptimization::get_current_platform_config();
    config.memory_alignment
}

/// 检查平台是否支持SIMD优化
pub fn is_simd_supported() -> bool {
    let config = PlatformOptimization::get_current_platform_config();
    config.simd_config.enabled
}

/// 获取支持的SIMD指令集
pub fn get_supported_simd_instructions() -> Vec<String> {
    let config = PlatformOptimization::get_current_platform_config();
    config.simd_config.instruction_sets
}
