/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-A Park-2T plasma-blackout coupling stages (Gap 2): the **Lagging-Equilibrium
//! Relaxation (LER)** mechanism applied to the existing incompressible QTT rollout.
//!
//! Three between-step [`PhysicsStage`]s, composed statically with the rest of the
//! coupling:
//! * [`RecoveryTemperatureStage`] rebuilds the driving translational temperature
//!   `T_tr` each step from the flow state — a recovery-temperature reconstruction
//!   off the per-cell speed with a **mandatory Rankine–Hugoniot** post-shock jump
//!   from the configured flight condition (isentropic recovery alone is too cold to
//!   ionize). Tier-A stand-in, not a true post-shock thermodynamic path.
//! * [`IonizationStage`] relaxes a carried ionization fraction `α` toward the
//!   Park-2T Saha surrogate `α_eq(T_tr)` with a timescale `τ_ion` grounded in the
//!   dominant associative-ionization rate (N + O → NO⁺ + e⁻), via the closed-form
//!   LER exponential, and writes back the electron density `n_e = α · n_tot`.
//! * [`EosStage`] writes a per-cell two-temperature pressure into a `"pressure"`
//!   scalar — the interface Tier-B reuses. On the incompressible Tier-A rollout the
//!   ambient effect is deliberately limited (the marcher does not consume it).

use super::coupling::{CoupledField, PhysicsStage, StepContext};
use crate::types::CfdScalar;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_physics::{
    AVOGADRO_CONSTANT, BOLTZMANN_CONSTANT, ElectronDensity, PARK_NO_IONIZATION_ACTIVATION_TEMP,
    PARK_NO_IONIZATION_EXPONENT, PARK_NO_IONIZATION_PREFACTOR, PhysicsError, Temperature,
    VibrationalTemperature, arrhenius_rate_kernel, park2t_ionization_surrogate_kernel,
    plasma_frequency_kernel, rankine_hugoniot_temperature_kernel, recovery_temperature_kernel,
    vibrational_relaxation_kernel,
};

/// The closed-form Lagging-Equilibrium Relaxation step:
/// `x(t+Δt) = x_eq − (x_eq − x)·exp(−Δt/τ)`.
///
/// Exact on the linear relaxation, unconditionally stable under stiffness, and it
/// returns the target exactly as `τ → 0` (the equilibrium limit). This is the one
/// mechanism the Tier-A stages share.
pub fn ler_step<R: CfdScalar>(x: R, x_eq: R, tau: R, dt: R) -> R {
    if tau <= R::zero() {
        // τ → 0: the increment jumps the state exactly to equilibrium.
        return x_eq;
    }
    x_eq - (x_eq - x) * (-(dt / tau)).exp()
}

/// Relax a named [`CoupledField`] scalar toward per-cell targets with per-cell
/// timescales, in place, by the closed-form [`ler_step`]. A no-op if the field is
/// absent; the `targets`/`taus` slices must match the field length.
pub fn ler_relax_scalar<R: CfdScalar>(
    field: &mut CoupledField<R>,
    name: &str,
    dt: R,
    targets: &[R],
    taus: &[R],
) -> Result<(), PhysicsError> {
    let Some(xs) = field.scalar_mut(name) else {
        return Ok(());
    };
    if xs.len() != targets.len() || xs.len() != taus.len() {
        return Err(PhysicsError::DimensionMismatch(
            "LER relax: scalar length must match targets/taus".into(),
        ));
    }
    for (i, x) in xs.iter_mut().enumerate() {
        *x = ler_step(*x, targets[i], taus[i], dt);
    }
    Ok(())
}

/// Rebuilds `T_tr` each step from the flow state: `T_tr = T_post − ½|u|²/c_p`, with
/// `T_post` from a Rankine–Hugoniot normal-shock jump on the configured flight Mach.
/// Reads the per-cell `"speed"` field (the state-derived `|u|`) and writes `"T_tr"`.
#[derive(Debug, Clone, Copy)]
pub struct RecoveryTemperatureStage<R: CfdScalar> {
    speed_field: &'static str,
    t_tr_field: &'static str,
    mach: R,
    gamma: R,
    t_inf: R,
    c_p: R,
}

