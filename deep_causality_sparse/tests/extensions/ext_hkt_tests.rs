/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Functor, Monad};
use deep_causality_sparse::{CsrMatrix, CsrMatrixWitness};

#[test]
fn test_functor_fmap() {
    // 1x3 Matrix: [1.0, 2.0, 3.0]
    let triplets = vec![(0, 0, 1.0), (0, 1, 2.0), (0, 2, 3.0)];
    let matrix = CsrMatrix::from_triplets(1, 3, &triplets).unwrap();

    // Multiply by 2.0
    let mapped = <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(matrix, |x| x * 2.0);

    assert_eq!(mapped.values(), &vec![2.0, 4.0, 6.0]);
    assert_eq!(mapped.shape(), (1, 3));
}

#[test]
fn test_applicative_pure() {
    let matrix = <CsrMatrixWitness as Applicative<CsrMatrixWitness>>::pure(42.0);

    assert_eq!(matrix.shape(), (1, 1));
    assert_eq!(matrix.values(), &vec![42.0]);
}

#[test]
fn test_monad_bind() {
    // 1x2 Matrix: [10.0, 20.0]
    let triplets = vec![(0, 0, 10.0), (0, 1, 20.0)];
    let matrix = CsrMatrix::from_triplets(1, 2, &triplets).unwrap();

    // Bind: x -> [x+1, x+2] (expands each element to 2 elements)
    // Result should be 1x4: [11.0, 12.0, 21.0, 22.0]
    let result = <CsrMatrixWitness as Monad<CsrMatrixWitness>>::bind(matrix, |x| {
        let t = vec![(0, 0, x + 1.0), (0, 1, x + 2.0)];
        CsrMatrix::from_triplets(1, 2, &t).unwrap()
    });

    assert_eq!(result.values(), &vec![11.0, 12.0, 21.0, 22.0]);
    // The bind implementation linearizes the result, so shape will be (1, 4)
    assert_eq!(result.shape(), (1, 4));
}
