/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for the compiled-stencil validation surface and the
//! build-time enumeration branches not pinned by the equivalence battery:
//!
//! * `stencil/mod.rs`: the `apply_convective` "pre scratch" length check and
//!   both scratch length checks of `apply_convective_vector_adjoint`.
//! * `build.rs`: the open-boundary transport branches (a target axis whose +1
//!   shift leaves the lattice, and a target whose entire offset star falls
//!   outside the open lattice so its row is empty) plus the duplicate-column
//!   merge that fires on a tiny extent-2 periodic axis (wrap aliasing two
//!   offsets onto the same source cell). These compile-time tables are
//!   re-validated against the generic operators on the same lattices.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, DecStencilTables, LatticeComplex, Manifold,
};

fn manifold<const D: usize>(
    lattice: LatticeComplex<D, f64>,
    metric: CubicalReggeGeometry<D, f64>,
) -> Manifold<LatticeComplex<D, f64>, f64> {
    let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn random(len: usize, seed: u64) -> Vec<f64> {
    let mut s = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (0..len)
        .map(|_| {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            2.0 * ((s >> 11) as f64 / (1u64 << 53) as f64) - 1.0
        })
        .collect()
}

// ---------------------------------------------------------------------------
// stencil/mod.rs:228 — apply_convective rejects a wrong-length pre scratch.
// ---------------------------------------------------------------------------

#[test]
fn apply_convective_rejects_wrong_pre_scratch() {
    let m = manifold(
        LatticeComplex::<2, f64>::square_torus(4),
        CubicalReggeGeometry::unit(),
    );
    let tables = DecStencilTables::compile(&m).unwrap();
    let n1 = m.complex().num_cells(1);
    let n2 = m.complex().num_cells(2);
    let (_pre_len, wedge_len) = tables.convective_scratch_lens();

    let w = vec![0.0; n2];
    let u = vec![0.0; n1];
    let mut pre = vec![0.0; 1]; // wrong length
    let mut wb = vec![0.0; wedge_len];
    let mut conv = vec![0.0; n1];
    let err = tables
        .apply_convective(&w, &u, &mut pre, &mut wb, &mut conv)
        .unwrap_err();
    assert!(format!("{err}").contains("expected"), "{err}");
}

// ---------------------------------------------------------------------------
// stencil/mod.rs:276,281 — apply_convective_vector_adjoint scratch checks.
// ---------------------------------------------------------------------------

#[test]
fn apply_convective_vector_adjoint_rejects_wrong_scratch() {
    let m = manifold(
        LatticeComplex::<2, f64>::square_torus(4),
        CubicalReggeGeometry::unit(),
    );
    let tables = DecStencilTables::compile(&m).unwrap();
    let n1 = m.complex().num_cells(1);
    let (pre_len, _wedge_len) = tables.convective_scratch_lens();
    let (s1_len, sw_len) = tables.convective_vector_adjoint_scratch_lens();

    let pre = vec![0.0; pre_len];
    let w = vec![0.0; n1];

    // Wrong n1 scratch length.
    let mut bad_s1 = vec![0.0; s1_len + 1];
    let mut sw = vec![0.0; sw_len];
    let mut out = vec![0.0; n1];
    let err = tables
        .apply_convective_vector_adjoint(&pre, &w, &mut bad_s1, &mut sw, &mut out)
        .unwrap_err();
    assert!(format!("{err}").contains("expected"), "{err}");

    // Wrong wedge scratch length.
    let mut s1 = vec![0.0; s1_len];
    let mut bad_sw = vec![0.0; sw_len + 1];
    let err = tables
        .apply_convective_vector_adjoint(&pre, &w, &mut s1, &mut bad_sw, &mut out)
        .unwrap_err();
    assert!(format!("{err}").contains("expected"), "{err}");
}

// ---------------------------------------------------------------------------
// build.rs:181,182,197,198 — open-boundary transport branches. On an open
// lattice the transport gather drops out-of-range offsets (and may empty a
// target row). Compiling and applying on an open lattice walks these paths;
// the result is re-checked against the generic interior product.
// ---------------------------------------------------------------------------

#[test]
fn compiled_convective_matches_generic_on_small_open_lattice() {
    let m = manifold(
        LatticeComplex::<2, f64>::open([3, 3]),
        CubicalReggeGeometry::unit(),
    );
    let tables = DecStencilTables::compile(&m).unwrap();
    let n1 = m.complex().num_cells(1);
    let n2 = m.complex().num_cells(2);

    let omega = random(n2, 71);
    let x = random(n1, 73);
    let (pre_len, wedge_len) = tables.convective_scratch_lens();
    let mut pre = vec![0.0; pre_len];
    let mut wb = vec![0.0; wedge_len];
    let mut conv = vec![0.0; n1];
    tables
        .apply_convective(&omega, &x, &mut pre, &mut wb, &mut conv)
        .unwrap();

    let x_t = CausalTensor::new(x, vec![n1]).unwrap();
    let w_t = CausalTensor::new(omega, vec![n2]).unwrap();
    let generic = m.interior_product(&x_t, &w_t, 2).unwrap();
    for (a, b) in conv.iter().zip(generic.as_slice().iter()) {
        assert!((a - b).abs() <= 1e-12, "stencil {a} vs generic {b}");
    }
}

// ---------------------------------------------------------------------------
// build.rs:209 — duplicate-column merge: on a tiny extent-2 periodic axis the
// −1 and +1 wraps alias two offsets onto the same source cell, so the row
// build merges coefficients in place. A 2-extent periodic lattice exercises it;
// the table must still reproduce the generic operator.
// ---------------------------------------------------------------------------

#[test]
fn compiled_operators_match_generic_on_extent_two_torus() {
    let m = manifold(
        LatticeComplex::<3, f64>::cubic_torus(2),
        CubicalReggeGeometry::unit(),
    );
    let tables = DecStencilTables::compile(&m).unwrap();
    let n1 = m.complex().num_cells(1);
    let n2 = m.complex().num_cells(2);

    // delta2 equivalence (touches build_delta + transport on the 2-extent torus).
    let w = random(n2, 81);
    let mut out = vec![0.0; n1];
    tables.apply_delta2(&w, &mut out).unwrap();
    let generic = m.codifferential_of(&w, 2);
    for (a, b) in out.iter().zip(generic.as_slice().iter()) {
        assert!((a - b).abs() <= 1e-12, "delta2 {a} vs generic {b}");
    }

    // convective equivalence (the transport rows with merged duplicates).
    let omega = random(n2, 83);
    let x = random(n1, 85);
    let (pre_len, wedge_len) = tables.convective_scratch_lens();
    let mut pre = vec![0.0; pre_len];
    let mut wb = vec![0.0; wedge_len];
    let mut conv = vec![0.0; n1];
    tables
        .apply_convective(&omega, &x, &mut pre, &mut wb, &mut conv)
        .unwrap();
    let x_t = CausalTensor::new(x, vec![n1]).unwrap();
    let w_t = CausalTensor::new(omega, vec![n2]).unwrap();
    let generic = m.interior_product(&x_t, &w_t, 2).unwrap();
    for (a, b) in conv.iter().zip(generic.as_slice().iter()) {
        assert!((a - b).abs() <= 1e-12, "convective {a} vs generic {b}");
    }
}

// ---------------------------------------------------------------------------
// `build::star_diag` reads only the diagonal of a Hodge ⋆ matrix. Off-diagonal
// entries stored ahead of the diagonal in a row must be scanned past.
// ---------------------------------------------------------------------------

/// A triangle (3 vertices, 3 edges, 1 face) with hand-supplied Hodge ⋆
/// operators, so a test controls the exact sparsity of `⋆₀`.
fn triangle_with_star_zero(
    star_zero: deep_causality_linear::CsrMatrix<f64>,
) -> deep_causality_topology::SimplicialManifold<f64, f64> {
    use deep_causality_linear::CsrMatrix;
    use deep_causality_topology::{ReggeGeometry, Simplex, SimplicialComplex, Skeleton};

    let sk0 = Skeleton::new(
        0,
        vec![
            Simplex::new(vec![0]),
            Simplex::new(vec![1]),
            Simplex::new(vec![2]),
        ],
    );
    let sk1 = Skeleton::new(
        1,
        vec![
            Simplex::new(vec![0, 1]),
            Simplex::new(vec![0, 2]),
            Simplex::new(vec![1, 2]),
        ],
    );
    let sk2 = Skeleton::new(2, vec![Simplex::new(vec![0, 1, 2])]);

    let d1 = CsrMatrix::from_triplets(
        3,
        3,
        &[
            (0, 0, -1i8),
            (1, 0, 1),
            (0, 1, -1),
            (2, 1, 1),
            (1, 2, -1),
            (2, 2, 1),
        ],
    )
    .unwrap();
    let d2 = CsrMatrix::from_triplets(3, 1, &[(0, 0, 1i8), (1, 0, -1), (2, 0, 1)]).unwrap();
    let cob = vec![d1.transpose(), d2.transpose()];

    let star_one =
        CsrMatrix::from_triplets(3, 3, &[(0, 0, 1.0f64), (1, 1, 1.0), (2, 2, 1.0)]).unwrap();
    let star_two = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0f64)]).unwrap();

    let complex = SimplicialComplex::new(
        vec![sk0, sk1, sk2],
        vec![d1, d2],
        cob,
        vec![star_zero, star_one, star_two],
    );
    let regge = ReggeGeometry::new(CausalTensor::new(vec![1.0f64; 3], vec![3]).unwrap());
    let data = CausalTensor::new(vec![0.0f64; 7], vec![7]).unwrap();
    Manifold::with_metric(complex, data, Some(regge), 0).unwrap()
}

