/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::Adjunction;
use deep_causality_linear::CsrMatrix;
use deep_causality_topology::{
    BaseTopology, Chain, DifferentialForm, Simplex, SimplicialComplex, SimplicialComplexBuilder,
    StokesAdjunction, StokesContext,
};
use std::sync::Arc;

fn simple_complex() -> SimplicialComplex<f64> {
    // Triangle: 3 vertices, 3 edges, 1 face
    let mut builder = SimplicialComplexBuilder::new(2);
    builder
        .add_simplex(Simplex::new(vec![0, 1, 2]))
        .expect("Failed to add simplex");
    builder.build::<f64>().expect("Failed to build complex")
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
fn test_stokes_context_from_arc() {
    let complex = simple_complex();
    let arc_complex = Arc::new(complex);
    let ctx = StokesContext::from_arc(arc_complex);
    assert_eq!(ctx.dim(), 2);
}

#[test]
fn test_stokes_context_complex_arc() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);
    let arc = ctx.complex_arc();
    assert_eq!(arc.dimension(), 2);
}

#[test]
fn test_num_simplices_out_of_bounds() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);

    // k=3 is beyond the 2D complex, should return 0
    assert_eq!(ctx.num_simplices(3), 0);
    assert_eq!(ctx.num_simplices(10), 0);
    assert_eq!(ctx.num_simplices(100), 0);
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
    // The comment above already derives the answer; assert it instead of "some entry is
    // non-zero", which any wrong-but-non-zero result satisfies.
    assert_eq!(dform.coefficients().as_slice(), &[1.0f64, 2.0, 1.0]);
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
fn test_exterior_derivative_beyond_coboundary() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);

    // 3-form on 2D complex (beyond dim)
    let form = DifferentialForm::from_coefficients(3, 2, vec![1.0]);

    let dform = StokesAdjunction::exterior_derivative(&ctx, &form);

    // "Should return zero form" was asserted only as a degree. Assert the zero as well.
    assert_eq!(dform.degree(), 4);
    assert!(
        dform.coefficients().as_slice().iter().all(|&x| x == 0.0),
        "a form beyond the coboundary range is zero"
    );
}

#[test]
fn test_boundary_placeholder() {
    // Current boundary implementation returns empty/zero chain placeholder
    let complex = simple_complex();
    let ctx = StokesContext::new(complex.clone());

    // Create dummy chain
    let weights: CsrMatrix<f64> = CsrMatrix::new();
    let chain = Chain::new(Arc::new(complex), 1, weights);

    let boundary = StokesAdjunction::boundary(&ctx, &chain);
    assert_eq!(boundary.grade(), 0);
    assert!(
        boundary.weights().values().is_empty(),
        "an empty 1-chain has an empty boundary"
    );
}

#[test]
fn test_boundary_0_chain() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex.clone());

    // A 0-chain that actually carries weight, so the k == 0 early return is exercised rather
    // than an empty input trivially producing an empty output.
    let weights = CsrMatrix::from_triplets(1, 3, &[(0, 0, 5.0f64)]).unwrap();
    let chain = Chain::new(Arc::new(complex), 0, weights);

    let boundary = StokesAdjunction::boundary(&ctx, &chain);
    assert_eq!(boundary.grade(), 0);
    assert!(
        boundary.weights().values().is_empty(),
        "the boundary of a 0-chain is empty"
    );
}

#[test]
fn test_boundary_k_exceeds_operators() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex.clone());

    // k > boundary_ops.len()
    let weights: CsrMatrix<f64> = CsrMatrix::new();
    let chain = Chain::new(Arc::new(complex), 10, weights);

    let boundary = StokesAdjunction::boundary(&ctx, &chain);
    assert_eq!(boundary.grade(), 9);
}

