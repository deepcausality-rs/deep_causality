/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::PropagatingEffect;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    einstein_tensor, energy_momentum_tensor_em, generate_schwarzschild_metric, lorentz_force,
};
use deep_causality_tensor::CausalTensor;

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
    pub metric_tensor: Option<CausalTensor<f64>>, // Stored for coupling
    // Coupling Results
    pub metric: Option<Metric>,
    pub metric_label: String,
    // MHD Results
    pub lorentz_force: f64,
    pub em_energy_density: f64, // T^00 component
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
    // Minkowski metric signature (- + + +) perturbed by gravity
    // g_00 = -(1 - 2GM/rc^2)
    let g_00 = -0.9; // Time dilation
    let g_11 = 1.1; // Radial stretching
    let g_22 = 1.0;
    let g_33 = 1.0;
    let g_uv =
        generate_schwarzschild_metric(g_00, g_11, g_22, g_33).expect("Failed to generate metric");

    // Synthetic Ricci Tensor Construction (Proxy for example)
    // Assumption: R_uv ~ -0.1 * g_uv, R ~ -0.4
    // G_uv = R_uv - 0.5 * R * g_uv = -0.1g - 0.5(-0.4)g = -0.1g + 0.2g = 0.1g
    // This matches the example's outcome (positive curvature intensity)
    let ricci = g_uv.clone() * -0.1;
    let scalar_r = -0.4;

    // Calculate Einstein tensor G_uv using physics kernel
    let g_tensor_wrapper = einstein_tensor(&ricci, scalar_r, &g_uv);

    let g_tensor = match g_tensor_wrapper.value.into_value() {
        Some(t) => t,
        None => {
            return PropagatingEffect::from_error(deep_causality::CausalityError(
                deep_causality::CausalityErrorEnum::Custom(
                    "Einstein Tensor calculation failed".into(),
                ),
            ));
        }
    };

    // Extract local curvature intensity from g_00 component
    let curvature_intensity = g_tensor.data()[0].abs();
    println!(
        "   -> Local Curvature Intensity: {:.4}",
        curvature_intensity
    );

    PropagatingEffect::pure(GrmhdState {
        curvature_intensity,
        metric_tensor: Some(g_uv),
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

    // 1. Setup Plasma Current vector (J) - axis 1
    let idx_current = 1 << 1;
    let mut j_data = vec![0.0; 1 << metric.dimension()];
    j_data[idx_current] = state.current_density;
    let j_vec = CausalMultiVector::new(j_data, metric).unwrap();

    // 2. Setup Magnetic Field vector (B) - axis 2
    // Note: Physics kernel expects B as a vector. J (e1) ^ B (e2) = F (e12 bivector)
    let idx_b = 1 << 2;
    let mut b_data = vec![0.0; 1 << metric.dimension()];
    b_data[idx_b] = state.magnetic_field;
    let b_vec = CausalMultiVector::new(b_data, metric).unwrap();

    // 3. Compute Lorentz Force using Wrapper: F = J ^ B
    let f_effect = lorentz_force(&j_vec, &b_vec);

    match f_effect.value.into_value() {
        Some(f_field) => {
            // Extract force component (e12 bivector)
            let idx_force = idx_current | idx_b;
            let force = *f_field.0.get(idx_force).unwrap_or(&0.0);

            println!("   -> Lorentz Force Bivector Intensity: {:.4}", force);

            PropagatingEffect::pure(GrmhdState {
                lorentz_force: force,
                ..state
            })
        }
        None => PropagatingEffect::from_error(deep_causality::CausalityError(
            deep_causality::CausalityErrorEnum::Custom("Lorentz Force calculation failed".into()),
        )),
    }
}

/// Step 3b: Calculate Energy-Momentum Tensor
///
/// Uses the new GRMHD module to compute T_uv for the electromagnetic field.
/// This couples the MHD field back to the GR metric geometry.
pub fn calculate_energy_momentum(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let g_uv = match &state.metric_tensor {
        Some(t) => t,
        None => return PropagatingEffect::pure(state),
    };

    // Construct Electromagnetic Tensor F^uv
    // Assuming B is along z-axis, B_z = F^12 (x,y component).
    // F^uv = [[0, 0, 0, 0], [0, 0, B, 0], [0, -B, 0, 0], [0, 0, 0, 0]]
    // (Indices: 0=t, 1=x, 2=y, 3=z)
    let b = state.magnetic_field;
    let mut f_data = vec![0.0; 16];
    f_data[4 + 2] = b; // F^12
    f_data[2 * 4 + 1] = -b; // F^21

    let f_tensor = match CausalTensor::new(f_data, vec![4, 4]) {
        Ok(t) => t,
        Err(e) => {
            return PropagatingEffect::from_error(deep_causality::CausalityError(
                deep_causality::CausalityErrorEnum::Custom(e.to_string()),
            ));
        }
    };

    // Calculate T^uv
    let t_effect = energy_momentum_tensor_em(&f_tensor, g_uv);

    match t_effect.value.into_value() {
        Some(t_tensor) => {
            // Extract energy density T^00
            let energy_density = t_tensor.data()[0];
            println!("   -> EM Energy Density (T^00): {:.4}", energy_density);

            PropagatingEffect::pure(GrmhdState {
                em_energy_density: energy_density,
                ..state
            })
        }
        None => PropagatingEffect::from_error(deep_causality::CausalityError(
            deep_causality::CausalityErrorEnum::Custom(
                "Energy Momentum Tensor calculation failed".into(),
            ),
        )),
    }
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
