/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer: the value/state/context types, the Candle anomaly **detector**, the reused BRCD
//! causal **explainer**, and the `PropagatingProcess` **gate** that bridges them.

use candle_core::{DType, Device, Tensor, Var};
use deep_causality_core::{
    CausalEffect, CausalityError, CausalityErrorEnum, EffectLog, PropagatingProcess,
};
use deep_causality_discovery::{BrcdConfig, CdlBuilder, CdlConfigBuilder};
use deep_causality_haft::LogAddEntry;

/// Number of metric columns in the Sock Shop case.
pub const N_FEATURES: usize = 44;
/// Gradient-descent epochs for the logistic-regression detector.
const EPOCHS: usize = 800;
/// Learning rate.
const LR: f64 = 0.3;

// ---------------------------------------------------------------------------
// Value / State / Context for the causal-monad bridge
// ---------------------------------------------------------------------------

/// The value channel carried through the `PropagatingProcess`.
#[derive(Debug, Clone)]
pub enum RcaSignal {
    /// Detector output before the gate decides (the anomaly score in `[0, 1]`).
    Score(f64),
    /// Below threshold: no escalation, no causal verdict.
    Healthy { score: f64 },
    /// Escalated: the causal stage named a culprit.
    RootCause {
        metric: String,
        index: usize,
        posterior: f64,
        matches_truth: bool,
    },
}

/// Carried state: the window label, for the narrative.
#[derive(Debug, Clone)]
pub struct RcaState {
    pub window_label: String,
}

/// Read-only context: data paths, the column header, and the shipped ground truth.
#[derive(Debug, Clone)]
pub struct RcaConfig {
    pub normal_path: String,
    pub anomalous_path: String,
    pub cpdag_path: String,
    pub header: Vec<String>,
    /// Ground-truth root-cause column index (first line of `expected.txt`).
    pub truth_index: usize,
    /// Gate threshold, calibrated at runtime to the detector's operating point (the midpoint
    /// between the held-out normal and anomalous mean scores). At/above this, the gate escalates.
    pub threshold: f64,
}

/// The process type threaded through the detect → gate → explain chain.
pub type RcaProcess = PropagatingProcess<RcaSignal, RcaState, RcaConfig>;

// ---------------------------------------------------------------------------
// Detect stage: a logistic-regression anomaly classifier on candle-core
// ---------------------------------------------------------------------------

/// The trained detector: weights + the standardization statistics fit on the training rows.
pub struct Detector {
    w: Tensor,
    b: Tensor,
    mean: Vec<f64>,
    std: Vec<f64>,
}

impl Detector {
    /// Score a single 44-feature row, returning the anomaly probability in `[0, 1]`.
    fn score_row(&self, row: &[f64], dev: &Device) -> Result<f64, candle_core::Error> {
        let z: Vec<f32> = standardize(row, &self.mean, &self.std)
            .into_iter()
            .map(|v| v as f32)
            .collect();
        let x = Tensor::from_vec(z, (1, N_FEATURES), dev)?;
        let logit = x.matmul(&self.w)?.broadcast_add(&self.b)?;
        let p = sigmoid(&logit)?;
        Ok(p.flatten_all()?.to_vec1::<f32>()?[0] as f64)
    }

    /// Mean anomaly score over a set of rows.
    pub fn mean_score(&self, rows: &[Vec<f64>], dev: &Device) -> Result<f64, candle_core::Error> {
        let mut acc = 0.0;
        for r in rows {
            acc += self.score_row(r, dev)?;
        }
        Ok(acc / rows.len() as f64)
    }
}

/// `sigmoid(z) = 1 / (1 + exp(-z))`, written with `candle-core` tensor ops only.
fn sigmoid(z: &Tensor) -> Result<Tensor, candle_core::Error> {
    // exp(-z), then (1·exp(-z) + 1), then reciprocal.
    z.neg()?.exp()?.affine(1.0, 1.0)?.recip()
}

/// Per-feature z-score standardization using fitted statistics.
pub fn standardize(row: &[f64], mean: &[f64], std: &[f64]) -> Vec<f64> {
    row.iter()
        .zip(mean.iter().zip(std.iter()))
        .map(|(&x, (&m, &s))| if s > 1e-12 { (x - m) / s } else { 0.0 })
        .collect()
}

/// Train the logistic-regression detector on the labeled, standardized rows.
///
/// Manual gradient descent on `candle-core` (no `candle-nn`): forward pass, binary-cross-entropy
/// loss, `loss.backward()`, then a hand-rolled `Var::set` update. Zero weight init keeps it
/// deterministic.
pub fn train_detector(
    train_rows: &[Vec<f64>],
    train_labels: &[f64],
    mean: Vec<f64>,
    std: Vec<f64>,
    dev: &Device,
) -> Result<Detector, candle_core::Error> {
    let n = train_rows.len();
    let mut x_flat = Vec::with_capacity(n * N_FEATURES);
    for r in train_rows {
        for v in standardize(r, &mean, &std) {
            x_flat.push(v as f32);
        }
    }
    let x = Tensor::from_vec(x_flat, (n, N_FEATURES), dev)?;
    let y = Tensor::from_vec(
        train_labels.iter().map(|&v| v as f32).collect::<Vec<_>>(),
        (n, 1),
        dev,
    )?;

    let w = Var::from_tensor(&Tensor::zeros((N_FEATURES, 1), DType::F32, dev)?)?;
    let b = Var::from_tensor(&Tensor::zeros((1, 1), DType::F32, dev)?)?;

    for _ in 0..EPOCHS {
        let logit = x.matmul(w.as_tensor())?.broadcast_add(b.as_tensor())?;
        let p = sigmoid(&logit)?.clamp(1e-7, 1.0 - 1e-7)?;
        // Binary cross-entropy: -mean( y·log(p) + (1-y)·log(1-p) ).
        let log_p = p.log()?;
        let log_1mp = p.affine(-1.0, 1.0)?.log()?;
        let term = (y.mul(&log_p)? + y.affine(-1.0, 1.0)?.mul(&log_1mp)?)?;
        let loss = term.mean_all()?.neg()?;

        let grads = loss.backward()?;
        let gw = grads.get(w.as_tensor()).expect("grad w");
        let gb = grads.get(b.as_tensor()).expect("grad b");
        w.set(&w.as_tensor().sub(&gw.affine(LR, 0.0)?)?)?;
        b.set(&b.as_tensor().sub(&gb.affine(LR, 0.0)?)?)?;
    }

    Ok(Detector {
        w: w.as_tensor().clone(),
        b: b.as_tensor().clone(),
        mean,
        std,
    })
}

