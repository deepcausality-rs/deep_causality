/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Cordell analytic SRP plume model — the plume-as-effective-obstruction
//! geometry of a single on-axis retro-nozzle. Implemented subset: terminal
//! (Mach-disk) shock via Sibulkin source flow + stagnation-pressure balance
//! (dissertation §3.2, Eqs. 7–16), Charwat barrel-shock shape (§3.3.1,
//! Eqs. 17–26), and the mass-flow-conservation radial scaling (§3.3.2,
//! Eqs. 27–40). The bow-shock construction (§3.4) is deliberately absent:
//! in this codebase the marched CFD layer forms its own bow shock around the
//! obstruction this kernel returns. The crossflow deflection (§3.5.2) is
//! identically zero for the on-axis single nozzle ("a plume exhausting
//! directly into the freestream gets no preferred direction", diss. p. 99),
//! and the free-shear-layer thickening (§3.5.1) is not modeled.
//!
//! Exact-anchor validation: the jet-edge Mach kernel reproduces the
//! dissertation's printed Table 13 values (C_T 0.47 → 3.86, 4.04 → 5.63,
//! 10.0 → 6.53 at the single-nozzle wind-tunnel conditions) and the
//! terminal-shock Mach matches Fig. 54 (≈ 15.5 analytic at C_T = 10).
//!
//! # References
//! * Cordell, C. E., Jr., "Computational Fluid Dynamics and Analytical
//!   Modeling of Supersonic Retropropulsion Flowfield Structures across a
//!   Wide Range of Potential Vehicle Configurations," Ph.D. dissertation,
//!   Georgia Institute of Technology, Dec. 2013, Ch. III
//!   (`papers/cordell_2013_srp_analytic.pdf`).
//! * Cordell, C. E., & Braun, R. D., "Steady State Modeling of Supersonic
//!   Retropropulsion Plume Structures," J. Spacecraft and Rockets
//!   50(4):763–770, 2013.

use crate::constants::{
    CORDELL_GAMMA_ENVELOPE_HI, CORDELL_GAMMA_ENVELOPE_LO, CORDELL_MACH_ENVELOPE_HI,
    CORDELL_MACH_ENVELOPE_LO, JARVINEN_ADAMS_TRANSITION_PRESSURE_RATIO_LO, real_from_f64,
    sibulkin_scaling_coefficient,
};
use crate::{
    Length, PhysicsError, PlumeGeometry, Pressure, Temperature, area_mach_ratio_kernel,
    isentropic_pressure_ratio_kernel,
};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Bisection iterations for the internal solves (deterministic across
/// precisions; the bracket halves each step).
const PLUME_SOLVE_ITERATIONS: usize = 200;
/// Simpson intervals of the Charwat barrel-shock shape integral (even).
const BARREL_SIMPSON_INTERVALS: usize = 2000;
/// The dissertation's own truncation of the shape integral short of the
/// cot singularity at `R_ND = 1` (diss. p. 84).
const BARREL_UPPER_LIMIT: f64 = 0.999;

#[inline]
fn lift<R: RealField + FromPrimitive>(x: f64, what: &'static str) -> Result<R, PhysicsError> {
    R::from_f64(x).ok_or_else(|| PhysicsError::NumericalInstability(what.into()))
}

/// Prandtl–Meyer function
///
/// $$ \nu(M) = \sqrt{\tfrac{\gamma+1}{\gamma-1}}\,
///    \tan^{-1}\sqrt{\tfrac{(M^2-1)(\gamma-1)}{\gamma+1}} - \tan^{-1}\sqrt{M^2-1} $$
///
/// (radians), the supersonic expansion-turn angle from Mach 1 to `M`.
/// Placed with its consumer (the Cordell plume model, dissertation Eq. (9),
/// p. 79); the relation itself is standard supersonic-flow theory.
///
/// # Arguments
/// * `mach` — Mach number `M` (≥ 1).
/// * `gamma` — ratio of specific heats `γ` (> 1).
///
/// # References
/// * Anderson, J. D., "Modern Compressible Flow," 3rd ed., McGraw-Hill
///   (2003), Ch. 4; Cordell dissertation Eq. (9).
pub fn prandtl_meyer_kernel<R>(mach: R, gamma: R) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let one = R::one();
    if mach < one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Prandtl-Meyer function requires Mach >= 1".into(),
        ));
    }
    if gamma <= one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Ratio of specific heats must be > 1".into(),
        ));
    }
    let a = ((gamma + one) / (gamma - one)).sqrt();
    let m2m1 = mach * mach - one;
    Ok(a * (m2m1.sqrt() / a).atan() - m2m1.sqrt().atan())
}

