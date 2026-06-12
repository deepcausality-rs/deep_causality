/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Equivalence tests for the compiled stencil tables: every compiled
//! operator must match the generic compositional operator it folds, on
//! periodic and mixed-periodicity lattices, across metric tiers, at f64
//! and Float106 (≤ 100·ε of the scalar). The generic operators are the
//! oracle; these tests are CI-permanent so the two evaluation strategies
//! cannot silently diverge.

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, DecStencilTables, LatticeComplex, Manifold,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn manifold_with_metric<const D: usize, R>(
    lattice: LatticeComplex<D, R>,
    metric: CubicalReggeGeometry<D, R>,
) -> Manifold<LatticeComplex<D, R>, R>
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![R::zero(); total], vec![total]).unwrap();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn random_cochain<R: RealField + deep_causality_par::MaybeParallel + FromPrimitive>(
    len: usize,
    seed: u64,
) -> Vec<R> {
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let unit = (state >> 11) as f64 / (1u64 << 53) as f64;
            R::from_f64(2.0 * unit - 1.0).expect("[-1,1] lifts into any RealField")
        })
        .collect()
}

fn assert_close_f64(a: &[f64], b: &[f64], tol: f64, what: &str) {
    assert_eq!(a.len(), b.len(), "{what}: length mismatch");
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        assert!(
            (x - y).abs() <= tol,
            "{what}[{i}]: stencil {x} vs generic {y} (|Δ| = {:e})",
            (x - y).abs()
        );
    }
}

/// Run the full operator-equivalence battery on one f64 manifold.
fn assert_all_operators_match<const D: usize>(
    manifold: &Manifold<LatticeComplex<D, f64>, f64>,
    seed: u64,
) {
    let tables = DecStencilTables::compile(manifold).expect("compile succeeds");
    let tol = 100.0 * f64::EPSILON;
    let n0 = manifold.complex().num_cells(0);
    let n1 = manifold.complex().num_cells(1);
    let n2 = manifold.complex().num_cells(2);

    // d0
    let phi = random_cochain::<f64>(n0, seed);
    let mut out = vec![0.0; n1];
    tables.apply_d0(&phi, &mut out).unwrap();
    let generic = manifold.exterior_derivative_of(&phi, 0);
    assert_close_f64(&out, generic.as_slice(), tol, "d0");

    // d1
    let u = random_cochain::<f64>(n1, seed + 1);
    let mut omega = vec![0.0; n2];
    tables.apply_d1(&u, &mut omega).unwrap();
    let generic = manifold.exterior_derivative_of(&u, 1);
    assert_close_f64(&omega, generic.as_slice(), tol, "d1");

    // delta1
    let mut out = vec![0.0; n0];
    tables.apply_delta1(&u, &mut out).unwrap();
    let generic = manifold.codifferential_of(&u, 1);
    assert_close_f64(&out, generic.as_slice(), tol, "delta1");

    // delta2
    let w = random_cochain::<f64>(n2, seed + 2);
    let mut out = vec![0.0; n1];
    tables.apply_delta2(&w, &mut out).unwrap();
    let generic = manifold.codifferential_of(&w, 2);
    assert_close_f64(&out, generic.as_slice(), tol, "delta2");

    // convective: i_x ω vs the generic interior product
    let x = random_cochain::<f64>(n1, seed + 3);
    let (pre_len, wedge_len) = tables.convective_scratch_lens();
    let mut pre = vec![0.0; pre_len];
    let mut wb = vec![0.0; wedge_len];
    let mut conv = vec![0.0; n1];
    tables
        .apply_convective(&w, &x, &mut pre, &mut wb, &mut conv)
        .unwrap();
    let x_t = CausalTensor::new(x, vec![n1]).unwrap();
    let w_t = CausalTensor::new(w, vec![n2]).unwrap();
    let generic = manifold.interior_product(&x_t, &w_t, 2).unwrap();
    assert_close_f64(&conv, generic.as_slice(), tol, "convective");

    // viscous composition: delta2(d1 u) + d0(delta1 u) vs laplacian_of
    let mut omega = vec![0.0; n2];
    tables.apply_d1(&u, &mut omega).unwrap();
    let mut visc_a = vec![0.0; n1];
    tables.apply_delta2(&omega, &mut visc_a).unwrap();
    let mut s0 = vec![0.0; n0];
    tables.apply_delta1(&u, &mut s0).unwrap();
    let mut visc_b = vec![0.0; n1];
    tables.apply_d0(&s0, &mut visc_b).unwrap();
    let composed: Vec<f64> = visc_a
        .iter()
        .zip(visc_b.iter())
        .map(|(a, b)| a + b)
        .collect();
    let generic = manifold.laplacian_of(&u, 1);
    assert_close_f64(&composed, generic.as_slice(), tol, "laplacian1");
}

// ---------------------------------------------------------------------------
// f64 equivalence across lattices and metric tiers
// ---------------------------------------------------------------------------

#[test]
fn stencils_match_generic_2d_periodic_unit() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::square_torus(8),
        CubicalReggeGeometry::unit(),
    );
    assert_all_operators_match(&m, 41);
}

