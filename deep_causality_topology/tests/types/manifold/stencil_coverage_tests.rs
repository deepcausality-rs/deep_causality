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
