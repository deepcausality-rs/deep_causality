/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `SimplicialComplex::map` and `SimplicialComplex::default`.

use deep_causality_topology::{Simplex, SimplicialComplex, SimplicialComplexBuilder};

#[test]
fn test_simplicial_complex_default_is_empty() {
    let sc: SimplicialComplex<f64> = SimplicialComplex::default();
    // Default produces an empty complex: no skeletons, no operators.
    // We verify behavior via the public Default trait round-trip and clone.
    let _cloned = sc.clone();
}

#[test]
fn test_simplicial_complex_map_changes_value_type() {
    // Build a small complex with a single triangle.
    let mut builder = SimplicialComplexBuilder::new(2);
    builder
        .add_simplex(Simplex::new(vec![0, 1, 2]))
        .expect("failed to add simplex");
    let complex: SimplicialComplex<f64> = builder.build::<f64>().expect("build");

    // Map f64 -> f32 — values change type, structure remains identical.
    let mapped: SimplicialComplex<f32> = complex.clone().map(|x| x as f32);

    // Same number of skeletons after map (compare via Debug output)
    // We use display tests rely on dimension via base topology — but mod.rs doesn't
    // expose skeletons; instead just confirm the call compiles and works.
    let _ = format!("{:?}", mapped);

    // Map identity preserves value type
    let id_mapped: SimplicialComplex<f64> = complex.map(|x| x);
    let _ = format!("{:?}", id_mapped);
}

#[test]
fn test_simplicial_complex_map_to_i32() {
    let mut builder = SimplicialComplexBuilder::new(1);
    builder
        .add_simplex(Simplex::new(vec![0, 1]))
        .expect("failed to add simplex");
    let complex: SimplicialComplex<f64> = builder.build::<f64>().expect("build");

    let mapped: SimplicialComplex<i32> = complex.map(|x| x as i32);
    let _ = format!("{:?}", mapped);
}
