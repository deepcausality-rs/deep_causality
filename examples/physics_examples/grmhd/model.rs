/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The GRMHD stages, each a pure `GrmhdState -> PropagatingEffect<GrmhdState>`.
//!
//! Geometric units throughout: `G = c = 1`, so a mass is a length and `M = r_s / 2`. Curvature
//! then carries units of `1 / length^2` and the Kretschmann scalar `1 / length^4`.
//!
//! Multivector blade indices are bitmasks over the basis vectors, so the grade of a blade is the
//! population count of its index. `J` on `e_1` and `B` on `e_2` put `J ^ B` on `e_1 ^ e_2`.

use crate::FloatType;
use deep_causality::{CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_algebra::Real;
use deep_causality_multivector::{CausalMultiVector, Metric};

use deep_causality_num::lift;
use deep_causality_physics::{
    energy_momentum_tensor_em, generate_schwarzschild_metric, lorentz_force,
};
use deep_causality_tensor::CausalTensor;

/// Spacetime is four-dimensional, so `g_uv` and `T^uv` are 4x4.
const DIM: usize = 4;
/// Blade indices in the algebra the plasma runs in: the current on `e_1`, the field on `e_2`.
const E_1: usize = 1 << 1;
const E_2: usize = 1 << 2;
/// `J ^ B` lands on the blade spanning both.
const E_12: usize = E_1 | E_2;

/// Configuration for the GRMHD run.
#[derive(Clone, Debug, Default)]
pub struct SimulationConfig {
    /// Schwarzschild radius of the central body, in metres.
    pub schwarzschild_radius: FloatType,
    /// Orbital radius of the plasma, in metres.
    pub radius: FloatType,
    /// Radial extent of the plasma column, in metres. Tidal stretch is measured across it.
    pub column_length: FloatType,
    /// Plasma current density.
    pub current_density: FloatType,
    /// Confining magnetic field.
    pub magnetic_field: FloatType,
    /// Tidal acceleration above which the plasma is treated relativistically, in m/s^2.
    pub tidal_threshold: FloatType,
}

/// State threaded through the causal chain.
#[derive(Clone, Debug, Default)]
pub struct GrmhdState {
    pub config: SimulationConfig,
    // --- GR results ---
    /// `M = r_s / 2` in geometric units.
    pub mass_geometric: FloatType,
    /// `K = R_abcd R^abcd = 48 M^2 / r^6`, the curvature invariant that survives in vacuum.
    pub kretschmann: FloatType,
    /// `R = 0` for a vacuum solution; carried so the run can show it is zero.
    pub ricci_scalar: FloatType,
    /// Tidal acceleration across the plasma column, `2 M L / r^3` in geometric units.
    pub tidal_acceleration: FloatType,
    pub metric_tensor: Option<CausalTensor<FloatType>>,
    // --- Coupling results ---
    pub metric: Option<Metric>,
    pub metric_label: &'static str,
    // --- MHD results ---
    pub lorentz_force: FloatType,
    pub em_energy_density: FloatType,
    // --- Analysis ---
    pub status: &'static str,
}

impl GrmhdState {
    pub fn new(config: &SimulationConfig) -> Self {
        Self {
            config: config.clone(),
            ..Default::default()
        }
    }
}

/// Stage 1: the GR solver.
///
/// Builds the Schwarzschild metric at the plasma's radius and reads real curvature off it. The
/// Ricci scalar vanishes because the exterior is vacuum, which is exactly why the *Kretschmann*
/// scalar is the invariant to use: it is built from the full Riemann tensor and stays non-zero
/// where Ricci does not. The tidal acceleration across the column is the same curvature in the
/// units an engineer cares about.
pub fn calculate_curvature(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let r = state.config.radius;
    let r_s = state.config.schwarzschild_radius;
    let two = lift::<FloatType>(2.0);

    if r <= r_s {
        return fail("the plasma radius is inside the horizon");
    }

    let mass_geometric = r_s / two;
    let r6 = r * r * r * r * r * r;
    let kretschmann = lift::<FloatType>(48.0) * mass_geometric * mass_geometric / r6;
    // Vacuum exterior: the Einstein equations give R_uv = 0, hence R = 0.
    let ricci_scalar = lift::<FloatType>(0.0);
    // Radial tidal acceleration over a proper length L, to leading order: a = 2 M L / r^3.
    let tidal_acceleration = two * mass_geometric * state.config.column_length / (r * r * r);

    // The Schwarzschild metric at this radius, in the (-,+,+,+) signature.
    let one = lift::<FloatType>(1.0);
    let lapse = one - r_s / r;
    let metric_tensor = match generate_schwarzschild_metric(-lapse, one / lapse, one, one) {
        Ok(g) => g,
        Err(e) => return fail(format!("schwarzschild metric: {e:?}")),
    };

    PropagatingEffect::pure(GrmhdState {
        mass_geometric,
        kretschmann,
        ricci_scalar,
        tidal_acceleration,
        metric_tensor: Some(metric_tensor),
        ..state
    })
}

/// Stage 2: the coupling.
///
/// A computed curvature quantity, not a flag, decides which algebra the plasma runs in. Above
/// the tidal threshold the flow is relativistic and needs the full `Cl(1,3)` spacetime algebra;
/// below it, the three-dimensional Euclidean algebra is enough.
pub fn select_metric(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let (metric, metric_label) = if state.tidal_acceleration > state.config.tidal_threshold {
        (Metric::Minkowski(4), "Relativistic (Minkowski 4D)")
    } else {
        (Metric::Euclidean(3), "Classical (Euclidean 3D)")
    };

    PropagatingEffect::pure(GrmhdState {
        metric: Some(metric),
        metric_label,
        ..state
    })
}

/// Stage 3: the MHD solver, in whatever algebra stage 2 selected.
///
/// `F = J ^ B`: the current on `e_1`, the confining field on `e_2`, and the force density on the
/// bivector they span.
pub fn calculate_lorentz_force(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let metric = match state.metric {
        Some(m) => m,
        None => return fail("no metric was selected"),
    };
    let blades = 1 << metric.dimension();

    let j_vec = match blade(blades, E_1, state.config.current_density, metric) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    let b_vec = match blade(blades, E_2, state.config.magnetic_field, metric) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };

    let f_field = match lorentz_force(&j_vec, &b_vec).value_cloned() {
        Some(f) => f,
        None => return fail("lorentz force produced no value"),
    };
    let lorentz_force = f_field
        .0
        .data()
        .get(E_12)
        .copied()
        .unwrap_or_else(|| lift::<FloatType>(0.0));

    PropagatingEffect::pure(GrmhdState {
        lorentz_force,
        ..state
    })
}

