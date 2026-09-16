/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The GRMHD stages, each a pure `GrmhdState -> PropagatingEffect<GrmhdState>`.
//!
//! Geometric units throughout: `G = c = 1`, so a mass is a length and `M = r_s / 2`. Curvature
//! then carries units of `1 / length^2`, the Kretschmann scalar `1 / length^4`, an acceleration
//! `1 / length`, and an energy density `1 / length^2`. Every comparison the model makes is made
//! in these units. The one conversion out of them multiplies the tidal acceleration by `c^2`
//! for display in `m/s^2`, and lives in [`tidal_acceleration_si`].
//!
//! Multivector blade indices are bitmasks over the basis vectors, so the grade of a blade is the
//! population count of its index. `J` on `e_1` and `B` on `e_2` put `J ^ B` on `e_1 ^ e_2`.

use crate::FloatType;
use deep_causality::{CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_algebra::Real;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift_usize};
use deep_causality_physics::{
    SPEED_OF_LIGHT, energy_momentum_tensor_em, generate_schwarzschild_metric, lorentz_force,
};
use deep_causality_tensor::CausalTensor;

/// Spacetime is four-dimensional, so `g_uv` and `T^uv` are 4x4.
const DIM: usize = 4;
/// Coordinate indices of Schwarzschild `(t, r, theta, phi)`.
const RADIAL: usize = 1;
const POLAR: usize = 2;
/// Blade indices in the algebra the plasma runs in: the current on `e_1`, the field on `e_2`.
const E_1: usize = 1 << 1;
const E_2: usize = 1 << 2;
/// `J ^ B` lands on the blade spanning both.
const E_12: usize = E_1 | E_2;

/// Small whole numbers and the Kretschmann coefficient, at the working type.
pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
pub const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
/// `K = 48 M^2 / r^6` for Schwarzschild.
const KRETSCHMANN_COEFFICIENT: FloatType = const_scalar_from_int!(FloatType, 48);
/// The exponent in the curvature radius `K^(-1/4)`.
const QUARTER: FloatType = const_scalar_from_float!(FloatType, 0.25);
/// The speed of light in `m/s`, for the one display conversion out of geometric units.
const LIGHT_SPEED: FloatType = const_scalar_from_float!(FloatType, SPEED_OF_LIGHT);

/// `8 pi`, the coupling in `G_uv = 8 pi T_uv` with `G = c = 1`.
fn eight_pi() -> FloatType {
    lift_usize::<FloatType>(8) * FloatType::pi()
}

/// Configuration for the GRMHD run.
#[derive(Clone, Debug, Default)]
pub struct SimulationConfig {
    /// Schwarzschild radius of the central body, in metres.
    pub schwarzschild_radius: FloatType,
    /// Orbital radius of the plasma, in metres.
    pub radius: FloatType,
    /// Radial extent of the plasma column, in metres. Tidal stretch is measured across it.
    pub column_length: FloatType,
    /// Plasma current density, in geometric units.
    pub current_density: FloatType,
    /// Confining magnetic field as the static observer measures it, in geometric units (`1/m`).
    pub magnetic_field: FloatType,
    /// Tidal acceleration above which the plasma is treated relativistically, in `1/m`.
    pub tidal_threshold: FloatType,
}

