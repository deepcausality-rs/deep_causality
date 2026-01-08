/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # General Relativity Gauge Field Pipeline
//!
//! Demonstrates **modular causal composition** via `CausalEffectPropagationProcess`
//! for General Relativity (GR) spacetime analysis.
//!
//! ## Generic Float Type Support
//!
//! This example supports `f32`, `f64`, and `DoubleFloat` by changing the `FloatType`
//! type alias. All numeric literals are converted using the `flt!` macro.
//!
use deep_causality_core::{CausalEffectPropagationProcess, EffectValue, PropagatingEffect};
use deep_causality_num::{DoubleFloat, Float};
use deep_causality_physics::{AdmOps, GrOps, LorentzianMetric};
use deep_causality_physics::{AdmState, EastCoastMetric, GR, SPEED_OF_LIGHT};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{GaugeField, Manifold, Simplex, SimplicialComplexBuilder};
use std::error::Error;

// =============================================================================
// FLOAT TYPE CONFIGURATION
// =============================================================================

// Change this to f32 or DoubleFloat to use different precision
type FloatType = DoubleFloat;
type GRTheory = GR<FloatType>;

/// Macro to convert f64 literals to FloatType
/// This enables writing `flt!(1.0)` instead of `<FloatType as From<f64>>::from(1.0)`
macro_rules! float_from_f64 {
    ($x:expr) => {
        <FloatType as From<f64>>::from($x)
    };
}

// =============================================================================
// MAIN: Pipeline Composition via Causal Monad
// =============================================================================

fn main() -> Result<(), Box<dyn Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  General Relativity Spacetime Analysis");
    println!("  (Float Type: {})", std::any::type_name::<FloatType>());
    println!("═══════════════════════════════════════════════════════════════\n");

    // Composed pipeline using bind_or_error
    let result = initial_stage_create_schwarzschild()
        .bind_or_error(stage_curvature_invariants, "Curvature computation failed")
        .bind_or_error(stage_geodesic_analysis, "Geodesic analysis failed")
        .bind_or_error(stage_adm_formalism, "ADM formalism failed")
        .bind_or_error(stage_event_horizon_detection, "Horizon detection failed");

    // Extract and display final result
    print_summary(&result);

    Ok(())
}

// =============================================================================
// GR State: Passed through pipeline stages
// =============================================================================

/// Accumulated results from pipeline stages
#[derive(Debug, Clone, Default)]
pub struct SpaceTimeData {
    /// General Relativity gauge field
    pub gr: Option<GRTheory>,
    /// Observation radius r
    pub r: FloatType,
    /// Schwarzschild radius r_s
    pub r_s: FloatType,
    /// Kretschmann scalar
    pub kretschmann: FloatType,
    /// Ricci scalar
    pub ricci_scalar: FloatType,
    /// Geodesic deviation
    pub deviation: FloatType,
    /// Hamiltonian constraint
    pub h_constraint: FloatType,
}

/// Custom PropagatingEffect for SpaceTimeData
type SpaceTimeEffect = PropagatingEffect<SpaceTimeData>;

/// Accumulated results from pipeline stages (final output)
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
struct GRState {
    /// Mass parameter M (in geometric units, G=c=1)
    mass: FloatType,
    /// Schwarzschild radius r_s = 2M
    schwarzschild_radius: FloatType,
    /// Observation radius r
    observation_radius: FloatType,
    /// Kretschmann scalar K = R_μνρσ R^μνρσ
    kretschmann: FloatType,
    /// Ricci scalar R = g^μν R_μν
    ricci_scalar: FloatType,
    /// Geodesic deviation magnitude
    geodesic_deviation: FloatType,
    /// ADM Hamiltonian constraint H
    hamiltonian_constraint: FloatType,
    /// Is inside event horizon
    inside_horizon: bool,
    /// Time dilation factor √(1 - r_s/r)
    time_dilation: FloatType,
}

// =============================================================================
// STAGE 1: Create Schwarzschild Metric
// =============================================================================

