/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! BOSS structure-learning verification.
//!
//! Two parts:
//!
//! 1. **Structural (self-contained).** BOSS learns from deterministically
//!    generated linear-Gaussian data with a fixed seed; the learned CPDAG's
//!    skeleton and v-structures must match the known generating structure
//!    (a chain `X — Y — Z`, and a collider `X → Z ← Y`).
//! 2. **End-to-end (real data).** On a committed Online Boutique case, BRCD runs
//!    with **no supplied CPDAG** (`cpdag = None`), so BOSS learns the structure
//!    from the pre-failure data and BRCD ranks against it. This exercises the
//!    full learn → rank pipeline on real ~50-variable data — the path that
//!    serves systems without a service map (Petshop in the paper; the Petshop
//!    dataset is not committed here, so Online Boutique stands in).
//!
//!    Reproduction here is **structural + downstream-ranking, not byte-exact**:
//!    BOSS is a heuristic search, and the paper reports BRCD underperforms with a
//!    *learned* (vs supplied) CPDAG on Online Boutique under limited samples. So
//!    this part asserts the pipeline produces a valid ranking and reports where
//!    the known fault lands, rather than demanding exact recovery.
//!
//! Run: `cargo run -p deep_causality_algorithms --example verification_boss`

mod common;

use common::{Report, load_csv, load_expected, tensor};
use deep_causality_algorithms::brcd::{BossConfig, BrcdConfig, boss_learn, brcd_run};
use deep_causality_rand::{Rng, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::EdgeKind;
use std::f64::consts::TAU;
use std::path::PathBuf;

/// One standard-normal draw via Box-Muller.
fn next_normal(rng: &mut Xoshiro256) -> f64 {
    let u1: f64 = (rng.random::<f64>()).max(1e-12);
    let u2: f64 = rng.random::<f64>();
    (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos()
}

/// `n × 3` linear-Gaussian chain X(0) → Y(1) → Z(2), fixed seed.
fn chain(n: usize, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let mut flat = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = next_normal(&mut rng);
        let y = x + next_normal(&mut rng);
        let z = y + next_normal(&mut rng);
        flat.extend([x, y, z]);
    }
    tensor(flat, n, 3)
}

/// `n × 3` collider X(0) → Z(2) ← Y(1): X, Y independent, Z = X + Y + 3·noise.
fn collider(n: usize, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let mut flat = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = next_normal(&mut rng);
        let y = next_normal(&mut rng);
        let z = x + y + 3.0 * next_normal(&mut rng);
        flat.extend([x, y, z]);
    }
    tensor(flat, n, 3)
}

fn structural_checks(report: &mut Report) {
    let cfg = BossConfig::<f64>::with_seed(0);

    // Chain: the essential graph is the undirected path X — Y — Z.
    let chain_cpdag = boss_learn(&chain(600, 11), &cfg).expect("boss_learn chain");
    report.check(
        "chain: X — Y undirected",
        chain_cpdag.edge_kind(0, 1) == Some(EdgeKind::Undirected),
    );
    report.check(
        "chain: Y — Z undirected",
        chain_cpdag.edge_kind(1, 2) == Some(EdgeKind::Undirected),
    );
    report.check(
        "chain: no spurious X — Z edge",
        chain_cpdag.edge_kind(0, 2).is_none(),
    );

    // Collider: the v-structure X → Z ← Y is compelled (directed into Z).
    let coll_cpdag = boss_learn(&collider(600, 7), &cfg).expect("boss_learn collider");
    report.check(
        "collider: X → Z directed",
        coll_cpdag.edge_kind(0, 2) == Some(EdgeKind::Directed),
    );
    report.check(
        "collider: Y → Z directed",
        coll_cpdag.edge_kind(1, 2) == Some(EdgeKind::Directed),
    );
    report.check(
        "collider: Z has both parents",
        coll_cpdag.parents(2).contains(&0) && coll_cpdag.parents(2).contains(&1),
    );
    report.check(
        "collider: no X — Y edge",
        coll_cpdag.edge_kind(0, 1).is_none(),
    );
}

fn end_to_end_check(report: &mut Report) {
    let case = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("verification/brcd/data/online-boutique/adservice_cpu_1");
    if !case.join("normal.csv").exists() {
        report.check(
            &format!("online-boutique case present at {}", case.display()),
            false,
        );
        return;
    }

    let (nd, nr, nc) = load_csv(&case.join("normal.csv")).expect("read normal.csv");
    let (ad, ar, ac) = load_csv(&case.join("anomalous.csv")).expect("read anomalous.csv");
    let expected = load_expected(&case.join("expected.txt")).expect("read expected.txt");
    let true_fault = expected
        .first()
        .copied()
        .expect("expected.txt has a top fault");

    let mut config = BrcdConfig::continuous(0);
    config.transform_parents = true;

    // cpdag = None → BOSS learns the structure from the normal data, then ranks.
    let result = brcd_run::<f64, ()>(&tensor(nd, nr, nc), &tensor(ad, ar, ac), None, &config)
        .expect("brcd_run(None) learns a CPDAG and ranks");

    let ranking: Vec<usize> = result
        .ranks()
        .iter()
        .filter_map(|c| c.first().copied())
        .collect();
    let fault_rank = ranking.iter().position(|&v| v == true_fault);

    println!(
        "  [adservice_cpu_1] learned-CPDAG top-5: {:?} | true fault (supplied-CPDAG #1): {} | learned rank: {:?}",
        &ranking[..5.min(ranking.len())],
        true_fault,
        fault_rank.map(|r| r + 1),
    );

    // The pipeline must produce a complete ranking over all candidates, and the
    // learned-CPDAG run must recover the true fault within the top 5 (it lands at
    // #1 here). Reproduction is structural + downstream-ranking, not byte-exact
    // against the supplied-CPDAG `expected.txt` (see module docs).
    report.check(
        "online-boutique: brcd_run(None) recovers the fault in the top-5",
        ranking.len() == nc && fault_rank.is_some_and(|r| r < 5),
    );
}

fn main() {
    let mut report = Report::new("BOSS structure learning");
    structural_checks(&mut report);
    end_to_end_check(&mut report);
    report.finish();
}
