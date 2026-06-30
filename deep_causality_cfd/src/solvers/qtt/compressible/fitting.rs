/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 4 — shock fitting on the RAM-C stagnation line (the buildable milestone, design D1/D9).
//!
//! On the stagnation streamline the bow shock is a 1-D **fitted interface**: the freestream crosses it and
//! the **exact Rankine–Hugoniot jump** sets the post-shock state. No flux is marched *through* the front
//! (the `studies/qtt_repin_marcher` lesson), so each side stays smooth and `O(1)` rank. The post-shock
//! translational temperature `T₂` is the **real transported energy** — this retires the Tier-A
//! recovery-temperature *reconstruction*. The smooth post-shock relaxation zone then drives the reused
//! Tier-A ionization kernels: Saha/Park-2T ionization → electron density → plasma frequency → blackout.
//! The gate is the peak electron density / blackout onset against the RAM-C II flight data.

use crate::tensor_bridge::quantize;
use crate::types::CfdScalar;
use alloc::vec;
use deep_causality_num::ConjugateScalar;
use deep_causality_physics::{
    AVOGADRO_CONSTANT, ElectronDensity, PARK_NO_IONIZATION_ACTIVATION_TEMP,
    PARK_NO_IONIZATION_EXPONENT, PARK_NO_IONIZATION_PREFACTOR, PhysicsError, Temperature,
    arrhenius_rate_kernel, electron_density_kernel, park2t_ionization_surrogate_kernel,
    plasma_frequency_kernel, rankine_hugoniot_temperature_kernel,
};
use deep_causality_tensor::{CausalTensor, Truncation};

/// The post-shock state from the exact Rankine–Hugoniot normal-shock jump.
#[derive(Clone, Copy, Debug)]
pub struct PostShockState<R> {
    /// Post-shock translational temperature `T₂` (K) — the transported energy, not a reconstruction.
    pub t2: R,
    /// Post-shock heavy-particle number density `n₂` (m⁻³).
    pub n_tot2: R,
    /// Density ratio `ρ₂/ρ₁`.
    pub rho_ratio: R,
    /// Velocity ratio `u₂/u₁`.
    pub u_ratio: R,
    /// Pressure ratio `p₂/p₁`.
    pub p_ratio: R,
}

/// The stagnation-line blackout outcome at the post-shock equilibrium (the peak).
#[derive(Clone, Copy, Debug)]
pub struct StagnationOutcome<R> {
    /// Peak electron density `n_e` (m⁻³).
    pub electron_density: R,
    /// Plasma (angular) frequency `ω_p` (rad/s).
    pub plasma_frequency: R,
    /// Ionization fraction `α`.
    pub ionization_fraction: R,
    /// Whether the plasma frequency exceeds the comms band (signal cutoff).
    pub blackout: bool,
}

/// A fitted normal shock on the stagnation streamline: the exact Rankine–Hugoniot interface (task 4.1).
pub struct FittedNormalShock<R> {
    gamma: R,
}