/// Stage 4: the EM stress-energy the plasma feeds back into the metric.
///
/// With the field along `z`, the only non-zero components of `F^uv` are `F^12 = -F^21 = B`, and
/// `T^00` is the energy density that would source the Einstein equations on the next cycle.
pub fn calculate_energy_momentum(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let g_uv = match &state.metric_tensor {
        Some(t) => t,
        None => return fail("no metric tensor was built"),
    };

    let b = state.config.magnetic_field;
    let mut f_data = vec![lift::<FloatType>(0.0); DIM * DIM];
    f_data[DIM + 2] = b; // F^12
    f_data[2 * DIM + 1] = -b; // F^21
    let f_tensor = match CausalTensor::new(f_data, vec![DIM, DIM]) {
        Ok(t) => t,
        Err(e) => return fail(format!("EM tensor: {e:?}")),
    };

    let t_tensor = match energy_momentum_tensor_em(&f_tensor, g_uv).value_cloned() {
        Some(t) => t,
        None => return fail("energy momentum tensor produced no value"),
    };
    let em_energy_density = t_tensor
        .as_slice()
        .first()
        .copied()
        .unwrap_or_else(|| lift::<FloatType>(0.0));

    PropagatingEffect::pure(GrmhdState {
        em_energy_density,
        ..state
    })
}

/// Stage 5: what the confinement looks like once both halves have run.
pub fn analyze_stability(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let zero = lift::<FloatType>(0.0);
    let status = if state.lorentz_force < zero {
        "Reversed confinement - containment field needs adjustment"
    } else if state.em_energy_density > state.kretschmann {
        "Magnetically dominated - the field, not the tide, sets the scale"
    } else {
        "Tidally dominated - curvature exceeds magnetic confinement"
    };

    PropagatingEffect::pure(GrmhdState { status, ..state })
}

/// A vector with one blade set, in the given algebra.
fn blade(
    blades: usize,
    index: usize,
    value: FloatType,
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, String> {
    let mut data = vec![lift::<FloatType>(0.0); blades];
    match data.get_mut(index) {
        Some(slot) => *slot = value,
        None => {
            return Err(format!(
                "blade {index} is outside a {blades}-coefficient algebra"
            ));
        }
    }
    CausalMultiVector::new(data, metric)
        .map_err(|e| format!("building a multivector failed: {e:?}"))
}

/// The curvature radius `K^(-1/4)`, the length over which spacetime bends appreciably.
pub fn curvature_radius(kretschmann: FloatType) -> FloatType {
    let quarter = lift::<FloatType>(0.25);
    lift::<FloatType>(1.0) / Real::powf(kretschmann, quarter)
}

fn fail(reason: impl Into<String>) -> PropagatingEffect<GrmhdState> {
    PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(reason.into())))
}