/// Creates a Schwarzschild black hole spacetime.
///
/// # Physics
/// The Schwarzschild metric in spherical coordinates (t, r, θ, φ):
/// ```text
/// ds² = -(1 - r_s/r)dt² + (1 - r_s/r)^{-1}dr² + r²(dθ² + sin²θ dφ²)
/// ```
/// where r_s = 2GM/c² is the Schwarzschild radius.
fn initial_stage_create_schwarzschild() -> SpaceTimeEffect {
    println!("Stage 1: Create Schwarzschild Spacetime");
    println!("────────────────────────────────────────");

    // Black hole parameters
    let mass_solar: FloatType = float_from_f64!(10.0); // 10 solar masses
    let mass_kg: FloatType = mass_solar * float_from_f64!(1.989e30); // kg
    let r_s: FloatType = GR::schwarzschild_radius(mass_kg); // kg → geometric units

    // Observation point (outside horizon)
    let r = float_from_f64!(3.0) * r_s; // At 3 Schwarzschild radii

    println!("  Mass:                {} M☉", mass_solar);
    println!("  Schwarzschild radius: {} m", r_s);
    println!("  Observation radius:   {} m ({:.1} r_s)", r, (r / r_s));

    // Create manifold for the GR field
    let mut builder = SimplicialComplexBuilder::new(0);
    builder
        .add_simplex(Simplex::new(vec![0]))
        .expect("add simplex");
    let complex = builder.build().expect("build complex");

    let num_simplices = complex.total_simplices();
    let data: CausalTensor<FloatType> = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).expect("create manifold");

    // Construct Schwarzschild metric tensor at radius r
    // g_μν = diag(-(1-r_s/r), (1-r_s/r)^{-1}, r², r²sin²θ)
    let one = float_from_f64!(1.0);
    let f = one - r_s / r; // Metric function
    let mut metric_data: Vec<FloatType> = vec![float_from_f64!(0.0); 16];
    metric_data[0] = float_from_f64!(0.0) - f; // g_tt = -f
    metric_data[5] = one / f; // g_rr
    metric_data[10] = r * r; // g_θθ
    metric_data[15] = r * r; // g_φφ (assuming θ = π/2)

    let connection = CausalTensor::from_vec(metric_data, &[1, 4, 4]);

    // Precompute curvature in Lie-algebra form [points, 4, 4, 6]
    let mut fs_data: Vec<FloatType> = vec![float_from_f64!(0.0); 4 * 4 * 6];
    let riemann_scale = r_s / (r * r * r);
    fs_data[0] = riemann_scale;
    let field_strength = CausalTensor::from_vec(fs_data, &[1, 4, 4, 6]);

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();

    match GaugeField::new(base, topo_metric, connection, field_strength) {
        Ok(gr) => {
            println!("  Metric function:     f(r) = 1 - r_s/r = {:.6}", (f));
            println!("  Time dilation:       √f = {}", (f.sqrt()));
            println!();

            let data = SpaceTimeData {
                gr: Some(gr),
                r,
                r_s,
                ..Default::default()
            };
            CausalEffectPropagationProcess::pure(data)
        }
        Err(e) => {
            println!("  [ERROR] Failed to create GR field: {:?}", e);
            CausalEffectPropagationProcess::pure(SpaceTimeData::default())
        }
    }
}

// =============================================================================
// STAGE 2: Curvature Invariants
// =============================================================================

/// Computes curvature invariants of the spacetime.
///
/// # Physics
/// - Kretschmann scalar: K = R_μνρσ R^μνρσ = 48M²/r⁶ (for Schwarzschild)
/// - Ricci scalar: R = 0 (vacuum spacetime)
fn stage_curvature_invariants(mut input: SpaceTimeData, _: (), _: Option<()>) -> SpaceTimeEffect {
    println!("Stage 2: Curvature Invariants");
    println!("─────────────────────────────");

    if let Some(_gr) = &input.gr {
        let r = input.r;
        let r_s = input.r_s;
        // For Schwarzschild spacetime, use the exact analytic expressions:
        // Kretschmann scalar: K = R_μνρσ R^μνρσ = 48 M²/r⁶ = 12 r_s²/r⁶
        // Ricci scalar: R = 0 (vacuum solution)
        let m = r_s / float_from_f64!(2.0); // M = r_s/2 in geometric units
        let r6 = r * r * r * r * r * r;
        let kretschmann = float_from_f64!(48.0) * m * m / r6;
        let ricci_scalar: FloatType = float_from_f64!(0.0); // Vacuum solution: R = 0

        println!("  Kretschmann scalar: K = {} (analytic)", (kretschmann));
        println!("  Ricci scalar:       R = {} (vacuum)", (ricci_scalar));

        // Physical interpretation
        if (ricci_scalar).abs() < 1e-10 {
            println!("\n  → Vacuum spacetime (T_μν = 0)");
        }
        if (kretschmann) > 0.0 {
            println!("  → Non-flat curvature: spacetime is curved");
        }

        // Curvature radius from Kretschmann scalar
        let curvature_radius = float_from_f64!(1.0) / kretschmann.powf(float_from_f64!(0.25));
        println!(
            "  → Curvature radius: {} m (via K^{{-1/4}})",
            (curvature_radius)
        );
        println!();

        input.kretschmann = kretschmann;
        input.ricci_scalar = ricci_scalar;

        CausalEffectPropagationProcess::pure(input)
    } else {
        CausalEffectPropagationProcess::pure(input)
    }
}

// =============================================================================
// STAGE 3: Geodesic Analysis
// =============================================================================

