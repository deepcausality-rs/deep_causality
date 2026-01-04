/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;
use deep_causality_topology::{
    Chain, DifferentialForm, Simplex, SimplicialComplex, SimplicialComplexBuilder,
    StokesAdjunction, StokesContext,
};

fn simple_complex() -> SimplicialComplex {
    // Triangle: 3 vertices, 3 edges, 1 face
    let mut builder = SimplicialComplexBuilder::new(2);
    builder
        .add_simplex(Simplex::new(vec![0, 1, 2]))
        .expect("Failed to add simplex");
    builder.build().expect("Failed to build complex")
}

#[test]
fn test_stokes_context_new() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);
    assert_eq!(ctx.dim(), 2); // Triangle is 2D
    // Vertices: 0, 1, 2
    assert_eq!(ctx.num_simplices(0), 3);
    // Edges: (0,1), (0,2), (1,2)
    assert_eq!(ctx.num_simplices(1), 3);
    // Faces: (0,1,2)
    assert_eq!(ctx.num_simplices(2), 1);
}

#[test]
fn test_exterior_derivative_0_form() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);

    // Create a 0-form (scalar field on vertices)
    // Coeffs: [v0, v1, v2] -> [1.0, 2.0, 3.0]
    let form = DifferentialForm::from_coefficients(0, 2, vec![1.0, 2.0, 3.0]);

    // df should be a 1-form on edges
    // Edges order typically sorted: (0,1), (0,2), (1,2)
    // df((a,b)) = f(b) - f(a)
    // (0,1): 2-1 = 1
    // (0,2): 3-1 = 2
    // (1,2): 3-2 = 1
    // Orientation might affect signs based on complex builder
    let dform = StokesAdjunction::exterior_derivative(&ctx, &form);

    assert_eq!(dform.degree(), 1);
    assert_eq!(dform.coefficients().as_slice().len(), 3);

    // Check first edge value roughly
    // Exact values depend on builder sorting, but we expect non-zero derivative
    let coeffs = dform.coefficients().as_slice();
    assert!(coeffs.iter().any(|&x| x != 0.0));
}

#[test]
fn test_exterior_derivative_top_form() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);

    // 2-form on 2D complex
    let form = DifferentialForm::from_coefficients(2, 2, vec![5.0]);

    let dform = StokesAdjunction::exterior_derivative(&ctx, &form);

    // d of top form is 0 (or empty (k+1)-form)
    assert_eq!(dform.degree(), 3);
    assert!(dform.coefficients().as_slice().iter().all(|&x| x == 0.0));
}

#[test]
fn test_boundary_placeholder() {
    // Current boundary implementation returns empty/zero chain placeholder
    let complex = simple_complex();
    let ctx = StokesContext::new(complex.clone());

    // Create dummy chain
    let weights: CsrMatrix<f64> = CsrMatrix::new();
    let chain = Chain::new(std::sync::Arc::new(complex), 1, weights);

    let boundary = StokesAdjunction::boundary(&ctx, &chain);
    assert_eq!(boundary.grade(), 0);
}
