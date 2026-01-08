/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{CWComplex, Cell, CellComplex};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum TestCell {
    Vertex(usize),
    Edge(usize, usize),        // connects vertices
    Face(usize, usize, usize), // triangle
}

impl Cell for TestCell {
    fn dim(&self) -> usize {
        match self {
            TestCell::Vertex(_) => 0,
            TestCell::Edge(_, _) => 1,
            TestCell::Face(_, _, _) => 2,
        }
    }

    fn boundary(&self) -> Vec<(Self, i8)> {
        match self {
            TestCell::Vertex(_) => vec![],
            TestCell::Edge(a, b) => {
                vec![(TestCell::Vertex(*b), 1), (TestCell::Vertex(*a), -1)]
            }
            TestCell::Face(a, b, c) => {
                vec![
                    (TestCell::Edge(*b, *c), 1),
                    (TestCell::Edge(*a, *c), -1),
                    (TestCell::Edge(*a, *b), 1),
                ]
            }
        }
    }
}

fn create_circle_complex() -> CellComplex<TestCell> {
    // A triangle is topologically a circle if we only include edges
    vec![
        TestCell::Vertex(0),
        TestCell::Vertex(1),
        TestCell::Vertex(2),
        TestCell::Edge(0, 1),
        TestCell::Edge(1, 2),
        TestCell::Edge(0, 2),
    ]
    .into_iter()
    .collect::<Vec<_>>()
    .into_iter() // Re-iterator? Just pass vec to from_cells
    .collect::<Vec<_>>()
    .pipe(CellComplex::from_cells)
}

// Helper trait to enable pipe syntax for cleaner setup (optional, but standard rust pattern)
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}
impl<T> Pipe for T {}

#[test]
fn test_homology_circle() {
    // Circle (empty triangle):
    // b0 = 1 (1 component)
    // b1 = 1 (1 hole)
    let complex = create_circle_complex();

    assert_eq!(CWComplex::betti_number(&complex, 0), 1);
    assert_eq!(CWComplex::betti_number(&complex, 1), 1);
}

#[test]
fn test_homology_disk() {
    // Disk (filled triangle):
    // b0 = 1
    // b1 = 0 (hole filled)
    let cells = vec![
        TestCell::Vertex(0),
        TestCell::Vertex(1),
        TestCell::Vertex(2),
        TestCell::Edge(0, 1),
        TestCell::Edge(1, 2),
        TestCell::Edge(0, 2),
        TestCell::Face(0, 1, 2),
    ];
    let complex = CellComplex::from_cells(cells);

    assert_eq!(CWComplex::betti_number(&complex, 0), 1);
    assert_eq!(CWComplex::betti_number(&complex, 1), 0);
}

#[test]
fn test_homology_two_components() {
    // Two disjoint edges: (0-1) and (2-3)
    let cells = vec![
        TestCell::Vertex(0),
        TestCell::Vertex(1),
        TestCell::Edge(0, 1),
        TestCell::Vertex(2),
        TestCell::Vertex(3),
        TestCell::Edge(2, 3),
    ];
    let complex = CellComplex::from_cells(cells);

    // b0 = 2 components
    assert_eq!(CWComplex::betti_number(&complex, 0), 2);
    // b1 = 0 loops
    assert_eq!(CWComplex::betti_number(&complex, 1), 0);
}

#[test]
fn test_boundary_matrix_invalid_k() {
    let complex = create_circle_complex();

    // k=0 boundary is empty
    let b0 = complex.compute_boundary_matrix(0);
    assert!(b0.values().is_empty());

    // k out of bounds is empty
    let b_inf = complex.compute_boundary_matrix(100);
    assert!(b_inf.values().is_empty());
}

#[test]
fn test_rank_of_matrix_calculation() {
    // This indirectly tests rank_of_matrix via betti numbers,
    // but let's verify specific rank logic if accessible or via property.
    // rank(∂1) for a triangle:
    // M is 3 vertices x 3 edges.
    // Rows sum to 0 (each col is +1, -1).
    // Matrix:
    //      e0(0-1) e1(1-2) e2(0-2)
    // v0:  -1       0      -1
    // v1:   1      -1       0
    // v2:   0       1       1
    //
    // v0 = -(v1 + v2) -> linearly dependent rows.
    // Rank is 2.

    // We can't access rank_of_matrix directly (private method),
    // but we can infer it:
    // b0 = n0 - rank(∂1) - rank(∂0)
    // 1  = 3  - rank(∂1) - 0
    // rank(∂1) = 2.
    // This confirms our calculation.
    let complex = create_circle_complex();
    // b0 logic check inside `betti_number(0)`:
    // b0 = n_0 - rank_0 - rank_1  (Wait, formula for b_k = dim Ker d_k - dim Im d_{k+1})
    // For k=0:
    // d_0: C_0 -> 0 (rank 0) -> Ker d_0 = C_0 (dim 3)
    // d_1: C_1 -> C_0 (rank 2) -> Im d_1 (dim 2)
    // b_0 = 3 - 2 = 1. Correct.

    assert_eq!(CWComplex::betti_number(&complex, 0), 1);
}
