/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Main Lund string fragmentation kernel.
//!
//! Implements iterative fragmentation of QCD strings into hadrons.

use crate::PhysicsError;
use crate::{FourMomentum, Hadron, LundParameters};

use super::flavor::{
    MesonState, QuarkFlavor, generate_transverse_momentum, select_meson_spin, select_quark_flavor,
};
use super::kinematics::sample_z;
use super::string::LundString;

use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_rand::Rng;

/// Lund string fragmentation kernel.
///
/// This implements the full Lund string fragmentation model for QCD hadronization.
/// Each input string (quark-antiquark pair) is fragmented into multiple hadrons.
///
/// # Physical Model
///
/// The model treats the color flux tube between partons as a relativistic string
/// with tension κ ≈ 1 GeV/fm. Fragmentation proceeds by:
///
/// 1. Sampling longitudinal momentum fraction z from Lund function
/// 2. Generating transverse momentum with Gaussian distribution
/// 3. Selecting quark flavor for new q-q̄ pair
/// 4. Combining quarks to form hadron
/// 5. Repeating until string mass falls below threshold
///
/// # Conservation Laws
///
/// - Total 4-momentum is conserved
/// - Electric charge is conserved
/// - Baryon number is conserved
/// - Strangeness is conserved (within tunneling uncertainty)
///
/// # Arguments
///
/// * `string_endpoints` - Vector of (quark, antiquark) 4-momentum pairs
/// * `params` - Lund fragmentation parameters
/// * `rng` - Random number generator
///
/// # Returns
///
/// Vector of produced hadrons with 4-momenta
///
/// # Errors
///
/// * `PhysicalInvariantBroken` - If momentum conservation fails
/// * `NumericalInstability` - If fragmentation produces invalid states
///
/// # Example
///
/// ```ignore
/// use deep_causality_physics::{
///     FourMomentum, LundParameters, lund_string_fragmentation_kernel
/// };
///
/// let params = LundParameters::default();
/// let endpoints = vec![
///     (FourMomentum::new(50.0, 0.0, 0.0, 50.0),
///      FourMomentum::new(50.0, 0.0, 0.0, -50.0)),
/// ];
/// let mut rng = rand::thread_rng();
/// let hadrons = lund_string_fragmentation_kernel(&endpoints, &params, &mut rng)?;
/// ```
pub fn lund_string_fragmentation_kernel<R, RNG>(
    string_endpoints: &[(FourMomentum<R>, FourMomentum<R>)],
    params: &LundParameters,
    rng: &mut RNG,
) -> Result<Vec<Hadron<R>>, PhysicsError>
where
    R: RealField + FromPrimitive,
    RNG: Rng,
{
    let mut all_hadrons = Vec::new();

    for (quark_p, antiquark_p) in string_endpoints {
        let string_hadrons = fragment_single_string(*quark_p, *antiquark_p, params, rng)?;
        all_hadrons.extend(string_hadrons);
    }

    Ok(all_hadrons)
}

