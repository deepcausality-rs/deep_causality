/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Finite-rate ionization network kernels: the two-way channel set that turns
//! the corridor's electron density from a calibrated closure into a
//! prediction from published rate data.
//!
//! All rates come from Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990),
//! Table II (`papers/gupta_1990_nasa_rp1232.pdf`, page 46), which pairs each
//! forward rate with its backward rate through the source's detailed-balance
//! relation `k_b = k_f / K_eq` (its eq. 5a) and states the pairs are valid
//! for flight velocities up to about 8 km/s. The equilibrium constant of a
//! pair is therefore `K_eq = k_f / k_b` from one table row: the network's
//! fixed point recovers the source-consistent equilibrium by construction
//! in the thermal-equilibrium limit. At genuine two-temperature states the
//! channels are rated at their controlling temperatures (heavy-particle
//! channels at the Park controller, electron channels at the electron
//! temperature) and the fixed point deliberately departs from any
//! single-temperature equilibrium.

use crate::constants::{
    rp1232_ei_n_activation_temp, rp1232_ei_n_exponent, rp1232_ei_n_prefactor,
    rp1232_ei_o_activation_temp, rp1232_ei_o_exponent, rp1232_ei_o_prefactor,
    rp1232_n2_diss_activation_temp, rp1232_n2_diss_exponent, rp1232_n2_diss_prefactor,
    rp1232_n2_recomb_exponent, rp1232_n2_recomb_prefactor, rp1232_no_dr_activation_temp,
    rp1232_no_dr_exponent, rp1232_no_dr_prefactor, rp1232_o2_diss_activation_temp,
    rp1232_o2_diss_exponent, rp1232_o2_diss_prefactor, rp1232_o2_recomb_exponent,
    rp1232_o2_recomb_prefactor,
};
use crate::kernels::hypersonic::thermochemistry::arrhenius_rate_kernel;
use crate::{
    DissociationFraction, ElectronDensity, ElectronTemperature, EquilibriumConstant, PhysicsError,
    ReactionRate, Temperature,
};
use deep_causality_num::{FromPrimitive, RealField};

/// Lift a plain numeral into `R` (model coefficients come through the
/// real-field constant accessors in `constants::hypersonic`).
fn lift<R: RealField + FromPrimitive>(x: f64, what: &str) -> Result<R, PhysicsError> {
    R::from_f64(x)
        .ok_or_else(|| PhysicsError::NumericalInstability(format!("R::from_f64({what}) failed")))
}

/// Dissociative recombination `NO⁺ + e⁻ → N + O`: the two-body backward rate
/// of the associative-ionization channel, rated at the **electron
/// temperature** (barrier-free; `k_b = 1.80e19 · T_e⁻¹` cm³·mol⁻¹·s⁻¹). This
/// is the physical blackout-exit mechanism: in cold dense air it decays a
/// carried electron population toward the local (low) equilibrium.
///
/// # References
/// * Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990), Table II, reaction 7
///   backward (`papers/gupta_1990_nasa_rp1232.pdf`).
pub fn no_dissociative_recombination_rate_kernel<R>(
    electron_temperature: ElectronTemperature<R>,
) -> Result<ReactionRate<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    arrhenius_rate_kernel(
        Temperature::new(electron_temperature.value())?,
        rp1232_no_dr_prefactor::<R>(),
        rp1232_no_dr_exponent::<R>(),
        rp1232_no_dr_activation_temp::<R>(),
    )
}

/// Thresholded electron-impact ionization `N + e⁻ → N⁺ + 2e⁻`, rated at the
/// **electron temperature** (`k_f = 1.1e32 · T_e⁻³·¹⁴ · exp(−1.69e5/T_e)`
/// cm³·mol⁻¹·s⁻¹). Table II states a ±36 percent spread on the prefactor and
/// the source notes these expansion-flow rates tend to be lower than
/// compressive-flow data; the validation band absorbs both.
///
/// # References
/// * Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990), Table II, reaction 9
///   forward (`papers/gupta_1990_nasa_rp1232.pdf`).
pub fn electron_impact_ionization_n_rate_kernel<R>(
    electron_temperature: ElectronTemperature<R>,
) -> Result<ReactionRate<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    arrhenius_rate_kernel(
        Temperature::new(electron_temperature.value())?,
        rp1232_ei_n_prefactor::<R>(),
        rp1232_ei_n_exponent::<R>(),
        rp1232_ei_n_activation_temp::<R>(),
    )
}