#[test]
fn test_integrate_grade_mismatch() {
    let complex = simple_complex();

    // 0-form
    let form = DifferentialForm::from_coefficients(0, 2, vec![1.0, 2.0, 3.0]);

    // 1-chain (grade mismatch)
    let weights: CsrMatrix<f64> = CsrMatrix::new();
    let chain = Chain::new(Arc::new(complex), 1, weights);

    // Should return zero due to grade mismatch
    let result = StokesAdjunction::integrate(&form, &chain);
    assert_eq!(result, 0.0);
}

#[test]
fn test_integrate_matching_grade() {
    let complex = simple_complex();

    // 0-form on vertices
    let form = DifferentialForm::from_coefficients(0, 2, vec![1.0, 2.0, 3.0]);

    // 0-chain matching grade
    let triplets = vec![(0, 0, 1.0), (0, 1, 1.0), (0, 2, 1.0)];
    let weights = CsrMatrix::from_triplets(1, 3, &triplets).unwrap();
    let chain = Chain::new(Arc::new(complex), 0, weights);

    // Integrate: sum of form values weighted by chain
    let result = StokesAdjunction::integrate(&form, &chain);
    assert_eq!(result, 6.0); // 1*1 + 1*2 + 1*3 = 6
}

// =============================================================================
// Adjunction Trait Tests
// =============================================================================

#[test]
fn test_adjunction_unit() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);

    // Unit: A → R(L(A)) = Chain<DifferentialForm<A>>
    let chain = <StokesAdjunction as Adjunction<_, _, StokesContext<f64>>>::unit(&ctx, 42.0f64);

    // The grade alone never observes the value 42.0 reaching the chain.
    assert_eq!(chain.grade(), 0);
    let form = &chain.weights().values()[0];
    assert_eq!(form.degree(), 0);
    assert_eq!(form.coefficients().as_slice(), &[42.0f64]);
}

#[test]
fn test_adjunction_left_adjunct() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);

    // left_adjunct: (L(A) → B) → (A → R(B))
    // Given f: DifferentialForm<A> → B, produce g: A → Chain<B, B>
    let chain = <StokesAdjunction as Adjunction<_, _, StokesContext<f64>>>::left_adjunct(
        &ctx,
        5.0f64,
        |form: DifferentialForm<f64>| {
            // Sum all coefficients
            form.coefficients().as_slice().iter().sum::<f64>()
        },
    );

    // `f` sums the coefficients of the form `unit` built, which is [5.0]. Asserting only the
    // grade means the closure's result is never observed at all.
    assert_eq!(chain.grade(), 0);
    assert_eq!(chain.weights().values(), &vec![5.0f64]);
}

#[test]
fn test_adjunction_right_adjunct() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);

    // Create a form with some coefficients
    let form = DifferentialForm::from_coefficients(0, 2, vec![10.0]);

    // right_adjunct: (A → R(B)) → (L(A) → B)
    let result = <StokesAdjunction as Adjunction<_, _, StokesContext<f64>>>::right_adjunct(
        &ctx,
        form,
        |a: f64| {
            // Create a chain with the value
            let triplets = vec![(0, 0, a * 2.0)];
            let weights = CsrMatrix::from_triplets(1, 1, &triplets).unwrap();
            Chain::new(ctx.complex_arc(), 0, weights)
        },
    );

    let result = result.expect("form and chain both store a value");
    assert_eq!(result, 20.0); // 10.0 * 2.0
}

#[test]
fn test_adjunction_right_adjunct_empty_output_chain() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);

    // Form with valid coefficient
    let form = DifferentialForm::from_coefficients(0, 2, vec![10.0]);

    // Function returns an empty chain: there is no B to return, and that is now reported rather
    // than panicked.
    let result = <StokesAdjunction as Adjunction<_, _, StokesContext<f64>>>::right_adjunct(
        &ctx,
        form,
        |_a: f64| {
            // Create a chain with empty weights
            let weights = CsrMatrix::<f64>::new();
            Chain::new(ctx.complex_arc(), 0, weights)
        },
    );

    let err = result.expect_err("an empty output chain has no B to return");
    assert!(
        err.to_string().contains("stores no weight"),
        "the error should name what was empty, got: {err}"
    );
}

