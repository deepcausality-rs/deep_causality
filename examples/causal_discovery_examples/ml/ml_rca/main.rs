/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # ML-gated causal root-cause analysis (Candle × DeepCausality)
//!
//! **"ML detects, causality explains."** A small **Candle** classifier learns to *detect* that a
//! microservice system is anomalous; when it fires, the existing **DeepCausality** BRCD
//! causal-discovery pipeline *explains* the anomaly by ranking the culprit service/metric. The two
//! stages are sequenced through a `PropagatingProcess` monad: the Candle anomaly score gates the
//! causal stage, the carried value becomes the root-cause verdict, and the escalation is recorded in
//! the `EffectLog`.
//!
//! Both halves run on the shipped, labeled RCAEval Sock Shop case (`data/sock-shop-2/carts_cpu_1`):
//! 44 service metrics, a `normal.csv` window (label 0) and an `anomalous.csv` window (label 1), and
//! a supplied service-call CPDAG. The detector is a logistic regression written directly on
//! `candle-core` (manual forward pass, binary-cross-entropy, hand-rolled gradient descent, no
//! `candle-nn`). The explainer reuses the same CDL BRCD pipeline as `example_brcd_discovery`; its
//! top-ranked culprit is checked against the case's shipped ground truth (`expected.txt`), which
//! ranks `shipping_latency` (column 42) first.
//!
//! Candle is an example-only dependency; no `deep_causality_*` library crate depends on it.
//!
//! The detector + causal/monad logic lives in [`model`]; data loading and console reporting live in
//! [`utils`]; this file is the lean orchestration.
//!
//! Run: `cargo run -p causal_discovery_examples --example example_ml_rca`

mod model;
mod utils;

use candle_core::Device;
use model::{RcaConfig, run_window, train_detector};
use utils::{
    build_training_set, fit_standardizer, load_csv, load_truth_index, print_banner,
    print_dataset_summary, print_detector_scores, print_ground_truth, print_threshold,
    print_window_result,
};

fn main() -> Result<(), String> {
    print_banner();

    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/data/sock-shop-2/carts_cpu_1");
    let normal_path = format!("{base}/normal.csv");
    let anomalous_path = format!("{base}/anomalous.csv");
    let cpdag_path = format!("{base}/cpdag.csv");
    let expected_path = format!("{base}/expected.txt");

    // 1. Load the labeled dataset (normal = 0, anomalous = 1) and the ground-truth root cause.
    let (header, normal_rows) = load_csv(&normal_path)?;
    let (_, anomalous_rows) = load_csv(&anomalous_path)?;
    print_dataset_summary(normal_rows.len(), anomalous_rows.len());

    let truth_index = load_truth_index(&expected_path)?;
    print_ground_truth(&header, truth_index);

    // 2. Split, standardize, and train the Candle detector (detect stage).
    let dev = Device::Cpu;
    let (train_rows, train_labels, norm_te, anom_te) =
        build_training_set(&normal_rows, &anomalous_rows);
    let (mean, std) = fit_standardizer(&train_rows);
    let detector = train_detector(&train_rows, &train_labels, mean, std, &dev)
        .map_err(|e| format!("training: {e}"))?;

    let normal_score = detector
        .mean_score(&norm_te, &dev)
        .map_err(|e| format!("score: {e}"))?;
    let anomalous_score = detector
        .mean_score(&anom_te, &dev)
        .map_err(|e| format!("score: {e}"))?;
    print_detector_scores(normal_score, anomalous_score);

    // 3. Calibrate the gate to the detector's operating point (midpoint of the class scores).
    let threshold = (normal_score + anomalous_score) / 2.0;
    print_threshold(threshold);

    let cfg = RcaConfig {
        normal_path,
        anomalous_path,
        cpdag_path,
        header,
        truth_index,
        threshold,
    };

    // 4. Drive the causal-monad bridge twice: a normal window (short-circuits) and an anomalous
    //    window (escalates to the causal explainer).
    for (label, score) in [
        ("normal-window", normal_score),
        ("anomalous-window", anomalous_score),
    ] {
        let result = run_window(label, score, &cfg);
        print_window_result(label, &result);
    }

    Ok(())
}
