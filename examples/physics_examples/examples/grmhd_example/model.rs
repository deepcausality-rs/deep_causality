/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::PropagatingEffect;
use deep_causality_haft::Applicative;
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

/// Configuration for the GRMHD simulation
#[derive(Clone, Debug, Default)]
pub struct SimulationConfig {
    pub current_density: f64,
    pub magnetic_field: f64,
    pub curvature_threshold: f64,
}

/// State propagated through the causal chain
#[derive(Clone, Debug, Default)]
pub struct GrmhdState {
    // Configuration
    pub current_density: f64,
    pub magnetic_field: f64,
    pub curvature_threshold: f64,

    // GR Results
    pub curvature_intensity: f64,

    // Coupling Results
    pub metric: Option<Metric>,
    pub metric_label: String,

    // MHD Results
    pub lorentz_force: f64,

    // Analysis Results
    pub stability_status: String,
}

impl GrmhdState {
    pub fn new(config: &SimulationConfig) -> Self {
        Self {
            current_density: config.current_density,
            magnetic_field: config.magnetic_field,
            curvature_threshold: config.curvature_threshold,
            ..Default::default()
        }
    }
}

/// Step 1: GR Solver - Calculate spacetime curvature
///
/// Uses Tensor monad to compute the Einstein tensor from the metric.
pub fn calculate_curvature(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    // Calculate the spacetime metric tensor (Schwarzschild-like)
    let g_uv = calculate_spacetime_metric();

    // Calculate Einstein tensor G_uv
    let g_tensor = calculate_einstein_tensor(&g_uv);

    // Extract local curvature intensity from g_00 component
    let curvature_intensity = g_tensor.data()[0].abs();
    println!(
        "   -> Local Curvature Intensity: {:.4}",
        curvature_intensity
    );

    PropagatingEffect::pure(GrmhdState {
        curvature_intensity,
        ..state
    })
}

/// Step 2: Coupling Layer - Select appropriate metric based on curvature
///
/// Dynamic type/value decision driven by physics:
/// - High curvature → Minkowski(4) relativistic metric
/// - Low curvature → Euclidean(3) classical metric
pub fn select_metric(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let (metric, label) = if state.curvature_intensity > state.curvature_threshold {
        (Metric::Minkowski(4), "Relativistic (Minkowski 4D)")
    } else {
        (Metric::Euclidean(3), "Classical (Euclidean 3D)")
    };

    println!("   -> Selected Metric: {}", label);

    PropagatingEffect::pure(GrmhdState {
        metric: Some(metric),
        metric_label: label.to_string(),
        ..state
    })
}

/// Step 3: MHD Solver - Calculate Lorentz force
///
/// Uses MultiVector monad to compute F = J · B with the selected metric.
pub fn calculate_lorentz_force(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let metric = match state.metric {
        Some(m) => m,
        None => return PropagatingEffect::pure(state),
    };

    let force = compute_lorentz_force_internal(state.current_density, state.magnetic_field, metric);

    println!("   -> Lorentz Force Density: {:.4}", force);

    PropagatingEffect::pure(GrmhdState {
        lorentz_force: force,
        ..state
    })
}

/// Step 4: Stability Analysis - Determine confinement status
pub fn analyze_stability(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let status = if state.lorentz_force < 0.0 {
        println!("   STATUS: Relativistic Reversal Detected!");
        println!("   Action: Adjusting containment field to compensate for frame dragging.");
        "Relativistic Reversal - Adjustment Required"
    } else {
        println!("   STATUS: Standard Confinement.");
        "Stable Confinement"
    };

    PropagatingEffect::pure(GrmhdState {
        stability_status: status.to_string(),
        ..state
    })
}

// =============================================================================
// Internal Physics Functions
// =============================================================================

/// Calculate the spacetime metric tensor (Schwarzschild-like)
fn calculate_spacetime_metric() -> CausalTensor<f64> {
    // Minkowski metric signature (- + + +) perturbed by gravity
    // g_00 = -(1 - 2GM/rc^2)
    let g_00 = -0.9; // Time dilation
    let g_11 = 1.1; // Radial stretching
    let g_22 = 1.0;
    let g_33 = 1.0;

    let metric_data = vec![
        g_00, 0.0, 0.0, 0.0, 0.0, g_11, 0.0, 0.0, 0.0, 0.0, g_22, 0.0, 0.0, 0.0, 0.0, g_33,
    ];

    CausalTensor::new(metric_data, vec![4, 4]).unwrap()
}

/// Calculate Einstein tensor G_uv (simplified: G_uv ~ R * g_uv)
fn calculate_einstein_tensor(g_uv: &CausalTensor<f64>) -> CausalTensor<f64> {
    let curvature = 0.1; // Scalar curvature R

    // G_uv ~ R * g_uv (Simplified EFE LHS)
    let scale_fn = |x: f64| x * curvature;
    let fn_tensor = <CausalTensorWitness as Applicative<CausalTensorWitness>>::pure(scale_fn);

    <CausalTensorWitness as Applicative<CausalTensorWitness>>::apply(fn_tensor, g_uv.clone())
}

/// Calculate Lorentz Force F = J · B using the appropriate metric
fn compute_lorentz_force_internal(
    current_density: f64,
    magnetic_field: f64,
    metric_signature: Metric,
) -> f64 {
    // 1. Setup Plasma Current (J) - flowing toroidally (X-axis)
    let idx_current = 1 << 1;
    let mut j_data = vec![0.0; 1 << metric_signature.dimension()];
    j_data[idx_current] = current_density;
    let j_vec = CausalMultiVector::new(j_data, metric_signature).unwrap();

    // 2. Setup Magnetic Field (B) - applied poloidally (XY-plane)
    let idx_field_plane = (1 << 1) | (1 << 2);
    let mut b_data = vec![0.0; 1 << metric_signature.dimension()];
    b_data[idx_field_plane] = magnetic_field;
    let b_field = CausalMultiVector::new(b_data, metric_signature).unwrap();

    // 3. Compute Force: F = J · B (inner product)
    let force = j_vec.inner_product(&b_field);

    // Extract force component in poloidal direction (dimension 2)
    let idx_force = 1 << 2;
    *force.get(idx_force).unwrap()
}
