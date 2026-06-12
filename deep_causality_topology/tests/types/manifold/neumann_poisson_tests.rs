/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the direct Neumann–Poisson path (neumann-poisson
//! capability): wall-bounded uniform boxes route the grade-0 solve
//! through DCT-I (wall axes) / DFT (periodic axes) transforms. Pinned
//! here: residual at rounding against the implemented boundary-corrected
//! `Δ₀` (the eigenbasis gate that decides DCT type), agreement with the
//! Jacobi-preconditioned CG fallback, the discrete no-penetration
//! property of the projection, and exactness under a starved iteration
//! budget (the direct path has no convergence-failure mode).

use deep_causality_sparse::cg_solve_preconditioned;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HasHodgeStar, LatticeComplex, Manifold,
};

fn manifold_with_metric<const D: usize>(
    lattice: LatticeComplex<D, f64>,
    metric: CubicalReggeGeometry<D, f64>,
) -> Manifold<LatticeComplex<D, f64>, f64> {
    let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn random_cochain(len: usize, seed: u64) -> Vec<f64> {
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let unit = (state >> 11) as f64 / (1u64 << 53) as f64;
            2.0 * unit - 1.0
        })
        .collect()
}

fn sup(v: impl IntoIterator<Item = f64>) -> f64 {
    v.into_iter().fold(0.0, |m, x| m.max(x.abs()))
}

fn subtract_mean(v: &mut [f64]) {
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    for x in v.iter_mut() {
        *x -= mean;
    }
}

/// `Δ₀φ` must reproduce the consistent RHS `δω` to rounding — the
/// eigenbasis/operator-match gate for the DCT-I choice.
fn assert_residual_at_rounding<const D: usize>(
    manifold: &Manifold<LatticeComplex<D, f64>, f64>,
    seed: u64,
    tol: f64,
) {
    let n0 = manifold.complex().num_cells(0);
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain(n1, seed), vec![n1]).unwrap();

    let mut rhs = manifold.codifferential_of(omega.as_slice(), 1).into_vec();
    rhs.resize(n0, 0.0);

    let projection = manifold.leray_project(&omega).expect("projection succeeds");
    let phi = projection.potential().as_slice();
    let mut residual = manifold.laplacian_of(phi, 0).into_vec();
    residual.resize(n0, 0.0);

    let err = sup(residual.iter().zip(rhs.iter()).map(|(a, b)| a - b));
    assert!(err < tol, "residual {err:e} exceeds {tol:e}");
}

#[test]
fn residual_at_rounding_all_walls_2d() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::open([6, 6]),
        CubicalReggeGeometry::unit(),
    );
    assert_residual_at_rounding(&m, 101, 1e-12);
}

#[test]
fn residual_at_rounding_all_walls_anisotropic() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::open([4, 6]),
        CubicalReggeGeometry::per_axis([0.5, 0.25]),
    );
    assert_residual_at_rounding(&m, 103, 1e-10);
}

#[test]
fn residual_at_rounding_mixed_axes() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::new([8, 6], [true, false]),
        CubicalReggeGeometry::unit(),
    );
    assert_residual_at_rounding(&m, 107, 1e-12);
}

#[test]
fn residual_at_rounding_all_walls_3d_uniform() {
    let m = manifold_with_metric(
        LatticeComplex::<3, f64>::open([4, 4, 4]),
        CubicalReggeGeometry::uniform(0.5),
    );
    assert_residual_at_rounding(&m, 109, 1e-10);
}