#[test]
fn test_boundary_1_chain_non_trivial() {
    // boundary of a 1-chain with weights produces a 0-chain via the inner dot-product path.
    let complex = simple_complex();
    let ctx = StokesContext::new(complex.clone());

    let num_edges = ctx.num_simplices(1);
    assert!(num_edges >= 1);

    // 1-chain with weight 1.0 on first edge
    let triplets: Vec<(usize, usize, f64)> = (0..num_edges).map(|i| (0, i, 1.0)).collect();
    let weights = deep_causality_linear::CsrMatrix::from_triplets(1, num_edges, &triplets).unwrap();
    let chain = Chain::new(Arc::new(complex), 1, weights);

    let bd = StokesAdjunction::boundary(&ctx, &chain);
    assert_eq!(bd.grade(), 0);
    // d(e01) = v1 - v0, d(e02) = v2 - v0, d(e12) = v2 - v1. With unit weights the sum is
    // -2*v0 + 0*v1 + 2*v2, so v1 cancels and only columns 0 and 2 carry weight.
    assert_eq!(bd.weights().col_indices(), &vec![0usize, 2]);
    assert_eq!(bd.weights().values(), &vec![-2.0f64, 2.0]);
}

#[test]
fn test_stokes_context_complex_getter() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);
    let c = ctx.complex();
    assert_eq!(c.dimension(), 2);
}

#[test]
fn test_adjunction_counit_empty_chain_in_form() {
    let complex = simple_complex();
    let ctx = StokesContext::new(complex.clone());

    // Create an empty chain
    let weights = CsrMatrix::<f64>::new();
    let chain = Chain::new(Arc::new(complex), 0, weights);

    // Embed this chain into a 0-form
    // DifferentialForm<Chain<f64, f64>>
    // We can use Adjunction::unit to wrap it, but unit() creates a chain of forms.
    // Counit input is DifferentialForm<Chain<B, B>>.
    // So we need to create a DifferentialForm where the coefficient is a Chain.

    // DifferentialForm::from_coefficients takes Vec<T>.
    // Here T is Chain<f64, f64>.
    let coeffs = vec![chain];
    let form_of_chains = DifferentialForm::from_coefficients(0, 2, coeffs);

    // The form's chain stores nothing, so there is no B to evaluate to. That is now reported
    // rather than panicked.
    let result =
        <StokesAdjunction as Adjunction<_, _, StokesContext<f64>>>::counit(&ctx, form_of_chains);

    let err = result.expect_err("an empty chain inside the form has no B to evaluate to");
    assert!(
        err.to_string().contains("stores no weight"),
        "the error should name what was empty, got: {err}"
    );
}

#[test]
fn test_adjunction_counit_returns_first_chain_value() {
    // Success path of `counit`: the form's single coefficient is a NON-empty chain,
    // so `chain.weights().values().first()` is `Some(val)` and the value is returned.
    // Covers src/extensions/hkt_gauge/hkt_adjunction_stokes.rs line 164.
    let complex = simple_complex();
    let ctx = StokesContext::new(complex.clone());

    // A 0-chain carrying the value 7.0 at vertex 0.
    let weights = CsrMatrix::from_triplets(1, 1, &[(0, 0, 7.0)]).unwrap();
    let chain = Chain::new(Arc::new(complex), 0, weights);

    // DifferentialForm<Chain<f64, f64>> with the chain as its single coefficient.
    let form_of_chains = DifferentialForm::from_coefficients(0, 2, vec![chain]);

    let result =
        <StokesAdjunction as Adjunction<_, _, StokesContext<f64>>>::counit(&ctx, form_of_chains);
    let result = result.expect("form and chain both store a value");
    assert_eq!(result, 7.0);
}