/// Choked (sonic-throat) mass flow
///
/// $$ \dot m = A^* p_0 \sqrt{\frac{\gamma}{R_s T_0}}
///    \left(\frac{2}{\gamma+1}\right)^{\frac{\gamma+1}{2(\gamma-1)}} $$
///
/// the maximum mass flow through a nozzle throat of area `A*` at chamber
/// stagnation conditions — the input mass flow the Cordell §3.3.2 scaling
/// conserves.
///
/// # Arguments
/// * `throat_area` — throat area `A*` (m², > 0).
/// * `chamber_pressure` — stagnation pressure `p₀` (Pa, > 0).
/// * `chamber_temperature` — stagnation temperature `T₀` (K, > 0).
/// * `gamma` — ratio of specific heats `γ` (> 1).
/// * `r_specific` — specific gas constant `R_s` (J·kg⁻¹·K⁻¹, > 0).
///
/// # References
/// * Anderson, "Modern Compressible Flow," 3rd ed. (2003), Ch. 5; Sutton &
///   Biblarz, "Rocket Propulsion Elements," 9th ed. (2017), Ch. 3.
pub fn choked_mass_flow_kernel<R>(
    throat_area: crate::Area<R>,
    chamber_pressure: Pressure<R>,
    chamber_temperature: Temperature<R>,
    gamma: R,
    r_specific: R,
) -> Result<crate::MassFlowRate<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let one = R::one();
    if gamma <= one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Ratio of specific heats must be > 1".into(),
        ));
    }
    let a_star = throat_area.value();
    let p0 = chamber_pressure.value();
    let t0 = chamber_temperature.value();
    if a_star <= R::zero() || p0 <= R::zero() || t0 <= R::zero() || r_specific <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Throat area, chamber state, and gas constant must be positive".into(),
        ));
    }
    let two: R = lift(2.0, "R::from_f64(2.0) failed")?;
    let exponent = (gamma + one) / (two * (gamma - one));
    let factor = (two / (gamma + one)).powf(exponent);
    crate::MassFlowRate::new(a_star * p0 * (gamma / (r_specific * t0)).sqrt() * factor)
}

/// Total pressure behind a normal shock relative to the upstream total
/// pressure (Cordell Eqs. (14)/(15); the standard Rayleigh-pitot loss):
/// monotone decreasing in `M` from 1 at `M = 1`.
fn normal_shock_total_pressure_ratio<R>(mach: R, gamma: R) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let one = R::one();
    let two: R = lift(2.0, "R::from_f64(2.0) failed")?;
    let m2 = mach * mach;
    let a = ((gamma + one) * m2 / ((gamma - one) * m2 + two)).powf(gamma / (gamma - one));
    let b = ((gamma + one) / (two * gamma * m2 - (gamma - one))).powf(one / (gamma - one));
    Ok(a * b)
}

