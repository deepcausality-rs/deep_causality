/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_linear::CsrMatrix;
use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::{Simplex, SimplicialComplex, Skeleton};

#[test]
fn test_simplicial_complex_display() {
    let complex = create_triangle_complex();
    let display_str = format!("{}", complex);

    assert!(display_str.contains("CausalComplex:"));
    assert!(display_str.contains("Number of Skeletons: 3"));
    assert!(display_str.contains("Skeleton 0: (Dim: 0, Num Simplices: 3)"));
    assert!(display_str.contains("Skeleton 1: (Dim: 1, Num Simplices: 3)"));
    assert!(display_str.contains("Skeleton 2: (Dim: 2, Num Simplices: 1)"));
    assert!(display_str.contains("Number of Boundary Operators: 2"));
    assert!(display_str.contains("Boundary Operator 0: (Shape: 3x3, Num Non-Zeros: 6)"));
    assert!(display_str.contains("Boundary Operator 1: (Shape: 3x1, Num Non-Zeros: 3)"));
    assert!(display_str.contains("Number of Coboundary Operators: 0"));
}

#[test]
fn test_simplicial_complex_display_with_coboundary_operators() {
    // Construct a minimal complex with at least one coboundary operator so the
    // coboundary-loop branch in Display is exercised.
    let vertices = vec![Simplex::new(vec![0]), Simplex::new(vec![1])];
    let edges = vec![Simplex::new(vec![0, 1])];
    let skeletons = vec![Skeleton::new(0, vertices), Skeleton::new(1, edges)];

    let d1: CsrMatrix<i8> = CsrMatrix::from_triplets(2, 1, &[(1, 0, 1i8), (0, 0, -1)]).unwrap();
    // Coboundary δ_0 = (∂_1)^T : a 1x2 matrix.
    let codelta_0: CsrMatrix<i8> =
        CsrMatrix::from_triplets(1, 2, &[(0, 0, -1i8), (0, 1, 1)]).unwrap();

    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(skeletons, vec![d1], vec![codelta_0], Vec::new());

    let display_str = format!("{}", complex);
    assert!(display_str.contains("Number of Coboundary Operators: 1"));
    assert!(display_str.contains("Coboundary Operator 0: (Shape: 1x2, Num Non-Zeros: 2)"));
}

// ============================================================================
// Writer-error propagation
//
// Every `writeln!` in the Display impl ends in `?`. A formatter whose sink
// fails must surface that failure to the caller rather than reporting a
// successful format of a truncated value.
// ============================================================================

use core::fmt::{self, Write};

/// Accepts `budget` writes and reports an error on the next one.
struct FailAfter {
    budget: usize,
}

impl Write for FailAfter {
    fn write_str(&mut self, _s: &str) -> fmt::Result {
        if self.budget == 0 {
            return Err(fmt::Error);
        }
        self.budget -= 1;
        Ok(())
    }
}

#[test]
fn test_simplicial_complex_display_propagates_every_writer_failure() {
    // A complex with a skeleton loop, a boundary-operator loop and a
    // coboundary-operator loop, so each `?` in the impl is on the path.
    let vertices = vec![Simplex::new(vec![0]), Simplex::new(vec![1])];
    let edges = vec![Simplex::new(vec![0, 1])];
    let skeletons = vec![Skeleton::new(0, vertices), Skeleton::new(1, edges)];
    let d1: CsrMatrix<i8> = CsrMatrix::from_triplets(2, 1, &[(1, 0, 1i8), (0, 0, -1)]).unwrap();
    let cd0: CsrMatrix<i8> = CsrMatrix::from_triplets(1, 2, &[(0, 0, -1i8), (0, 1, 1)]).unwrap();
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(skeletons, vec![d1], vec![cd0], Vec::new());

    let mut first_ok = None;
    for budget in 0..4096 {
        let mut sink = FailAfter { budget };
        if fmt::write(&mut sink, format_args!("{}", complex)).is_ok() {
            first_ok = Some(budget);
            break;
        }
    }
    let first_ok = first_ok.expect("a large enough budget formats the whole value");
    assert!(first_ok > 0, "the value needs at least one write");

    // Cutting the sink short at any earlier point must be reported, never swallowed.
    for budget in 0..first_ok {
        let mut sink = FailAfter { budget };
        assert!(
            fmt::write(&mut sink, format_args!("{}", complex)).is_err(),
            "a sink that fails after {budget} writes must yield Err"
        );
    }
}