impl<R> FittedNormalShock<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// Build the fitted shock for a ratio of specific heats `gamma` (> 1; an effective post-shock value
    /// for reacting air narrows the `T₂` over-prediction of the perfect-gas value).
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] if `gamma <= 1`.
    pub fn new(gamma: R) -> Result<Self, PhysicsError> {
        if gamma <= R::one() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "ratio of specific heats must be > 1".into(),
            ));
        }
        Ok(Self { gamma })
    }

    /// The exact RH post-shock state for freestream `(T₁, n₁)` at Mach `mach` (≥ 1).
    ///
    /// # Errors
    /// Propagates the RH temperature kernel's invariants; numeric-conversion failures.
    pub fn post_shock(
        &self,
        t_inf: R,
        n_tot_inf: R,
        mach: R,
    ) -> Result<PostShockState<R>, PhysicsError> {
        let g = self.gamma;
        let one = R::one();
        let two = R::from_f64(2.0)
            .ok_or_else(|| PhysicsError::NumericalInstability("from_f64(2.0)".into()))?;
        let m2 = mach * mach;

        let t2 = rankine_hugoniot_temperature_kernel(Temperature::new(t_inf)?, mach, g)?.value();
        let rho_ratio = (g + one) * m2 / ((g - one) * m2 + two);
        let u_ratio = one / rho_ratio;
        let p_ratio = (two * g * m2 - (g - one)) / (g + one);
        let n_tot2 = n_tot_inf * rho_ratio;

        Ok(PostShockState {
            t2,
            n_tot2,
            rho_ratio,
            u_ratio,
            p_ratio,
        })
    }

    /// The equilibrium blackout state at the post-shock condition (the stagnation-line peak): Saha/Park-2T
    /// ionization → electron density → plasma frequency → blackout vs the `comms_band` (rad/s).
    ///
    /// # Errors
    /// Propagates the ionization / electron-density / plasma-frequency kernels.
    pub fn stagnation_blackout(
        &self,
        post: &PostShockState<R>,
        comms_band: R,
    ) -> Result<StagnationOutcome<R>, PhysicsError> {
        let alpha = park2t_ionization_surrogate_kernel(Temperature::new(post.t2)?, post.n_tot2)?;
        let n_e = electron_density_kernel(alpha, post.n_tot2)?;
        let omega_p = plasma_frequency_kernel(n_e)?;
        Ok(StagnationOutcome {
            electron_density: n_e.value(),
            plasma_frequency: omega_p.value(),
            ionization_fraction: alpha.value(),
            blackout: omega_p.value() > comms_band,
        })
    }

    /// The **nonequilibrium** stagnation-line blackout (the physically correct peak): ionization lags
    /// Saha equilibrium because the residence time is short against the ionization time
    /// `τ_ion = 1 / (k_f · n₂)`, with `k_f` the **dominant associative-ionization rate** N + O → NO⁺ + e⁻
    /// (Park / Gupta), grounded — not a free fit. The closed-form LER relaxation gives
    /// `α = α_eq·(1 − e^{−t_res/τ_ion})`, so the peak `n_e` sits well below the Saha equilibrium of
    /// [`stagnation_blackout`], toward the RAM-C flight value. `residence_time` is `t_res` (s).
    ///
    /// # Errors
    /// Propagates the ionization / rate / electron-density / plasma-frequency kernels.
    pub fn stagnation_line_blackout(
        &self,
        post: &PostShockState<R>,
        residence_time: R,
        comms_band: R,
    ) -> Result<StagnationOutcome<R>, PhysicsError> {
        let t2 = Temperature::new(post.t2)?;
        let alpha_eq = park2t_ionization_surrogate_kernel(t2, post.n_tot2)?.value();

        // k_f in Park/Gupta units (cm³·mol⁻¹·s⁻¹) → SI (m³·s⁻¹ per particle): ×1e-6 / N_A.
        let prefactor = R::from_f64(PARK_NO_IONIZATION_PREFACTOR)
            .ok_or_else(|| PhysicsError::NumericalInstability("Park prefactor".into()))?;
        let exponent = R::from_f64(PARK_NO_IONIZATION_EXPONENT)
            .ok_or_else(|| PhysicsError::NumericalInstability("Park exponent".into()))?;
        let theta_d = R::from_f64(PARK_NO_IONIZATION_ACTIVATION_TEMP)
            .ok_or_else(|| PhysicsError::NumericalInstability("Park activation temp".into()))?;
        let k_cgs = arrhenius_rate_kernel(t2, prefactor, exponent, theta_d)?.value();
        let cm3_per_m3 = R::from_f64(1.0e-6)
            .ok_or_else(|| PhysicsError::NumericalInstability("cm³→m³".into()))?;
        let avogadro = R::from_f64(AVOGADRO_CONSTANT)
            .ok_or_else(|| PhysicsError::NumericalInstability("Avogadro".into()))?;
        let k_si = k_cgs * cm3_per_m3 / avogadro;
        let tau_ion = R::one() / (k_si * post.n_tot2);

        let frac = R::one() - (R::zero() - residence_time / tau_ion).exp();
        let alpha = alpha_eq * frac;
        let n_e = ElectronDensity::new(alpha * post.n_tot2)?;
        let omega_p = plasma_frequency_kernel(n_e)?;
        Ok(StagnationOutcome {
            electron_density: n_e.value(),
            plasma_frequency: omega_p.value(),
            ionization_fraction: alpha,
            blackout: omega_p.value() > comms_band,
        })
    }

    /// The post-shock ionization relaxation profile `n_e(s) = α_eq·n₂·(1 − e^{−s/L})` along the streamline
    /// as a 1-D QTT field (the smooth post-shock zone). Returns `(max_bond, peak n_e)` — the bond witnesses
    /// the "each side `O(1)` rank" of task 4.1.
    ///
    /// # Errors
    /// Propagates the ionization kernel / codec; numeric-conversion failures.
    pub fn relaxation_profile_bond(
        &self,
        post: &PostShockState<R>,
        l: usize,
        relax_length: R,
        trunc: &Truncation<R>,
    ) -> Result<(usize, R), PhysicsError> {
        let alpha_eq =
            park2t_ionization_surrogate_kernel(Temperature::new(post.t2)?, post.n_tot2)?.value();
        let peak = alpha_eq * post.n_tot2;
        let n = 1usize << l;
        let n_r = R::from_usize(n)
            .ok_or_else(|| PhysicsError::NumericalInstability("from_usize(n)".into()))?;
        let mut data = vec![R::zero(); n];
        for (i, d) in data.iter_mut().enumerate() {
            let s = R::from_usize(i)
                .ok_or_else(|| PhysicsError::NumericalInstability("from_usize(i)".into()))?
                / n_r;
            let frac = R::one() - (R::zero() - s / relax_length).exp();
            *d = peak * frac;
        }
        let field = quantize(&CausalTensor::new(data, vec![n])?, trunc)?;
        Ok((field.max_bond(), peak))
    }
}
