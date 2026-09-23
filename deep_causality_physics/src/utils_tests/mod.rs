/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared test fixtures for the physics test suite.
//!
//! These helpers build discrete manifolds and cochains used by the
//! `SolenoidalField` tests. They live in the source tree (rather than inside
//! the `tests/` folder) because the Bazel `rust_test_suite` compiles each
//! `*_tests.rs` file as a standalone crate, so test files cannot share helpers
//! across one another — only the crate under test is visible to all of them.

use alloc::{vec, vec::Vec};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

/// A 2D square-torus lattice manifold of side `n` with unit cubical metric.
pub fn unit_manifold<R>(n: usize) -> Manifold<LatticeComplex<2, R>, R>
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let lattice: LatticeComplex<2, R> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![R::zero(); total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, R> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// A deterministic pseudo-random cochain of length `len` in `[-1, 1]`.
pub fn random_cochain<R: RealField + FromPrimitive>(len: usize, seed: u64) -> Vec<R> {
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let unit = (state >> 11) as f64 / (1u64 << 53) as f64;
            R::from_f64(2.0 * unit - 1.0).expect("[-1,1] lifts")
        })
        .collect()
}

/// Discrete divergence of an edge cochain: place at grade 1, apply δ.
pub fn divergence<R>(manifold: &Manifold<LatticeComplex<2, R>, R>, one_form: &[R]) -> Vec<R>
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let lattice = LatticeComplex::<2, R>::square_torus(
        // shape is square by fixture construction
        manifold.complex().shape()[0],
    );
    let total: usize = (0..=2).map(|g| lattice.num_cells(g)).sum();
    let n0 = lattice.num_cells(0);
    let mut data = vec![R::zero(); total];
    data[n0..n0 + one_form.len()].copy_from_slice(one_form);
    let tensor = CausalTensor::new(data, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, R> = CubicalReggeGeometry::unit();
    let m = Manifold::from_cubical_with_metric(lattice, tensor, metric, 0);
    m.codifferential(1).as_slice().to_vec()
}

/// Supremum (max-abs) norm of a slice.
pub fn sup_norm<R: RealField>(v: &[R]) -> R {
    v.iter()
        .map(|x| x.abs())
        .fold(R::zero(), |m, x| if x > m { x } else { m })
}

// =================================================================================================
// Chronometric fixtures
// =================================================================================================

/// The 1PN clock-rate model for a J2-corrected monopole, used to forward-model a
/// `SpaceTimeCoordinate` from a chosen `GM`.
///
/// `1/r_eff = 1/r - J2 R_eq^2 P2(cos theta) / r^3` and `drift = Phi/c^2 - v^2/(2 c^2)` with
/// `Phi = -GM / r_eff`. At `J2 = 0` this is exactly
/// [`crate::relativistic_clock_drift_rate_kernel`], which is checked against the published GPS
/// relativistic split; `solve_gm_tests.rs` asserts that agreement.
///
/// It lives here because the Bazel test suite compiles each `*_tests.rs` as its own crate, so the
/// two chronometric test files cannot otherwise share it and were carrying a copy each.
pub fn chronometric_forward_drift_rate(
    target_gm: f64,
    r: f64,
    v: f64,
    z: f64,
    body: &crate::CentralBody<f64>,
) -> f64 {
    let cos_theta = z / r;
    let legendre_p2 = 0.5 * (3.0 * cos_theta * cos_theta - 1.0);
    let r_cubed = r * r * r;
    let req_sq = body.equatorial_radius_m * body.equatorial_radius_m;
    let inv_r_eff = 1.0 / r - body.j2 * req_sq * legendre_p2 / r_cubed;
    let phi = -target_gm * inv_r_eff;
    let c_sq = crate::SPEED_OF_LIGHT * crate::SPEED_OF_LIGHT;
    phi / c_sq - 0.5 * v * v / c_sq
}

/// A `SpaceTimeCoordinate` whose `clock_drift_rate` is forward-modelled from `target_gm` by
/// [`chronometric_forward_drift_rate`].
pub fn chronometric_build_coord(
    target_gm: f64,
    r: f64,
    v: f64,
    position: [f64; 3],
    velocity: [f64; 3],
    body: &crate::CentralBody<f64>,
) -> crate::SpaceTimeCoordinate<f64> {
    crate::SpaceTimeCoordinate::<f64> {
        timestamp: 0,
        sat_id: 0,
        r_m: r,
        v_ms: v,
        clock_bias_s: 0.0,
        position,
        velocity,
        clock_drift_rate: chronometric_forward_drift_rate(target_gm, r, v, position[2], body),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sup_norm() {
        assert_eq!(sup_norm(&[-3.0_f64, 1.0, 2.0]), 3.0);
        assert_eq!(sup_norm::<f64>(&[]), 0.0);
    }

    #[test]
    fn test_random_cochain_is_bounded_and_deterministic() {
        let a = random_cochain::<f64>(16, 7);
        let b = random_cochain::<f64>(16, 7);
        assert_eq!(a, b);
        assert!(a.iter().all(|&x| (-1.0..=1.0).contains(&x)));
    }

    #[test]
    fn test_chronometric_forward_model_matches_the_shipped_kernel_without_j2() {
        // At J2 = 0 the model reduces to Phi/c^2 - v^2/(2c^2), which is the shipped kernel.
        let body = crate::CentralBody::<f64>::new(crate::EARTH_GM, 6.378e6, 0.0);
        for (r, v) in [(6.378e6_f64, 0.0_f64), (2.6561e7, 3.874e3)] {
            let ours = chronometric_forward_drift_rate(crate::EARTH_GM, r, v, 0.0, &body);
            let shipped =
                crate::relativistic_clock_drift_rate_kernel(r, v, crate::EARTH_GM).unwrap();
            assert!((ours - shipped).abs() <= 1e-15 * shipped.abs().max(1e-18));
        }
    }

    #[test]
    fn test_chronometric_build_coord_carries_the_forward_modelled_drift() {
        let body = crate::CentralBody::<f64>::new(crate::EARTH_GM, 6.378e6, 0.0);
        let coord = chronometric_build_coord(
            crate::EARTH_GM,
            2.93e7,
            3650.0,
            [2.93e7, 0.0, 0.0],
            [0.0, 3650.0, 0.0],
            &body,
        );
        assert_eq!(coord.r_m, 2.93e7);
        assert_eq!(coord.v_ms, 3650.0);
        assert_eq!(
            coord.clock_drift_rate,
            chronometric_forward_drift_rate(crate::EARTH_GM, 2.93e7, 3650.0, 0.0, &body)
        );
    }

    #[test]
    fn test_unit_manifold_and_divergence_dimensions() {
        let manifold = unit_manifold::<f64>(4);
        let n1 = manifold.complex().num_cells(1);
        let div = divergence(&manifold, &random_cochain::<f64>(n1, 1));
        assert_eq!(div.len(), manifold.complex().num_cells(0));
    }
}
