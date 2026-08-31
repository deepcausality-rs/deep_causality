/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Foldable, Functor};
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

// ============================================================================
// Additional HKT Tests for Coverage
// ============================================================================

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
