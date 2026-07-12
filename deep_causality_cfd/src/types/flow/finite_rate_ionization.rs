/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The finite-rate ionization network stage: the two-way replacement for the
//! calibrated [`IonizationStage`](super::IonizationStage) closure.
//!
//! Per cell, the stage evaluates the three-channel RP-1232 network on the
//! evolved carrier state: associative ionization `N + O -> NO+ + e-` forward
//! and its dissociative-recombination reverse (the physical blackout-exit
//! mechanism), thresholded electron-impact ionization of N and O, and a
//! **lagged** neutral atom pool whose fractions relax toward their
//! dissociation equilibria (the pool inherits the same rate-versus-residence
//! honesty as the electrons). The N-pool clock carries both production
//! paths: direct dissociation and the low-activation Zeldovich exchange
//! `N2 + O -> NO + N` that feeds N atoms while direct dissociation is still
//! frozen. Every rate runs at its controlling temperature (the
//! ionization-chemistry note's coupling 2): ionization at the calibrated
//! geometric mean `sqrt(T_tr*T_ve)`, dissociation at Park's classic
//! `T_tr^0.7 * T_ve^0.3` (the published Park-lineage exponent for the Park
//! rate set), electron channels at `T_e = T_ve`. The carried fraction relaxes toward the network's
//! closed-form fixed point with the two-way clock
//! `tau = 1/(k_f[M] + beta*[e])`; integration is the shipped LER kernel, so
//! there is no stiff ODE solver and no per-cell iteration.
//!
//! There is **no Saha calibration target anywhere in this stage**: the fixed
//! point is where cited production balances cited loss.

use super::blackout::ler_step;
use super::coupling::{CoupledField, PhysicsStage, StepContext};
use crate::CfdScalar;
use alloc::vec::Vec;
use deep_causality_physics::{
    AVOGADRO_CONSTANT, ElectronTemperature, PhysicsError, Temperature, air_n2_mole_fraction,
    air_o2_mole_fraction, arrhenius_rate_kernel, dissociation_equilibrium_fraction_kernel,
    electron_impact_ionization_n_rate_kernel, electron_impact_ionization_o_rate_kernel,
    finite_rate_ionization_fixed_point_kernel, n2_dissociation_equilibrium_kernel,
    no_associative_ionization_rate_kernel, no_dissociative_recombination_rate_kernel,
    o2_dissociation_equilibrium_kernel, park_controlling_temperature_kernel, park_dissociation_q,
    rp1232_n2_diss_activation_temp, rp1232_n2_diss_exponent, rp1232_n2_diss_prefactor,
    rp1232_o2_diss_activation_temp, rp1232_o2_diss_exponent, rp1232_o2_diss_prefactor,
    zeldovich_exchange_rate_kernel,
};

/// The finite-rate ionization network stage. Reads the **translational**
/// temperature (default `"T_tr"`; rename with [`driven_by`](Self::driven_by)),
/// the vibrational-electron temperature (default `"T_ve"`, falling back to
/// the translational value), and the heavy-particle density (per-cell via
/// [`with_density_field`](Self::with_density_field), else the configured
/// constant); each channel's controlling temperature is computed internally.
/// Carries `"alpha"` and the two lagged atom-pool fractions
/// (`"atom_frac_n"`, `"atom_frac_o"`); writes `"n_e"`.
#[derive(Debug, Clone, Copy)]
pub struct FiniteRateIonizationStage<R: CfdScalar> {
    t_ctrl_field: &'static str,
    t_e_field: &'static str,
    alpha_field: &'static str,
    ne_field: &'static str,
    number_density: R,
    density_field: Option<&'static str>,
    sheath_renewal: Option<R>,
}

impl<R: CfdScalar> FiniteRateIonizationStage<R> {
    /// Drive the network at the configured total heavy-particle number
    /// density `number_density` (m⁻³) when no per-cell field is present.
    pub fn new(number_density: R) -> Self {
        Self {
            t_ctrl_field: "T_tr",
            t_e_field: "T_ve",
            alpha_field: "alpha",
            ne_field: "n_e",
            number_density,
            density_field: None,
            sheath_renewal: None,
        }
    }

    /// Read the translational temperature from a different field. The
    /// controlling temperatures per channel are computed internally from
    /// this and the vibrational-electron field, so pass the *translational*
    /// projection, not a precomputed controller.
    pub fn driven_by(mut self, field: &'static str) -> Self {
        self.t_ctrl_field = field;
        self
    }

    /// Rate the electron channels (impact ionization, dissociative
    /// recombination) off a different field. Default `"T_ve"` per the
    /// recorded `T_e = T_ve` insight; falls back to the controller
    /// temperature when the field is absent.
    pub fn with_electron_temperature_field(mut self, field: &'static str) -> Self {
        self.t_e_field = field;
        self
    }

