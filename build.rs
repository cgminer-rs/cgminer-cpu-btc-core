// CPU BTC 挖矿核心专用构建脚本
// 针对CPU密集型SHA-256双重哈希算法的构建时优化

use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let profile = env::var("PROFILE").unwrap();

    println!("cargo:warning=🔧 CPU BTC Core: Building for {}-{} ({})", target_os, target_arch, profile);

    // 设置CPU挖矿特定的编译特性
    setup_cpu_mining_features(&target_os, &target_arch);

    // 检测CPU特性并启用相应优化
    detect_and_enable_cpu_features(&target_os, &target_arch);

    // 设置发布版本的额外优化
    if profile == "release" {
        setup_release_optimizations(&target_os, &target_arch);
    }

    // 检查依赖库
    check_cpu_mining_dependencies(&target_os);

    println!("cargo:warning=✅ CPU BTC Core: Build configuration completed");
}

/// 设置CPU挖矿特定的编译特性
fn setup_cpu_mining_features(target_os: &str, target_arch: &str) {
    println!("cargo:warning=⚙️  Setting up CPU mining features");

    // 启用CPU挖矿特定特性
    println!("cargo:rustc-cfg=cpu_mining");
    println!("cargo:rustc-cfg=sha256_optimized");
    println!("cargo:rustc-cfg=double_sha256_optimized");

    // 根据平台启用特定特性
    match (target_os, target_arch) {
        ("macos", "aarch64") => {
            println!("cargo:rustc-cfg=apple_silicon_mining");
            println!("cargo:rustc-cfg=neon_sha256");
            println!("cargo:rustc-cfg=crypto_ext_sha256");
            println!("cargo:rustc-cfg=has_hardware_sha");
            println!("cargo:warning=🍎 Apple Silicon: Hardware SHA-256 acceleration enabled");
        }
        ("macos", "x86_64") => {
            println!("cargo:rustc-cfg=intel_mac_mining");
            println!("cargo:rustc-cfg=aes_ni_mining");
            println!("cargo:rustc-cfg=sha_ext_mining");
            println!("cargo:warning=💻 Intel Mac: SHA extensions enabled");
        }
        ("linux", "x86_64") => {
            println!("cargo:rustc-cfg=x86_64_linux_mining");
            println!("cargo:rustc-cfg=aes_ni_mining");
            println!("cargo:rustc-cfg=sha_ext_mining");
            println!("cargo:rustc-cfg=avx2_mining");
            println!("cargo:warning=🐧 Linux x86_64: Full CPU optimization enabled");
        }
        ("linux", "aarch64") => {
            println!("cargo:rustc-cfg=aarch64_linux_mining");
            println!("cargo:rustc-cfg=neon_mining");
            println!("cargo:rustc-cfg=crypto_ext_mining");
            println!("cargo:warning=🦾 ARM64 Linux: NEON and Crypto extensions enabled");
        }
        ("windows", "x86_64") => {
            println!("cargo:rustc-cfg=windows_x86_64_mining");
            println!("cargo:rustc-cfg=aes_ni_mining");
            println!("cargo:rustc-cfg=sha_ext_mining");
            println!("cargo:warning=🪟 Windows: CPU mining optimizations enabled");
        }
        _ => {
            println!("cargo:warning=❓ Unknown platform: Using generic optimizations");
        }
    }

    // 启用CPU绑定特性（如果平台支持）
    match target_os {
        "linux" | "windows" => {
            println!("cargo:rustc-cfg=has_cpu_affinity");
            println!("cargo:rustc-cfg=cpu_binding_supported");
        }
        "macos" => {
            println!("cargo:rustc-cfg=limited_cpu_affinity");
        }
        _ => {}
    }
}