/// Terminal (Mach-disk) shock Mach number of the retro-plume: the `M` at
/// which the jet's post-terminal-shock stagnation pressure balances the
/// freestream's post-bow-shock stagnation pressure (Cordell §3.2,
/// Eqs. (13)–(15); interface at rest). Solved by deterministic bisection —
/// the loss ratio is monotone decreasing in `M`.
///
/// # Arguments
/// * `chamber_pressure` — jet stagnation pressure `P_T,jet` (Pa).
/// * `post_bow_shock_total_pressure` — `P_T,1` (Pa), from
///   [`srp_post_bow_shock_total_pressure_kernel`].
/// * `gamma_jet` — jet ratio of specific heats (> 1).
pub fn srp_terminal_shock_mach_kernel<R>(
    chamber_pressure: Pressure<R>,
    post_bow_shock_total_pressure: Pressure<R>,
    gamma_jet: R,
) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if gamma_jet <= R::one() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Ratio of specific heats must be > 1".into(),
        ));
    }
    let pt_jet = chamber_pressure.value();
    let pt_1 = post_bow_shock_total_pressure.value();
    if pt_jet <= R::zero() || pt_1 <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Stagnation pressures must be positive".into(),
        ));
    }
    let target = pt_1 / pt_jet;
    if target >= R::one() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Jet stagnation pressure must exceed the post-bow-shock stagnation pressure for a \
             terminal shock to form (the low-thrust regime is outside the model)"
                .into(),
        ));
    }
    let half: R = lift(0.5, "R::from_f64(0.5) failed")?;
    let mut lo = R::one() + lift(1.0e-9, "R::from_f64(1e-9) failed")?;
    let mut hi = lift(1000.0, "R::from_f64(1000.0) failed")?;
    if normal_shock_total_pressure_ratio(hi, gamma_jet)? > target {
        return Err(PhysicsError::NumericalInstability(
            "srp_terminal_shock_mach_kernel: terminal Mach exceeds the solve bracket".into(),
        ));
    }
    for _ in 0..PLUME_SOLVE_ITERATIONS {
        let mid = (lo + hi) * half;
        if mid <= lo || mid >= hi {
            break;
        }
        if normal_shock_total_pressure_ratio(mid, gamma_jet)? > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok((lo + hi) * half)
}

/// Freestream stagnation pressure behind the normal (bow) shock, `P_T,1`
/// (Cordell Eqs. (13)–(14)): the backpressure the retro-plume expands
/// against.
///
/// # Arguments
/// * `p_inf` — freestream static pressure (Pa, > 0).
/// * `mach_inf` — freestream Mach number (> 1; a bow shock must exist).
/// * `gamma_inf` — freestream ratio of specific heats (> 1).
pub fn srp_post_bow_shock_total_pressure_kernel<R>(
    p_inf: Pressure<R>,
    mach_inf: R,
    gamma_inf: R,
) -> Result<Pressure<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if mach_inf <= R::one() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Freestream must be supersonic for a bow shock".into(),
        ));
    }
    let pt_inf = p_inf.value() * isentropic_pressure_ratio_kernel(mach_inf, gamma_inf)?;
    let loss = normal_shock_total_pressure_ratio(mach_inf, gamma_inf)?;
    Pressure::new(pt_inf * loss)
}

/// Jet-edge Mach number of the barrel shock: the Prandtl–Meyer expansion of
/// the nozzle exit flow to the post-bow-shock backpressure `P_T,1`
/// (Cordell Eq. (19)). Note the exit-Mach dependence cancels against the
/// exit static pressure, so `M_edge` depends only on `P_T,jet/P_T,1` — the
/// property that makes the dissertation's printed Table 13 an exact anchor.
///
/// # Arguments
/// * `exit_mach` — nozzle exit Mach `M_exit` (≥ 1).
/// * `exit_pressure` — static exit pressure `P_exit` (Pa, > 0).
/// * `post_bow_shock_total_pressure` — backpressure `P_T,1` (Pa, > 0).
/// * `gamma_jet` — jet ratio of specific heats (> 1).
pub fn srp_jet_edge_mach_kernel<R>(
    exit_mach: R,
    exit_pressure: Pressure<R>,
    post_bow_shock_total_pressure: Pressure<R>,
    gamma_jet: R,
) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let one = R::one();
    if exit_mach < one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Nozzle exit flow must be supersonic".into(),
        ));
    }
    if gamma_jet <= one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Ratio of specific heats must be > 1".into(),
        ));
    }
    let p_exit = exit_pressure.value();
    let pt_1 = post_bow_shock_total_pressure.value();
    if p_exit <= R::zero() || pt_1 <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Pressures must be positive".into(),
        ));
    }
    let two: R = lift(2.0, "R::from_f64(2.0) failed")?;
    let half: R = lift(0.5, "R::from_f64(0.5) failed")?;
    let stag = one + (gamma_jet - one) * half * exit_mach * exit_mach;
    let val = stag * (pt_1 / p_exit).powf((one - gamma_jet) / gamma_jet) - one;
    if val <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Backpressure too high: the jet does not expand supersonically at its edge".into(),
        ));
    }
    Ok((two / (gamma_jet - one) * val).sqrt())
}

