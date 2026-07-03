/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain logic (the "how" of the physics): the atmosphere interpolation and the per-point
//! placard computation. Configuration (matrix location, shock model) lives in `model_config`;
//! tuned values in `constants`.

use crate::FloatType;
use crate::constants;
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::FittedNormalShock;
use deep_causality_num::Real;

/// The freestream `(n_tot m⁻³, T K, a m/s)` at `alt_m`, linearly interpolated between the
/// cited atmosphere rows. Altitudes outside the table are an error naming the valid range.
pub fn atmosphere_at(alt_m: FloatType) -> Result<(FloatType, FloatType, FloatType), String> {
    let floor = constants::ATMOSPHERE[0].0;
    let ceiling = constants::ATMOSPHERE[constants::ATMOSPHERE.len() - 1].0;
    if alt_m < ft(floor) || alt_m > ft(ceiling) {
        return Err(format!(
            "altitude {:.1} km is outside the atmosphere table ({:.0} to {:.0} km); \
             fix the matrix row or extend constants::ATMOSPHERE",
            alt_m / ft(1000.0),
            floor / 1000.0,
            ceiling / 1000.0
        ));
    }
    for pair in constants::ATMOSPHERE.windows(2) {
        let (a0, n0, temp0, c0) = pair[0];
        let (a1, n1, temp1, c1) = pair[1];
        if alt_m <= ft(a1) {
            let w = (alt_m - ft(a0)) / (ft(a1) - ft(a0));
            return Ok((
                ft(n0) + w * (ft(n1) - ft(n0)),
                ft(temp0) + w * (ft(temp1) - ft(temp0)),
                ft(c0) + w * (ft(c1) - ft(c0)),
            ));
        }
    }
    Err(format!(
        "altitude {:.1} km found no bracketing atmosphere rows (table not ascending?)",
        alt_m / ft(1000.0)
    ))
}

/// One placard row for one grid point: `[mach, alt km, q kPa, T₀ K, q̇ W/cm²]`.
///
/// Above Mach 1 the stagnation temperature is taken through the exact Rankine-Hugoniot jump:
/// the post-shock static state `(T₂, u₂/u₁)` from the fitted shock, the post-shock Mach number
/// `M₂ = M₁·(u₂/u₁)·√(T₁/T₂)`, then the isentropic re-stagnation
/// `T₀ = T₂·(1 + (γ−1)/2·M₂²)`. For a calorically perfect gas this equals the freestream total
/// temperature (the shock is adiabatic), so the branch is exactly continuous at Mach 1, where
/// the shock-free isentropic form takes over.
pub fn placard_point(
    shock: &FittedNormalShock<FloatType>,
    mach: FloatType,
    alt_km: FloatType,
) -> Result<[FloatType; 5], String> {
    let here = format!("M {mach:.2} / {alt_km:.1} km");
    let (n_inf, t_inf, a_inf) =
        atmosphere_at(alt_km * ft(1000.0)).map_err(|e| format!("grid point {here}: {e}"))?;
    let rho_inf = n_inf * ft(constants::AIR_MEAN_MOLECULAR_MASS_KG);
    let v = mach * a_inf;
    let q_pa = ft(0.5) * rho_inf * v * v;

    let half_gm1 = ft(0.5) * (ft(constants::GAMMA) - ft(1.0));
    let t0_k = if mach >= ft(1.0) {
        let post = shock
            .post_shock(t_inf, n_inf, mach)
            .map_err(|e| format!("grid point {here}: post-shock state failed: {e}"))?;
        let m2 = mach * post.u_ratio * Real::sqrt(t_inf / post.t2);
        post.t2 * (ft(1.0) + half_gm1 * m2 * m2)
    } else {
        // No shock below Mach 1: the isentropic stagnation temperature, the exact shock-free
        // limit of the branch above.
        t_inf * (ft(1.0) + half_gm1 * mach * mach)
    };

    let qdot_w_m2 = ft(constants::SUTTON_GRAVES_K)
        * Real::sqrt(rho_inf / ft(constants::NOSE_RADIUS_M))
        * v
        * v
        * v;

    Ok([
        mach,
        alt_km,
        q_pa / ft(1000.0), // Pa -> kPa
        t0_k,
        qdot_w_m2 / ft(1.0e4), // W/m² -> W/cm²
    ])
}