/// The mass weighting of the grade-0 Poisson solve is the diagonal of `⋆₀`.
/// Adding an off-diagonal entry to `⋆₀` — stored ahead of that row's diagonal —
/// leaves the diagonal it extracts unchanged, so the Leray projection is
/// identical to the one computed against the purely diagonal star.
#[test]
fn star_diag_ignores_off_diagonal_entries_of_the_hodge_star() {
    use deep_causality_linear::CsrMatrix;

    let diagonal =
        CsrMatrix::from_triplets(3, 3, &[(0, 0, 1.0f64), (1, 1, 1.0), (2, 2, 1.0)]).unwrap();
    let with_off_diagonal = CsrMatrix::from_triplets(
        3,
        3,
        &[(0, 0, 1.0f64), (1, 0, 0.25), (1, 1, 1.0), (2, 2, 1.0)],
    )
    .unwrap();

    let field = CausalTensor::new(vec![1.0f64, 2.0, -0.5], vec![3]).unwrap();

    let reference = triangle_with_star_zero(diagonal)
        .leray_project(&field)
        .expect("plain projection on a triangle converges");
    let perturbed = triangle_with_star_zero(with_off_diagonal)
        .leray_project(&field)
        .expect("plain projection on a triangle converges");

    assert_eq!(
        reference.projected().as_slice(),
        perturbed.projected().as_slice()
    );
    assert_eq!(
        reference.potential().as_slice(),
        perturbed.potential().as_slice()
    );
}