#[test]
fn test_exterior_derivative_k_beyond_coboundary_ops_len() {
    // Reach the `k >= coboundary_ops.len()` early-return *without* tripping the
    // earlier `k >= dim` guard. We build a dim-2 complex (three skeletons) but pass an
    // EMPTY coboundary-operator list, so `coboundary_ops.len() == 0`. A degree-0 form
    // then satisfies `0 < dim(=2)` yet `0 >= len(=0)`.
    // Covers src/extensions/hkt_gauge/hkt_adjunction_stokes.rs line 258.
    let v = vec![Simplex::new(vec![0]), Simplex::new(vec![1])];
    let e = vec![Simplex::new(vec![0, 1])];
    let f = vec![Simplex::new(vec![0, 1, 2])];
    let complex: SimplicialComplex<f64> = SimplicialComplex::new(
        vec![
            deep_causality_topology::Skeleton::new(0, v),
            deep_causality_topology::Skeleton::new(1, e),
            deep_causality_topology::Skeleton::new(2, f),
        ],
        vec![],
        vec![], // empty coboundary operators -> len 0
        vec![],
    );
    let ctx = StokesContext::new(complex);
    assert_eq!(ctx.dim(), 2);

    let form = DifferentialForm::from_coefficients(0, 2, vec![1.0, 2.0]);
    let dform = StokesAdjunction::exterior_derivative(&ctx, &form);

    // Returns a zero (k+1)-form because the coboundary operator is absent.
    assert_eq!(dform.degree(), 1);
}

#[test]
fn test_exterior_derivative_accumulates_signed_sum() {
    // Drives the inner accumulation `sum += coeffs[col] * sign_t` over a real coboundary
    // row with both a +1 and a -1 entry.
    // Covers src/extensions/hkt_gauge/hkt_adjunction_stokes.rs line 288.
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);

    // df((a,b)) = f(b) - f(a). With f = [0, 10, 100] on the 3 vertices, every edge
    // derivative is non-zero, so the signed accumulation must execute.
    let form = DifferentialForm::from_coefficients(0, 2, vec![0.0, 10.0, 100.0]);
    let dform = StokesAdjunction::exterior_derivative(&ctx, &form);

    // df(e01) = 10 - 0, df(e02) = 100 - 0, df(e12) = 100 - 10. "At least one is non-zero"
    // is satisfied by any wrong answer that is not identically zero.
    assert_eq!(dform.coefficients().as_slice(), &[10.0f64, 100.0, 90.0]);
}

#[test]
fn test_boundary_accumulates_signed_sum() {
    // Drives the inner accumulation `sum += *val * sign_t` of the boundary operator over
    // a 1-chain whose weighted edges map onto shared vertices.
    // Covers src/extensions/hkt_gauge/hkt_adjunction_stokes.rs line 357.
    let complex = simple_complex();
    let ctx = StokesContext::new(complex.clone());

    let num_edges = ctx.num_simplices(1);
    assert!(num_edges >= 1);

    // Distinct non-zero weights on every edge so the per-vertex dot product is non-trivial.
    let triplets: Vec<(usize, usize, f64)> =
        (0..num_edges).map(|i| (0, i, (i as f64) + 1.0)).collect();
    let weights = CsrMatrix::from_triplets(1, num_edges, &triplets).unwrap();
    let chain = Chain::new(Arc::new(complex), 1, weights);

    let bd = StokesAdjunction::boundary(&ctx, &chain);
    assert_eq!(bd.grade(), 0);
    // Weights 1, 2, 3 on e01, e02, e12:
    //   1*(v1 - v0) + 2*(v2 - v0) + 3*(v2 - v1) = -3*v0 - 2*v1 + 5*v2
    // `any(|x| x != 0.0)` is satisfied by any non-zero output at all, including a wrong one.
    assert_eq!(bd.weights().col_indices(), &vec![0usize, 1, 2]);
    assert_eq!(bd.weights().values(), &vec![-3.0f64, -2.0, 5.0]);
}