    /// Read the heavy-particle density per cell from a named field (e.g. the
    /// evolved `"n_tot"`). A single-cell field broadcasts; the config value
    /// is the fallback.
    pub fn with_density_field(mut self, field: &'static str) -> Self {
        self.density_field = Some(field);
        self
    }

    /// Sheath renewal (the parcel picture): each step the sheath is refreshed
    /// by parcels whose chemistry has run for one **residence time**, so the
    /// carried scalars take their residence-limited lag values instead of
    /// accumulating. With the network's real loss channel the carried
    /// fraction self-limits either way; the A/B against the stagnation
    /// closure decides which mode the corridor keeps.
    pub fn with_sheath_renewal(mut self, residence_time: R) -> Self {
        self.sheath_renewal = Some(residence_time);
        self
    }

    fn lift(x: f64) -> Result<R, PhysicsError> {
        R::from_f64(x).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64 failed in the network stage".into())
        })
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for FiniteRateIonizationStage<R> {
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(t_ctrl) = field.scalar(self.t_ctrl_field) else {
            return Ok(());
        };
        let t_ctrl = t_ctrl.to_vec();
        let n = t_ctrl.len();
        let t_e = field.scalar(self.t_e_field).map(|t| t.to_vec());
        let density = self
            .density_field
            .and_then(|name| field.scalar(name))
            .map(|d| d.to_vec());
        let alpha_carried = field.scalar(self.alpha_field).map(|a| a.to_vec());
        let pool_n_carried = field.scalar("atom_frac_n").map(|a| a.to_vec());
        let pool_o_carried = field.scalar("atom_frac_o").map(|a| a.to_vec());

        // [M] in mol·cm⁻³; rates are cm³·mol⁻¹·s⁻¹ (RP-1232 Table II basis).
        let avogadro = Self::lift(AVOGADRO_CONSTANT)?;
        let cm3_per_m3 = Self::lift(1.0e6)?;
        let to_conc = avogadro * cm3_per_m3;
        let two = Self::lift(2.0)?;
        // A frozen-chemistry timescale >> dt: when every rate vanishes the
        // LER step leaves the carried scalars unchanged.
        let huge = Self::lift(1.0e30)?;
        let frozen_tau = ctx.dt() * huge;
        let x_n2 = air_n2_mole_fraction::<R>();
        let x_o2 = air_o2_mole_fraction::<R>();

        let dt = ctx.dt();
        let mut alpha_out = Vec::with_capacity(n);
        let mut ne_out = Vec::with_capacity(n);
        let mut pool_n_out = Vec::with_capacity(n);
        let mut pool_o_out = Vec::with_capacity(n);

        // Shape contract for every optional per-cell input: length n is
        // per-cell, length 1 broadcasts, anything else is a shape bug that
        // must surface, not silently read cell 0.
        let validate = |name: &str, cells: &Option<Vec<R>>| -> Result<(), PhysicsError> {
            match cells {
                Some(c) if c.len() != n && c.len() != 1 => {
                    Err(PhysicsError::DimensionMismatch(format!(
                        "the per-cell field '{}' has length {} but the controller field '{}' has length {} (expected {} or 1)",
                        name,
                        c.len(),
                        self.t_ctrl_field,
                        n,
                        n
                    )))
                }
                _ => Ok(()),
            }
        };
        validate(self.t_e_field, &t_e)?;
        if let Some(name) = self.density_field {
            validate(name, &density)?;
        }
        validate(self.alpha_field, &alpha_carried)?;
        validate("atom_frac_n", &pool_n_carried)?;
        validate("atom_frac_o", &pool_o_carried)?;

        let per_cell = |cells: &Option<Vec<R>>, i: usize, fallback: R| match cells {
            Some(c) if c.len() == n => c[i],
            Some(c) if c.len() == 1 => c[0],
            _ => fallback,
        };

        for (i, &tc) in t_ctrl.iter().enumerate() {
            let n_tot = per_cell(&density, i, self.number_density);
            let conc = n_tot / to_conc;
            let tve_val = per_cell(&t_e, i, tc);
            let t_tr = Temperature::new(tc)?;
            let t_ve = Temperature::new(tve_val)?;
            // Controlling temperatures per channel (coupling 2 of the
            // ionization-chemistry note): ionization at the calibrated
            // geometric mean, dissociation at Park's classic q = 0.7,
            // electron channels at T_e = T_ve.
            let half = R::one() / two;
            let t_ion = park_controlling_temperature_kernel(t_tr, t_ve, half)?;
            let t_diss = park_controlling_temperature_kernel(t_tr, t_ve, park_dissociation_q())?;
            let t_electron = ElectronTemperature::new(tve_val)?;

            // ── The lagged atom pool: equilibrium targets at the dissociation
            // controller; the O clock is direct dissociation, the N clock is
            // direct dissociation plus the Zeldovich exchange through the
            // already-relaxed O pool.
            let nuclei_n = two * x_n2 * conc;
            let nuclei_o = two * x_o2 * conc;
            let k_eq_n2 = n2_dissociation_equilibrium_kernel(t_diss)?;
            let k_eq_o2 = o2_dissociation_equilibrium_kernel(t_diss)?;
            let x_n_eq = dissociation_equilibrium_fraction_kernel(k_eq_n2, nuclei_n)?.value();
            let x_o_eq = dissociation_equilibrium_fraction_kernel(k_eq_o2, nuclei_o)?.value();
            let k_d_n2 = arrhenius_rate_kernel(
                t_diss,
                rp1232_n2_diss_prefactor::<R>(),
                rp1232_n2_diss_exponent::<R>(),
                rp1232_n2_diss_activation_temp::<R>(),
            )?
            .value();
            let k_d_o2 = arrhenius_rate_kernel(
                t_diss,
                rp1232_o2_diss_prefactor::<R>(),
                rp1232_o2_diss_exponent::<R>(),
                rp1232_o2_diss_activation_temp::<R>(),
            )?
            .value();
            let k_z = zeldovich_exchange_rate_kernel(t_diss)?.value();
            let tau_pool = |rate_sum: R| {
                if rate_sum > R::zero() {
                    R::one() / rate_sum
                } else {
                    frozen_tau
                }
            };
            let tau_o2 = tau_pool(k_d_o2 * conc);
            let x_o = match self.sheath_renewal {
                Some(t_res) => ler_step(R::zero(), x_o_eq, tau_o2, t_res),
                None => ler_step(per_cell(&pool_o_carried, i, R::zero()), x_o_eq, tau_o2, dt),
            };
            let conc_o = x_o * nuclei_o;
            let tau_n2 = tau_pool(k_d_n2 * conc + k_z * conc_o);
            let x_n = match self.sheath_renewal {
                Some(t_res) => ler_step(R::zero(), x_n_eq, tau_n2, t_res),
                None => ler_step(per_cell(&pool_n_carried, i, R::zero()), x_n_eq, tau_n2, dt),
            };
            let conc_n = x_n * nuclei_n;

            // ── The network channels.
            let k_f = no_associative_ionization_rate_kernel(t_ion)?.value();
            let production = k_f * conc_n * conc_o;
            let k_impact_n = electron_impact_ionization_n_rate_kernel(t_electron)?.value();
            let k_impact_o = electron_impact_ionization_o_rate_kernel(t_electron)?.value();
            let linear = k_impact_n * conc_n + k_impact_o * conc_o;
            let beta = no_dissociative_recombination_rate_kernel(t_electron)?.value();

            let target_conc =
                finite_rate_ionization_fixed_point_kernel(production, linear, beta)?.value();
            let alpha_target = {
                let a = target_conc / conc;
                if a > R::one() { R::one() } else { a }
            };

            // ── The two-way clock and the LER update of the carried fraction.
            // The clock is evaluated at the fixed point in renewal mode (a
            // fresh parcel's approach, state-independent so the renewed value
            // is stateless per step) and at the carried population otherwise.
            let alpha_prev = per_cell(&alpha_carried, i, R::zero());
            let e_for_tau = match self.sheath_renewal {
                // The clock's electron concentration is capped like the
                // target itself: alpha_target <= 1 means the electrons can
                // never exceed the heavy-particle concentration.
                Some(_) => {
                    if target_conc < conc {
                        target_conc
                    } else {
                        conc
                    }
                }
                None => alpha_prev * conc,
            };
            let denom = k_f * conc + beta * e_for_tau;
            let tau = if denom > R::zero() {
                R::one() / denom
            } else {
                frozen_tau
            };
            let alpha = match self.sheath_renewal {
                Some(t_res) => ler_step(R::zero(), alpha_target, tau, t_res),
                None => ler_step(alpha_prev, alpha_target, tau, dt),
            };

            alpha_out.push(alpha);
            ne_out.push(alpha * n_tot);
            pool_n_out.push(x_n);
            pool_o_out.push(x_o);
        }

        field.set_scalar(self.alpha_field, alpha_out);
        field.set_scalar(self.ne_field, ne_out);
        field.set_scalar("atom_frac_n", pool_n_out);
        field.set_scalar("atom_frac_o", pool_o_out);
        Ok(())
    }
}
