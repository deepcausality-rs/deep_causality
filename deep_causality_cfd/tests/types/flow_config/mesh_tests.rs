/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `Mesh` constructors and `materialize`/`manifold`, including the metric `Grading`
//! (cosine-graded edge lengths) path that the marching cases do not exercise directly.

use deep_causality_cfd::{Body, Grading, Mesh};
use deep_causality_topology::ChainComplex;

#[test]
fn test_periodic_cube_materializes() {
    // n^3 vertices on a fully periodic torus.
    let manifold = Mesh::<3, f64>::periodic_cube(4)
        .manifold()
        .expect("materializes");
    assert_eq!(manifold.complex().num_cells(0), 4 * 4 * 4);
}

#[test]
fn test_torus_alias_matches_periodic_cube() {
    let cube = Mesh::<2, f64>::periodic_cube(6).manifold().expect("cube");
    let torus = Mesh::<2, f64>::torus(6).manifold().expect("torus");
    assert_eq!(cube.complex().num_cells(0), torus.complex().num_cells(0));
}

#[test]
fn test_box_domain_and_channel_materialize() {
    assert!(Mesh::<2, f64>::box_domain([5, 7]).manifold().is_ok());
    assert!(Mesh::<2, f64>::channel([8, 4]).manifold().is_ok());
    // 1-D channel still sets the streamwise axis periodic without panicking.
    assert!(Mesh::<1, f64>::channel([8]).manifold().is_ok());
}

#[test]
fn test_spacing_is_applied() {
    // A non-unit spacing materializes; the manifold shape is unchanged.
    let manifold = Mesh::<2, f64>::box_domain([4, 4])
        .spacing(0.25)
        .manifold()
        .expect("materializes");
    assert_eq!(manifold.complex().num_cells(0), 4 * 4);
}

#[test]
fn test_cosine_graded_metric_materializes() {
    // The cosine grading varies edge lengths along axis 0; structure is unchanged so the
    // manifold still materializes, exercising the `base_geometry` graded branch.
    let grading = Grading::cosine(0, 0.2_f64);
    let manifold = Mesh::<2, f64>::torus(8)
        .graded(grading)
        .manifold()
        .expect("graded mesh materializes");
    assert_eq!(manifold.complex().num_cells(0), 8 * 8);
    // Grading is Copy/Debug.
    let _copy = grading;
    assert!(format!("{grading:?}").contains("Cosine"));
}

#[test]
fn test_immersed_body_materializes_cut_cells() {
    // An immersed disk drives the cut-cell branch of `materialize`.
    let body = Body::disk([4.0_f64, 4.0], 1.5).merge_floor(0.25);
    let manifold = Mesh::<2, f64>::box_domain([8, 8])
        .immersed(body)
        .manifold()
        .expect("immersed mesh materializes");
    assert_eq!(manifold.complex().num_cells(0), 8 * 8);
    assert!(format!("{body:?}").contains("Body"));
}

#[test]
fn test_mesh_is_clone_debug() {
    let mesh = Mesh::<2, f64>::periodic_cube(4);
    let cloned = mesh.clone();
    assert!(cloned.manifold().is_ok());
    assert!(format!("{mesh:?}").contains("Mesh"));
}
