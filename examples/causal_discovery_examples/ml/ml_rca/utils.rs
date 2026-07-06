/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Utility layer: dataset loading / preparation and the console-reporting functions. Keeping I/O
//! and presentation here leaves `main` a lean orchestration of the model stages.

use crate::model::{N_FEATURES, RcaProcess, RcaSignal};
use std::fs;

// ---------------------------------------------------------------------------
// Data loading & preparation
// ---------------------------------------------------------------------------

/// Parse a Sock Shop metrics CSV into (header, rows). Each row is 44 f64 features.
pub fn load_csv(path: &str) -> Result<(Vec<String>, Vec<Vec<f64>>), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    let mut lines = text.lines();
    let header: Vec<String> = lines
        .next()
        .ok_or_else(|| format!("{path}: empty file"))?
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let row: Vec<f64> = line
            .split(',')
            .map(|s| s.trim().parse::<f64>().unwrap_or(0.0))
            .collect();
        if row.len() >= N_FEATURES {
            rows.push(row[..N_FEATURES].to_vec());
        }
    }
    Ok((header, rows))
}

/// Read the ground-truth root-cause column index (first line of `expected.txt`).
pub fn load_truth_index(path: &str) -> Result<usize, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("read {path}: {e}"))?
        .lines()
        .next()
        .and_then(|l| l.trim().parse().ok())
        .ok_or_else(|| "expected.txt: no root-cause index".to_string())
}

/// Build the labeled training set (normal = 0, anomalous = 1) and a held-out evaluation split.
/// Deterministic: the last 20% of each class is held out.
#[allow(clippy::type_complexity)]
pub fn build_training_set(
    normal_rows: &[Vec<f64>],
    anomalous_rows: &[Vec<f64>],
) -> (Vec<Vec<f64>>, Vec<f64>, Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let split = |rows: &[Vec<f64>]| -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let cut = rows.len() * 4 / 5;
        (rows[..cut].to_vec(), rows[cut..].to_vec())
    };
    let (norm_tr, norm_te) = split(normal_rows);
    let (anom_tr, anom_te) = split(anomalous_rows);

    let mut train_rows = norm_tr.clone();
    train_rows.extend(anom_tr.clone());
    let mut train_labels = vec![0.0; norm_tr.len()];
    train_labels.extend(vec![1.0; anom_tr.len()]);

    (train_rows, train_labels, norm_te, anom_te)
}

/// Column-wise mean and standard deviation over the given rows.
pub fn fit_standardizer(rows: &[Vec<f64>]) -> (Vec<f64>, Vec<f64>) {
    let n = rows.len() as f64;
    let mut mean = vec![0.0; N_FEATURES];
    for r in rows {
        for (m, &v) in mean.iter_mut().zip(r.iter()) {
            *m += v / n;
        }
    }
    let mut var = vec![0.0; N_FEATURES];
    for r in rows {
        for (j, &v) in r.iter().enumerate() {
            var[j] += (v - mean[j]).powi(2) / n;
        }
    }
    let std = var.iter().map(|v| v.sqrt()).collect();
    (mean, std)
}

// ---------------------------------------------------------------------------
// Console reporting
// ---------------------------------------------------------------------------

/// The example banner.
pub fn print_banner() {
    println!("=== ML-gated Causal Root-Cause Analysis (Candle × DeepCausality) ===\n");
}

/// Summarize the loaded dataset.
pub fn print_dataset_summary(n_normal: usize, n_anomalous: usize) {
    println!("Loaded {n_normal} normal + {n_anomalous} anomalous rows, {N_FEATURES} features.");
}

/// Report the shipped ground-truth root cause.
pub fn print_ground_truth(header: &[String], truth_index: usize) {
    let metric = header
        .get(truth_index)
        .cloned()
        .unwrap_or_else(|| format!("col_{truth_index}"));
    println!("Ground-truth root cause: {metric} (col {truth_index}).\n");
}

/// Report the detector's held-out mean scores and their separation.
pub fn print_detector_scores(normal_score: f64, anomalous_score: f64) {
    println!("Candle detector, held-out mean anomaly score:");
    println!("  normal window    : {normal_score:.3}");
    println!("  anomalous window : {anomalous_score:.3}");
    println!(
        "  separation       : anomalous {} normal\n",
        if anomalous_score > normal_score {
            ">"
        } else {
            "≤ (!)"
        }
    );
}

/// Report the calibrated gate threshold.
pub fn print_threshold(threshold: f64) {
    println!("Gate threshold (calibrated midpoint): {threshold:.3}\n");
}

/// Print one window's verdict (healthy / root cause) and its `EffectLog`.
pub fn print_window_result(label: &str, result: &RcaProcess) {
    println!("--- Window: {label} ---");
    if let Some(err) = result.error() {
        println!("  verdict: ERROR (causal stage failed): {err}");
        println!("  {}", result.logs());
        return;
    }
    match result.value() {
        Some(RcaSignal::Healthy { score }) => {
            println!("  verdict: HEALTHY (score {score:.3}); causal stage not run.");
        }
        Some(RcaSignal::RootCause {
            metric,
            index,
            posterior,
            matches_truth,
            ..
        }) => {
            println!("  verdict: ROOT CAUSE = {metric} (col {index}, posterior {posterior:.4})");
            println!(
                "  ground-truth check: {}",
                if *matches_truth {
                    "MATCH ✓ (ML detected, causality explained, ground truth confirmed)"
                } else {
                    "no match; see the ranking vs expected.txt"
                }
            );
        }
        other => println!("  verdict: {other:?}"),
    }
    println!("  {}", result.logs());
}
