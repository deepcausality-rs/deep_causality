/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # General Relativity Gauge Field Pipeline
//!
//! Demonstrates **modular causal composition** via `CausalEffectPropagationProcess`
//! for General Relativity (GR) spacetime analysis.
//!
//! The central body's mass and the observer's radius are the pipeline's context: a `Context` of
//! `Data` contextoids that every stage reads, deriving `r_s` and `r` from it.
//!
//! ## Generic Float Type Support
//!
//! This example supports `f32`, `f64`, and `DoubleFloat` by changing the `FloatType`
//! type alias. All numeric literals are converted using the `flt!` macro.
//!
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{
    CausalEffectPropagationProcess, CausalFlow, CausalityError, CausalityErrorEnum,
    PropagatingEffect, PropagatingProcess,
};
use deep_causality_num::{Float, Float106, const_scalar_from_float, const_scalar_from_int};
use deep_causality_num_dual::Dual;
use deep_causality_physics::{AdmOps, GrOps, LorentzianMetric};
use deep_causality_physics::{AdmState, EastCoastMetric, GR, SPEED_OF_LIGHT};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{GaugeField, Manifold, Simplex, SimplicialComplexBuilder};
use std::error::Error;
// =============================================================================
// FLOAT TYPE CONFIGURATION
// =============================================================================

// Change this to f32 , f64, or Float106 to use different precision
type FloatType = Float106;

/// Small whole numbers and fractions, declared once at the working type.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
const QUARTER: FloatType = const_scalar_from_float!(FloatType, 0.25);
/// One solar mass in kilograms.
const SOLAR_MASS_KG: FloatType = const_scalar_from_float!(FloatType, 1.989e30);
/// `K = 48 M^2 / r^6` for Schwarzschild.
const KRETSCHMANN_COEFFICIENT: FloatType = const_scalar_from_int!(FloatType, 48);
/// The photon sphere sits at `1.5 r_s` and the ISCO at `3 r_s`.
const PHOTON_SPHERE_RADII: FloatType = const_scalar_from_float!(FloatType, 1.5);
const ISCO_RADII: FloatType = const_scalar_from_int!(FloatType, 3);
/// The speed of light, at the working type.
const LIGHT_SPEED: FloatType = const_scalar_from_float!(FloatType, SPEED_OF_LIGHT);
type GRTheory = GR<FloatType>;

/// The central body and the observer as a context: one `Data` contextoid per world fact. The
/// stages work in Schwarzschild coordinates, which no context space or spacetime type models, so
/// the spatial, temporal and spacetime slots are empty.
type SchwarzschildContext =
    Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: mass of the central body, in solar masses.
const CENTRAL_MASS: ContextoidId = 1;
/// Contextoid id: observation radius, in Schwarzschild radii.
const OBSERVATION_RADIUS: ContextoidId = 2;

// =============================================================================
// MAIN: Pipeline Composition via Causal Monad
// =============================================================================

fn main() -> Result<(), Box<dyn Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  General Relativity Spacetime Analysis");
    println!("  (Float Type: {})", std::any::type_name::<FloatType>());
    println!("═══════════════════════════════════════════════════════════════\n");

    // The central body and the observer: the context every stage reads.
    let spacetime = schwarzschild_spacetime()?;

    // Composed pipeline using bind_or_error
    let result = CausalFlow::from(initial_stage_create_schwarzschild(&spacetime))
        .context(spacetime)
        .bind_or_error(stage_curvature_invariants, "Curvature computation failed")
        .bind_or_error(stage_geodesic_analysis, "Geodesic analysis failed")
        .bind_or_error(stage_adm_formalism, "ADM formalism failed")
        .bind_or_error(stage_event_horizon_detection, "Horizon detection failed")
        .into_process();

    // Extract and display final result; a stage that failed is the error the process exits with.
    print_summary(&result);
    match result.error() {
        Some(error) => Err(error.clone().into()),
        None => Ok(()),
    }
}

// =============================================================================
// GR State: Passed through pipeline stages
// =============================================================================