#[test]
fn stencils_match_generic_2d_mixed_periodicity_per_axis() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::new([8, 6], [true, false]),
        CubicalReggeGeometry::per_axis([0.5, 0.25]),
    );
    assert_all_operators_match(&m, 43);
}

#[test]
fn stencils_match_generic_3d_periodic_uniform() {
    let m = manifold_with_metric(
        LatticeComplex::<3, f64>::cubic_torus(4),
        CubicalReggeGeometry::uniform(0.5),
    );
    assert_all_operators_match(&m, 47);
}

#[test]
fn stencils_match_generic_3d_open_unit() {
    let m = manifold_with_metric(
        LatticeComplex::<3, f64>::open([4, 5, 4]),
        CubicalReggeGeometry::unit(),
    );
    assert_all_operators_match(&m, 53);
}

// ---------------------------------------------------------------------------
// Float106 equivalence (precision-generic gate)
// ---------------------------------------------------------------------------

#[test]
fn stencils_match_generic_float106() {
    let m = manifold_with_metric(
        LatticeComplex::<2, Float106>::square_torus(6),
        CubicalReggeGeometry::unit(),
    );
    let tables = DecStencilTables::compile(&m).expect("compile succeeds");
    let n1 = m.complex().num_cells(1);
    let n2 = m.complex().num_cells(2);
    let tol = 1e-29; // ~100·ε for the double-double

    let u = random_cochain::<Float106>(n1, 59);
    let x = random_cochain::<Float106>(n1, 61);
    let mut omega = vec![Float106::from_f64(0.0); n2];
    tables.apply_d1(&u, &mut omega).unwrap();
    let generic = m.exterior_derivative_of(&u, 1);
    for (a, b) in omega.iter().zip(generic.as_slice().iter()) {
        let d = *a - *b;
        assert!(d.hi().abs() < tol, "d1 Float106: {:e}", d.hi());
    }

    let (pre_len, wedge_len) = tables.convective_scratch_lens();
    let mut pre = vec![Float106::from_f64(0.0); pre_len];
    let mut wb = vec![Float106::from_f64(0.0); wedge_len];
    let mut conv = vec![Float106::from_f64(0.0); n1];
    tables
        .apply_convective(&omega, &x, &mut pre, &mut wb, &mut conv)
        .unwrap();
    let x_t = CausalTensor::new(x, vec![n1]).unwrap();
    let w_t = CausalTensor::new(omega, vec![n2]).unwrap();
    let generic = m.interior_product(&x_t, &w_t, 2).unwrap();
    for (a, b) in conv.iter().zip(generic.as_slice().iter()) {
        let d = *a - *b;
        assert!(d.hi().abs() < tol, "convective Float106: {:e}", d.hi());
    }
}

// ---------------------------------------------------------------------------
// Construction and validation surface
// ---------------------------------------------------------------------------

#[test]
fn compile_rejects_1d_lattice() {
    let m = manifold_with_metric(
        LatticeComplex::<1, f64>::new([8], [true]),
        CubicalReggeGeometry::unit(),
    );
    let err = DecStencilTables::compile(&m).unwrap_err();
    assert!(format!("{err}").contains("dimension >= 2"), "{err}");
}

#[test]
fn compile_rejects_missing_metric() {
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let m = Manifold::from_cubical(lattice, data, 0);
    let err = DecStencilTables::compile(&m).unwrap_err();
    assert!(format!("{err}").contains("requires a metric"), "{err}");
}

#[test]
fn apply_validates_lengths() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::square_torus(4),
        CubicalReggeGeometry::unit(),
    );
    let tables = DecStencilTables::compile(&m).unwrap();
    let n1 = m.complex().num_cells(1);

    let mut out = vec![0.0; n1];
    let err = tables.apply_d1(&[0.0; 3], &mut out).unwrap_err();
    assert!(format!("{err}").contains("expected"), "{err}");

    let u = vec![0.0; n1];
    let mut bad_out = vec![0.0; 3];
    let err = tables.apply_delta1(&u, &mut bad_out).unwrap_err();
    assert!(format!("{err}").contains("expected"), "{err}");

    let (pre_len, wedge_len) = tables.convective_scratch_lens();
    let n2 = m.complex().num_cells(2);
    let w = vec![0.0; n2];
    let mut pre = vec![0.0; pre_len];
    let mut wb = vec![0.0; wedge_len.saturating_sub(1)];
    let mut conv = vec![0.0; n1];
    let err = tables
        .apply_convective(&w, &u, &mut pre, &mut wb, &mut conv)
        .unwrap_err();
    assert!(format!("{err}").contains("expected"), "{err}");
}

#[test]
fn tables_are_cloneable_and_debuggable() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::square_torus(4),
        CubicalReggeGeometry::unit(),
    );
    let tables = DecStencilTables::compile(&m).unwrap();
    let cloned = tables.clone();
    assert_eq!(cloned.num_cells(1), m.complex().num_cells(1));
    assert!(format!("{tables:?}").contains("DecStencilTables"));
}