/// Analyzes geodesic deviation and tidal forces.
///
/// # Physics
/// - Geodesic deviation: D²ξ^μ/Dτ² = R^μ_νρσ u^ν ξ^ρ u^σ
/// - Measures how nearby geodesics converge/diverge
fn stage_geodesic_analysis(mut input: SpaceTimeData, _: (), _: Option<()>) -> SpaceTimeEffect {
    println!("Stage 3: Geodesic Analysis");
    println!("──────────────────────────");

    if let Some(gr) = &input.gr {
        let r = input.r;
        let r_s = input.r_s;
        // Static observer 4-velocity: u^μ = (1/√f, 0, 0, 0)
        let one = float_from_f64!(1.0);
        let zero = float_from_f64!(0.0);
        let f = one - r_s / r;
        let u = CausalTensor::from_vec(vec![one / f.sqrt(), zero, zero, zero], &[4]);

        // Separation vector (radial): ξ^μ = (0, 1, 0, 0)
        let xi = CausalTensor::from_vec(vec![zero, one, zero, zero], &[4]);

        // Compute geodesic deviation in SI units. For geometric units, use geodesic_deviation()
        let tidal_acceleration: FloatType =
            match gr.geodesic_deviation_si(u.as_slice(), xi.as_slice()) {
                Ok(d) => {
                    // Magnitude of acceleration (already in m/s²)
                    d.iter().map(|x| (*x) * (*x)).sum::<FloatType>().sqrt()
                }
                Err(_) => {
                    // Analytic fallback: radial tidal acceleration ~ c² * M/r³
                    let m = (r_s) / 2.0;
                    let c = SPEED_OF_LIGHT;
                    c * c * 2.0 * m / (r * r * r)
                }
            };

        // Also show the geometric deviation for reference
        let deviation_geometric = tidal_acceleration / (SPEED_OF_LIGHT * SPEED_OF_LIGHT);
        println!(
            "  Geodesic deviation:      {} m⁻² (geometric)",
            deviation_geometric
        );
        println!(
            "  Tidal acceleration:      {} m/s² (SI)",
            tidal_acceleration
        );

        // Spaghettification distance (where tidal force ~ g)
        let g = 9.8; // Earth gravity
        if tidal_acceleration > g {
            println!(
                "  → Tidal force (at 1m) exceeds Earth gravity ({:.1} g)",
                tidal_acceleration / g
            );
        }

        input.deviation = deviation_geometric;
        let tau_ratio = f.sqrt();
        println!("\n  Proper time dilation: dτ/dt = {:.6}", (tau_ratio));
        println!(
            "  Gravitational redshift:  z = {:.6}",
            (one / tau_ratio - one)
        );
        println!();

        CausalEffectPropagationProcess::pure(input)
    } else {
        CausalEffectPropagationProcess::pure(input)
    }
}

// =============================================================================
// STAGE 4: ADM Formalism
// =============================================================================

/// Applies the ADM 3+1 decomposition.
///
/// # Physics
/// - Splits spacetime into spatial slices Σ_t
/// - Hamiltonian constraint: H = R + K² - K_ij K^ij = 16πρ
/// - For vacuum: H = 0
fn stage_adm_formalism(mut input: SpaceTimeData, _: (), _: Option<()>) -> SpaceTimeEffect {
    println!("Stage 4: ADM 3+1 Formalism");
    println!("──────────────────────────");

    let r = input.r;
    let r_s = input.r_s;
    let one = float_from_f64!(1.0);
    let zero = float_from_f64!(0.0);
    let f = one - r_s / r;

    // Spatial 3-metric
    let gamma = CausalTensor::from_vec(
        vec![one / f, zero, zero, zero, r * r, zero, zero, zero, r * r],
        &[3, 3],
    );

    // Extrinsic curvature K_ij = 0 for static slice
    let k = CausalTensor::zeros(&[3, 3]);

    // Lapse and shift
    let alpha = CausalTensor::from_vec(vec![f.sqrt()], &[1]);
    let beta = CausalTensor::zeros(&[3]);

    // ADM state with zero spatial Ricci scalar (vacuum)
    let adm_state = AdmState::new(gamma, k, alpha.clone(), beta, float_from_f64!(0.0));

    // Compute Hamiltonian constraint
    let h_constraint = match adm_state.hamiltonian_constraint(None) {
        Ok(h) => h.as_slice().first().copied().unwrap_or(zero),
        Err(_) => float_from_f64!(0.0),
    };

    println!("  Lapse function α:        {}", (alpha.as_slice()[0]));
    println!("  Shift vector β:          (0, 0, 0)");
    println!("  Extrinsic curvature K:   0 (static slice)");
    println!("  Hamiltonian constraint:  H = {}", h_constraint);

    if h_constraint.abs() < 1e-10 {
        println!("\n  → Constraint satisfied (vacuum solution)");
    }

    // Compute mean curvature
    let mean_curv = match adm_state.mean_curvature() {
        Ok(k) => k.as_slice().first().copied().unwrap_or(zero),
        Err(_) => float_from_f64!(0.0),
    };
    println!("  Mean curvature K:        {}", mean_curv);
    println!();

    input.h_constraint = h_constraint;

    CausalEffectPropagationProcess::pure(input)
}