/// The direct solution must agree with the Jacobi-preconditioned CG
/// fallback on the identical weighted system.
fn assert_agrees_with_pcg<const D: usize>(
    manifold: &Manifold<LatticeComplex<D, f64>, f64>,
    seed: u64,
) {
    let n0 = manifold.complex().num_cells(0);
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain(n1, seed), vec![n1]).unwrap();

    // Direct path through leray (mean-gauged).
    let direct = manifold
        .leray_project(&omega)
        .expect("projection succeeds")
        .potential()
        .as_slice()
        .to_vec();

    // PCG reference on the mass-weighted system.
    let mut rhs = manifold.codifferential_of(omega.as_slice(), 1).into_vec();
    rhs.resize(n0, 0.0);
    let metric_binding = manifold.metric();
    let metric = metric_binding.as_ref().unwrap();
    let star0 = metric.hodge_star_matrix(manifold.complex(), 0).unwrap();
    let star1 = metric.hodge_star_matrix(manifold.complex(), 1).unwrap();
    let diag_of = |m: &deep_causality_sparse::CsrMatrix<f64>, n: usize| -> Vec<f64> {
        let mut d = vec![0.0; n];
        for (i, slot) in d.iter_mut().enumerate() {
            for e in m.row_indices()[i]..m.row_indices()[i + 1] {
                if m.col_indices()[e] == i {
                    *slot = m.values()[e];
                }
            }
        }
        d
    };
    let m0 = diag_of(star0.as_ref(), n0);
    let m1 = diag_of(star1.as_ref(), n1);
    let mut wrhs: Vec<f64> = rhs.iter().zip(m0.iter()).map(|(r, m)| r * m).collect();
    subtract_mean(&mut wrhs);
    let apply = |v: &[f64]| -> Vec<f64> {
        let mut out = manifold.laplacian_of(v, 0).into_vec();
        out.resize(n0, 0.0);
        for (o, m) in out.iter_mut().zip(m0.iter()) {
            *o *= *m;
        }
        out
    };
    let boundary = manifold.complex().boundary_matrix(1);
    let mut jacobi = vec![0.0; n0];
    for (i, slot) in jacobi.iter_mut().enumerate() {
        for e in boundary.row_indices()[i]..boundary.row_indices()[i + 1] {
            *slot += m1[boundary.col_indices()[e]];
        }
    }
    let mut pcg =
        cg_solve_preconditioned(apply, &jacobi, &wrhs, 1e-13, 10_000).expect("PCG converges");

    // Align gauges (both potentials are defined up to a constant).
    let mut direct_gauged = direct;
    subtract_mean(&mut direct_gauged);
    subtract_mean(&mut pcg);

    let gap = sup(direct_gauged.iter().zip(pcg.iter()).map(|(a, b)| a - b));
    assert!(gap < 1e-9, "direct vs PCG gap {gap:e}");
}

#[test]
fn direct_agrees_with_pcg_all_walls() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::open([6, 5]),
        CubicalReggeGeometry::unit(),
    );
    assert_agrees_with_pcg(&m, 113);
}

#[test]
fn direct_agrees_with_pcg_mixed_anisotropic() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::new([8, 5], [true, false]),
        CubicalReggeGeometry::per_axis([0.5, 0.25]),
    );
    assert_agrees_with_pcg(&m, 127);
}

/// Discrete no-penetration: the projected field's divergence vanishes at
/// every vertex — including wall and corner vertices, whose clipped dual
/// volumes encode the no-flux wall condition (DEC form of the spec's
/// no-flux scenario).
#[test]
fn projection_is_divergence_free_up_to_the_walls() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::open([6, 6]),
        CubicalReggeGeometry::unit(),
    );
    let n0 = m.complex().num_cells(0);
    let n1 = m.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain(n1, 131), vec![n1]).unwrap();
    let projection = m.leray_project(&omega).expect("projection succeeds");
    let mut div = m
        .codifferential_of(projection.projected().as_slice(), 1)
        .into_vec();
    div.resize(n0, 0.0);
    let err = sup(div.iter().copied());
    assert!(err < 1e-12, "boundary-inclusive divergence {err:e}");
}

/// The direct path ignores the CG iteration budget entirely.
#[test]
fn direct_path_ignores_iteration_budget() {
    use deep_causality_topology::HodgeDecomposeOptions;
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::open([6, 6]),
        CubicalReggeGeometry::unit(),
    );
    let n1 = m.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain(n1, 137), vec![n1]).unwrap();
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-12),
        max_iterations: Some(1),
    };
    assert!(m.leray_project_opts(&omega, &opts).is_ok());
}

/// Extent-1 wall axes cannot carry the DCT-I basis: the solve falls back
/// to the (preconditioned) CG path and still succeeds.
#[test]
fn degenerate_wall_extent_falls_back_to_cg() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::new([8, 1], [true, false]),
        CubicalReggeGeometry::unit(),
    );
    let n1 = m.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain(n1, 139), vec![n1]).unwrap();
    assert!(m.leray_project(&omega).is_ok());
}