/// The Cordell analytic plume-boundary geometry of a single **on-axis**
/// retro-nozzle: terminal-shock standoff (Sibulkin + stagnation balance,
/// §3.2), maximum plume radius (Charwat barrel shape, §3.3.1, scaled by
/// mass-flow conservation, §3.3.2), and the penetration/obstruction length
/// (the barrel profile extended at its maximum radius to the terminal-shock
/// station, diss. p. 84). Returns geometry only — shaping any discrete
/// forcing region from it is the CFD stage's job.
///
/// Validity envelope, enforced by rejection (the note's §6 discipline pin):
/// freestream Mach within the validated `[2, 4]` band, jet gamma within
/// `[1.2, 1.4]` (diss. Tables 7/11/12), a formed terminal shock
/// (`P_T,jet > P_T,1`), and a jet-exit pressure ratio at or above the
/// Jarvinen–Adams blunt-flow transition (`P_exit/P∞ ≥ 7.0` — below it the
/// flow is the unsteady jet-penetration regime where "the current model is
/// not valid," diss. p. 135).
///
/// Accuracy, per the dissertation's own comparisons: terminal-shock Mach
/// and jet-edge Mach reproduce the printed values (Table 13; Fig. 54);
/// terminal-shock standoff is consistently slightly underpredicted
/// (isentropic-jet assumption, p. 135); radial extent errors up to ~13%
/// across C_T = 1–10 (p. 148).
///
/// # Arguments
/// * `chamber_pressure` / `chamber_temperature` — jet stagnation state
///   (Pa, K; > 0).
/// * `r_specific` — jet specific gas constant (J·kg⁻¹·K⁻¹, > 0).
/// * `gamma_jet` — jet ratio of specific heats.
/// * `exit_mach` — nozzle exit Mach (≥ 1).
/// * `nozzle_half_angle_rad` — conical nozzle half-angle θ_n (rad, ≥ 0).
/// * `throat_diameter` / `exit_radius` / `cone_length` — nozzle geometry (m).
/// * `p_inf` — freestream static pressure (Pa).
/// * `mach_inf` / `gamma_inf` — freestream Mach and gamma.
#[allow(clippy::too_many_arguments)]
pub fn cordell_braun_plume_boundary_kernel<R>(
    chamber_pressure: Pressure<R>,
    chamber_temperature: Temperature<R>,
    r_specific: R,
    gamma_jet: R,
    exit_mach: R,
    nozzle_half_angle_rad: R,
    throat_diameter: Length<R>,
    exit_radius: Length<R>,
    cone_length: Length<R>,
    p_inf: Pressure<R>,
    mach_inf: R,
    gamma_inf: R,
) -> Result<PlumeGeometry<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let one = R::one();
    let two: R = lift(2.0, "R::from_f64(2.0) failed")?;
    let half: R = lift(0.5, "R::from_f64(0.5) failed")?;

    // ── Validity envelope (rejected, never extrapolated) ──
    let m_lo: R = real_from_f64(CORDELL_MACH_ENVELOPE_LO);
    let m_hi: R = real_from_f64(CORDELL_MACH_ENVELOPE_HI);
    if mach_inf < m_lo || mach_inf > m_hi {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Freestream Mach outside the Cordell model's validated envelope [2, 4]".into(),
        ));
    }
    let g_lo: R = real_from_f64(CORDELL_GAMMA_ENVELOPE_LO);
    let g_hi: R = real_from_f64(CORDELL_GAMMA_ENVELOPE_HI);
    if gamma_jet < g_lo || gamma_jet > g_hi {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Jet gamma outside the Cordell model's validated envelope [1.2, 1.4]".into(),
        ));
    }
    if exit_mach < one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Nozzle exit flow must be supersonic".into(),
        ));
    }
    if nozzle_half_angle_rad < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Nozzle half-angle cannot be negative".into(),
        ));
    }
    let d_throat = throat_diameter.value();
    let r_exit = exit_radius.value();
    let l_cone = cone_length.value();
    if d_throat <= R::zero() || r_exit <= R::zero() || l_cone < R::zero() {
        return Err(PhysicsError::Singularity(
            "Nozzle geometry must be positive".into(),
        ));
    }
    if chamber_temperature.value() <= R::zero() || r_specific <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Chamber temperature and gas constant must be positive".into(),
        ));
    }

    // ── Jet-penetration gate (Jarvinen–Adams blunt-flow transition) ──
    let pt_jet = chamber_pressure.value();
    let p_exit = pt_jet / isentropic_pressure_ratio_kernel(exit_mach, gamma_jet)?;
    let transition: R = real_from_f64(JARVINEN_ADAMS_TRANSITION_PRESSURE_RATIO_LO);
    if p_exit / p_inf.value() < transition {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Jet exit pressure ratio below the blunt-flow transition (P_exit/P_inf < 7): the \
             unsteady jet-penetration regime is outside the model"
                .into(),
        ));
    }

    // ── Step 1: terminal shock (Eqs. 7–16) ──
    let pt_1 = srp_post_bow_shock_total_pressure_kernel(p_inf, mach_inf, gamma_inf)?;
    let m_terminal = srp_terminal_shock_mach_kernel(chamber_pressure, pt_1, gamma_jet)?;

    let nu_vacuum = {
        // Eq. (8): the vacuum-expansion Prandtl–Meyer limit.
        let pi_: R = lift(core::f64::consts::PI, "R::from_f64(pi) failed")?;
        pi_ * half * (((gamma_jet + one) / (gamma_jet - one)).sqrt() - one)
    };
    let nu_exit = prandtl_meyer_kernel(exit_mach, gamma_jet)?;
    let theta_max = nu_vacuum - nu_exit + nozzle_half_angle_rad; // Eq. (10)
    if theta_max <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Maximum jet turn angle must be positive".into(),
        ));
    }
    let pi_: R = lift(core::f64::consts::PI, "R::from_f64(pi) failed")?;
    let psi_solid = two * pi_ * (one - theta_max.cos()); // Eq. (11)
    let b_sibulkin = sibulkin_scaling_coefficient::<R>() * pi_ / psi_solid; // Eq. (12)

    let p_terminal = pt_jet / isentropic_pressure_ratio_kernel(m_terminal, gamma_jet)?;
    let rho_ratio = (p_terminal / pt_jet).powf(one / gamma_jet); // Eq. (16)
    let x_terminal_throat = d_throat * (b_sibulkin / rho_ratio).sqrt(); // inverted Eq. (7)
    let standoff_exit = x_terminal_throat - l_cone;
    if standoff_exit <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Terminal shock inside the nozzle: the thrust is too low for the model".into(),
        ));
    }

    // ── Step 2: Charwat barrel shock (Eqs. 17–26) ──
    let m_edge = srp_jet_edge_mach_kernel(exit_mach, Pressure::new(p_exit)?, pt_1, gamma_jet)?;
    let ar_edge = area_mach_ratio_kernel(m_edge, gamma_jet)?;
    let ar_exit = area_mach_ratio_kernel(exit_mach, gamma_jet)?;
    let psi_charwat = two / (gamma_jet + one) * (m_edge * m_edge - one).sqrt()
        / (m_edge * m_edge * ar_edge)
        * ar_exit; // Eq. (20)
    let nu_edge = prandtl_meyer_kernel(m_edge, gamma_jet)?;
    let theta_0 = nu_edge - nu_exit + nozzle_half_angle_rad; // Eq. (21)
    if theta_0 <= R::zero() || psi_charwat <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Charwat shape parameters must be positive".into(),
        ));
    }
    let rho_a = (one + theta_0 / psi_charwat).sqrt(); // Eq. (22)
    let phi = psi_charwat + theta_0; // Eq. (23)
    if phi >= pi_ {
        return Err(PhysicsError::NumericalInstability(
            "Charwat shape angle exceeds pi: cot integrand undefined".into(),
        ));
    }

    // Eq. (26): X_ND(R) = ∫ cot(phi (1 - R²)) dR from 1/rho_a to 0.999,
    // Simpson with the dissertation's own truncation short of the R = 1
    // singularity. This is quadrature of a shape function, not a spatial
    // discretization.
    let upper: R = lift(BARREL_UPPER_LIMIT, "R::from_f64(0.999) failed")?;
    let a_lim = one / rho_a;
    if a_lim >= upper {
        return Err(PhysicsError::NumericalInstability(
            "Degenerate barrel-shock bracket".into(),
        ));
    }
    let n = BARREL_SIMPSON_INTERVALS;
    let n_r: R = lift(n as f64, "R::from_f64(N) failed")?;
    let h = (upper - a_lim) / n_r;
    let cot = |r: R| -> Result<R, PhysicsError> {
        let arg = phi * (one - r * r);
        let s = arg.sin();
        if s <= R::zero() {
            return Err(PhysicsError::NumericalInstability(
                "cot argument left (0, pi) in the barrel integral".into(),
            ));
        }
        Ok(arg.cos() / s)
    };
    let mut sum = cot(a_lim)? + cot(upper)?;
    let four: R = lift(4.0, "R::from_f64(4.0) failed")?;
    let mut r_i = a_lim;
    for i in 1..n {
        r_i += h;
        let w = if i % 2 == 1 { four } else { two };
        sum += w * cot(r_i)?;
    }
    let three: R = lift(3.0, "R::from_f64(3.0) failed")?;
    let x_apex_nd = sum * h / three;
    let r_max_raw = upper * rho_a * r_exit;
    let x_apex = x_apex_nd * rho_a * r_exit;

    // ── Step 3: mass-flow-conservation radial scaling (Eqs. 27–40) ──
    // The truncated Charwat plume loses boundary outflow; the radial
    // coordinate is scaled by C so terminal-disk + barrel outflow equal the
    // choked input mass flow.
    let rho_t_jet = pt_jet / (r_specific * chamber_temperature.value()); // Eq. (29)
    let a_star = pi_ * d_throat * d_throat / four;
    let mdot_in = choked_mass_flow_kernel(
        crate::Area::new(a_star)?,
        chamber_pressure,
        chamber_temperature,
        gamma_jet,
        r_specific,
    )?
    .value();
    let rho_terminal = rho_ratio * rho_t_jet;
    let v_terminal = m_terminal * (gamma_jet * p_terminal / rho_terminal).sqrt(); // Eq. (30)

    // Barrel curve points (raw radial coordinate), extended at max radius to
    // the terminal station (diss. p. 84).
    const CURVE_POINTS: usize = 400;
    let np: R = lift(CURVE_POINTS as f64, "R::from_f64(points) failed")?;
    let dr_step = (upper - a_lim) / np;
    let mut xs: alloc::vec::Vec<R> = alloc::vec::Vec::with_capacity(CURVE_POINTS + 2);
    let mut rs: alloc::vec::Vec<R> = alloc::vec::Vec::with_capacity(CURVE_POINTS + 2);
    xs.push(R::zero());
    rs.push(a_lim * rho_a * r_exit);
    let mut acc = R::zero();
    let mut f_prev = cot(a_lim)?;
    for i in 1..=CURVE_POINTS {
        let r_now = a_lim + dr_step * lift(i as f64, "R::from_f64(i) failed")?;
        let f_now = cot(r_now)?;
        acc += (f_prev + f_now) * half * dr_step;
        xs.push(acc * rho_a * r_exit);
        rs.push(r_now * rho_a * r_exit);
        f_prev = f_now;
    }
    // Terminal extension point.
    let x_end = if standoff_exit > xs[CURVE_POINTS] {
        standoff_exit
    } else {
        xs[CURVE_POINTS]
    };
    xs.push(x_end);
    rs.push(rs[CURVE_POINTS]);
    let last = CURVE_POINTS + 1;
    let r_term_raw = rs[last];

    let mdot_total = |c: R| -> Result<R, PhysicsError> {
        // Eq. (31): terminal-disk outflow through the scaled disk.
        let m_term = rho_terminal * v_terminal * pi_ * (c * r_term_raw) * (c * r_term_raw);
        // Eqs. (32)–(40): barrel outflow over the scaled curve.
        let mut m_bar = R::zero();
        for i in 1..last {
            let x = xs[i];
            let r = rs[i];
            let dx = (xs[i + 1] - xs[i - 1]) * half;
            let dr = (rs[i + 1] - rs[i - 1]) * half;
            let d = ((x + l_cone) * (x + l_cone) + (c * r) * (c * r)).sqrt(); // Eq. (32)
            let rr = b_sibulkin * (d_throat / d) * (d_throat / d); // Eq. (7)
            if rr >= one {
                continue; // inside the source core; no supersonic barrel point
            }
            let m_i = (two / (gamma_jet - one) * (rr.powf(-(gamma_jet - one)) - one)).sqrt(); // Eq. (33)
            let p_i = pt_jet / isentropic_pressure_ratio_kernel(m_i, gamma_jet)?;
            let rho_i = rr * rho_t_jet;
            let v_i = m_i * (gamma_jet * p_i / rho_i).sqrt(); // Eq. (34)
            let norm = ((c * dr) * (c * dr) + dx * dx).sqrt();
            if norm <= R::zero() {
                continue;
            }
            // Eqs. (35)/(38)/(39), transcribed verbatim: V along the throat ray
            // and the area vector A_i = (dx/norm)·[−C·dr, dx] (dissertation
            // Eq. 38). Note this is the paper's own formulation — its magnitude
            // is the *axial-projected* width dx, not the slant arc ds = norm, so
            // V·A carries the (dx/norm) factor. A first-principles
            // surface-of-revolution flux would drop it (the norm cancels
            // ds against the unit normal); Cordell does not, and the
            // mass-flow-conservation scaling on C absorbs the difference. Kept
            // faithful to the cited equations — do not "correct" toward ds.
            let v_dot_a = v_i / d * (dx / norm) * (c * r * dx - (x + l_cone) * c * dr);
            m_bar += two * pi_ * c * r * rho_i * v_dot_a; // Eq. (40) sum
        }
        Ok(m_term + m_bar) // Eq. (40) + Eq. (31)
    };

    // Iterate C (bisection; outflow grows with C).
    let mut c_lo = lift(0.2, "R::from_f64(0.2) failed")?;
    let mut c_hi = lift(10.0, "R::from_f64(10.0) failed")?;
    if mdot_total(c_lo)? > mdot_in || mdot_total(c_hi)? < mdot_in {
        return Err(PhysicsError::NumericalInstability(
            "Mass-flow scaling parameter outside the solve bracket".into(),
        ));
    }
    for _ in 0..PLUME_SOLVE_ITERATIONS {
        let mid = (c_lo + c_hi) * half;
        if mid <= c_lo || mid >= c_hi {
            break;
        }
        if mdot_total(mid)? < mdot_in {
            c_lo = mid;
        } else {
            c_hi = mid;
        }
    }
    let c_scale = (c_lo + c_hi) * half;

    let max_radius = Length::new(c_scale * r_max_raw)?;
    let penetration = if x_apex > standoff_exit {
        x_apex
    } else {
        standoff_exit
    };
    Ok(PlumeGeometry::new(
        max_radius,
        Length::new(penetration)?,
        Length::new(standoff_exit)?,
    ))
}
