/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Adjunction, Foldable, Functor};
use deep_causality_linear::CsrMatrix;
use deep_causality_topology::{Chain, ChainWitness, Simplex, SimplicialComplex, Skeleton};
use std::sync::Arc;

fn create_simple_complex() -> Arc<SimplicialComplex<f64>> {
    // Single triangle: {0, 1, 2}
    let vertices = vec![
        Simplex::new(vec![0]),
        Simplex::new(vec![1]),
        Simplex::new(vec![2]),
    ];
    let skeleton_0 = Skeleton::new(0, vertices);

    // We only need 0-skeleton for the current HKT implementation of unit/left_adjunct
    // as it defaults to 0-skeleton in the code I read.
    Arc::new(SimplicialComplex::new(
        vec![skeleton_0],
        vec![],
        vec![],
        vec![],
    ))
}

#[test]
fn test_simplicial_complex_unit() {
    let complex = create_simple_complex();
    let val = 42.0;

    // Unit: Embed scalar into Chain<Chain<A, A>>
    let chain_of_chains: Chain<f64, Chain<f64, f64>> =
        ChainWitness::<f64>::unit(&(complex, 0), val);

    // Verify outer chain
    assert_eq!(chain_of_chains.grade(), 0);
    // Outer chain contains 1 element (the inner chain) at index 0 (unit impl details).
    let outer_w = chain_of_chains.weights();
    // Pure creates 1x1 matrix with value at (0,0).
    // Check first element of outer chain
    if let Some(inner_chain) = outer_w.values().iter().next() {
        let w: &CsrMatrix<f64> = inner_chain.weights();
        assert_eq!(w.get_value_at(0, 0), 42.0);
    } else {
        panic!("Unit resulted in empty outer chain");
    }
}

#[test]
fn test_simplicial_complex_left_adjunct() {
    let complex = create_simple_complex();

    // Left Adjunct: (Chain<A, A> -> B) -> (A -> Chain<B, B>)
    // f: Chain<f64, f64> -> f64
    let f = |c: Chain<f64, f64>| c.weights().values().iter().sum::<f64>();

    let chain: Chain<f64, f64> = ChainWitness::<f64>::left_adjunct(&(complex, 0), 0.0, f);

    // Expect Chain<f64, f64> containing f(unit(0.0)).
    // unit(0.0) -> Chain<f64, Chain<f64, f64>> with inner value 0.0.
    // f(inner) -> sum(0.0) -> 0.0.
    // So distinct chain with single value 0.0.

    assert_eq!(chain.weights().get_value_at(0, 0), 0.0);
}

#[test]
#[allow(unused_variables)]
fn test_simplicial_complex_counit() {
    let complex = create_simple_complex();

    // Counit: Chain<Chain<B, B>> -> B
    // We construct a nested chain manually or via unit.
    let inner_val = 100.0;
    // We can use unit to create Chain<f64, Chain<f64, f64>> easily.
    let chain_chain = ChainWitness::<f64>::unit(&(complex.clone(), 0), inner_val);

    let result = ChainWitness::<f64>::counit(&(complex, 0), chain_chain);
    assert_eq!(result, 100.0);
}

#[test]
fn test_simplicial_complex_right_adjunct() {
    let complex = create_simple_complex();

    // Right Adjunct: (A -> Chain<B, B>) -> (Chain<A, A> -> B)
    // Chain<A, A> with weights at 0 and 2.
    let size = 3;
    let weights =
        CsrMatrix::from_triplets(1, size, &[(0, 0, 2.0), (0, 2, 3.0)]).expect("Matrix failed");

    let chain = Chain::new(complex.clone(), 0, weights);

    // f: f64 -> Chain<f64, f64>
    // f(w) -> Chain with weight w*10 at index 0.
    let f = |w: f64| -> Chain<f64, f64> {
        let val = w * 10.0;
        let w_matrix = CsrMatrix::from_triplets(1, 1, &[(0, 0, val)]).unwrap();
        Chain::new(complex.clone(), 0, w_matrix)
    };

    // Execution:
    // fmap(chain, f) -> Chain<f64, Chain<f64, f64>>.    // Execution:
    // right_adjunct expects context: &(Arc<SimplicialComplex>, usize).
    // We clone complex for the context tuple to avoid move errors since closure borrows it.
    let ctx_complex = complex.clone();
    let result = ChainWitness::<f64>::right_adjunct(&(ctx_complex, 0), chain, f);

    assert_eq!(result, 20.0);
}

// ============================================================================
// Additional HKT Tests for Coverage
// ============================================================================

#[test]
fn test_simplicial_complex_right_adjunct_returns_inner_value() {
    // Drives the success path of `right_adjunct`: the produced `Chain<Chain<B, B>>` has a
    // non-empty outer chain whose first inner chain has a non-empty value list, so the
    // function returns `val` from the innermost `if let`.
    // Covers src/extensions/hkt_simplicial_complex/mod.rs line 144.
    let complex = create_simple_complex();

    // Chain<A, A> with a single weight at index 0.
    let weights = CsrMatrix::from_triplets(1, 1, &[(0, 0, 4.0)]).expect("Matrix failed");
    let chain = Chain::new(complex.clone(), 0, weights);

    let ctx_complex = complex.clone();
    let f = |w: f64| -> Chain<f64, f64> {
        let w_matrix = CsrMatrix::from_triplets(1, 1, &[(0, 0, w + 1.0)]).unwrap();
        Chain::new(complex.clone(), 0, w_matrix)
    };

    let result = ChainWitness::<f64>::right_adjunct(&(ctx_complex, 0), chain, f);
    assert_eq!(result, 5.0);
}

