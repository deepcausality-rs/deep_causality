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
    fn test_unit_manifold_and_divergence_dimensions() {
        let manifold = unit_manifold::<f64>(4);
        let n1 = manifold.complex().num_cells(1);
        let div = divergence(&manifold, &random_cochain::<f64>(n1, 1));
        assert_eq!(div.len(), manifold.complex().num_cells(0));
    }
}