// ---------------------------------------------------------------------------
// Degenerate lattices and metrics in the table build.
// ---------------------------------------------------------------------------

/// An open axis of extent 1 leaves every dual volume unresolvable, so every star
/// diagonal is zero. `build_delta` compiles a zero-mass row to an empty stencil
/// row, which is the same answer the generic codifferential's zero-mass guard
/// gives: the zero form.
#[test]
fn zero_mass_rows_compile_to_empty_delta_rows() {
    let lattice = LatticeComplex::<2, f64>::new([1, 3], [false, false]);
    let n0 = lattice.num_cells(0);
    let n1 = lattice.num_cells(1);
    let m = manifold(
        lattice,
        CubicalReggeGeometry::from_edge_lengths(vec![1.0; n1]),
    );
    let tables = DecStencilTables::compile(&m).unwrap();

    let field = random(n1, 91);
    let mut out = vec![0.0; n0];
    tables.apply_delta1(&field, &mut out).unwrap();
    let generic = m.codifferential_of(&field, 1);
    assert!(out.iter().all(|&x| x == 0.0));
    assert_eq!(out.as_slice(), generic.as_slice());
}

/// An open axis of extent 1 carries no cells oriented along it, so a transport
/// target whose complement includes that axis gathers from nothing and its row
/// compiles empty. The convective table still agrees with the generic interior
/// product, which is zero everywhere on a lattice with no 2-cells.
#[test]
fn transport_rows_with_no_source_cells_compile_empty() {
    let lattice = LatticeComplex::<2, f64>::new([1, 3], [false, false]);
    assert_eq!(
        lattice.num_cells(2),
        0,
        "no plaquette fits an extent-1 axis"
    );
    let n1 = lattice.num_cells(1);
    let m = manifold(lattice, CubicalReggeGeometry::unit());
    let tables = DecStencilTables::compile(&m).unwrap();

    let x = random(n1, 95);
    let omega: Vec<f64> = Vec::new();
    let (pre_len, wedge_len) = tables.convective_scratch_lens();
    let mut pre = vec![0.0; pre_len];
    let mut wb = vec![0.0; wedge_len];
    let mut conv = vec![0.0; n1];
    tables
        .apply_convective(&omega, &x, &mut pre, &mut wb, &mut conv)
        .unwrap();
    assert!(conv.iter().all(|&c| c == 0.0));
}

/// A periodic axis of extent 1 wraps every shift back onto the same cell, so two
/// offsets of the transport gather alias onto one source column and their
/// coefficients merge. The merged table must still reproduce the generic
/// interior product.
#[test]
fn transport_merges_aliased_columns_on_a_periodic_extent_one_axis() {
    let lattice = LatticeComplex::<2, f64>::new([1, 3], [true, true]);
    let n1 = lattice.num_cells(1);
    let n2 = lattice.num_cells(2);
    assert!(n2 > 0);
    let m = manifold(lattice, CubicalReggeGeometry::unit());
    let tables = DecStencilTables::compile(&m).unwrap();

    let omega = random(n2, 97);
    let x = random(n1, 99);
    let (pre_len, wedge_len) = tables.convective_scratch_lens();
    let mut pre = vec![0.0; pre_len];
    let mut wb = vec![0.0; wedge_len];
    let mut conv = vec![0.0; n1];
    tables
        .apply_convective(&omega, &x, &mut pre, &mut wb, &mut conv)
        .unwrap();

    let x_t = CausalTensor::new(x, vec![n1]).unwrap();
    let w_t = CausalTensor::new(omega, vec![n2]).unwrap();
    let generic = m.interior_product(&x_t, &w_t, 2).unwrap();
    for (a, b) in conv.iter().zip(generic.as_slice().iter()) {
        assert!((a - b).abs() <= 1e-12, "stencil {a} vs generic {b}");
    }
}