#[test]
#[should_panic(expected = "f returned a Chain that stores nothing")]
fn test_simplicial_complex_right_adjunct_panics_when_f_yields_an_empty_chain() {
    // Failure mode two of two: the input chain has a value, so `f` runs, but the chain `f`
    // returns stores nothing. There is no `B` to return and `B` carries no `Default`.
    let complex = create_simple_complex();

    let weights = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0)]).expect("Matrix failed");
    let chain = Chain::new(complex.clone(), 0, weights);

    let ctx_complex = complex.clone();
    let f = |_w: f64| -> Chain<f64, f64> {
        let empty: CsrMatrix<f64> = CsrMatrix::new();
        Chain::new(complex.clone(), 0, empty)
    };

    let _ = ChainWitness::<f64>::right_adjunct(&(ctx_complex, 0), chain, f);
}

#[test]
fn test_chain_functor_fmap() {
    let complex = create_simple_complex();
    let size = 3;
    // Create chain with weights [1.0, 2.0, 3.0]
    let weights = CsrMatrix::from_triplets(1, size, &[(0, 0, 1.0), (0, 1, 2.0), (0, 2, 3.0)])
        .expect("Matrix failed");
    let chain: Chain<f64, f64> = Chain::new(complex, 0, weights);

    // Apply fmap to double all values
    let doubled: Chain<f64, f64> = ChainWitness::<f64>::fmap(chain, |x| x * 2.0);

    // Verify doubled values
    assert_eq!(doubled.weights().get_value_at(0, 0), 2.0);
    assert_eq!(doubled.weights().get_value_at(0, 1), 4.0);
    assert_eq!(doubled.weights().get_value_at(0, 2), 6.0);
}

#[test]
fn test_chain_functor_fmap_type_change() {
    let complex = create_simple_complex();
    let size = 2;
    // Create chain with f64 weights
    let weights =
        CsrMatrix::from_triplets(1, size, &[(0, 0, 1.5), (0, 1, 2.5)]).expect("Matrix failed");
    let chain: Chain<f64, f64> = Chain::new(complex, 0, weights);

    // Apply fmap to convert to i32 (truncating)
    // The complex stays `f64`; only the coefficients become `i32`. Before the parameter
    // split this had to be `Chain<i32, i32>`, which is why the geometry could not survive.
    let ints: Chain<f64, i32> = ChainWitness::<f64>::fmap(chain, |x| x as i32);

    // Verify converted values
    assert_eq!(ints.weights().get_value_at(0, 0), 1);
    assert_eq!(ints.weights().get_value_at(0, 1), 2);
}

#[test]
fn test_chain_foldable_fold() {
    let complex = create_simple_complex();
    let size = 3;
    // Create chain with weights [1.0, 2.0, 3.0]
    let weights = CsrMatrix::from_triplets(1, size, &[(0, 0, 1.0), (0, 1, 2.0), (0, 2, 3.0)])
        .expect("Matrix failed");
    let chain: Chain<f64, f64> = Chain::new(complex, 0, weights);

    // Fold to compute sum
    let sum: f64 = ChainWitness::<f64>::fold(chain, 0.0, |acc, x| acc + x);

    // 1 + 2 + 3 = 6
    assert_eq!(sum, 6.0);
}

#[test]
fn test_chain_foldable_fold_product() {
    let complex = create_simple_complex();
    let size = 3;
    // Create chain with weights [1.0, 2.0, 3.0]
    let weights = CsrMatrix::from_triplets(1, size, &[(0, 0, 1.0), (0, 1, 2.0), (0, 2, 3.0)])
        .expect("Matrix failed");
    let chain: Chain<f64, f64> = Chain::new(complex, 0, weights);

    // Fold to compute product
    let product: f64 = ChainWitness::<f64>::fold(chain, 1.0, |acc, x| acc * x);

    // 1 * 2 * 3 = 6
    assert_eq!(product, 6.0);
}

#[test]
fn test_chain_foldable_fold_to_string() {
    let complex = create_simple_complex();
    let size = 2;
    let weights =
        CsrMatrix::from_triplets(1, size, &[(0, 0, 10.0), (0, 1, 20.0)]).expect("Matrix failed");
    let chain: Chain<f64, f64> = Chain::new(complex, 0, weights);

    // Fold to collect values as string
    let result: String = ChainWitness::<f64>::fold(chain, String::new(), |mut acc, x| {
        if !acc.is_empty() {
            acc.push_str(", ");
        }
        acc.push_str(&x.to_string());
        acc
    });

    assert_eq!(result, "10, 20");
}

#[test]
#[should_panic(expected = "cannot be called on a Chain that stores nothing")]
fn test_simplicial_complex_right_adjunct_empty_outer_chain_panics() {
    // An input chain with no stored weight produces an outer `Chain<Chain<B, B>>` with nothing to
    // unpack, so `f` is never applied. The message distinguishes this from the case where `f`
    // ran and returned an empty chain: a caller seeing this one passed an empty chain in.
    let complex = create_simple_complex();

    let empty: CsrMatrix<f64> = CsrMatrix::new();
    let chain = Chain::new(complex.clone(), 0, empty);

    let ctx_complex = complex.clone();
    let f = |w: f64| -> Chain<f64, f64> {
        let w_matrix = CsrMatrix::from_triplets(1, 1, &[(0, 0, w)]).unwrap();
        Chain::new(complex.clone(), 0, w_matrix)
    };

    let _ = ChainWitness::<f64>::right_adjunct(&(ctx_complex, 0), chain, f);
}
