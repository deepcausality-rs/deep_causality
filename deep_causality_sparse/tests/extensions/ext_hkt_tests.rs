/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Adjunction, Applicative, CoMonad, Foldable, Functor, Monad};
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
fn test_foldable_fold() {
    let triplets = vec![(0, 0, 1.0), (0, 1, 2.0), (0, 2, 3.0)];
    let matrix = CsrMatrix::from_triplets(1, 3, &triplets).unwrap();

    // Sum values
    let sum = <CsrMatrixWitness as Foldable<CsrMatrixWitness>>::fold(matrix, 0.0, |acc, x| acc + x);
    assert_eq!(sum, 6.0);
}

#[test]
fn test_applicative_pure() {
    let matrix = <CsrMatrixWitness as Applicative<CsrMatrixWitness>>::pure(42.0);

    assert_eq!(matrix.shape(), (1, 1));
    assert_eq!(matrix.values(), &vec![42.0]);
}

#[test]
fn test_applicative_apply_broadcast() {
    // Function wrapped in 1x1 matrix
    let func_ptr: fn(f64) -> f64 = |x: f64| x * 2.0;
    let f_matrix = <CsrMatrixWitness as Applicative<CsrMatrixWitness>>::pure(func_ptr);

    // Data matrix
    let triplets = vec![(0, 0, 10.0), (0, 1, 20.0)];
    let data_matrix = CsrMatrix::from_triplets(1, 2, &triplets).unwrap();

    let result = <CsrMatrixWitness as Applicative<CsrMatrixWitness>>::apply(f_matrix, data_matrix);

    assert_eq!(result.values(), &vec![20.0, 40.0]);
    assert_eq!(result.shape(), (1, 2));
}

#[test]
fn test_applicative_apply_intersection() {
    // Function matrix (1x2) - Shapes match data matrix (1x2)
    // This triggers the intersection logic.

    let triplets = vec![(0, 0, 1.0), (0, 1, 2.0)];
    let base_matrix = CsrMatrix::from_triplets(1, 2, &triplets).unwrap();

    let func_ptr: fn(f64) -> f64 = |x| x * 2.0;

    // Map f64 -> fn(f64)->f64
    let f_matrix = <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(base_matrix, |_| func_ptr);

    let triplets_data = vec![(0, 0, 10.0), (0, 1, 20.0)];
    let data_matrix = CsrMatrix::from_triplets(1, 2, &triplets_data).unwrap();

    let result = <CsrMatrixWitness as Applicative<CsrMatrixWitness>>::apply(f_matrix, data_matrix);

    // Should return result of applying function element-wise
    assert_eq!(result.values(), &vec![20.0, 40.0]);
    assert_eq!(result.shape(), (1, 2));
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

#[test]
fn test_comonad_extract() {
    let triplets = vec![(0, 0, 99.0)];
    let matrix = CsrMatrix::from_triplets(1, 1, &triplets).unwrap();

    let val = <CsrMatrixWitness as CoMonad<CsrMatrixWitness>>::extract(&matrix);
    assert_eq!(val, 99.0);
}

#[test]
#[should_panic(expected = "Comonad::extract cannot be called on an empty CsrMatrix")]
fn test_comonad_extract_panic() {
    let matrix: CsrMatrix<f64> = CsrMatrix::new();
    <CsrMatrixWitness as CoMonad<CsrMatrixWitness>>::extract(&matrix);
}

#[test]
fn test_comonad_extend() {
    // Matrix [10.0, 20.0]
    let triplets = vec![(0, 0, 10.0), (0, 1, 20.0)];
    let matrix = CsrMatrix::from_triplets(1, 2, &triplets).unwrap();

    // Extend: sum of all values in the matrix
    let extended =
        <CsrMatrixWitness as CoMonad<CsrMatrixWitness>>::extend(&matrix, |m: &CsrMatrix<f64>| {
            m.values().iter().sum::<f64>()
        });

    // Sum is 30.0 for first element (view [10, 20]), and 20.0 for second (view [20]).
    // Because shift_view(0, 1) crops column 0 (value 10).
    assert_eq!(extended.values(), &vec![30.0, 20.0]);
    assert_eq!(extended.shape(), (1, 2));
}

#[test]
fn test_adjunction_units() {
    // Unit creates CsrMatrix<CsrMatrix<A>>
    // ctx shape (2,2), value 5.0
    let ctx = (2, 2);
    let val = 5.0;

    let outer =
        <CsrMatrixWitness as Adjunction<CsrMatrixWitness, CsrMatrixWitness, (usize, usize)>>::unit(
            &ctx, val,
        );

    // Outer should be 1x1 wrapper
    assert_eq!(outer.shape(), (1, 1));
    assert_eq!(outer.values().len(), 1);

    let inner = &outer.values()[0];
    // Inner should be created with ctx shape (2,2) and value at 0,0
    assert_eq!(inner.shape(), (2, 2));
    assert_eq!(inner.get_value_at(0, 0), 5.0);
}

#[test]
fn test_adjunction_unit_invalid_shape() {
    // (0,0) shape means no elements. (0,0, val) would be out of bounds.
    let ctx = (0, 0);
    let val = 5.0;

    let outer =
        <CsrMatrixWitness as Adjunction<CsrMatrixWitness, CsrMatrixWitness, (usize, usize)>>::unit(
            &ctx, val,
        );
    // Should contain an empty inner matrix because from_triplets failed
    let inner = &outer.values()[0];
    assert_eq!(inner.values().len(), 0);
}

#[test]
fn test_adjunction_counit() {
    // Counit: Flatten then extract.
    // Use unit to create the nested structure properly (bypasses direct from_triplets for outer matrix which needs Copy)
    let ctx = (1, 1);
    let val = 7.0;
    let outer =
        <CsrMatrixWitness as Adjunction<CsrMatrixWitness, CsrMatrixWitness, (usize, usize)>>::unit(
            &ctx, val,
        );

    let result = <CsrMatrixWitness as Adjunction<
        CsrMatrixWitness,
        CsrMatrixWitness,
        (usize, usize),
    >>::counit(&ctx, outer);
    assert_eq!(result, 7.0);
}

#[test]
fn test_adjunction_left_adjunct() {
    // left_adjunct: a -> f(unit(a))
    // a = 3.0
    let ctx = (1, 1);
    let a = 3.0;

    let result_matrix = <CsrMatrixWitness as Adjunction<
        CsrMatrixWitness,
        CsrMatrixWitness,
        (usize, usize),
    >>::left_adjunct(&ctx, a, |inner_mat: CsrMatrix<f64>| {
        inner_mat.values()[0] * 10.0
    });

    // result_matrix should be 1x1 containing 30.0.
    assert_eq!(result_matrix.values()[0], 30.0);
}

#[test]
fn test_adjunction_right_adjunct() {
    // right_adjunct: la -> counit(fmap(la, f))
    let triplets = vec![(0, 0, 2.0)];
    let la = CsrMatrix::from_triplets(1, 1, &triplets).unwrap();

    // f turns 2.0 into [[20.0]] (inner matrix)
    let f = |x: f64| CsrMatrix::from_triplets(1, 1, &[(0, 0, x * 10.0)]).unwrap();

    let result = <CsrMatrixWitness as Adjunction<
        CsrMatrixWitness,
        CsrMatrixWitness,
        (usize, usize),
    >>::right_adjunct(&(1, 1), la, f);

    // Result is B (f64) -> 20.0
    assert_eq!(result, 20.0);
}
