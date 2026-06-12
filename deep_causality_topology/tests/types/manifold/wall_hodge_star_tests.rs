/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the boundary-corrected Hodge star (the wall-hodge-star
//! capability): clip exponents at faces/edges/corners, interior and
//! periodic invariance, and the M-weighted symmetry the CG solves rely
//! on.

use deep_causality_sparse::CsrMatrix;
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

fn star_diag<const D: usize>(
    metric: &CubicalReggeGeometry<D, f64>,
    complex: &LatticeComplex<D, f64>,
    k: usize,
) -> Vec<f64> {
    let m: std::borrow::Cow<'_, CsrMatrix<f64>> = metric.hodge_star_matrix(complex, k).unwrap();
    let n = complex.num_cells(k);
    let mut diag = vec![0.0; n];
    let ptr = m.row_indices();
    let cols = m.col_indices();
    let vals = m.values();
    for (i, d) in diag.iter_mut().enumerate() {
        for e in ptr[i]..ptr[i + 1] {
            if cols[e] == i {
                *d = vals[e];
            }
        }
    }
    diag
}

#[test]
fn fully_periodic_star_is_unchanged() {
    // No wall axes → no clipping: the unit star is the identity.
    let complex = LatticeComplex::<2, f64>::square_torus(4);
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    for k in 0..=2 {
        for v in star_diag(&metric, &complex, k) {
            assert_eq!(v, 1.0);
        }
    }
}

#[test]
fn grade0_clip_exponents_3d_uniform() {
    // Vertices on an open 3D lattice: interior h³, face h³/2, edge h³/4,
    // corner h³/8 (the spec's face/edge/corner scenario).
    let n = 4usize;
    let h = 0.5f64;
    let complex = LatticeComplex::<3, f64>::open([n, n, n]);
    let metric: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::uniform(h);
    let diag = star_diag(&metric, &complex, 0);
    let h3 = h * h * h;
    for (idx, v) in diag.iter().enumerate() {
        // Vertex positions in axis-0-fastest order.
        let x = idx % n;
        let y = (idx / n) % n;
        let z = idx / (n * n);
        let b = [x, y, z].iter().filter(|&&p| p == 0 || p + 1 == n).count();
        let expected = h3 / (1u32 << b) as f64;
        assert!(
            (v - expected).abs() < 1e-15,
            "vertex {idx} (b={b}): {v} vs {expected}"
        );
    }
}

#[test]
fn interior_entries_unchanged_2d_per_axis() {
    // Interior cells keep the closed-form dual/primal ratios.
    let complex = LatticeComplex::<2, f64>::open([5, 5]);
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::per_axis([0.5, 0.25]);
    let diag0 = star_diag(&metric, &complex, 0);
    // Vertex (2,2) is interior: dual = 0.5·0.25.
    let idx = 2 + 2 * 5;
    assert!((diag0[idx] - 0.125).abs() < 1e-15);
}

#[test]
fn periodic_axes_do_not_clip_on_mixed_lattices() {
    // periodic-x, wall-y: only y-boundary incidences clip.
    let n = 4usize;
    let complex = LatticeComplex::<2, f64>::new([n, n], [true, false]);
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    let diag0 = star_diag(&metric, &complex, 0);
    for (idx, v) in diag0.iter().enumerate() {
        let y = idx / n;
        let expected = if y == 0 || y + 1 == n { 0.5 } else { 1.0 };
        assert!((v - expected).abs() < 1e-15, "vertex {idx}: {v}");
    }
}

#[test]
fn x_edges_clip_only_at_y_walls_2d() {
    // An x-edge's dual extends along y only: clipping is driven by the
    // y-position, never by sitting at the x-extremes.
    let n = 4usize;
    let complex = LatticeComplex::<2, f64>::new([n, n], [false, false]);
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    let diag1 = star_diag(&metric, &complex, 1);
    // Orientation-major: x-edges (orientation 0b01) come first,
    // (n−1)·n of them, axis-0-fastest.
    let nx_edges = (n - 1) * n;
    for (idx, v) in diag1.iter().enumerate().take(nx_edges) {
        let y = idx / (n - 1);
        let expected = if y == 0 || y + 1 == n { 0.5 } else { 1.0 };
        assert!(
            (v - expected).abs() < 1e-15,
            "x-edge {idx}: {v} vs {expected}"
        );
    }
}

/// The mass-weighted Laplacian `M_k Δ_k` must be Euclidean-symmetric on
/// walled lattices — the property the CG solves rely on (design D7/D8 of
/// add-walls-and-dec-stencils).
fn assert_weighted_laplacian_symmetric<const D: usize>(
    manifold: &Manifold<LatticeComplex<D, f64>, f64>,
    k: usize,
    tol: f64,
) {
    let complex = manifold.complex();
    let n = complex.num_cells(k);
    let metric_binding = manifold.metric();
    let metric = metric_binding.as_ref().unwrap();
    let mass = star_diag(metric, complex, k);

    // Assemble M_k·Δ_k column by column and check symmetry.
    let mut columns: Vec<Vec<f64>> = Vec::with_capacity(n);
    for j in 0..n {
        let mut e = vec![0.0; n];
        e[j] = 1.0;
        let mut col = manifold.laplacian_of(&e, k).into_vec();
        col.resize(n, 0.0);
        for (i, c) in col.iter_mut().enumerate() {
            *c *= mass[i];
        }
        columns.push(col);
    }
    for (i, col_i) in columns.iter().enumerate() {
        for (j, col_j) in columns.iter().enumerate().take(i) {
            let a = col_j[i];
            let b = col_i[j];
            assert!(
                (a - b).abs() < tol,
                "M·Δ_{k} asymmetric at ({i},{j}): {a} vs {b}"
            );
        }
    }
}

#[test]
fn weighted_laplacian_symmetric_open_2d_grade0() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::open([3, 3]),
        CubicalReggeGeometry::unit(),
    );
    assert_weighted_laplacian_symmetric(&m, 0, 1e-13);
}

