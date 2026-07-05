/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # RAM-C stagnation line — the Tier-B Stage-4 milestone (shock fitting + reused Tier-A LER)
//!
//! On the stagnation streamline the bow shock is a 1-D **fitted interface**: the freestream crosses it and
//! the **exact Rankine–Hugoniot jump** sets the post-shock state, with `T₂` the **real transported energy**
//! (retiring the Tier-A recovery-temperature reconstruction). The smooth post-shock relaxation zone drives
//! the reused Tier-A Saha/Park-2T ionization → electron density → plasma frequency → blackout. The example
//! self-verifies (exit nonzero on break) against the RAM-C II flight data: the post-shock temperature band,
//! the peak electron density, blackout onset, and the `O(1)` rank of the smooth post-shock profile.
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p deep_causality_cfd --example qtt_ramc_stagline
//! ```

mod config;
mod print_utils;

use deep_causality_cfd::{
    Ambient, CoupledField, FiniteRateIonizationStage, FittedNormalShock, Park2tClosure,
    PhysicsStage, StepContext,
};
use deep_causality_physics::{
    AVOGADRO_CONSTANT, ElectronTemperature, THETA_VIB_N2, Temperature, VibrationalTemperature,
    air_n2_mole_fraction, air_o2_mole_fraction, finite_rate_ionization_fixed_point_kernel,
    no_associative_ionization_rate_kernel, no_dissociative_recombination_rate_kernel,
    vibrational_relaxation_kernel,
};
use deep_causality_tensor::Truncation;

/// Working precision.
pub type FloatType = f64;