/// State threaded through the causal chain.
#[derive(Clone, Debug, Default)]
pub struct GrmhdState {
    pub config: SimulationConfig,
    // --- GR results ---
    /// `M = r_s / 2` in geometric units.
    pub mass_geometric: FloatType,
    /// `1 - r_s / r`: `-g_tt`, and the square of the static observer's time leg.
    pub lapse: FloatType,
    /// `K = R_abcd R^abcd = 48 M^2 / r^6`, the curvature invariant that survives in vacuum.
    pub kretschmann: FloatType,
    /// `R = 0` for a vacuum solution; carried so the run can show it is zero.
    pub ricci_scalar: FloatType,
    /// Tidal acceleration across the plasma column, `2 M L / r^3`, in `1/m`.
    pub tidal_acceleration: FloatType,
    /// The Schwarzschild metric at the plasma, in the equatorial plane.
    pub metric_tensor: Option<CausalTensor<FloatType>>,
    // --- Coupling results ---
    pub metric: Option<Metric>,
    pub metric_label: &'static str,
    // --- MHD results ---
    pub lorentz_force: FloatType,
    /// `F^{r theta}` in Schwarzschild coordinates, built from the field the observer measures.
    pub em_tensor_component: FloatType,
    /// `T^{tt}` in Schwarzschild coordinates, as the stress-energy kernel returns it.
    pub em_energy_coordinate: FloatType,
    /// The energy density the static observer measures, `T^{tt}` projected onto the frame.
    pub em_energy_density: FloatType,
    // --- Analysis ---
    /// `8 pi rho_EM`: the curvature the plasma's own stress-energy sources, in `1/m^2`.
    pub plasma_curvature: FloatType,
    /// `sqrt(K)`: the curvature the hole imposes at the plasma, in `1/m^2`.
    pub tidal_curvature: FloatType,
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
/// Builds the Schwarzschild metric at the plasma's radius and reads curvature off the solution.
/// The Ricci scalar vanishes because the exterior is vacuum, which is why the *Kretschmann*
/// scalar is the invariant to use: it is built from the full Riemann tensor and stays non-zero
/// where Ricci does not. The tidal acceleration across the column is the same curvature as an
/// engineer meets it.
///
/// The metric is the coordinate metric in `(t, r, theta, phi)` with the plasma in the equatorial
/// plane, `theta = pi / 2`, so `g_{theta theta} = r^2` and `g_{phi phi} = r^2 sin^2 theta = r^2`.
pub fn calculate_curvature(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let r = state.config.radius;
    let r_s = state.config.schwarzschild_radius;

    if r <= r_s {
        return fail("the plasma radius is inside the horizon");
    }

    let mass_geometric = r_s / TWO;
    let r6 = r * r * r * r * r * r;
    let kretschmann = KRETSCHMANN_COEFFICIENT * mass_geometric * mass_geometric / r6;
    // Vacuum exterior: the Einstein equations give R_uv = 0, hence R = 0.
    let ricci_scalar = ZERO;
    // Radial tidal acceleration over a proper length L, to leading order: a = 2 M L / r^3.
    let tidal_acceleration = TWO * mass_geometric * state.config.column_length / (r * r * r);

    let lapse = ONE - r_s / r;
    let metric_tensor = match generate_schwarzschild_metric(-lapse, ONE / lapse, r * r, r * r) {
        Ok(g) => g,
        Err(e) => return fail(format!("schwarzschild metric: {e:?}")),
    };

    PropagatingEffect::pure(GrmhdState {
        mass_geometric,
        lapse,
        kretschmann,
        ricci_scalar,
        tidal_acceleration,
        metric_tensor: Some(metric_tensor),
        ..state
    })
}

/// Stage 2: the coupling.
///
/// A computed curvature quantity decides which algebra the plasma runs in. Above the tidal
/// threshold the flow is relativistic and needs the full `Cl(1,3)` spacetime algebra; below it,
/// the three-dimensional Euclidean algebra is enough. Both sides of the comparison are in `1/m`.
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
/// bivector they span. These are the components a static observer measures, so the algebra's
/// flat metric is the right one for them.
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
    let lorentz_force = f_field.0.data().get(E_12).copied().unwrap_or(ZERO);

    PropagatingEffect::pure(GrmhdState {
        lorentz_force,
        ..state
    })
}

