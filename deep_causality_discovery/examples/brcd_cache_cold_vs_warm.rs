/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Paper-grade evaluation harness: cold (learn) vs warm (cached) CPDAG resolution.
//!
//! Evidence for claim (2) — *the CPDAG cache makes a warm (cached) run much cheaper
//! than a cold (learn) run*. A synthetic linear-Gaussian dataset large enough that
//! BOSS structure learning is measurably expensive (~30 variables, ~800 normal +
//! ~800 anomalous rows, a planted shifted root cause) is written to a unique temp
//! dir, then the CDL pipeline is driven end to end via the builder:
//!
//!   CdlConfigBuilder::build_brcd_config()
//!     .with_normal_path(..).with_anomalous_path(..)
//!     .with_cpdag_cache_path(cache)               // NO supplied cpdag_path
//!     .with_brcd_config(BrcdConfig::continuous(seed)).build()
//!   -> CdlBuilder::build_brcd(&cfg).brcd_load_input().brcd_discover()
//!
//! The cache resolves inside `brcd_load_input`: a cold rep starts from an empty
//! cache (BOSS learns + persists + ranks); a warm rep reuses the populated cache
//! (load + rank). We report cold ms, warm ms, the delta (= structure-learning cost
//! avoided) and the speedup, assert warm < cold, and assert the warm ranking equals
//! the cold ranking — the cache must be correct, not merely fast.
//!
//! Run (release):
//!   cargo run --release -p deep_causality_discovery --example brcd_cache_cold_vs_warm

use deep_causality_discovery::{BrcdConfig, BrcdLoaderConfig, CdlBuilder, CdlConfigBuilder};
use deep_causality_rand::{Distribution, Normal, Rng, Xoshiro256};
use std::fs;
use std::io::Write;
use std::time::Instant;

const N_VARS: usize = 30;
const N_ROWS: usize = 800;
const SEED: u64 = 0xB0C5_CACE;
const REPS: usize = 3;