/// Thresholded electron-impact ionization `O + e⁻ → O⁺ + 2e⁻`, rated at the
/// **electron temperature** (`k_f = 3.6e31 · T_e⁻²·⁹¹ · exp(−1.58e5/T_e)`
/// cm³·mol⁻¹·s⁻¹). Same data-quality caveats as the N channel.
///
/// # References
/// * Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990), Table II, reaction 8
///   forward (`papers/gupta_1990_nasa_rp1232.pdf`).
pub fn electron_impact_ionization_o_rate_kernel<R>(
    electron_temperature: ElectronTemperature<R>,
) -> Result<ReactionRate<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    arrhenius_rate_kernel(
        Temperature::new(electron_temperature.value())?,
        rp1232_ei_o_prefactor::<R>(),
        rp1232_ei_o_exponent::<R>(),
        rp1232_ei_o_activation_temp::<R>(),
    )
}

/// Concentration-basis equilibrium constant of N₂ dissociation
/// (`N₂ + M ⇌ 2N + M`) from the RP-1232 Table II pair: `K = k_f / k_b`, unit
/// mol·cm⁻³ (the backward rate is three-body). Detailed balance holds by
/// construction because both rates come from one table row.
///
/// # References
/// * Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990), Table II, reaction 2
///   (`papers/gupta_1990_nasa_rp1232.pdf`).
pub fn n2_dissociation_equilibrium_kernel<R>(
    temperature: Temperature<R>,
) -> Result<EquilibriumConstant<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let kf = arrhenius_rate_kernel(
        temperature,
        rp1232_n2_diss_prefactor::<R>(),
        rp1232_n2_diss_exponent::<R>(),
        rp1232_n2_diss_activation_temp::<R>(),
    )?;
    let kb = arrhenius_rate_kernel(
        temperature,
        rp1232_n2_recomb_prefactor::<R>(),
        rp1232_n2_recomb_exponent::<R>(),
        R::zero(),
    )?;
    EquilibriumConstant::new(kf.value() / kb.value())
}

/// Concentration-basis equilibrium constant of O₂ dissociation
/// (`O₂ + M ⇌ 2O + M`) from the RP-1232 Table II pair: `K = k_f / k_b`, unit
/// mol·cm⁻³.
///
/// # References
/// * Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990), Table II, reaction 1
///   (`papers/gupta_1990_nasa_rp1232.pdf`).
pub fn o2_dissociation_equilibrium_kernel<R>(
    temperature: Temperature<R>,
) -> Result<EquilibriumConstant<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let kf = arrhenius_rate_kernel(
        temperature,
        rp1232_o2_diss_prefactor::<R>(),
        rp1232_o2_diss_exponent::<R>(),
        rp1232_o2_diss_activation_temp::<R>(),
    )?;
    let kb = arrhenius_rate_kernel(
        temperature,
        rp1232_o2_recomb_prefactor::<R>(),
        rp1232_o2_recomb_exponent::<R>(),
        R::zero(),
    )?;
    EquilibriumConstant::new(kf.value() / kb.value())
}

/// Equilibrium dissociation fraction of a diatomic pool `A₂ ⇌ 2A` at fixed
/// nuclei density: solve `[A]²/[A₂] = K` with `[A₂] = (n_nuclei − [A])/2` for
/// the atom share `x = [A]/n_nuclei`. Closed form:
/// `[A] = (−K + √(K² + 8·K·n_nuclei))/4`. `k_eq` and `nuclei_density` must be
/// in one consistent concentration basis.
pub fn dissociation_equilibrium_fraction_kernel<R>(
    k_eq: EquilibriumConstant<R>,
    nuclei_density: R,
) -> Result<DissociationFraction<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if nuclei_density <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Nuclei density must be positive for the dissociation equilibrium".into(),
        ));
    }
    let k = k_eq.value();
    let four = lift::<R>(4.0, "4.0")?;
    let eight = lift::<R>(8.0, "8.0")?;
    let atoms = (-k + (k * k + eight * k * nuclei_density).sqrt()) / four;
    let x = atoms / nuclei_density;
    // Guard the [0, 1] invariant against rounding at the endpoints.
    let x = if x < R::zero() {
        R::zero()
    } else if x > R::one() {
        R::one()
    } else {
        x
    };
    DissociationFraction::new(x)
}

