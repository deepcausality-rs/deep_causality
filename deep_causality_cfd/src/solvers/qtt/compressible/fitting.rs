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

use crate::CfdScalar;
use crate::tensor_bridge::quantize;
use alloc::vec;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::{
    AVOGADRO_CONSTANT, ElectronDensity, PARK_NO_IONIZATION_ACTIVATION_TEMP,
    PARK_NO_IONIZATION_EXPONENT, PARK_NO_IONIZATION_PREFACTOR, PhysicsError, Temperature,
    VibrationalTemperature, arrhenius_rate_kernel, electron_density_kernel,
    park2t_ionization_surrogate_kernel, plasma_frequency_kernel,
    rankine_hugoniot_temperature_kernel, vibrational_relaxation_kernel,
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

/// Park two-temperature ionization closure — the gas-property inputs that turn the *translational*
/// post-shock state into the **lagging vibrational-electron controller** that actually governs ionization.
///
/// Behind the shock the heavy-particle translational temperature `T_tr = T₂` jumps immediately, but the
/// vibrational / electronic / free-electron bath is still cold (frozen at the free-stream value) and
/// relaxes up over the residence time on the Millikan–White clock `τ_vt`. Ionization is a heavy-particle ↔
/// electron handshake, so Park drives it off the **rate-controlling temperature** `Tₐ = √(T_tr·T_ve)`, not
/// the hot translation. This struct carries the four gas properties the relaxation needs.
#[derive(Clone, Copy, Debug)]
pub struct Park2tClosure<R> {
    /// Free-stream (pre-shock) temperature `T_∞` — the frozen initial vibrational temperature `T_ve(0)`
    /// just behind the shock, before relaxation toward `T₂` begins.
    pub t_ve_initial: R,
    /// Post-shock pressure `p₂` in **atm** — sets the Millikan–White relaxation time `τ_vt ∝ 1/p`.
    pub pressure_atm: R,
    /// Reduced mass `μ_sr` of the dominant relaxing collision pair, in **amu**. For the RAM-C closure
    /// this is [`REDUCED_MASS_AMU`] for the N₂–N₂ pair (`μ = m(N₂)/2`). The value is not restated here;
    /// see that constant for the derivation, the pair selection, and the citation.
    pub reduced_mass_amu: R,
    /// Characteristic vibrational temperature `θ_v` of the dominant species, in **K** (N₂ ≈ 3393).
    pub theta_vib: R,
}

/// Standard atomic weight of nitrogen, in amu (IUPAC 2021: 14.007).
const N_ATOMIC_MASS_AMU: f64 = 14.007;
/// Molecular mass of the diatomic N₂, in amu: two nitrogen atoms. This is the mass the reduced-mass
/// formula needs. Substituting the *atomic* 14 here is the arithmetic slip that produced the meaningless
/// 7.0 (`14·14/28`), which is the N–N pair of two atoms.
const N2_MOLECULAR_MASS_AMU: f64 = 2.0 * N_ATOMIC_MASS_AMU;

/// Millikan–White reduced mass `μ_sr` of the dominant relaxing collision pair, in **amu**.
///
/// **Derived, not asserted.** `μ_sr = m_s·m_r / (m_s + m_r)` for the chosen **N₂–N₂** pair. Both
/// partners are the diatomic N₂ (`m = 28.014` amu), so `μ = m(N₂)/2 = 14.007` (14.0 to three figures,
/// which is what the change documents it as). The value follows from the two named masses above; it is
/// not a literal. A future edit to a mass without the value becomes a compile-time recomputation rather
/// than a comment nobody re-derives. [`reduced_mass_amu`] is the checked constructor, and it rejects the
/// monatomic species this pair's history motivates.
///
/// **The pair is a physical choice (design D2), not a default.** At the RAM-C II post-shock condition
/// (`M = 25`, `T₂ ≈ 8044 K`) the air is partially dissociated: roughly `x_N₂ ≈ 0.46`, `x_O ≈ 0.31`,
/// `x_N ≈ 0.23`. N₂–N₂ is therefore not automatically the dominant collision partner. The alternatives,
/// and the `τ·p` they imply at 8044 K:
///
/// | Pair  | μ (amu) | τ·p (atm·s) |
/// |-------|---------|-------------|
/// | N₂–N  | 9.33    | 6.9e-7 (shortest) |
/// | N₂–O  | 10.18   | 7.5e-7 |
/// | **N₂–N₂** | **14.007** | **1.02e-6 (longest live partner)** |
/// | N₂–O₂ | 14.93   | 1.08e-6 (O₂ fully dissociated at 8044 K, no population) |
///
/// The live partners at this condition are N₂, N and O; O₂ is fully dissociated by ~5000 K, so N₂–O₂,
/// though heavier, carries no population and is listed only for the `μ`-sensitivity. Among the live
/// partners N₂–N₂ is the largest `μ`. That gives the longest `τ_vt`, the coldest `T_ve`, and the lowest
/// predicted `n_e` of them, so the choice cannot be accused of chasing a verdict. The lighter live
/// partners (N₂–N) would shorten `τ_vt` and raise `n_e` toward the former headline; picking one would be
/// indistinguishable from back-fitting. N₂–N₂ is also the conventional
/// two-temperature-literature baseline.
///
/// **This single pair stands in for a mixed bath, and it biases `τ_vt` long (design D2a).** Lighter
/// partners relax faster, so pure N₂–N₂ is the slowest relaxation and the lowest `n_e` of the live
/// options. It therefore under-predicts electron density, which under-predicts blackout. That is the
/// optimistic-about-comms direction, the wrong way to be wrong for an avionics consumer. The faithful
/// form is the mixture-weighted `1/τ_mix = Σ_r x_r / τ_(s,r)` over the composition the harness already
/// computes. Replacing this stand-in is the named next step, not a standing disclaimer.
///
/// **Millikan–White form and units (design D1 citation).** `μ` enters the relaxation time as
/// `τ_vt·p = exp[A_sr·(T^{-1/3} − B_sr) − 18.42]`, with `A_sr = 1.16e-3·μ^{1/2}·θ_v^{4/3}` and
/// `B_sr = 0.015·μ^{1/4}`. Units: `μ` in amu, `p` in atm, `τ` in s, `θ_v` and `T` in K. The correlation
/// is implemented in [`deep_causality_physics::vibrational_relaxation_kernel`]; this constant is the
/// RAM-C closure's `μ` input. (Millikan & White, J. Chem. Phys. 39:3209, 1963; Park 1990 for the
/// natural-log constants.)
pub const REDUCED_MASS_AMU: f64 =
    N2_MOLECULAR_MASS_AMU * N2_MOLECULAR_MASS_AMU / (N2_MOLECULAR_MASS_AMU + N2_MOLECULAR_MASS_AMU);

/// The Millikan–White reduced mass `μ_sr = m_s·m_r / (m_s + m_r)` (amu) for a named collision pair. This
/// is the checked constructor behind [`REDUCED_MASS_AMU`].
///
/// The relaxing species `s` **must** be a diatomic (or polyatomic). A monatomic species has no
/// vibrational mode and cannot be the relaxing partner, so `relaxing_is_diatomic = false` is rejected
/// rather than assigned a `μ`. This is the guard the committed `7.0` lacked: 7.00 amu is the N–N pair,
/// two nitrogen atoms, which this constructor refuses because atomic nitrogen cannot relax vibrationally.
///
/// # Errors
/// [`PhysicsError::PhysicalInvariantBroken`] if the relaxing species is monatomic, or if either mass is
/// not strictly positive and finite.
pub fn reduced_mass_amu(
    relaxing_mass_amu: f64,
    partner_mass_amu: f64,
    relaxing_is_diatomic: bool,
) -> Result<f64, PhysicsError> {
    if !relaxing_is_diatomic {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "the relaxing species is monatomic and has no vibrational mode; it cannot be the \
             Millikan–White relaxing partner (this is why μ = 7.0, the N–N atomic pair, is invalid)"
                .into(),
        ));
    }
    if relaxing_mass_amu <= 0.0
        || partner_mass_amu <= 0.0
        || !relaxing_mass_amu.is_finite()
        || !partner_mass_amu.is_finite()
    {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "collision-pair masses must be strictly positive and finite".into(),
        ));
    }
    Ok(relaxing_mass_amu * partner_mass_amu / (relaxing_mass_amu + partner_mass_amu))
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
    /// [`Self::stagnation_blackout`], toward the RAM-C flight value. `residence_time` is `t_res` (s).
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

    /// The **Park two-temperature** stagnation-line blackout (the chemistry-fidelity upgrade, Gap-3): the
    /// physically faithful peak. Ionization is no longer evaluated at the hot translational `T₂` — it is
    /// driven off the **rate-controlling temperature** `Tₐ = √(T_tr·T_ve)`, where the lagging
    /// vibrational-electron temperature `T_ve` is relaxed from the free-stream value toward `T₂` over the
    /// residence time by the closed-form Landau–Teller / Millikan–White LER kernel. Both the Saha
    /// equilibrium target and the associative-ionization rate use `Tₐ`, so the cold electron bath suppresses
    /// the equilibrium the single-temperature surrogate over-counted. Under the corrected N₂–N₂ closure
    /// ([`REDUCED_MASS_AMU`], `μ = 14.007`) the controller `α` falls `4.6×10⁻³ → ~2×10⁻⁵` and peak `n_e`
    /// lands `5.31e17`, 1.27 decades below the RAM-C II anchor. That offset is reported, not re-admitted to
    /// the retired `+0.0`-decade headline the invalid `μ = 7.0` produced.
    ///
    /// `residence_time` is `t_res = standoff/u₂` (s); `closure` carries the gas properties the relaxation
    /// needs (free-stream `T_ve(0)`, post-shock pressure, reduced mass, `θ_v`). Returns the same outcome
    /// shape as [`Self::stagnation_line_blackout`], plus the controller is recorded in `ionization_fraction`.
    ///
    /// # Errors
    /// Propagates the vibrational-relaxation / ionization / rate / electron-density / plasma-frequency kernels.
    ///
    /// # References
    /// * Park, "Nonequilibrium Hypersonic Aerothermodynamics," Wiley (1990) — the two-temperature model and
    ///   the geometric-mean rate-controlling temperature `Tₐ = √(T_tr·T_ve)`.
    /// * Park, J. Thermophys. Heat Transfer 7(3):385 (1993).
    pub fn stagnation_line_blackout_2t(
        &self,
        post: &PostShockState<R>,
        residence_time: R,
        closure: &Park2tClosure<R>,
        comms_band: R,
    ) -> Result<StagnationOutcome<R>, PhysicsError> {
        let t_tr = Temperature::new(post.t2)?;

        // 1. Relax the lagging vibrational-electron temperature T_ve over the residence time (closed-form
        //    LER exponential): T_ve(0) = free-stream value (frozen behind the shock) → T_tr = T₂.
        let t_ve = vibrational_relaxation_kernel(
            VibrationalTemperature::new(closure.t_ve_initial)?,
            t_tr,
            closure.pressure_atm,
            closure.reduced_mass_amu,
            closure.theta_vib,
            residence_time,
        )?
        .value();

        // 2. Park rate-controlling temperature Tₐ = √(T_tr·T_ve) — the heavy-particle ↔ electron handshake.
        let t_a_val = (post.t2 * t_ve).sqrt();
        let t_a = Temperature::new(t_a_val)?;

        // 3. Saha equilibrium target at the *controller* Tₐ (not T_tr) — the cold electron bath suppresses it.
        let alpha_eq = park2t_ionization_surrogate_kernel(t_a, post.n_tot2)?.value();

        // 4. Associative-ionization lag, also at Tₐ. k_f in Park/Gupta units (cm³·mol⁻¹·s⁻¹) → SI per
        //    particle (m³·s⁻¹): ×1e-6 / N_A. τ_ion = 1/(k·n₂); α = α_eq·(1 − e^{−t_res/τ_ion}).
        let prefactor = R::from_f64(PARK_NO_IONIZATION_PREFACTOR)
            .ok_or_else(|| PhysicsError::NumericalInstability("Park prefactor".into()))?;
        let exponent = R::from_f64(PARK_NO_IONIZATION_EXPONENT)
            .ok_or_else(|| PhysicsError::NumericalInstability("Park exponent".into()))?;
        let theta_d = R::from_f64(PARK_NO_IONIZATION_ACTIVATION_TEMP)
            .ok_or_else(|| PhysicsError::NumericalInstability("Park activation temp".into()))?;
        let k_cgs = arrhenius_rate_kernel(t_a, prefactor, exponent, theta_d)?.value();
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