/// Stage 4: the EM stress-energy the plasma feeds back into the metric.
///
/// The observer measures `B` in an orthonormal frame; the stress-energy kernel works in
/// coordinates. The frame component `F^{r^ theta^}` becomes the coordinate component
/// `F^{r theta} = B sqrt(g_rr g_thetatheta)^-1 = B sqrt(lapse) / r`, the kernel returns
/// `T^{tt}` in coordinates, and projecting that back onto the observer's time leg,
/// `T^{t^ t^} = (-g_tt) T^{tt} = lapse T^{tt}`, gives the energy density the observer measures.
/// For a pure magnetic field that must be `B^2 / 2`, whatever the metric, and the run checks
/// that it is.
pub fn calculate_energy_momentum(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let g_uv = match &state.metric_tensor {
        Some(t) => t,
        None => return fail("no metric tensor was built"),
    };
    let r = state.config.radius;
    let lapse = state.lapse;

    let em_tensor_component = state.config.magnetic_field * Real::sqrt(lapse) / r;
    let mut f_data = vec![ZERO; DIM * DIM];
    f_data[RADIAL * DIM + POLAR] = em_tensor_component;
    f_data[POLAR * DIM + RADIAL] = -em_tensor_component;
    let f_tensor = match CausalTensor::new(f_data, vec![DIM, DIM]) {
        Ok(t) => t,
        Err(e) => return fail(format!("EM tensor: {e:?}")),
    };

    let t_tensor = match energy_momentum_tensor_em(&f_tensor, g_uv).value_cloned() {
        Some(t) => t,
        None => return fail("energy momentum tensor produced no value"),
    };
    let em_energy_coordinate = match t_tensor.as_slice().first() {
        Some(&t_tt) => t_tt,
        None => return fail("the stress-energy tensor is empty"),
    };
    let em_energy_density = lapse * em_energy_coordinate;

    PropagatingEffect::pure(GrmhdState {
        em_tensor_component,
        em_energy_coordinate,
        em_energy_density,
        ..state
    })
}

/// Stage 5: what the confinement looks like once both halves have run.
///
/// The comparison is between two curvatures in `1/m^2`: `8 pi rho_EM`, the source term the
/// plasma's own stress-energy would put on the right of the Einstein equations, and `sqrt(K)`,
/// the curvature the hole imposes at the plasma.
pub fn analyze_stability(state: GrmhdState) -> PropagatingEffect<GrmhdState> {
    let plasma_curvature = eight_pi() * state.em_energy_density;
    let tidal_curvature = Real::sqrt(state.kretschmann);

    let status = if state.lorentz_force < ZERO {
        "Reversed confinement: the force points out of the column"
    } else if plasma_curvature > tidal_curvature {
        "Magnetically dominated: 8 pi rho_EM exceeds sqrt(K), the field sets the scale"
    } else {
        "Tidally dominated: sqrt(K) exceeds 8 pi rho_EM, the hole sets the scale"
    };

    PropagatingEffect::pure(GrmhdState {
        plasma_curvature,
        tidal_curvature,
        status,
        ..state
    })
}

/// The frame energy density of a pure magnetic field, `B^2 / 2`, in closed form.
pub fn frame_energy_density(magnetic_field: FloatType) -> FloatType {
    magnetic_field * magnetic_field / TWO
}

/// A geometric acceleration in `1/m` as an SI acceleration in `m/s^2`: multiply by `c^2`.
pub fn tidal_acceleration_si(geometric: FloatType) -> FloatType {
    geometric * LIGHT_SPEED * LIGHT_SPEED
}

/// The curvature radius `K^(-1/4)`, the length over which spacetime bends appreciably.
pub fn curvature_radius(kretschmann: FloatType) -> FloatType {
    ONE / Real::powf(kretschmann, QUARTER)
}

/// A vector with one blade set, in the given algebra.
fn blade(
    blades: usize,
    index: usize,
    value: FloatType,
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, String> {
    let mut data = vec![ZERO; blades];
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

fn fail(reason: impl Into<String>) -> PropagatingEffect<GrmhdState> {
    PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(reason.into())))
}
