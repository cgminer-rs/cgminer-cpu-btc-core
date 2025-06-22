//! CPU BTC 核心基准测试套件
//!
//! 专门测试 Bitcoin CPU 挖矿核心算法的性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use sha2::{Sha256, Digest};
use std::time::Duration;

/// 创建测试用的区块头数据 (80字节)
fn create_test_block_header() -> [u8; 80] {
    let mut header = [0u8; 80];

    // 版本 (4字节)
    header[0..4].copy_from_slice(&1u32.to_le_bytes());

    // 前一个区块哈希 (32字节) - 使用测试数据
    header[4..36].copy_from_slice(&[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68,
        0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e, 0x93,
        0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1,
        0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c, 0xe2, 0x6f
    ]);

    // Merkle根 (32字节) - 使用测试数据
    header[36..68].copy_from_slice(&[
        0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2,
        0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f, 0x61,
        0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32,
        0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e, 0x5e, 0x4a
    ]);

    // 时间戳 (4字节)
    header[68..72].copy_from_slice(&1231006505u32.to_le_bytes());

    // 难度目标 (4字节)
    header[72..76].copy_from_slice(&0x1d00ffffu32.to_le_bytes());

    // Nonce (4字节) - 将在测试中修改
    header[76..80].copy_from_slice(&0u32.to_le_bytes());

    header
}



/// SHA256 双重哈希基准测试
fn bench_double_sha256(c: &mut Criterion) {
    let mut group = c.benchmark_group("sha256_double_hash");

    let test_data = create_test_block_header();

    group.bench_function("single_hash", |b| {
        b.iter(|| {
            let data = black_box(&test_data);

            // 第一次 SHA256
            let mut hasher = Sha256::new();
            hasher.update(data);
            let hash1 = hasher.finalize();

            // 第二次 SHA256
            let mut hasher = Sha256::new();
            hasher.update(&hash1);
            let hash2 = hasher.finalize();

            black_box(hash2)
        })
    });

    // 批量哈希测试
    for batch_size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_hash", batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    let mut results = Vec::with_capacity(batch_size);
                    for i in 0..batch_size {
                        let mut data = test_data;
                        // 修改 nonce
                        data[76..80].copy_from_slice(&(i as u32).to_le_bytes());

                        // 双重哈希
                        let mut hasher = Sha256::new();
                        hasher.update(&data);
                        let hash1 = hasher.finalize();

                        let mut hasher = Sha256::new();
                        hasher.update(&hash1);
                        let hash2 = hasher.finalize();

                        results.push(hash2);
                    }
                    black_box(results)
                })
            },
        );
    }

    group.finish();
}





/// 内存使用效率基准测试
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    let header = create_test_block_header();

    group.bench_function("preallocated_vectors", |b| {
        b.iter(|| {
            let batch_size = 1000;
            let mut hashes = Vec::with_capacity(batch_size); // 预分配

            for i in 0..batch_size {
                let mut work_header = header;
                work_header[76..80].copy_from_slice(&(i as u32).to_le_bytes());

                let mut hasher = Sha256::new();
                hasher.update(&work_header);
                let hash1 = hasher.finalize();

                let mut hasher = Sha256::new();
                hasher.update(&hash1);
                let hash2 = hasher.finalize();

                hashes.push(hash2.to_vec());
            }
            black_box(hashes)
        })
    });

    group.bench_function("dynamic_vectors", |b| {
        b.iter(|| {
            let batch_size = 1000;
            let mut hashes = Vec::new(); // 动态分配

            for i in 0..batch_size {
                let mut work_header = header;
                work_header[76..80].copy_from_slice(&(i as u32).to_le_bytes());

                let mut hasher = Sha256::new();
                hasher.update(&work_header);
                let hash1 = hasher.finalize();

                let mut hasher = Sha256::new();
                hasher.update(&hash1);
                let hash2 = hasher.finalize();

                hashes.push(hash2.to_vec());
            }
            black_box(hashes)
        })
    });

    group.finish();
}



// 定义基准测试组
criterion_group!(
    benches,
    bench_double_sha256,
    bench_memory_efficiency
);

// 主函数
criterion_main!(benches);