/// 检测CPU特性并启用相应优化
fn detect_and_enable_cpu_features(_target_os: &str, target_arch: &str) {
    println!("cargo:warning=🔍 CPU feature detection is handled by dependencies at runtime (e.g., sha2 crate).");

    match target_arch {
        "x86_64" => {
            println!("cargo:rustc-cfg=has_sse2"); // SSE2 is a baseline feature for x86_64
        }
        "aarch64" => {
            println!("cargo:rustc-cfg=has_neon"); // NEON is a baseline feature for aarch64
        }
        _ => {}
    }
}

/// 设置发布版本的额外优化
fn setup_release_optimizations(target_os: &str, target_arch: &str) {
    println!("cargo:warning=🚀 Setting up release optimizations");

    // 启用发布版本特定优化
    println!("cargo:rustc-cfg=release_optimized");
    println!("cargo:rustc-cfg=fast_math");
    println!("cargo:rustc-cfg=aggressive_inlining");

    // 平台特定的发布优化
    match (target_os, target_arch) {
        ("macos", "aarch64") => {
            println!("cargo:rustc-cfg=apple_silicon_release");
            println!("cargo:warning=🍎 Apple Silicon release optimizations enabled");
        }
        ("linux", "x86_64") => {
            println!("cargo:rustc-cfg=linux_x86_64_release");
            println!("cargo:warning=🐧 Linux x86_64 release optimizations enabled");
        }
        _ => {}
    }
}

/// 检查CPU挖矿相关依赖
fn check_cpu_mining_dependencies(target_os: &str) {
    println!("cargo:warning=📦 Checking CPU mining dependencies");

    match target_os {
        "linux" => {
            // 检查Linux特定依赖
            check_library_exists("pthread");
            check_library_exists("m"); // 数学库

            // 检查CPU绑定相关头文件
            if check_header_exists("sched.h") {
                println!("cargo:rustc-cfg=has_sched_h");
            }

            if check_header_exists("sys/sysinfo.h") {
                println!("cargo:rustc-cfg=has_sysinfo");
            }
        }
        "macos" => {
            // 检查macOS框架
            check_framework_exists("CoreFoundation");
            check_framework_exists("IOKit");
        }
        "windows" => {
            // Windows特定检查
            println!("cargo:rustc-cfg=has_windows_api");
        }
        _ => {}
    }
}

/// 检查CPU特性是否可用（简化版本）
fn is_feature_available(_feature: &str) -> bool {
    // This function is incorrect as it cannot know the target CPU's features at compile time.
    // True detection should happen at runtime or be passed via RUSTFLAGS.
    // Dependencies like `sha2` with the 'asm' feature handle runtime detection themselves.
    // Returning false to avoid incorrect cfg flags.
    false
}

/// 检查库是否存在
fn check_library_exists(lib_name: &str) -> bool {
    let output = Command::new("pkg-config")
        .args(["--exists", lib_name])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("cargo:rustc-cfg=has_lib_{}", lib_name.replace("-", "_"));
            true
        }
        _ => {
            println!("cargo:warning=Library {} not found", lib_name);
            false
        }
    }
}

/// 检查头文件是否存在
fn check_header_exists(header_path: &str) -> bool {
    let paths = [
        format!("/usr/include/{}", header_path),
        format!("/usr/local/include/{}", header_path),
        format!("/opt/homebrew/include/{}", header_path),
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            let header_name = header_path.replace("/", "_").replace(".", "_");
            println!("cargo:rustc-cfg=has_header_{}", header_name);
            return true;
        }
    }

    println!("cargo:warning=Header {} not found", header_path);
    false
}

/// 检查框架是否存在 (macOS)
fn check_framework_exists(framework_name: &str) -> bool {
    let framework_path = format!("/System/Library/Frameworks/{}.framework", framework_name);

    if std::path::Path::new(&framework_path).exists() {
        println!("cargo:rustc-cfg=has_framework_{}", framework_name.to_lowercase());
        true
    } else {
        println!("cargo:warning=Framework {} not found", framework_name);
        false
    }
}