// ---------------------------------------------------------------------------
// Explain stage: the existing CDL BRCD root-cause pipeline (reused verbatim)
// ---------------------------------------------------------------------------

/// Run the BRCD causal-discovery pipeline and return the top-ranked culprit column index and its
/// posterior weight. Reuses the same surface as `example_brcd_discovery`.
fn brcd_top_culprit(cfg: &RcaConfig) -> Result<(usize, f64), String> {
    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(&cfg.normal_path)
        .with_anomalous_path(&cfg.anomalous_path)
        .with_brcd_config(BrcdConfig::<f64>::continuous(0))
        .with_cpdag_path(&cfg.cpdag_path)
        .build()
        .map_err(|e| format!("BRCD config: {e}"))?;

    let effect = CdlBuilder::build_brcd(&config)
        .brcd_load_input()
        .brcd_discover();

    let cdl = effect.inner.map_err(|e| format!("BRCD discover: {e}"))?;
    let result = &cdl.state.brcd_result;
    let ranks = result.ranks();
    let posterior = result.posterior();
    let top = ranks
        .first()
        .and_then(|set| set.first())
        .copied()
        .ok_or_else(|| "BRCD ranked no candidates".to_string())?;
    let weight = posterior.first().copied().unwrap_or(f64::NAN);
    Ok((top, weight))
}

// ---------------------------------------------------------------------------
// The gate: a single PropagatingProcess bind stage bridging detect -> explain
// ---------------------------------------------------------------------------

/// The gate bind stage. Reads the detector score from the value channel; below threshold it
/// short-circuits to `Healthy` (no causal stage); at/above threshold it escalates, runs the BRCD
/// explainer, compares the culprit against the shipped ground truth, and carries a `RootCause`
/// verdict, recording the escalation in the `EffectLog`.
fn gate_stage(
    value: CausalEffect<RcaSignal>,
    state: RcaState,
    ctx: Option<RcaConfig>,
) -> RcaProcess {
    let cfg = ctx.as_ref().expect("RcaConfig required");
    let score = match value.into_value() {
        Some(RcaSignal::Score(s)) => s,
        _ => 0.0,
    };
    let threshold = cfg.threshold;
    let mut logs = EffectLog::new();

    if score < threshold {
        logs.add_entry(&format!(
            "[{}] anomaly score {:.3} < {:.3} → healthy, no escalation (causal stage skipped)",
            state.window_label, score, threshold
        ));
        return RcaProcess::new(
            Ok(CausalEffect::value(RcaSignal::Healthy { score })),
            state,
            ctx,
            logs,
        );
    }

    logs.add_entry(&format!(
        "[{}] anomaly score {:.3} ≥ {:.3} → ESCALATE to causal root-cause analysis",
        state.window_label, score, threshold
    ));

    match brcd_top_culprit(cfg) {
        Ok((index, posterior)) => {
            let metric = cfg
                .header
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("col_{index}"));
            let matches_truth = index == cfg.truth_index;
            logs.add_entry(&format!(
                "[{}] causal verdict: root cause = {} (col {}, posterior {:.4}); ground-truth match = {}",
                state.window_label, metric, index, posterior, matches_truth
            ));
            RcaProcess::new(
                Ok(CausalEffect::value(RcaSignal::RootCause {
                    metric,
                    index,
                    posterior,
                    matches_truth,
                })),
                state,
                ctx,
                logs,
            )
        }
        Err(e) => {
            // Propagate the causal-stage failure into the process error channel so the chain
            // enters an error state. Logging alone would leave a failed run looking like a normal,
            // empty result.
            logs.add_entry(&format!(
                "[{}] causal stage failed: {e}",
                state.window_label
            ));
            RcaProcess::new(
                Err(CausalityError::new(CausalityErrorEnum::Custom(format!(
                    "causal root-cause analysis failed: {e}"
                )))),
                state,
                ctx,
                logs,
            )
        }
    }
}

/// Run one window through the causal-monad bridge: seed the process with the detector score, then
/// `bind` the gate (which escalates to the explainer or short-circuits to healthy).
pub fn run_window(label: &str, score: f64, cfg: &RcaConfig) -> RcaProcess {
    let start = RcaProcess::new(
        Ok(CausalEffect::value(RcaSignal::Score(score))),
        RcaState {
            window_label: label.to_string(),
        },
        Some(cfg.clone()),
        EffectLog::new(),
    );
    start.bind(gate_stage)
}