fn main() {
    println!();
    println!("BRCD evaluation harness — cold (learn) vs warm (cached) CPDAG resolution");
    println!(
        "Dataset: {N_VARS} vars, {N_ROWS} normal + {N_ROWS} anomalous rows, planted root cause."
    );
    println!("Median of {REPS} reps; fresh cache per cold rep. Times are wall-clock (indicative).");
    println!();

    // Unique temp dir derived from a fixed seed so reruns are isolated yet reproducible.
    let mut dir = std::env::temp_dir();
    dir.push(format!("dcl_brcd_cache_{SEED:016x}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let normal_path = dir.join("normal.csv").to_str().unwrap().to_string();
    let anomalous_path = dir.join("anomalous.csv").to_str().unwrap().to_string();
    let cache_path = dir.join("cpdag_cache.csv").to_str().unwrap().to_string();
    let cache_key = format!("{cache_path}.key");

    let rc = N_VARS - 5; // a deep node so the anomaly propagates a detectable signal
    write_dataset(&normal_path, 0.0, rc, SEED ^ 0x1111);
    write_dataset(&anomalous_path, 5.0, rc, SEED ^ 0x2222);
    println!("Wrote dataset to {}", dir.display());
    println!("Planted root cause: v{rc}");
    println!();

    // --- Cold reps: empty cache each time (BOSS learns + persists + ranks). ----
    let mut cold_ms: Vec<f64> = Vec::new();
    let mut cold_ranks: Option<Vec<Vec<usize>>> = None;
    for rep in 0..REPS {
        // Ensure a true cold start: remove any cache + sidecar.
        let _ = fs::remove_file(&cache_path);
        let _ = fs::remove_file(&cache_key);
        let cfg = make_config(&normal_path, &anomalous_path, &cache_path);
        let (ms, ranks) = run_once(&cfg);
        println!("  cold rep {rep}: {ms:.1} ms");
        cold_ms.push(ms);
        cold_ranks = Some(ranks);
    }

    // After the cold reps the cache is populated. Confirm it exists.
    assert!(
        fs::metadata(&cache_path).is_ok(),
        "cache CSV should exist after a cold run"
    );

    // --- Warm reps: populated cache (load + rank, no BOSS). --------------------
    let mut warm_ms: Vec<f64> = Vec::new();
    let mut warm_ranks: Option<Vec<Vec<usize>>> = None;
    for rep in 0..REPS {
        let cfg = make_config(&normal_path, &anomalous_path, &cache_path);
        let (ms, ranks) = run_once(&cfg);
        println!("  warm rep {rep}: {ms:.1} ms");
        warm_ms.push(ms);
        warm_ranks = Some(ranks);
    }

    let cold = median_ms(&mut cold_ms);
    let warm = median_ms(&mut warm_ms);
    let delta = cold - warm;
    let speedup = cold / warm;

    println!();
    println!("============================================================================");
    println!("COLD vs WARM (median of {REPS} reps)");
    println!("============================================================================");
    println!("  cold (BOSS learns + persists + rank): {cold:>10.1} ms");
    println!("  warm (cache load + rank):             {warm:>10.1} ms");
    println!("  delta (structure-learning avoided):   {delta:>10.1} ms");
    println!("  speedup:                              {speedup:>10.2} x");
    println!("============================================================================");

    let cold_ranks = cold_ranks.expect("cold ranks");
    let warm_ranks = warm_ranks.expect("warm ranks");

    // Correctness: the cache must reproduce the cold ranking exactly.
    assert_eq!(
        cold_ranks, warm_ranks,
        "warm (cached) ranking must equal cold (learned) ranking — cache must be correct"
    );
    // Speed: the warm path must be cheaper than the cold path.
    assert!(
        warm < cold,
        "warm run ({warm:.1} ms) must be cheaper than cold run ({cold:.1} ms)"
    );

    let top_cold = cold_ranks.first().map(|v| v.as_slice());
    println!();
    println!("Cache correctness: warm ranking == cold ranking (asserted).");
    println!("Top-ranked candidate set (both): {top_cold:?}");
    println!(
        "Reading: the warm run avoids BOSS structure learning entirely, so it is {speedup:.1}x\n\
         cheaper than the cold run while producing the identical ranking."
    );
}

/// Writes a linear-Gaussian dataset (random DAG over `N_VARS`, parents from lower
/// indices) to `path` as a headed CSV. In the anomalous regime the root cause
/// node's intercept is shifted by `perturb`.
fn write_dataset(path: &str, perturb: f64, rc: usize, seed: u64) {
    let mut rng = Xoshiro256::from_seed(seed);
    let eps = Normal::new(0.0_f64, 1.0).unwrap();

    // Fixed structure: derive weights from a structure seed independent of the
    // regime so normal and anomalous share the same mechanism (only rc's intercept
    // differs in the anomalous regime).
    let mut srng = Xoshiro256::from_seed(0xDA15_5EED);
    let mut parents: Vec<Vec<usize>> = vec![Vec::new(); N_VARS];
    let mut weight: Vec<Vec<f64>> = vec![Vec::new(); N_VARS];
    for i in 0..N_VARS {
        for j in 0..i {
            if srng.random_range(0.0..1.0) < 0.25 {
                parents[i].push(j);
                let sign = if srng.random_range(0.0..1.0) < 0.5 {
                    -1.0
                } else {
                    1.0
                };
                weight[i].push(sign * (0.6 + srng.random_range(0.0..1.0)));
            }
        }
    }

    let mut header = String::new();
    for i in 0..N_VARS {
        if i > 0 {
            header.push(',');
        }
        header.push_str(&format!("v{i}"));
    }
    let mut out = String::with_capacity(N_ROWS * N_VARS * 8);
    out.push_str(&header);
    out.push('\n');

    for _ in 0..N_ROWS {
        let mut x = vec![0.0_f64; N_VARS];
        for i in 0..N_VARS {
            let mut mean = if i == rc { perturb } else { 0.0 };
            for (k, &j) in parents[i].iter().enumerate() {
                mean += weight[i][k] * x[j];
            }
            x[i] = mean + eps.sample(&mut rng);
        }
        for (i, v) in x.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!("{v:.6}"));
        }
        out.push('\n');
    }

    let mut f = fs::File::create(path).expect("create dataset csv");
    f.write_all(out.as_bytes()).expect("write dataset csv");
}

/// Builds a fresh loader config keyed to the given cache path. NO supplied CPDAG,
/// so a cache miss forces BOSS to learn.
fn make_config(normal: &str, anomalous: &str, cache: &str) -> BrcdLoaderConfig<f64> {
    CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal)
        .with_anomalous_path(anomalous)
        .with_brcd_config(BrcdConfig::<f64>::continuous(SEED))
        .with_cpdag_cache_path(cache)
        .build()
        .expect("dataset files exist")
}

/// Drives the BRCD pipeline through `brcd_discover`, returning `(elapsed_ms,
/// ranks)`. The cache resolves inside `brcd_load_input`.
fn run_once(config: &BrcdLoaderConfig<f64>) -> (f64, Vec<Vec<usize>>) {
    let t = Instant::now();
    let effect = CdlBuilder::build_brcd(config)
        .brcd_load_input()
        .brcd_discover();
    let ms = t.elapsed().as_secs_f64() * 1e3;
    let cdl = effect.inner.expect("pipeline succeeds");
    (ms, cdl.state.brcd_result.ranks().to_vec())
}

/// Median of a millisecond slice.
fn median_ms(samples: &mut [f64]) -> f64 {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = samples.len() / 2;
    if samples.len() % 2 == 1 {
        samples[m]
    } else {
        0.5 * (samples[m - 1] + samples[m])
    }
}
