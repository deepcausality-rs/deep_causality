/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Base verification — self-contained synthetic recovery.
//!
//! Generates a linear-Gaussian chain `X → Y → Z` under normal conditions, then
//! an anomalous dataset that perturbs `p(Y | X)` (Y's intercept jumps). Because
//! only Y's conditional mechanism changes, BRCD must rank **Y** as the top root
//! cause. No Python or external data is required — this is the base smoke
//! verification gating the real-world examples.
//!
//! Run: `cargo run -p deep_causality_algorithms --example verification_base`

mod common;

use common::{Report, cpdag};
use deep_causality_algorithms::brcd::{BrcdConfig, brcd_run};
use deep_causality_rand::{Distribution, Normal, Xoshiro256};
use deep_causality_tensor::CausalTensor;

/// Linear-Gaussian chain `X = εx`, `Y = y_intercept + 1.5·X + εy`, `Z = 2·Y + εz`.
/// Columns are `[X, Y, Z]`.
fn chain_data(n: usize, y_intercept: f64, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let dist = Normal::new(0.0_f64, 1.0).unwrap();
    let mut data = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = dist.sample(&mut rng);
        let y = y_intercept + 1.5 * x + dist.sample(&mut rng);
        let z = 2.0 * y + dist.sample(&mut rng);
        data.extend_from_slice(&[x, y, z]);
    }
    CausalTensor::new(data, vec![n, 3]).unwrap()
}

fn main() {
    let mut report = Report::new("base — synthetic X→Y→Z recovery");

    // Normal vs anomalous: the anomaly shifts Y's intercept (perturbs p(Y|X)).
    let normal = chain_data(120, 0.0, 1);
    let anomalous = chain_data(120, 4.0, 2);
    // The undirected chain CPDAG X — Y — Z (the path-of-3 chain component).
    let graph = cpdag(3, &[(0, 1), (1, 2)], &[]);

    let result = brcd_run(
        &normal,
        &anomalous,
        Some(&graph),
        &BrcdConfig::continuous(7),
    )
    .expect("brcd_run on the synthetic chain");

    println!("{result}");
    // X = 0, Y = 1, Z = 2. The perturbed mechanism is Y's.
    report.check(
        "top-ranked root cause is Y (index 1)",
        result.top() == Some(&[1][..]),
    );
    report.finish();
}
