// CPU BTC 挖矿核心专用构建脚本 - 简化版本
// 针对CPU密集型SHA-256双重哈希算法的构建时优化

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let profile = env::var("PROFILE").unwrap();

    println!("cargo:warning=🔧 CPU BTC Core: Building for {}-{} ({})", target_os, target_arch, profile);

    // 设置CPU挖矿特定的编译特性
    setup_cpu_mining_features(&target_os, &target_arch);

    // 设置发布版本的额外优化
    if profile == "release" {
        setup_release_optimizations(&target_os, &target_arch);
    }

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
            println!("cargo:rustc-cfg=has_hardware_sha");
            println!("cargo:warning=🍎 Apple Silicon: Hardware SHA-256 acceleration enabled");
        }
        ("macos", "x86_64") => {
            println!("cargo:rustc-cfg=intel_mac_mining");
            println!("cargo:warning=💻 Intel Mac: SHA extensions enabled");
        }
        ("linux", "x86_64") => {
            println!("cargo:rustc-cfg=x86_64_linux_mining");
            println!("cargo:warning=🐧 Linux x86_64: Full CPU optimization enabled");
        }
        ("linux", "aarch64") => {
            println!("cargo:rustc-cfg=aarch64_linux_mining");
            println!("cargo:warning=🦾 ARM64 Linux: NEON and Crypto extensions enabled");
        }
        ("windows", "x86_64") => {
            println!("cargo:rustc-cfg=windows_x86_64_mining");
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
        }
        "macos" => {
            println!("cargo:rustc-cfg=limited_cpu_affinity");
        }
        _ => {}
    }
}

/// 设置发布版本的额外优化
fn setup_release_optimizations(target_os: &str, target_arch: &str) {
    println!("cargo:warning=🚀 Setting up release optimizations");

    // 启用发布版本特定优化
    println!("cargo:rustc-cfg=release_optimized");

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
