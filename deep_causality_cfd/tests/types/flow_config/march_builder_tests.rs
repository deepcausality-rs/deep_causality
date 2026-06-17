/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `MarchConfigBuilder::build` validation — the required-piece error paths the marching
//! cases never hit (they always supply both a mesh and a solver).

use deep_causality_cfd::{CfdConfigBuilder, Mesh};
use deep_causality_physics::PhysicsErrorEnum;

#[test]
fn test_build_requires_a_mesh() {
    let solver = CfdConfigBuilder::dec_ns()
        .viscosity(0.1)
        .time_step(0.01)
        .build()
        .expect("valid solver config");
    // `MarchConfig` is not `Debug`, so the error is destructured directly.
    let Err(err) = CfdConfigBuilder::march::<2, f64>("no-mesh")
        .solver(solver)
        .build()
    else {
        panic!("a missing mesh must be a build error");
    };
    assert!(matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)));
}

#[test]
fn test_build_requires_a_solver() {
    let Err(err) = CfdConfigBuilder::march::<2, f64>("no-solver")
        .mesh(Mesh::box_domain([4, 4]))
        .build()
    else {
        panic!("a missing solver must be a build error");
    };
    assert!(matches!(
        err.0,
        PhysicsErrorEnum::PhysicalInvariantBroken(_)
    ));
}