impl<R: CfdScalar> RecoveryTemperatureStage<R> {
    /// Reconstruct `T_tr` from the configured flight condition (`mach`, `gamma`,
    /// freestream `t_inf`) and the mixture `c_p`.
    pub fn new(mach: R, gamma: R, t_inf: R, c_p: R) -> Self {
        Self {
            speed_field: "speed",
            t_tr_field: "T_tr",
            mach,
            gamma,
            t_inf,
            c_p,
        }
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for RecoveryTemperatureStage<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(speed) = field.scalar(self.speed_field) else {
            return Ok(());
        };
        let speed = speed.to_vec();
        // The mandatory post-shock stagnation temperature (uniform from the flight
        // condition); the spatial structure of T_tr comes from the speed field.
        let t_inf = Temperature::new(self.t_inf)?;
        let t_post = rankine_hugoniot_temperature_kernel(t_inf, self.mach, self.gamma)?;
        let mut t_tr = Vec::with_capacity(speed.len());
        for &u in &speed {
            t_tr.push(recovery_temperature_kernel(t_post, u, self.c_p)?.value());
        }
        field.set_scalar(self.t_tr_field, t_tr);
        Ok(())
    }
}

/// The Park two-temperature vibrational lag: turns the per-cell translational `T_tr` into the
/// **rate-controlling temperature** `Tₐ = √(T_tr·T_ve)` that actually governs ionization.
///
/// Behind the shock the heavy-particle translation jumps at once while the vibrational-electron
/// bath is still frozen at the free-stream value; it relaxes up on the Millikan-White clock
/// `τ_vt ∝ 1/p`. The sheath is continuously renewed by fresh free-stream parcels, so each step
/// this stage evaluates the closed-form relaxation over one **residence time** (`t_res =
/// standoff/u₂`, the parcel-renewal picture), per cell against that cell's `T_tr`. It writes the
/// lagged `"T_ve"` and the controller `"T_a"`; feed the latter to
/// [`IonizationStage::driven_by`]. A no-op if `"T_tr"` is absent.
///
/// This is the marched form of the calibrated stagnation-line closure
/// (`Park2tClosure`/`stagnation_line_blackout_2t`), which lands the RAM-C II peak within the
/// production chemistry spread.
///
/// # References
/// * Park, "Nonequilibrium Hypersonic Aerothermodynamics," Wiley (1990) — the two-temperature
///   model and the geometric-mean rate-controlling temperature.
/// * Park, J. Thermophys. Heat Transfer 7(3):385 (1993).
/// * Millikan & White, J. Chem. Phys. 39:3209 (1963) — the `τ_vt` correlation.
#[derive(Debug, Clone, Copy)]
pub struct VibrationalLagStage<R: CfdScalar> {
    t_tr_field: &'static str,
    t_ve_field: &'static str,
    t_a_field: &'static str,
    t_ve_initial: R,
    pressure_atm: R,
    reduced_mass_amu: R,
    theta_vib: R,
    residence_time: R,
}

impl<R: CfdScalar> VibrationalLagStage<R> {
    /// A lag stage with the gas properties the Millikan-White relaxation needs: the frozen
    /// initial bath temperature `t_ve_initial` (the free-stream value), the post-shock pressure
    /// in atm, the reduced mass of the dominant colliding pair in amu (N₂-N₂ ≈ 7), the
    /// characteristic vibrational temperature in K (N₂ ≈ 3393), and the sheath residence time
    /// `t_res = standoff/u₂` in s.
    pub fn new(
        t_ve_initial: R,
        pressure_atm: R,
        reduced_mass_amu: R,
        theta_vib: R,
        residence_time: R,
    ) -> Self {
        Self {
            t_tr_field: "T_tr",
            t_ve_field: "T_ve",
            t_a_field: "T_a",
            t_ve_initial,
            pressure_atm,
            reduced_mass_amu,
            theta_vib,
            residence_time,
        }
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for VibrationalLagStage<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(t_tr) = field.scalar(self.t_tr_field) else {
            return Ok(());
        };
        let t_tr = t_tr.to_vec();
        let mut t_ve = Vec::with_capacity(t_tr.len());
        let mut t_a = Vec::with_capacity(t_tr.len());
        for &t in &t_tr {
            let relaxed = vibrational_relaxation_kernel(
                VibrationalTemperature::new(self.t_ve_initial)?,
                Temperature::new(t)?,
                self.pressure_atm,
                self.reduced_mass_amu,
                self.theta_vib,
                self.residence_time,
            )?
            .value();
            t_ve.push(relaxed);
            t_a.push((t * relaxed).sqrt());
        }
        field.set_scalar(self.t_ve_field, t_ve);
        field.set_scalar(self.t_a_field, t_a);
        Ok(())
    }
}

