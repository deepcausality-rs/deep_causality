/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Integration tests for the "learn-once, rank-many" CPDAG cache in the BRCD
//! sub-pipeline. They cover the three CPDAG resolution paths (supplied, cached,
//! none) plus cache correctness: a poisoned-cache hit proving the cache is read,
//! and a stale-key miss proving re-learn + overwrite.

use deep_causality_discovery::{
    BrcdConfig, CdlBuilder, CdlConfigBuilder, MixedGraph, load_cpdag_csv, save_cpdag_csv,
};
use deep_causality_tensor::CausalTensor;
use std::io::Write;
use tempfile::NamedTempFile;

/// Deterministic x -> y -> z chain. `intercept` shifts y's conditional mean so the
/// anomalous regime injects a root cause at y (variable index 1).
fn write_chain(intercept: f64, seed: u64) -> NamedTempFile {
    let mut state = seed | 1;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        ((state >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    };
    let mut csv = String::from("x,y,z\n");
    for _ in 0..120 {
        let x = next();
        let y = intercept + 1.5 * x + next();
        let z = 2.0 * y + next();
        csv.push_str(&format!("{:.6},{:.6},{:.6}\n", x, y, z));
    }
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(csv.as_bytes()).unwrap();
    f
}

/// The undirected x -- y -- z chain CPDAG, matching the structure of `write_chain`.
fn write_supplied_cpdag() -> NamedTempFile {
    let s = "# vertices=3\nsrc,dst,mark_src,mark_dst\n0,1,Tail,Tail\n1,2,Tail,Tail\n";
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(s.as_bytes()).unwrap();
    f
}

/// A unique, not-yet-existing cache CSV path under the temp dir.
fn fresh_cache_path(tag: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut p = std::env::temp_dir();
    p.push(format!("dcl_cpdag_cache_{tag}_{nanos}.csv"));
    p.to_str().unwrap().to_string()
}

fn sidecar(cache_path: &str) -> String {
    format!("{cache_path}.key")
}

/// Runs the BRCD pipeline end-to-end, returning the top-ranked candidate set.
fn run_top(config: &deep_causality_discovery::BrcdLoaderConfig<f64>) -> Vec<usize> {
    let effect = CdlBuilder::build_brcd(config)
        .brcd_load_input()
        .brcd_discover();
    let cdl = effect.inner.expect("pipeline succeeds");
    cdl.state
        .brcd_result
        .top()
        .expect("non-empty result")
        .to_vec()
}

// ---------------------------------------------------------------------------
// Path 1: supplied CPDAG path still works (regression).
// ---------------------------------------------------------------------------
#[test]
fn test_supplied_cpdag_path_unchanged() {
    let normal = write_chain(0.0, 0x1234_5678);
    let anomalous = write_chain(4.0, 0x9abc_def0);
    let cpdag = write_supplied_cpdag();

    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .with_cpdag_path(cpdag.path().to_str().unwrap())
        .build()
        .expect("files exist");

    // Supplied path wins; no cache files involved. Root cause is y (index 1).
    assert_eq!(run_top(&config), vec![1usize]);
    assert!(config.cpdag_cache_path().is_none());
}

// ---------------------------------------------------------------------------
// Path 2: cache MISS — learns, writes CSV + sidecar, and warm == cold.
// ---------------------------------------------------------------------------
#[test]
fn test_cache_miss_learns_persists_and_warm_equals_cold() {
    let normal = write_chain(0.0, 0x1111_2222);
    let anomalous = write_chain(4.0, 0x3333_4444);
    let cache_path = fresh_cache_path("miss");

    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .with_cpdag_cache_path(&cache_path)
        .build()
        .expect("files exist");

    // Files must not exist before the first run.
    assert!(!std::path::Path::new(&cache_path).exists());
    assert!(!std::path::Path::new(&sidecar(&cache_path)).exists());

    let cached_top = run_top(&config);

    // Cache miss persisted both the CSV and the key sidecar.
    assert!(std::path::Path::new(&cache_path).exists());
    assert!(std::path::Path::new(&sidecar(&cache_path)).exists());

    // Cold baseline: the same pipeline with NO cache (brcd_run learns internally
    // via BossConfig::with_seed(config.seed)). Warm (cached learn) must equal cold.
    let cold_config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .build()
        .expect("files exist");
    let cold_top = run_top(&cold_config);

    assert_eq!(
        cached_top, cold_top,
        "cached learn must equal brcd_run(None) (warm == cold)"
    );

    let _ = std::fs::remove_file(&cache_path);
    let _ = std::fs::remove_file(sidecar(&cache_path));
}

// ---------------------------------------------------------------------------
// Path 3: cache HIT — proven by poisoning the CSV while keeping the key valid.
// ---------------------------------------------------------------------------
#[test]
fn test_cache_hit_trusts_poisoned_graph() {
    let normal = write_chain(0.0, 0x5555_6666);
    let anomalous = write_chain(4.0, 0x7777_8888);
    let cache_path = fresh_cache_path("hit");

    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .with_cpdag_cache_path(&cache_path)
        .build()
        .expect("files exist");

    // Run 1: populates the cache (CSV + key) with the genuinely learned graph.
    let honest_top = run_top(&config);

    // Poison the cache CSV with a DIFFERENT but dimension-valid (3-vertex) graph,
    // leaving the .key sidecar intact so the recomputed key still matches. The
    // single arc x -> z (0 -> 2) leaves y (index 1) isolated, structurally unlike
    // the learned x -> y -> z chain. It deliberately moves the top-ranked root
    // cause OFF y, so a loader that re-learned would NOT produce this ranking.
    let mut poison: MixedGraph<()> = {
        let data = CausalTensor::new(vec![(); 3], vec![3]).unwrap();
        MixedGraph::new(3, data, 0).unwrap()
    };
    poison.add_arc(0, 2).unwrap();
    save_cpdag_csv(&poison, &cache_path).unwrap();

    // Independently compute what BRCD yields on the poisoned graph by feeding it
    // through the SUPPLIED path; this is the unambiguous "the cache was used"
    // oracle.
    let poison_supplied = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    save_cpdag_csv(&poison, poison_supplied.path().to_str().unwrap()).unwrap();
    let poison_oracle_config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .with_cpdag_path(poison_supplied.path().to_str().unwrap())
        .build()
        .expect("files exist");
    let poison_oracle_top = run_top(&poison_oracle_config);

    // Run 2: same data/seed/cache path. A valid key + present CSV is a hit, so the
    // loader must read the POISONED graph, not re-learn.
    let poisoned_run_top = run_top(&config);

    assert_eq!(
        poisoned_run_top, poison_oracle_top,
        "cache hit must rank using the poisoned graph (cache was read, not re-learned)"
    );
    // The honest learned result and the poisoned result differ, confirming the
    // poison actually changed the ranking (otherwise the test would be vacuous).
    assert_ne!(
        poisoned_run_top, honest_top,
        "poisoned graph must yield a different ranking than the honest learned graph"
    );

    let _ = std::fs::remove_file(&cache_path);
    let _ = std::fs::remove_file(sidecar(&cache_path));
}

// ---------------------------------------------------------------------------
// Path 4: cache STALE — a changed seed invalidates the key; re-learn + overwrite.
// ---------------------------------------------------------------------------
#[test]
fn test_stale_cache_relearns_and_overwrites() {
    let normal = write_chain(0.0, 0xaaaa_bbbb);
    let anomalous = write_chain(4.0, 0xcccc_dddd);
    let cache_path = fresh_cache_path("stale");

    // Run 1 with seed 7 populates the cache with the learned graph + key.
    let config_seed7 = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .with_cpdag_cache_path(&cache_path)
        .build()
        .expect("files exist");
    let _ = run_top(&config_seed7);

    let key_after_run1 = std::fs::read_to_string(sidecar(&cache_path)).unwrap();

    // Poison the CSV with an unrelated graph but DO NOT touch the key. If the
    // loader ignored the key on the next (stale) run it would return this graph.
    let mut poison: MixedGraph<()> = {
        let data = CausalTensor::new(vec![(); 3], vec![3]).unwrap();
        MixedGraph::new(3, data, 0).unwrap()
    };
    poison.add_arc(2, 0).unwrap();
    save_cpdag_csv(&poison, &cache_path).unwrap();

    // Run 2 with a DIFFERENT seed (9). The recomputed key (seed 9) differs from
    // the stored sidecar (seed 7), so the entry is stale: the loader must
    // re-learn and OVERWRITE both the CSV and the sidecar.
    let config_seed9 = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(9))
        .with_cpdag_cache_path(&cache_path)
        .build()
        .expect("files exist");
    let stale_run_top = run_top(&config_seed9);

    // The sidecar key changed (overwritten with the seed-9 key).
    let key_after_run2 = std::fs::read_to_string(sidecar(&cache_path)).unwrap();
    assert_ne!(
        key_after_run1, key_after_run2,
        "stale entry must overwrite the sidecar with the new key"
    );

    // The CSV no longer holds the poisoned graph: it was overwritten by the freshly
    // learned graph, which must equal the cold baseline for seed 9.
    let overwritten = load_cpdag_csv(&cache_path).unwrap();
    assert_ne!(
        overwritten.num_edges(),
        poison.num_edges(),
        "stale CSV must be overwritten, not reused"
    );

    let cold_seed9_config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(9))
        .build()
        .expect("files exist");
    let cold_seed9_top = run_top(&cold_seed9_config);

    assert_eq!(
        stale_run_top, cold_seed9_top,
        "stale run must re-learn (equal cold baseline), not use the poisoned graph"
    );

    let _ = std::fs::remove_file(&cache_path);
    let _ = std::fs::remove_file(sidecar(&cache_path));
}