#[test]
fn test_boundary_partial_chain_misses_some_columns() {
    // The boundary inner loop looks up each boundary column in the chain's weight
    // map. When the chain covers only a *subset* of the edges, the lookup misses
    // for the absent columns, exercising the `if let Some(val) = chain_map.get(&col)`
    // false arm (the loop continues without accumulating). Covers the not-found
    // branch at src/extensions/hkt_gauge/hkt_adjunction_stokes.rs line 357.
    let complex = simple_complex();
    let ctx = StokesContext::new(complex.clone());

    let num_edges = ctx.num_simplices(1);
    assert!(num_edges >= 2, "triangle has 3 edges");

    // Populate only the first edge; the remaining boundary columns must miss.
    let triplets: Vec<(usize, usize, f64)> = vec![(0, 0, 2.0)];
    let weights = CsrMatrix::from_triplets(1, num_edges, &triplets).unwrap();
    let chain = Chain::new(Arc::new(complex), 1, weights);

    let bd = StokesAdjunction::boundary(&ctx, &chain);
    assert_eq!(bd.grade(), 0);
    // A single weighted edge contributes to its two endpoint vertices only:
    //   2*(v1 - v0) = -2*v0 + 2*v1, so exactly columns 0 and 1 are touched.
    // The comment this test carried said as much, while the assertion allowed any non-zero
    // output including one that touched all three columns.
    assert_eq!(bd.weights().col_indices(), &vec![0usize, 1]);
    assert_eq!(bd.weights().values(), &vec![-2.0f64, 2.0]);
}

#[test]
fn test_exterior_derivative_short_coefficients_skips_out_of_range_columns() {
    // `exterior_derivative` guards each coboundary column with `col < coeffs.len()`.
    // Supplying a 0-form with FEWER coefficients than there are vertices forces the
    // guard's false arm for the out-of-range columns, so those terms are skipped.
    // Covers the column-bound guard skip at
    // src/extensions/hkt_gauge/hkt_adjunction_stokes.rs line 288.
    let complex = simple_complex();
    let ctx = StokesContext::new(complex);

    // The triangle has 3 vertices, but we deliberately supply only 1 coefficient.
    // Coboundary columns 1 and 2 therefore exceed `coeffs.len() == 1` and are skipped.
    let form = DifferentialForm::from_coefficients(0, 2, vec![7.0]);
    let dform = StokesAdjunction::exterior_derivative(&ctx, &form);

    // Result is still a well-formed 1-form on the 3 edges. Only vertex 0 is in range, so
    //   df(e01) = 0 - 7 = -7, df(e02) = 0 - 7 = -7, df(e12) = 0 - 0 = 0.
    assert_eq!(dform.degree(), 1);
    assert_eq!(dform.coefficients().as_slice(), &[-7.0f64, -7.0, 0.0]);
    // Every entry is finite (only the in-range column 0 ever contributes).
    assert!(
        dform
            .coefficients()
            .as_slice()
            .iter()
            .all(|x| x.is_finite())
    );
}

#[test]
fn test_boundary_of_a_chain_supported_off_the_operator_columns_is_zero() {
    // The inner accumulation looks each of the operator's column indices up in
    // the chain's weight map. A chain whose only stored weight sits at an index
    // the operator never names misses on every lookup, so no row accumulates and
    // the resulting chain carries no stored entry.
    let complex = simple_complex();
    let ctx = StokesContext::new(complex.clone());

    let weights = CsrMatrix::from_triplets(1, 3, &[(0, 2, 5.0f64)]).unwrap();
    let chain = Chain::new(Arc::new(complex), 1, weights);

    let bd = StokesAdjunction::boundary(&ctx, &chain);
    assert_eq!(bd.grade(), 0);
    // Column 2 is edge e12, which the boundary operator does name: d(5*e12) = -5*v1 + 5*v2.
    // The empty result this test used to assert, and the reasoning above it, describe a
    // lookup that misses when it should hit.
    assert_eq!(bd.weights().col_indices(), &vec![1usize, 2]);
    assert_eq!(bd.weights().values(), &vec![-5.0f64, 5.0]);
}