/// Relaxes the carried ionization fraction `α` toward the Park-2T Saha surrogate
/// `α_eq(T_tr)` with `τ_ion = 1/(k_f·[M])` (the dominant associative-ionization
/// rate, computed from state), via the closed-form LER exponential, then writes the
/// electron density `n_e = α · n_tot`. Reads `"T_tr"` by default (see
/// [`driven_by`](Self::driven_by)), carries `"alpha"`, writes `"n_e"`.
#[derive(Debug, Clone, Copy)]
pub struct IonizationStage<R: CfdScalar> {
    t_tr_field: &'static str,
    alpha_field: &'static str,
    ne_field: &'static str,
    number_density: R,
    sheath_renewal: Option<R>,
}

impl<R: CfdScalar> IonizationStage<R> {
    /// Drive ionization at the configured total heavy-particle number density
    /// `number_density` (m⁻³).
    pub fn new(number_density: R) -> Self {
        Self {
            t_tr_field: "T_tr",
            alpha_field: "alpha",
            ne_field: "n_e",
            number_density,
            sheath_renewal: None,
        }
    }

    /// Drive ionization off a different temperature field, e.g. the Park rate-controlling
    /// `"T_a"` a [`VibrationalLagStage`] writes (both the Saha target and the associative
    /// ionization rate then use the lagged controller instead of the hot translation).
    pub fn driven_by(mut self, field: &'static str) -> Self {
        self.t_tr_field = field;
        self
    }

    /// Sheath renewal (the parcel picture): each step the sheath is refreshed by free-stream
    /// parcels whose chemistry has run for one **residence time**, so `α` is the
    /// residence-limited lag value `α_eq·(1 − e^{−t_res/τ_ion})` instead of an accumulating
    /// relaxation. This is the marched form of the calibrated stagnation-line lag
    /// (`stagnation_line_blackout_2t`); without it a carried parcel ionizes for the whole march
    /// and reaches equilibrium regardless of the lag.
    pub fn with_sheath_renewal(mut self, residence_time: R) -> Self {
        self.sheath_renewal = Some(residence_time);
        self
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for IonizationStage<R> {
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(t_tr) = field.scalar(self.t_tr_field) else {
            return Ok(());
        };
        let t_tr = t_tr.to_vec();
        let n = t_tr.len();
        let n_tot = self.number_density;

        // [M] in mol·cm⁻³ for τ_ion = 1/(k_f·[M]); k_f is in cm³·mol⁻¹·s⁻¹ (RP-1232).
        let avogadro = R::from_f64(AVOGADRO_CONSTANT).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(AVOGADRO_CONSTANT) failed".into())
        })?;
        let cm3_per_m3 = R::from_f64(1.0e6)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(1e6) failed".into()))?;
        let conc_mol_cm3 = n_tot / (avogadro * cm3_per_m3);
        // A frozen-chemistry timescale ≫ dt: when the forward rate vanishes the LER
        // step leaves α effectively unchanged (no spurious jump to equilibrium).
        let huge = R::from_f64(1.0e30).unwrap_or_else(R::one);
        let frozen_tau = ctx.dt() * huge;

        let mut targets = Vec::with_capacity(n);
        let mut taus = Vec::with_capacity(n);
        for &t in &t_tr {
            let temp = Temperature::new(t)?;
            let alpha_eq = park2t_ionization_surrogate_kernel(temp, n_tot)?.value();
            let k_f = arrhenius_rate_kernel(
                temp,
                R::from_f64(PARK_NO_IONIZATION_PREFACTOR).ok_or_else(|| {
                    PhysicsError::NumericalInstability("R::from_f64(prefactor) failed".into())
                })?,
                R::from_f64(PARK_NO_IONIZATION_EXPONENT).ok_or_else(|| {
                    PhysicsError::NumericalInstability("R::from_f64(exponent) failed".into())
                })?,
                R::from_f64(PARK_NO_IONIZATION_ACTIVATION_TEMP).ok_or_else(|| {
                    PhysicsError::NumericalInstability("R::from_f64(activation) failed".into())
                })?,
            )?
            .value();
            // τ_ion = 1/(k_f·[M]); a vanishing forward rate is frozen chemistry (τ ≫ dt),
            // not instant equilibrium.
            let denom = k_f * conc_mol_cm3;
            let tau = if denom > R::zero() {
                R::one() / denom
            } else {
                frozen_tau
            };
            targets.push(alpha_eq);
            taus.push(tau);
        }