/// Accumulated results from pipeline stages
#[derive(Debug, Clone, Default)]
pub struct SpaceTimeData {
    /// General Relativity gauge field
    pub gr: Option<GRTheory>,
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

/// What a stage hands on: its value, with the spacetime context for the next stage.
type SpaceTimeProcess<T> = PropagatingProcess<T, (), SchwarzschildContext>;

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
fn initial_stage_create_schwarzschild(spacetime: &SchwarzschildContext) -> SpaceTimeEffect {
    println!("Stage 1: Create Schwarzschild Spacetime");
    println!("────────────────────────────────────────");

    // Black hole parameters and the observation point (outside horizon), from the context
    let mass_solar: FloatType = match read(spacetime, CENTRAL_MASS) {
        Ok(mass) => mass,
        Err(e) => return CausalEffectPropagationProcess::from_error(e),
    };
    let (r, r_s) = match radii(spacetime) {
        Ok(radii) => radii,
        Err(e) => return CausalEffectPropagationProcess::from_error(e),
    };

    println!("  Mass:                {} M☉", mass_solar);
    println!("  Schwarzschild radius: {} m", r_s);
    println!("  Observation radius:   {} m ({:.1} r_s)", r, (r / r_s));

    // Create manifold for the GR field
    let mut builder = SimplicialComplexBuilder::new(0);
    if let Err(e) = builder.add_simplex(Simplex::new(vec![0])) {
        return CausalEffectPropagationProcess::from_error(custom(format!("add simplex: {e:?}")));
    }
    let complex = match builder.build() {
        Ok(complex) => complex,
        Err(e) => {
            return CausalEffectPropagationProcess::from_error(custom(format!(
                "build complex: {e:?}"
            )));
        }
    };

    let num_simplices = complex.total_simplices();
    let data: CausalTensor<FloatType> = CausalTensor::zeros(&[num_simplices]);
    let base = match Manifold::new(complex, data, 0) {
        Ok(base) => base,
        Err(e) => {
            return CausalEffectPropagationProcess::from_error(custom(format!(
                "create manifold: {e:?}"
            )));
        }
    };

    // Construct Schwarzschild metric tensor at radius r
    // g_μν = diag(-(1-r_s/r), (1-r_s/r)^{-1}, r², r²sin²θ)
    let one = ONE;
    let f = one - r_s / r; // Metric function (the lapse) f(r) = 1 − r_s/r

    // The lapse f(r) = 1 − r_s/r; its radial derivative f'(r) = r_s/r² is the gravitational
    // redshift gradient (and fixes the surface gravity). Read it straight off the tangent functor:
    // seed r as a Dual variable and the ε-channel returns f'(r), exact (no finite differences) and
    // at the working Float106 precision, since r_s never leaves the scalar.
    let f_prime = (Dual::<FloatType>::constant(one)
        - Dual::<FloatType>::constant(r_s) / Dual::<FloatType>::variable(r))
    .derivative();
    println!("  Metric function:     f(r) = 1 - r_s/r = {:.6}", f);
    println!("  Time dilation:       √f = {}", f.sqrt());
    println!(
        "  Lapse gradient:      f'(r) = r_s/r²  (AD)       {}",
        f_prime
    );
    println!(
        "                                       (analytic) {}",
        r_s / (r * r)
    );
    println!();

    // Connection 1-form valued in the so(3,1) Lie algebra: shape [points=1, spacetime=4, lie=6].
    let connection = CausalTensor::from_vec(vec![ZERO; 4 * 6], &[1, 4, 6]);

    // Precompute curvature in Lie-algebra form [points, 4, 4, 6]
    let mut fs_data: Vec<FloatType> = vec![ZERO; 4 * 4 * 6];
    let riemann_scale = r_s / (r * r * r);
    fs_data[0] = riemann_scale;
    let field_strength = CausalTensor::from_vec(fs_data, &[1, 4, 4, 6]);

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();

    match GaugeField::new(base, topo_metric, connection, field_strength) {
        Ok(gr) => {
            let data = SpaceTimeData {
                gr: Some(gr),
                ..Default::default()
            };
            CausalEffectPropagationProcess::pure(data)
        }
        Err(e) => CausalEffectPropagationProcess::from_error(custom(format!(
            "failed to create the GR field: {e:?}"
        ))),
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
fn stage_curvature_invariants(
    mut input: SpaceTimeData,
    _: (),
    spacetime: Option<SchwarzschildContext>,
) -> SpaceTimeProcess<SpaceTimeData> {
    let (spacetime, r, r_s) = match stage_radii(spacetime) {
        Ok(stage) => stage,
        Err(e) => return PropagatingProcess::from_error(e),
    };
    println!("Stage 2: Curvature Invariants");
    println!("─────────────────────────────");

    if let Some(_gr) = &input.gr {
        // For Schwarzschild spacetime, use the exact analytic expressions:
        // Kretschmann scalar: K = R_μνρσ R^μνρσ = 48 M²/r⁶ = 12 r_s²/r⁶
        // Ricci scalar: R = 0 (vacuum solution)
        let m = r_s / TWO;
        let r6 = r * r * r * r * r * r;
        let kretschmann = KRETSCHMANN_COEFFICIENT * m * m / r6;
        let ricci_scalar: FloatType = ZERO; // Vacuum solution: R = 0

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
        let curvature_radius = ONE / kretschmann.powf(QUARTER);
        println!(
            "  → Curvature radius: {} m (via K^{{-1/4}})",
            (curvature_radius)
        );
        println!();

        input.kretschmann = kretschmann;
        input.ricci_scalar = ricci_scalar;

        proceed(input, spacetime)
    } else {
        proceed(input, spacetime)
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
fn stage_geodesic_analysis(
    mut input: SpaceTimeData,
    _: (),
    spacetime: Option<SchwarzschildContext>,
) -> SpaceTimeProcess<SpaceTimeData> {
    let (spacetime, r, r_s) = match stage_radii(spacetime) {
        Ok(stage) => stage,
        Err(e) => return PropagatingProcess::from_error(e),
    };
    println!("Stage 3: Geodesic Analysis");
    println!("──────────────────────────");

    if let Some(gr) = &input.gr {
        // Static observer 4-velocity: u^μ = (1/√f, 0, 0, 0)
        let one = ONE;
        let zero = ZERO;
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
                Err(e) => return PropagatingProcess::from_error(e.into()),
            };

        // Also show the geometric deviation for reference
        let c = LIGHT_SPEED;
        let deviation_geometric = tidal_acceleration / (c * c);
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

        proceed(input, spacetime)
    } else {
        proceed(input, spacetime)
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
fn stage_adm_formalism(
    mut input: SpaceTimeData,
    _: (),
    spacetime: Option<SchwarzschildContext>,
) -> SpaceTimeProcess<SpaceTimeData> {
    let (spacetime, r, r_s) = match stage_radii(spacetime) {
        Ok(stage) => stage,
        Err(e) => return PropagatingProcess::from_error(e),
    };
    println!("Stage 4: ADM 3+1 Formalism");
    println!("──────────────────────────");

    let one = ONE;
    let zero = ZERO;
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
    let adm_state = AdmState::new(gamma, k, alpha.clone(), beta, ZERO);

    // Compute Hamiltonian constraint
    let h_constraint = match adm_state.hamiltonian_constraint(None) {
        Ok(h) => match h.as_slice().first() {
            Some(&h) => h,
            None => {
                return PropagatingProcess::from_error(custom(
                    "the Hamiltonian constraint is empty",
                ));
            }
        },
        Err(e) => return PropagatingProcess::from_error(e.into()),
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
        Ok(k) => match k.as_slice().first() {
            Some(&k) => k,
            None => return PropagatingProcess::from_error(custom("the mean curvature is empty")),
        },
        Err(e) => return PropagatingProcess::from_error(e.into()),
    };
    println!("  Mean curvature K:        {}", mean_curv);
    println!();

    input.h_constraint = h_constraint;

    proceed(input, spacetime)
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
    spacetime: Option<SchwarzschildContext>,
) -> SpaceTimeProcess<GRState> {
    let (spacetime, r, r_s) = match stage_radii(spacetime) {
        Ok(stage) => stage,
        Err(e) => return PropagatingProcess::from_error(e),
    };
    println!("Stage 5: Horizon Detection");
    println!("──────────────────────────");

    let inside_horizon = r < r_s;
    let in_photon_sphere = r < PHOTON_SPHERE_RADII * r_s;
    let in_isco = r < ISCO_RADII * r_s;

    println!("  Event horizon (r = r_s):     {} m", (r_s));
    println!(
        "  Photon sphere (r = 1.5 r_s): {} m",
        (PHOTON_SPHERE_RADII * r_s)
    );
    println!("  ISCO (r = 3 r_s):            {} m", (ISCO_RADII * r_s));
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
    let one = ONE;
    let time_dilation: FloatType = if r > r_s {
        (one - r_s / r).sqrt()
    } else {
        ZERO
    };
    println!("  Time dilation factor:        {}", (time_dilation));
    println!();

    // Drop gr_opt to avoid unused warning
    let _ = input.gr;

    let state = GRState {
        mass: r_s / TWO,
        schwarzschild_radius: r_s,
        observation_radius: r,
        kretschmann: input.kretschmann,
        ricci_scalar: input.ricci_scalar,
        geodesic_deviation: input.deviation,
        hamiltonian_constraint: input.h_constraint,
        inside_horizon,
        time_dilation,
    };

    proceed(state, spacetime)
}

// =============================================================================
// CONTEXT: the central body and the observer
// =============================================================================

/// Builds the spacetime's world facts: each as a `Data` contextoid keyed by its contextoid id.
fn schwarzschild_spacetime() -> Result<SchwarzschildContext, ContextIndexError> {
    let facts = [
        (CENTRAL_MASS, const_scalar_from_int!(FloatType, 10)),
        (OBSERVATION_RADIUS, const_scalar_from_int!(FloatType, 3)),
    ];

    let mut spacetime = Context::with_capacity(1, "Schwarzschild black hole", facts.len());
    for (id, value) in facts {
        spacetime.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(spacetime)
}

/// Reads one world fact out of the spacetime, or the error naming the contextoid id it lacks.
fn read(spacetime: &SchwarzschildContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    spacetime.get_data_by_id(id).ok_or_else(|| {
        custom(format!(
            "the spacetime holds no Datoid with contextoid id {id}"
        ))
    })
}

/// The observation radius `r` and the Schwarzschild radius `r_s`, in metres, of the central
/// body and observer the context holds.
fn radii(spacetime: &SchwarzschildContext) -> Result<(FloatType, FloatType), CausalityError> {
    let mass_kg: FloatType = read(spacetime, CENTRAL_MASS)? * SOLAR_MASS_KG;
    let r_s: FloatType = GR::schwarzschild_radius(mass_kg); // kg → geometric units
    Ok((read(spacetime, OBSERVATION_RADIUS)? * r_s, r_s))
}

/// Hands `value` and the spacetime on to the next stage.
fn proceed<T>(value: T, spacetime: SchwarzschildContext) -> SpaceTimeProcess<T>
where
    T: Default + Clone + core::fmt::Debug,
{
    PropagatingProcess::with_state(PropagatingEffect::pure(value), (), Some(spacetime))
}

/// The spacetime the flow carries, with its observation radius `r` and Schwarzschild radius
/// `r_s`, or the error a stage short-circuits with when the flow carries no spacetime.
fn stage_radii(
    spacetime: Option<SchwarzschildContext>,
) -> Result<(SchwarzschildContext, FloatType, FloatType), CausalityError> {
    let spacetime = spacetime.ok_or_else(|| custom("the flow carries no spacetime"))?;
    let (r, r_s) = radii(&spacetime)?;
    Ok((spacetime, r, r_s))
}

/// A stage failure, as the pipeline's error.
fn custom(reason: impl Into<String>) -> CausalityError {
    CausalityError::new(CausalityErrorEnum::Custom(reason.into()))
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Prints the final pipeline summary.
fn print_summary(result: &SpaceTimeProcess<GRState>) {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Pipeline Summary");
    println!("═══════════════════════════════════════════════════════════════");

    match result.value() {
        Some(state) => {
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