#[test]
fn weighted_laplacian_symmetric_open_2d_grade1() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::open([3, 3]),
        CubicalReggeGeometry::unit(),
    );
    assert_weighted_laplacian_symmetric(&m, 1, 1e-13);
}

#[test]
fn weighted_laplacian_symmetric_open_2d_grade2() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::open([3, 3]),
        CubicalReggeGeometry::unit(),
    );
    assert_weighted_laplacian_symmetric(&m, 2, 1e-13);
}

#[test]
fn weighted_laplacian_symmetric_mixed_3d_grade1() {
    let m = manifold_with_metric(
        LatticeComplex::<3, f64>::new([3, 3, 3], [true, false, false]),
        CubicalReggeGeometry::uniform(0.5),
    );
    assert_weighted_laplacian_symmetric(&m, 1, 1e-12);
}

/// Diagnostic probe for the property-test divergence: replicate the two
/// hodge_decompose solves on the failing fixture with the M-weighted
/// operator and report consistency/convergence per grade.
#[test]
fn probe_weighted_solves_on_open_3x3() {
    use deep_causality_sparse::cg_solve;
    let lattice = LatticeComplex::<2, f64>::open([3, 3]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let mut data = vec![0.0; total];
    for (i, s) in data.iter_mut().enumerate() {
        *s = ((i as f64) * 0.31).sin();
    }
    let tensor = CausalTensor::new(data, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    let m = Manifold::from_cubical_with_metric(lattice, tensor, metric, 0);

    let omega = m.exterior_derivative(0);
    let n0 = m.complex().num_cells(0);
    let n2 = m.complex().num_cells(2);
    let metric_binding = m.metric();
    let metric = metric_binding.as_ref().unwrap();

    // alpha step: grade 0
    let mass0 = star_diag(metric, m.complex(), 0);
    let mut rhs_a = m.codifferential_of(omega.as_slice(), 1).into_vec();
    rhs_a.resize(n0, 0.0);
    let consistency: f64 = rhs_a.iter().zip(mass0.iter()).map(|(r, mw)| r * mw).sum();
    println!("grade0: sum(M*rhs) = {consistency:e}");
    let wrhs: Vec<f64> = rhs_a
        .iter()
        .zip(mass0.iter())
        .map(|(r, mw)| r * mw)
        .collect();
    let apply0 = |v: &[f64]| -> Vec<f64> {
        let mut out = m.laplacian_of(v, 0).into_vec();
        out.resize(n0, 0.0);
        for (o, mw) in out.iter_mut().zip(mass0.iter()) {
            *o *= *mw;
        }
        out
    };
    match cg_solve(apply0, &wrhs, 1e-10, 1000) {
        Ok(_) => println!("grade0: converged"),
        Err(e) => println!("grade0: FAILED iters={} res={:e}", e.iterations, e.residual),
    }

    // beta step: grade 2
    let mass2 = star_diag(metric, m.complex(), 2);
    let mut rhs_b = m.exterior_derivative_of(omega.as_slice(), 1).into_vec();
    rhs_b.resize(n2, 0.0);
    let wrhs2: Vec<f64> = rhs_b
        .iter()
        .zip(mass2.iter())
        .map(|(r, mw)| r * mw)
        .collect();
    let apply2 = |v: &[f64]| -> Vec<f64> {
        let mut out = m.laplacian_of(v, 2).into_vec();
        out.resize(n2, 0.0);
        for (o, mw) in out.iter_mut().zip(mass2.iter()) {
            *o *= *mw;
        }
        out
    };
    match cg_solve(apply2, &wrhs2, 1e-10, 1000) {
        Ok(_) => println!("grade2: converged"),
        Err(e) => println!("grade2: FAILED iters={} res={:e}", e.iterations, e.residual),
    }
}