        // Sheath renewal: the residence-limited lag value replaces the accumulating relaxation
        // (fresh parcels each step, each exposed for one t_res).
        if let Some(t_res) = self.sheath_renewal {
            let alpha: Vec<R> = targets
                .iter()
                .zip(&taus)
                .map(|(&a_eq, &tau)| {
                    if tau <= R::zero() {
                        a_eq
                    } else {
                        a_eq * (R::one() - (-(t_res / tau)).exp())
                    }
                })
                .collect();
            let n_e: Vec<R> = alpha.iter().map(|&a| a * n_tot).collect();
            field.set_scalar(self.alpha_field, alpha);
            field.set_scalar(self.ne_field, n_e);
            return Ok(());
        }

        // Seed the carried fraction on first contact (cold start at α = 0).
        if field.scalar(self.alpha_field).is_none() {
            field.set_scalar(self.alpha_field, vec![R::zero(); n]);
        }
        ler_relax_scalar(field, self.alpha_field, ctx.dt(), &targets, &taus)?;

        // Write back n_e = α · n_tot.
        let alpha = field
            .scalar(self.alpha_field)
            .map(|a| a.to_vec())
            .unwrap_or_default();
        let n_e: Vec<R> = alpha.iter().map(|&a| a * n_tot).collect();
        field.set_scalar(self.ne_field, n_e);
        Ok(())
    }
}

/// A two-temperature ideal-gas pressure closure `p = n·k_B·T_tr` written into a
/// per-cell `"pressure"` scalar — the interface the Tier-B compressible marcher
/// reuses. On the incompressible Tier-A rollout the marcher does not read it, so the
/// in-scope ambient effect is intentionally limited. Reads `"T_tr"`, writes `"pressure"`.
#[derive(Debug, Clone, Copy)]
pub struct EosStage<R: CfdScalar> {
    t_tr_field: &'static str,
    pressure_field: &'static str,
    number_density: R,
}

impl<R: CfdScalar> EosStage<R> {
    /// Close the pressure at the configured number density `number_density` (m⁻³).
    pub fn new(number_density: R) -> Self {
        Self {
            t_tr_field: "T_tr",
            pressure_field: "pressure",
            number_density,
        }
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for EosStage<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(t_tr) = field.scalar(self.t_tr_field) else {
            return Ok(());
        };
        let kb = R::from_f64(BOLTZMANN_CONSTANT).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(BOLTZMANN_CONSTANT) failed".into())
        })?;
        let n_tot = self.number_density;
        let pressure: Vec<R> = t_tr.iter().map(|&t| n_tot * kb * t).collect();
        field.set_scalar(self.pressure_field, pressure);
        Ok(())
    }
}

/// The blackout classification at a point: the (angular) plasma frequency and whether the link is
/// denied (plasma frequency above the configured comms band).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BlackoutState<R: CfdScalar> {
    /// Angular plasma frequency `ω_p` (rad/s).
    pub plasma_frequency: R,
    /// GNSS / comms link denied (the plasma frequency exceeds the comms band).
    pub denied: bool,
}

/// Maps an electron density to a blackout decision: `n_e → ω_p` (the plasma-frequency kernel) →
/// compare to the **configured** comms band → GNSS/comms-denied flag. The canonical causal-monad
/// seam: [`classify`](Self::classify) returns a `PropagatingEffect` (matching the crate's other
/// `PropagatingEffect` wrappers). The comparison threshold is config; the plasma frequency it
/// compares is computed from state.
#[derive(Debug, Clone, Copy)]
pub struct BlackoutTrigger<R: CfdScalar> {
    comms_band_rad_s: R,
}

impl<R: CfdScalar> BlackoutTrigger<R> {
    /// A trigger that denies the link when the angular plasma frequency exceeds `comms_band_rad_s`.
    pub fn new(comms_band_rad_s: R) -> Self {
        Self { comms_band_rad_s }
    }

    /// Classify an electron density into a [`BlackoutState`] (plain `Result`). A non-positive `n_e`
    /// (no plasma) leaves the link available; otherwise the plasma frequency is compared to the band.
    pub fn evaluate(&self, n_e: ElectronDensity<R>) -> Result<BlackoutState<R>, PhysicsError> {
        if n_e.value() <= R::zero() {
            return Ok(BlackoutState {
                plasma_frequency: R::zero(),
                denied: false,
            });
        }
        let omega_p = plasma_frequency_kernel(n_e)?;
        Ok(BlackoutState {
            plasma_frequency: omega_p.value(),
            denied: omega_p.value() > self.comms_band_rad_s,
        })
    }

    /// The causal-monad form: classify into a `PropagatingEffect<BlackoutState>`.
    pub fn classify(&self, n_e: ElectronDensity<R>) -> PropagatingEffect<BlackoutState<R>> {
        match self.evaluate(n_e) {
            Ok(state) => PropagatingEffect::pure(state),
            Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
        }
    }
}