/// The network's closed-form fixed point: the electron concentration where
/// production balances loss, `β·x² − k_lin·x − p = 0` with quadratic loss
/// through quasi-neutrality (`n_NO⁺ ≈ n_e`), solved as
/// `x* = (k_lin + √(k_lin² + 4·β·p)) / (2β)`.
///
/// # Arguments
/// * `production` — the electron-independent production rate `p` (channel 1
///   forward, `k_f·[N]·[O]`), in the caller's concentration basis per second.
/// * `linear_coefficient` — the electron-linear production coefficient
///   `k_lin` (electron impact, `k₈·[O] + k₉·[N]`), s⁻¹.
/// * `loss_coefficient` — the quadratic loss coefficient `β` (dissociative
///   recombination), concentration⁻¹·s⁻¹.
///
/// The returned value is in the caller's concentration basis (the kernel is
/// basis-agnostic; the corridor stage works in mol·cm⁻³ and converts at its
/// boundaries).
pub fn finite_rate_ionization_fixed_point_kernel<R>(
    production: R,
    linear_coefficient: R,
    loss_coefficient: R,
) -> Result<ElectronDensity<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if production < R::zero() || linear_coefficient < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Production terms cannot be negative".into(),
        ));
    }
    if loss_coefficient <= R::zero() {
        return Err(PhysicsError::Singularity(
            "The quadratic loss coefficient must be positive (dissociative recombination is \
             barrier-free)"
                .into(),
        ));
    }
    let two = lift::<R>(2.0, "2.0")?;
    let four = lift::<R>(4.0, "4.0")?;
    let disc = linear_coefficient * linear_coefficient + four * loss_coefficient * production;
    let x = (linear_coefficient + disc.sqrt()) / (two * loss_coefficient);
    ElectronDensity::new(x)
}

/// Associative ionization `N + O → NO⁺ + e⁻` forward rate, rated at the
/// heavy-particle controlling temperature
/// (`k_f = 9.03e9 · T⁰·⁵ · exp(−3.24e4/T)` cm³·mol⁻¹·s⁻¹): the convenience
/// form of the shipped Arrhenius evaluation over the shipped reaction-7
/// constants, paired with
/// [`no_dissociative_recombination_rate_kernel`] as one Table II row.
///
/// # References
/// * Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990), Table II, reaction 7
///   forward (`papers/gupta_1990_nasa_rp1232.pdf`).
pub fn no_associative_ionization_rate_kernel<R>(
    temperature: Temperature<R>,
) -> Result<ReactionRate<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    arrhenius_rate_kernel(
        temperature,
        crate::constants::park_no_ionization_prefactor::<R>(),
        crate::constants::park_no_ionization_exponent::<R>(),
        crate::constants::park_no_ionization_activation_temp::<R>(),
    )
}

/// Zeldovich exchange `N₂ + O → NO + N` forward rate
/// (`k = 6.75e13 · exp(−3.75e4/T)` cm³·mol⁻¹·s⁻¹): the low-activation
/// N-atom production path (37,500 K barrier against direct dissociation's
/// 113,100 K) that feeds associative ionization while direct N₂
/// dissociation is still frozen. Rated at the heavy-particle controlling
/// temperature.
///
/// # References
/// * Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990), Table II, reaction 6
///   forward (`papers/gupta_1990_nasa_rp1232.pdf`).
pub fn zeldovich_exchange_rate_kernel<R>(
    temperature: Temperature<R>,
) -> Result<ReactionRate<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    arrhenius_rate_kernel(
        temperature,
        crate::constants::rp1232_zeldovich_prefactor::<R>(),
        crate::constants::rp1232_zeldovich_exponent::<R>(),
        crate::constants::rp1232_zeldovich_activation_temp::<R>(),
    )
}

/// Park's controlling temperature `T_q = T_tr^q · T_v^(1−q)` for the given
/// exponent `q` (the dissociation closure uses [`PARK_DISSOCIATION_Q`]
/// = 0.7, Park 1990; the ionization controller keeps the geometric mean).
/// The controlling-temperature choice is the largest closure divergence
/// among production codes; the exponent here is the Park lineage's own
/// published value, a citation rather than a fit.
///
/// [`PARK_DISSOCIATION_Q`]: crate::constants::PARK_DISSOCIATION_Q
///
/// # References
/// * Park, "Nonequilibrium Hypersonic Aerothermodynamics," Wiley (1990).
pub fn park_controlling_temperature_kernel<R>(
    t_translational: Temperature<R>,
    t_vibrational: Temperature<R>,
    q: R,
) -> Result<Temperature<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    // An inclusion test rather than an exclusion test: NaN fails `>=`, so a
    // NaN exponent is rejected here instead of propagating as Ok(NaN).
    let q_in_unit_interval = q >= R::zero() && q <= R::one();
    if !q_in_unit_interval {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "The controlling-temperature exponent q must lie in [0, 1]".into(),
        ));
    }
    let t = t_translational.value();
    let tv = t_vibrational.value();
    Temperature::new(t.powf(q) * tv.powf(R::one() - q))
}