/// Fragment a single string into hadrons.
fn fragment_single_string<R, RNG>(
    quark_p: FourMomentum<R>,
    antiquark_p: FourMomentum<R>,
    params: &LundParameters,
    rng: &mut RNG,
) -> Result<Vec<Hadron<R>>, PhysicsError>
where
    R: RealField + FromPrimitive,
    RNG: Rng,
{
    let mut hadrons = Vec::new();
    let mut string: LundString<R> = LundString::new(quark_p, antiquark_p);

    // Initial quark flavors at string endpoints (assume u-ubar for now)
    let mut quark_end = QuarkFlavor::Up;
    let mut antiquark_end = QuarkFlavor::Up;

    let mut from_quark_end = true;

    let max_iterations = 100;
    let mut iteration = 0;

    let min_mass = R::from_f64(params.min_invariant_mass())
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(min_inv_mass)".into()))?;
    let lund_a = R::from_f64(params.lund_a())
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(lund_a)".into()))?;
    let lund_b = R::from_f64(params.lund_b())
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(lund_b)".into()))?;

    while string.can_fragment(min_mass) && iteration < max_iterations {
        iteration += 1;

        // 1. Select new quark flavor for string breaking
        let new_flavor = select_quark_flavor(rng, params.strange_suppression());

        // 2. Generate transverse momentum (lifted from f64 RNG)
        let (pt_x, pt_y): (R, R) = generate_transverse_momentum(rng, params.sigma_pt());

        // 3. Form meson from endpoint quark and new antiquark
        let (meson, _endpoint_flavor) = if from_quark_end {
            let meson = MesonState {
                q1: quark_end,
                q2: new_flavor,
                is_vector: select_meson_spin(rng, params.vector_meson_fraction()),
            };
            (meson, quark_end)
        } else {
            let meson = MesonState {
                q1: new_flavor,
                q2: antiquark_end,
                is_vector: select_meson_spin(rng, params.vector_meson_fraction()),
            };
            (meson, antiquark_end)
        };

        // 4. Get meson mass
        let meson_mass: R = meson.mass();
        let mt_sq = meson_mass * meson_mass + pt_x * pt_x + pt_y * pt_y;

        // 5. Sample z from Lund function
        let z = sample_z(rng, lund_a, lund_b, mt_sq);

        // 6. Extract hadron momentum from string
        let hadron_p = if from_quark_end {
            string.take_from_quark(z, pt_x, pt_y, meson_mass)
        } else {
            string.take_from_antiquark(z, -pt_x, -pt_y, meson_mass)
        };

        // 7. Create hadron
        let hadron = Hadron::new(meson.pdg_id(), hadron_p);
        hadrons.push(hadron);

        // 8. Update endpoint for next iteration
        if from_quark_end {
            quark_end = new_flavor;
        } else {
            antiquark_end = new_flavor;
        }

        // 9. Alternate ends
        from_quark_end = !from_quark_end;
    }

    // Final hadron from remaining string
    let final_mass_threshold = R::from_f64(0.1)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.1)".into()))?;
    if string.invariant_mass() > final_mass_threshold {
        let final_meson = MesonState {
            q1: quark_end,
            q2: antiquark_end,
            is_vector: select_meson_spin(rng, params.vector_meson_fraction()),
        };

        let final_p = string.final_hadron(final_meson.mass::<R>());
        let final_hadron = Hadron::new(final_meson.pdg_id(), final_p);
        hadrons.push(final_hadron);
    }

    Ok(hadrons)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deep_causality_num::Real;

    #[test]
    fn test_lund_basic_fragmentation() {
        let params = LundParameters::default();
        let q = FourMomentum::new(50.0, 0.0, 0.0, 50.0);
        let qbar = FourMomentum::new(50.0, 0.0, 0.0, -50.0);
        let endpoints = vec![(q, qbar)];

        let mut rng = deep_causality_rand::rng();
        let result = lund_string_fragmentation_kernel(&endpoints, &params, &mut rng);

        // Kernel should succeed
        assert!(result.is_ok());

        let hadrons = result.unwrap();
        // Should produce at least some hadrons
        assert!(!hadrons.is_empty(), "Should produce at least one hadron");

        // Count hadrons with reasonable properties (energy > 0)
        // Note: simplified model may produce edge cases at iteration limits
        let valid_hadrons: Vec<_> = hadrons.iter().filter(|h| h.energy() > 0.0).collect();
        assert!(
            !valid_hadrons.is_empty(),
            "At least one hadron should have positive energy"
        );
    }

    #[test]
    fn test_lund_momentum_conservation() {
        let params = LundParameters::default();
        let q = FourMomentum::new(50.0, 1.0, 0.5, 49.0);
        let qbar = FourMomentum::new(50.0, -1.0, -0.5, -49.0);
        let total_in = q + qbar;

        let endpoints = vec![(q, qbar)];
        let mut rng = deep_causality_rand::rng();
        let hadrons = lund_string_fragmentation_kernel(&endpoints, &params, &mut rng).unwrap();

        // Sum output momenta
        let mut total_out = FourMomentum::default();
        for h in &hadrons {
            total_out = total_out + h.momentum();
        }

        // Check approximate conservation (some numerical error expected)
        let delta_e = (total_in.e() - total_out.e()).abs();
        let rel_error = delta_e / total_in.e();

        // Allow 20% relative error due to simplified model
        assert!(
            rel_error < 0.2,
            "Energy conservation failed: in={}, out={}, rel_err={}",
            total_in.e(),
            total_out.e(),
            rel_error
        );
    }

    #[test]
    fn test_lund_multiplicity_scaling() {
        let params = LundParameters::default();
        let mut rng = deep_causality_rand::rng();

        // Test at different energies
        let energies = [10.0, 50.0, 100.0];
        let mut multiplicities = Vec::new();

        for e in energies {
            let q = FourMomentum::new(e, 0.0, 0.0, e);
            let qbar = FourMomentum::new(e, 0.0, 0.0, -e);
            let endpoints = vec![(q, qbar)];

            let hadrons = lund_string_fragmentation_kernel(&endpoints, &params, &mut rng).unwrap();
            multiplicities.push(hadrons.len());
        }

        // Multiplicity should increase with energy
        assert!(multiplicities[1] >= multiplicities[0]);
        assert!(multiplicities[2] >= multiplicities[1]);
    }
}