// =============================================================================
// STAGE 5: Horizon Detection
// =============================================================================

/// Detects event horizons and causal structure.
///
/// # Physics
/// - Event horizon at r = r_s (g_tt = 0)
/// - Photon sphere at r = 3M = 1.5 r_s
/// - ISCO at r = 6M = 3 r_s
fn stage_event_horizon_detection(
    input: SpaceTimeData,
    _: (),
    _: Option<()>,
) -> PropagatingEffect<GRState> {
    println!("Stage 5: Horizon Detection");
    println!("──────────────────────────");

    let r = input.r;
    let r_s = input.r_s;
    let inside_horizon = r < r_s;
    let in_photon_sphere = r < float_from_f64!(1.5) * r_s;
    let in_isco = r < float_from_f64!(3.0) * r_s;

    println!("  Event horizon (r = r_s):     {} m", (r_s));
    println!(
        "  Photon sphere (r = 1.5 r_s): {} m",
        (float_from_f64!(1.5) * r_s)
    );
    println!(
        "  ISCO (r = 3 r_s):            {} m",
        (float_from_f64!(3.0) * r_s)
    );
    println!();
    println!("  Current radius:              {:.6e} m", (r));
    println!("  Inside event horizon:        {}", inside_horizon);
    println!("  Inside photon sphere:        {}", in_photon_sphere);
    println!("  Inside ISCO:                 {}", in_isco);

    // Escape velocity
    if !inside_horizon {
        let v_escape = (r_s / r).sqrt();
        println!("\n  Escape velocity:             {} c", (v_escape));
    } else {
        println!("\n  → No escape possible (inside horizon)");
    }

    // Time dilation factor
    let one = float_from_f64!(1.0);
    let time_dilation: FloatType = if r > r_s {
        (one - r_s / r).sqrt()
    } else {
        float_from_f64!(0.0)
    };
    println!("  Time dilation factor:        {}", (time_dilation));
    println!();

    // Drop gr_opt to avoid unused warning
    let _ = input.gr;

    let state = GRState {
        mass: r_s / float_from_f64!(2.0),
        schwarzschild_radius: r_s,
        observation_radius: r,
        kretschmann: input.kretschmann,
        ricci_scalar: input.ricci_scalar,
        geodesic_deviation: input.deviation,
        hamiltonian_constraint: input.h_constraint,
        inside_horizon,
        time_dilation,
    };

    CausalEffectPropagationProcess::pure(state)
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Prints the final pipeline summary.
fn print_summary(result: &PropagatingEffect<GRState>) {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Pipeline Summary");
    println!("═══════════════════════════════════════════════════════════════");

    match result.value() {
        EffectValue::Value(state) => {
            println!("\n  ┌─────────────────────────────────────────────────────────┐");
            println!("  │  Schwarzschild Black Hole Parameters                    │");
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!(
                "  │  Mass (geometric):        {:>12.6e} m                │",
                (state.mass)
            );
            println!(
                "  │  Schwarzschild radius:    {:>12.6e} m                │",
                (state.schwarzschild_radius)
            );
            println!(
                "  │  Observation radius:      {:>12.6e} m                │",
                (state.observation_radius)
            );
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!("  │  Curvature Invariants                                   │");
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!(
                "  │  Kretschmann scalar:      {:>12.6e}                  │",
                (state.kretschmann)
            );
            println!(
                "  │  Ricci scalar:            {:>12.6e}                  │",
                (state.ricci_scalar)
            );
            println!(
                "  │  Geodesic deviation:      {:>12.6e}                  │",
                (state.geodesic_deviation)
            );
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!("  │  ADM Constraint                                         │");
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!(
                "  │  Hamiltonian constraint:  {:>12.6e}                  │",
                (state.hamiltonian_constraint)
            );
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!("  │  Causal Structure                                       │");
            println!("  ├─────────────────────────────────────────────────────────┤");
            println!(
                "  │  Inside event horizon:        {}                       │",
                if state.inside_horizon { "Yes" } else { "No " }
            );
            println!(
                "  │  Time dilation factor:    {:>12.6}                  │",
                (state.time_dilation)
            );
            println!("  └─────────────────────────────────────────────────────────┘");
            println!("\n[SUCCESS] GR Pipeline Completed.\n");
        }
        _ => {
            println!("  Pipeline returned unexpected result");
            println!("\n[WARN] Check individual stage outputs.\n");
        }
    }
}
