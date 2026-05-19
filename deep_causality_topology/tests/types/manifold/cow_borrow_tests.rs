/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Runtime verification that the `Cow::Borrowed` zero-copy guarantee holds on the
//! `SimplicialComplex` path. Stage B routes Manifold differential reads through
//! `ChainComplex::coboundary_matrix(k)`; the simplicial impl returns `Cow::Borrowed`
//! from its pre-computed cache, so no `CsrMatrix` clone is performed.

use deep_causality_topology::{ChainComplex, Simplex, SimplicialComplex, SimplicialComplexBuilder};
use std::borrow::Cow;

fn make_triangle_complex() -> SimplicialComplex<f64> {
    let mut builder = SimplicialComplexBuilder::new(2);
    builder
        .add_simplex(Simplex::new(vec![0, 1, 2]))
        .expect("add triangle");
    builder.build::<f64>().expect("build complex")
}

#[test]
fn coboundary_matrix_returns_borrowed_on_simplicial() {
    let complex = make_triangle_complex();
    let cob0 = ChainComplex::coboundary_matrix(&complex, 0);
    assert!(
        matches!(cob0, Cow::Borrowed(_)),
        "expected Cow::Borrowed from SimplicialComplex's coboundary_matrix(0) — got Cow::Owned (read-path clone regression)"
    );
}

#[test]
fn boundary_matrix_returns_borrowed_on_simplicial() {
    let complex = make_triangle_complex();
    let bnd1 = ChainComplex::boundary_matrix(&complex, 1);
    assert!(
        matches!(bnd1, Cow::Borrowed(_)),
        "expected Cow::Borrowed from SimplicialComplex's boundary_matrix(1) — got Cow::Owned (read-path clone regression)"
    );
    let bnd2 = ChainComplex::boundary_matrix(&complex, 2);
    assert!(
        matches!(bnd2, Cow::Borrowed(_)),
        "expected Cow::Borrowed from SimplicialComplex's boundary_matrix(2)"
    );
}