fn main() {
    println!(
        "=== RAM-C stagnation line: exact Rankine–Hugoniot fit + reused Tier-A ionization ===\n"
    );
    println!(
        "Flight: M = {}, γ = {}, T_inf = {} K, n_inf = {:.0e} m^-3 (RAM-C II ~71 km)\n",
        config::MACH,
        config::GAMMA,
        config::T_INF,
        config::NUMBER_DENSITY
    );

    let shock = FittedNormalShock::<FloatType>::new(config::ft(config::GAMMA))
        .unwrap_or_else(|e| fail("fitted shock", e));
    let post = shock
        .post_shock(
            config::ft(config::T_INF),
            config::ft(config::NUMBER_DENSITY),
            config::ft(config::MACH),
        )
        .unwrap_or_else(|e| fail("post-shock state", e));
    // Post-shock residence time t_res = standoff / u₂ over which ionization lags equilibrium.
    let u2 = config::FREESTREAM_VELOCITY * post.u_ratio;
    let residence_time = config::STANDOFF_M / u2;
    let equilibrium = shock
        .stagnation_blackout(&post, config::ft(config::COMMS_BAND_RAD_S))
        .unwrap_or_else(|e| fail("Saha equilibrium reference", e));
    // Single-temperature (translational-T) surrogate — kept only as the over-predicting reference the
    // chemistry-fidelity upgrade replaces (ionizes at the hot T₂, not the lagging electron bath).
    let outcome_1t = shock
        .stagnation_line_blackout(
            &post,
            config::ft(residence_time),
            config::ft(config::COMMS_BAND_RAD_S),
        )
        .unwrap_or_else(|e| fail("single-temperature stagnation blackout", e));

    // Gap-3 Park two-temperature controller: ionization driven off Tₐ = √(T_tr·T_ve), with T_ve relaxed
    // from the free-stream value over the residence time. Post-shock pressure (atm) sets τ_vt.
    let pressure_atm = post.n_tot2 * config::ft(1.380_649e-23) * post.t2
        / config::ft(config::STANDARD_ATMOSPHERE_PA);
    let closure = Park2tClosure {
        t_ve_initial: config::ft(config::T_INF),
        pressure_atm,
        reduced_mass_amu: config::ft(config::REDUCED_MASS_AMU),
        theta_vib: config::ft(THETA_VIB_N2),
    };
    let outcome = shock
        .stagnation_line_blackout_2t(
            &post,
            config::ft(residence_time),
            &closure,
            config::ft(config::COMMS_BAND_RAD_S),
        )
        .unwrap_or_else(|e| fail("Park-2T stagnation blackout", e));
    println!(
        "Residence time t_res = standoff/u2 = {:.3e} s  (Saha-equilibrium upper bound n_e = {:.3e} m^-3)",
        residence_time, equilibrium.electron_density
    );
    println!(
        "Single-T surrogate (ionizes at T₂, over-predicts): α = {:.3e}, n_e = {:.3e} m^-3 ({:+.1} dec vs RAM-C)\n",
        outcome_1t.ionization_fraction,
        outcome_1t.electron_density,
        (outcome_1t.electron_density / config::RAMC_NE_REFERENCE).log10()
    );

    let trunc = Truncation::<FloatType>::by_tol(1e-10).unwrap_or_else(|e| {
        eprintln!("truncation: {e:?}");
        std::process::exit(2);
    });
    let (profile_bond, _peak) = shock
        .relaxation_profile_bond(
            &post,
            config::PROFILE_L,
            config::ft(config::RELAX_LENGTH),
            &trunc,
        )
        .unwrap_or_else(|e| fail("relaxation profile", e));

    // ── The uncalibrated finite-rate network, on the stagnation-line
    // transit-age profile. No Saha calibration target anywhere below: the
    // numbers are predictions from the RP-1232 Table II rate pairs and the
    // post-shock state.
    //
    // Behind the shock the stagnation-line velocity decelerates linearly to
    // zero at the body, u(ξ) ≈ u₂(1−ξ), so a parcel's age at fractional
    // depth ξ is age(ξ) = t_res · ln(1/(1−ξ)): geometry and the fitted
    // Rankine-Hugoniot state only, no free parameter. The reflectometers
    // measured the layer's peak, so the gate reads the profile's peak.
    let profile_cells = 64_usize;
    let carried_steps = 256_usize;
    let mut ne_profile: Vec<FloatType> = Vec::with_capacity(profile_cells);
    let mut peak = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64); // (n_e, x_n, x_o, t_ve)
    let mut ne_carried = 0.0_f64;
    for j in 1..=profile_cells {
        let xi = j as FloatType / (profile_cells as FloatType + 1.0);
        let age = residence_time * (1.0 / (1.0 - xi)).ln();
        // The vibrational bath relaxes over the same age as the chemistry.
        let t_ve_xi = vibrational_relaxation_kernel(
            VibrationalTemperature::new(closure.t_ve_initial).unwrap_or_else(|e| fail("T_ve", e)),
            Temperature::new(post.t2).unwrap_or_else(|e| fail("T2", e)),
            closure.pressure_atm,
            closure.reduced_mass_amu,
            closure.theta_vib,
            config::ft(age),
        )
        .unwrap_or_else(|e| fail("vibrational relaxation", e))
        .value();
        // Arm A (kept): sheath renewal — the closed-form exposure at exactly
        // age(ξ), each depth an independent parcel, matching the transit-age
        // closure the anchor gate is pinned on.
        let stage =
            FiniteRateIonizationStage::new(post.n_tot2).with_sheath_renewal(config::ft(age));
        let mut cell = CoupledField::new(Ambient::new(config::ft(0.01), config::ft(0.0), None));
        cell.set_scalar("T_tr", vec![post.t2]);
        cell.set_scalar("T_ve", vec![t_ve_xi]);
        let ctx = StepContext::<2, FloatType>::qtt(config::ft(age), 1);
        stage
            .apply(&ctx, &mut cell)
            .unwrap_or_else(|e| fail("finite-rate network stage", e));
        let ne = cell.scalar("n_e").expect("n_e written")[0];
        ne_profile.push(ne);
        if ne > peak.0 {
            peak = (
                ne,
                cell.scalar("atom_frac_n").expect("pool written")[0],
                cell.scalar("atom_frac_o").expect("pool written")[0],
                t_ve_xi,
            );
        }
        // Arm B: carried — the same parcel integrated as a marched history
        // (no renewal), the mode the time-marched corridor runs in. Under
        // recombination the two-way clock is self-limiting, so the carried
        // march stops at the network fixed point instead of running away as
        // the forward-only surrogate did.
        let carried = FiniteRateIonizationStage::new(post.n_tot2);
        let mut cell_b = CoupledField::new(Ambient::new(config::ft(0.01), config::ft(0.0), None));
        cell_b.set_scalar("T_tr", vec![post.t2]);
        cell_b.set_scalar("T_ve", vec![t_ve_xi]);
        let dt = age / carried_steps as FloatType;
        for s in 0..carried_steps {
            let ctx_b = StepContext::<2, FloatType>::qtt(dt, s + 1);
            carried
                .apply(&ctx_b, &mut cell_b)
                .unwrap_or_else(|e| fail("carried-mode network stage", e));
        }
        let ne_b = cell_b.scalar("n_e").expect("n_e written")[0];
        if ne_b > ne_carried {
            ne_carried = ne_b;
        }
    }
    let (ne_network, x_n, x_o, t_ve) = peak;
    let t_a = (post.t2 * t_ve).sqrt();

    // Channel 1 plus the lagged pool alone (no electron impact): the same
    // fixed point with the linear term dropped, for the D7 attribution.
    let to_conc = AVOGADRO_CONSTANT * 1.0e6;
    let conc = post.n_tot2 / to_conc;
    let conc_n = x_n * 2.0 * air_n2_mole_fraction::<FloatType>() * conc;
    let conc_o = x_o * 2.0 * air_o2_mole_fraction::<FloatType>() * conc;
    let k_f = no_associative_ionization_rate_kernel(
        Temperature::new(t_a).unwrap_or_else(|e| fail("T_a lift", e)),
    )
    .unwrap_or_else(|e| fail("associative forward", e))
    .value();
    let beta = no_dissociative_recombination_rate_kernel(
        ElectronTemperature::new(t_ve).unwrap_or_else(|e| fail("T_e lift", e)),
    )
    .unwrap_or_else(|e| fail("dissociative recombination", e))
    .value();
    let target_c1 = finite_rate_ionization_fixed_point_kernel(k_f * conc_n * conc_o, 0.0, beta)
        .unwrap_or_else(|e| fail("channel-1 fixed point", e))
        .value();
    let tau_c1 = 1.0 / (k_f * conc + beta * target_c1);
    let alpha_c1 = (target_c1 / conc) * (1.0 - (-(residence_time / tau_c1)).exp());
    let ne_channel1 = alpha_c1 * post.n_tot2;

    println!(
        "Uncalibrated finite-rate network (RP-1232 Table II pairs; no Saha target):\n  \
         lagged atom pool: x_N = {:.3e}, x_O = {:.3e}\n  \
         channel 1 + pool: n_e = {:.3e} m^-3 ({:+.2} dec vs RAM-C)\n  \
         full network:     n_e = {:.3e} m^-3 ({:+.2} dec vs RAM-C)\n",
        x_n,
        x_o,
        ne_channel1,
        (ne_channel1 / config::RAMC_NE_REFERENCE).log10(),
        ne_network,
        (ne_network / config::RAMC_NE_REFERENCE).log10(),
    );
    println!(
        "Sheath-renewal A/B under recombination (peak over the transit-age profile):\n  \
         renewal (kept):    n_e = {:.3e} m^-3 ({:+.2} dec vs RAM-C)\n  \
         carried (marched): n_e = {:.3e} m^-3 ({:+.2} dec vs RAM-C)\n",
        ne_network,
        (ne_network / config::RAMC_NE_REFERENCE).log10(),
        ne_carried,
        (ne_carried / config::RAMC_NE_REFERENCE).log10(),
    );

    print_utils::render(&post, &outcome, profile_bond);
    if print_utils::verify(&post, &outcome, profile_bond)
        && print_utils::verify_network(ne_channel1, ne_network)
        && print_utils::verify_renewal_ab(ne_network, ne_carried)
    {
        print_utils::summary(&outcome);
    } else {
        std::process::exit(1);
    }
}

/// Print a stage-failure context and its error on stderr, then exit the process non-zero.
fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(1);
}
