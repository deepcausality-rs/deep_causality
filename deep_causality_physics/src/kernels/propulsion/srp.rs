/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Supersonic-retropropulsion similarity numbers and the digitized
//! Jarvinen–Adams central-nozzle drag correlation. All correlations
//! interpolate the digitized tables in `constants/propulsion.rs` by value
//! and reject out-of-domain inputs — extrapolating a wind-tunnel correlation
//! would fabricate physics.

use crate::constants::{
    JARVINEN_ADAMS_BASELINE_CA0, JARVINEN_ADAMS_PRESERVED_DRAG_M2, real_from_f64,
};
use crate::{Area, Density, Force, PhysicsError, Pressure, Speed};
use alloc::format;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Linear interpolation over a digitized `(x, y)` table, bracketing by
/// value. The tables are authored ascending in `x`; inputs outside the
/// digitized domain are rejected, never clamped or extrapolated.
fn interp_digitized_table<R>(
    table: &[(f64, f64)],
    x: R,
    what: &'static str,
) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if !x.is_finite() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Interpolation input must be finite".into(),
        ));
    }
    let lo: R = real_from_f64(table[0].0);
    let hi: R = real_from_f64(table[table.len() - 1].0);
    if x < lo || x > hi {
        return Err(PhysicsError::PhysicalInvariantBroken(format!(
            "{what}: input outside the digitized domain"
        )));
    }
    for w in table.windows(2) {
        let (x0, y0) = w[0];
        let (x1, y1) = w[1];
        let x0r: R = real_from_f64(x0);
        let x1r: R = real_from_f64(x1);
        if x <= x1r {
            let y0r: R = real_from_f64(y0);
            let y1r: R = real_from_f64(y1);
            let t = (x - x0r) / (x1r - x0r);
            return Ok(y0r + t * (y1r - y0r));
        }
    }
    // Unreachable: the domain check above guarantees a bracket.
    Err(PhysicsError::NumericalInstability(
        "interp_digitized_table: bracket not found".into(),
    ))
}

/// SRP thrust coefficient
///
/// $$ C_T = \frac{T}{q_\infty \cdot S_{ref}} $$
///
/// the freestream-normalized similarity number of the Jarvinen–Adams
/// retropropulsion dataset (`C_T = T/(q∞·A_m)`, model base area as the
/// reference). Distinct from the duct solver's nozzle thrust coefficient
/// (normalized by `p₀·A*`) and from the Korzun–Cruz–Braun survey's
/// nozzle-referenced definition — same words, different normalizations.
///
/// # Arguments
/// * `thrust` — retro-thrust `T` (N, ≥ 0).
/// * `q_inf` — freestream dynamic pressure `q∞` (Pa, > 0).
/// * `s_ref` — aerodynamic reference area `S_ref` (m², > 0).
///
/// # References
/// * Jarvinen & Adams, MC 70-3001-R2, NASA NAS 7-576 (1970), Nomenclature
///   p. v (`papers/jarvinen_adams_1970_ntrs_19720005324.pdf`).
pub fn srp_thrust_coefficient_kernel<R>(
    thrust: Force<R>,
    q_inf: Pressure<R>,
    s_ref: Area<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let t = thrust.value();
    if t < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Thrust cannot be negative".into(),
        ));
    }
    let q = q_inf.value();
    if q <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Freestream dynamic pressure must be positive".into(),
        ));
    }
    let s = s_ref.value();
    if s <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Reference area must be positive".into(),
        ));
    }
    Ok(t / (q * s))
}

/// Jet-to-freestream momentum-flux ratio
///
/// $$ J = \frac{\rho_j u_j^2}{\rho_\infty u_\infty^2} $$
///
/// the second similarity input of the Cordell plume model.
///
/// # References
/// * Cordell & Braun, "Steady State Modeling of Supersonic Retropropulsion
///   Plume Structures," JSR 50(4):763–770, 2013; Cordell dissertation,
///   Georgia Tech (2013) (`papers/cordell_2013_srp_analytic.pdf`).
pub fn momentum_flux_ratio_kernel<R>(
    rho_jet: Density<R>,
    u_jet: Speed<R>,
    rho_inf: Density<R>,
    u_inf: Speed<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let denom = rho_inf.value() * u_inf.value() * u_inf.value();
    if denom <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Freestream momentum flux must be positive".into(),
        ));
    }
    Ok(rho_jet.value() * u_jet.value() * u_jet.value() / denom)
}

