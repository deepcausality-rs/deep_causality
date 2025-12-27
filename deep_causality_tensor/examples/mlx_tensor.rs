/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! MLX GPU Acceleration: CPU vs GPU Performance Comparison
//!
//! Benchmarks matrix multiplication showing GPU scaling benefits.
//!
//! Run with:
//! ```bash
//! cargo run --example mlx_tensor --features mlx --release
//! ```

use deep_causality_tensor::{CausalTensor, MlxCausalTensor, Tensor};
use std::time::{Duration, Instant};

const CPU_GPU_SIZES: [usize; 4] = [128, 256, 512, 1024];
const GPU_ONLY_SIZES: [usize; 3] = [2048, 4096, 8192];
const NUM_ITERATIONS: usize = 3;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║           MLX GPU Acceleration: CPU vs GPU Benchmark                   ║");
    println!("║                  Matrix Multiplication Scaling Test                    ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝\n");

    println!("Iterations per size: {}\n", NUM_ITERATIONS);

    // ═══════════════════════════════════════════════════════════════════════════
    // Part 1: CPU vs GPU Comparison (128 - 1024)
    // ═══════════════════════════════════════════════════════════════════════════
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║                  Part 1: CPU vs GPU Comparison                         ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝\n");

    let mut cpu_gpu_results: Vec<(usize, Duration, Duration, f64)> = Vec::new();

    for &n in &CPU_GPU_SIZES {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Matrix Size: {}×{} ({} elements)", n, n, n * n);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        let a_data: Vec<f32> = (0..n * n).map(|i| (i as f32) * 0.001).collect();
        let b_data: Vec<f32> = (0..n * n).map(|i| ((n * n - i) as f32) * 0.001).collect();

        // CPU Benchmark
        let cpu_a = CausalTensor::new(a_data.clone(), vec![n, n]).unwrap();
        let cpu_b = CausalTensor::new(b_data.clone(), vec![n, n]).unwrap();

        let start = Instant::now();
        for _ in 0..NUM_ITERATIONS {
            let _ = cpu_a.matmul(&cpu_b);
        }
        let cpu_time = start.elapsed();
        let cpu_avg = cpu_time / NUM_ITERATIONS as u32;

        // GPU Benchmark
        let gpu_a = MlxCausalTensor::new_f32(a_data.clone(), vec![n, n]).unwrap();
        let gpu_b = MlxCausalTensor::new_f32(b_data.clone(), vec![n, n]).unwrap();

        let _ = gpu_a.matmul(&gpu_b).unwrap().to_causal_tensor(); // Warm-up

        let start = Instant::now();
        for _ in 0..NUM_ITERATIONS {
            let result = gpu_a.matmul(&gpu_b).unwrap();
            let _ = result.to_causal_tensor();
        }
        let gpu_time = start.elapsed();
        let gpu_avg = gpu_time / NUM_ITERATIONS as u32;

        let speedup = cpu_avg.as_nanos() as f64 / gpu_avg.as_nanos() as f64;

        println!("  CPU matmul: {:>10.3} ms", cpu_avg.as_secs_f64() * 1000.0);
        println!("  GPU matmul: {:>10.3} ms", gpu_avg.as_secs_f64() * 1000.0);
        println!("  Speedup:    {:>10.2}x\n", speedup);

        cpu_gpu_results.push((n, cpu_avg, gpu_avg, speedup));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Part 2: GPU-Only (Large Matrices)
    // ═══════════════════════════════════════════════════════════════════════════
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║          Part 2: GPU-Only (sizes impractical for CPU)                  ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝\n");

    let mut gpu_only_results: Vec<(usize, Duration)> = Vec::new();

    for &n in &GPU_ONLY_SIZES {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!(
            "  Matrix Size: {}×{} ({} elements, {:.2} GB)",
            n,
            n,
            n * n,
            (n * n * 4) as f64 / 1e9
        );
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        let a_data: Vec<f32> = (0..n * n).map(|i| (i as f32) * 0.001).collect();
        let b_data: Vec<f32> = (0..n * n).map(|i| ((n * n - i) as f32) * 0.001).collect();

        let gpu_a = MlxCausalTensor::new_f32(a_data, vec![n, n]).unwrap();
        let gpu_b = MlxCausalTensor::new_f32(b_data, vec![n, n]).unwrap();

        let _ = gpu_a.matmul(&gpu_b).unwrap().to_causal_tensor(); // Warm-up

        let start = Instant::now();
        for _ in 0..NUM_ITERATIONS {
            let result = gpu_a.matmul(&gpu_b).unwrap();
            let _ = result.to_causal_tensor();
        }
        let gpu_time = start.elapsed();
        let gpu_avg = gpu_time / NUM_ITERATIONS as u32;

        println!(
            "  GPU matmul: {:>10.3} ms\n",
            gpu_avg.as_secs_f64() * 1000.0
        );

        gpu_only_results.push((n, gpu_avg));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Summary Tables
    // ═══════════════════════════════════════════════════════════════════════════
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║                     Summary: CPU vs GPU                                ║");
    println!("╠════════════════════════════════════════════════════════════════════════╣");
    println!("║     Size     │  CPU (ms)  │  GPU (ms)  │  Speedup                      ║");
    println!("╟──────────────┼────────────┼────────────┼───────────────────────────────╢");

    for (n, cpu_avg, gpu_avg, speedup) in &cpu_gpu_results {
        println!(
            "║ {:>5}×{:<5}  │ {:>10.3} │ {:>10.3} │ {:>8.2}x                      ║",
            n,
            n,
            cpu_avg.as_secs_f64() * 1000.0,
            gpu_avg.as_secs_f64() * 1000.0,
            speedup
        );
    }
    println!("╚════════════════════════════════════════════════════════════════════════╝\n");

    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║                   Summary: GPU-Only (Large Matrices)                   ║");
    println!("╠════════════════════════════════════════════════════════════════════════╣");
    println!("║     Size     │  Elements  │  Memory    │  GPU (ms)                     ║");
    println!("╟──────────────┼────────────┼────────────┼───────────────────────────────╢");

    for (n, gpu_avg) in &gpu_only_results {
        println!(
            "║ {:>5}×{:<5}  │ {:>10} │ {:>7.2} GB │ {:>10.3}                     ║",
            n,
            n,
            n * n,
            (n * n * 4) as f64 / 1e9,
            gpu_avg.as_secs_f64() * 1000.0
        );
    }
    println!("╚════════════════════════════════════════════════════════════════════════╝\n");

    // Notes
    println!("Notes:");
    println!("  • GPU acceleration scales dramatically with matrix size");
    println!("  • Matrices >1024×1024 are impractical on CPU (>1 second per matmul)");
    println!("  • GPU handles 8192×8192 (268M elements) in seconds");
    println!("  • All GPU operations run in f32 (Metal limitation)");
}