/// Preserved-drag fraction of the central-nozzle SRP configuration — the
/// digitized **Jarvinen–Adams** curve `C_A_F/C_A0` vs `C_T` at M∞ = 2.0
/// ([`JARVINEN_ADAMS_PRESERVED_DRAG_M2`]). The low-C_T structure is the
/// report's measured drag collapse: the fraction falls from 1.0 to 0.22 by
/// C_T ≈ 0.46 and drops sharply across the jet-penetration → blunt-flow
/// transition near C_T ≈ 1; slightly negative values past C_T ≈ 2 are the
/// measured wake-type forebody force. Domain: C_T ∈ [0.0, 8.8]; inputs
/// outside it are rejected, never extrapolated. Corroboration: the
/// Korzun–Cruz–Braun survey's "minimum value of approximately 10% of the
/// no-jet value" brackets this curve's 0.12 → 0.02 transition values.
///
/// # References
/// * Jarvinen & Adams (1970), Fig. 32 (p. 54) and Fig. 56 (p. 81),
///   digitized ±0.02 (`papers/jarvinen_adams_1970_ntrs_19720005324.pdf`).
pub fn srp_preserved_drag_fraction_kernel<R>(c_t: R) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if c_t < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Thrust coefficient cannot be negative".into(),
        ));
    }
    interp_digitized_table(
        &JARVINEN_ADAMS_PRESERVED_DRAG_M2,
        c_t,
        "srp_preserved_drag_fraction_kernel",
    )
}

/// Unpowered baseline axial-force coefficient `C_A0(M)` of the
/// Jarvinen–Adams 60° single-engine aeroshell, digitized over the tested
/// envelope M ∈ [0.6, 2.0] ([`JARVINEN_ADAMS_BASELINE_CA0`]). Inputs
/// outside the digitized envelope are rejected.
///
/// # References
/// * Jarvinen & Adams (1970), Figs. 32–33 intercepts corroborated by
///   Fig. 11 (p. 27), digitized ±0.03.
pub fn jarvinen_adams_baseline_axial_coefficient_kernel<R>(mach: R) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    interp_digitized_table(
        &JARVINEN_ADAMS_BASELINE_CA0,
        mach,
        "jarvinen_adams_baseline_axial_coefficient_kernel",
    )
}

/// Total axial force coefficient of the central-nozzle SRP configuration
///
/// $$ C_{A,total} = C_T + f(C_T) \cdot C_{A0} $$
///
/// composed from the sibling kernels ([`srp_preserved_drag_fraction_kernel`]
/// and [`jarvinen_adams_baseline_axial_coefficient_kernel`]); no correlation
/// is restated here. At M = 2.0 this reproduces the report's Fig. 56
/// directly (`C_A0 = 1.25`); at other Mach numbers in [0.6, 2.0] the M = 2.0
/// preserved-drag shape is scaled by the local baseline — an approximation
/// the report's M = 1.5 data (Fig. 33) supports in shape, labeled here as
/// such. The composition carries the measured non-monotone band: the total
/// axial force *dips below* the unpowered value for low C_T (minimum ≈ 0.73
/// vs 1.25 unpowered at C_T ≈ 0.46, M = 2.0) before thrust dominates —
/// lighting the engine gently buys less deceleration than coasting.
///
/// # References
/// * Jarvinen & Adams (1970), Figs. 32/56; Conclusion 5, p. 145: "as the
///   thrusting coefficient increases (C_T ≥ 1), the total retroforce becomes
///   approximately equal to the retrothrust."
pub fn srp_total_axial_force_coefficient_kernel<R>(c_t: R, mach: R) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let fraction = srp_preserved_drag_fraction_kernel(c_t)?;
    let ca0 = jarvinen_adams_baseline_axial_coefficient_kernel(mach)?;
    Ok(c_t + fraction * ca0)
}

/// Flow-regime margin of the central-nozzle SRP configuration
///
/// $$ m = C_T - C_{T,transition} $$
///
/// positive in the steady blunt-flow regime, negative in the unsteady
/// jet-penetration regime (the jet pushes the bow shock up to six body
/// diameters upstream and the flow is unsteady — the physical reason a
/// throttle *floor* is a stability constraint for central SRP). The
/// transition C_T is supplied by the caller: at M∞ = 2.0 it is the
/// report's sharp transition near unity
/// ([`crate::constants::JARVINEN_ADAMS_TRANSITION_CT_M2`]); across the
/// tested conditions the transition is fixed in jet-exit pressure ratio
/// (P_ej/P∞ ≈ 7.0–7.2) with C_T in 0.5–3.0. The separate peripheral-
/// configuration bow-shock rippling bound (C_T ≳ 3, Keyes–Hefner via the
/// Korzun survey) is recorded as
/// [`crate::constants::KEYES_HEFNER_PERIPHERAL_RIPPLE_CT`] and does not
/// apply to the central configuration this kernel serves.
///
/// # References
/// * Jarvinen & Adams (1970), §3.1.2–3.1.3 (pp. 25–32), Fig. 18 (p. 35),
///   Conclusion 3 (p. 145); Korzun, Cruz & Braun, IEEE Aerospace (2008), p. 6.
pub fn srp_flow_regime_margin_kernel<R>(c_t: R, transition_c_t: R) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if c_t < R::zero() || !c_t.is_finite() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Thrust coefficient must be finite and non-negative".into(),
        ));
    }
    if transition_c_t <= R::zero() || !transition_c_t.is_finite() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Transition thrust coefficient must be finite and positive".into(),
        ));
    }
    Ok(c_t - transition_c_t)
}
