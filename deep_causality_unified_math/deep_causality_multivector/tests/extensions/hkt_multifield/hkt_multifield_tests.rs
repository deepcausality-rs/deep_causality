/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{CoMonad, Functor, Pure};
use deep_causality_multivector::{CausalMultiField, CausalMultiFieldWitness, Metric};

fn create_test_field() -> CausalMultiField<f32> {
    let shape = [2, 2, 2];
    let metric = Metric::from_signature(2, 0, 0);
    let dx = [0.1f32, 0.1, 0.1];
    CausalMultiField::zeros(shape, metric, dx)
}

#[test]
fn test_witness_new() {
    let _witness = CausalMultiFieldWitness::<f32>::new();
}

#[test]
fn test_functor_fmap() {
    // Both functor laws, asserted structurally on a non-constant field over an asymmetric grid
    // so an index or stride confusion in the rebuild is visible. The previous form mapped an
    // all-zeros field and looped over the slice, which cannot distinguish a correct rebuild
    // from a scrambled one and passes vacuously when the slice is empty.
    let w = CausalMultiField::<f32>::ones(
        [2, 1, 3],
        Metric::from_signature(2, 0, 0),
        [0.1f32, 0.2, 0.4],
    );
    assert_eq!(
        CausalMultiFieldWitness::<f32>::fmap(w.clone(), |x: f32| x),
        w
    );
    assert_eq!(
        CausalMultiFieldWitness::<f32>::fmap(w.clone(), |x: f32| (x + 1.0) * 2.0),
        CausalMultiFieldWitness::<f32>::fmap(
            CausalMultiFieldWitness::<f32>::fmap(w, |x: f32| x + 1.0),
            |x: f32| x * 2.0
        )
    );
}

#[test]
fn test_pure_lifts_into_a_single_cell() {
    let val = 42.0f32;
    let field = CausalMultiFieldWitness::<f32>::pure(val);

    // The defining property: `pure` is handed one value and `A` has no `Clone`, so the
    // context it builds must hold exactly one coefficient. Asserted as a count, not as a
    // loop over the slice, which would pass vacuously on an empty field.
    assert_eq!(
        field.data().as_slice().len(),
        1,
        "pure must build exactly one cell; more would require duplicating the value"
    );
    assert_eq!(field.data().as_slice()[0], val);

    // The minimal algebra, Cl(0,0,0), is what makes the cell count 1.
    assert_eq!(field.metric().dimension(), 0);
    assert_eq!(field.matrix_dim(), 1);
    assert_eq!(field.shape(), &[1, 1, 1]);
    assert_eq!(field.num_cells(), 1);
}

#[test]
fn test_comonad_left_counit_on_pure() {
    // extract(pure(a)) == a. An independent oracle: it names no internal shape, only the
    // comonad/pure agreement that any lawful pair must satisfy.
    for val in [0.0f32, 42.0, -7.5, f32::MIN_POSITIVE] {
        let field = CausalMultiFieldWitness::<f32>::pure(val);
        assert_eq!(CausalMultiFieldWitness::<f32>::extract(&field), val);
    }
}

#[test]
fn test_comonad_extract() {
    // A `pure` field holds one cell, so every candidate `extract` agrees on it. Use a field
    // whose buffer is provably non-constant, so reading anything but cell 0 is caught.
    let w = CausalMultiField::<f32>::ones(
        [1, 1, 1],
        Metric::from_signature(2, 0, 0),
        [0.1f32, 0.2, 0.4],
    );
    assert_eq!(w.data().as_slice(), &[1.0f32, 0.0, 0.0, 1.0]);
    assert_eq!(CausalMultiFieldWitness::<f32>::extract(&w), 1.0f32);
}

#[test]
fn test_comonad_extend_satisfies_the_left_counit_law() {
    // extend(w, extract) == w. The previous form asserted the opposite of the law: it described
    // `extend` as applying `f` to the *whole* field and broadcasting one answer to every cell,
    // then checked that every cell held that one answer. A comonad whose `extend` broadcasts
    // cannot satisfy this law on any field with more than one distinct coefficient.
    let w = CausalMultiField::<f32>::ones(
        [1, 1, 1],
        Metric::from_signature(2, 0, 0),
        [0.1f32, 0.2, 0.4],
    );
    assert_eq!(
        w.data().as_slice(),
        &[1.0f32, 0.0, 0.0, 1.0],
        "fixture is non-constant"
    );
    assert_eq!(
        CausalMultiFieldWitness::<f32>::extend(&w, |f: &CausalMultiField<f32>| {
            CausalMultiFieldWitness::<f32>::extract(f)
        }),
        w
    );
}

#[test]
fn test_multifield_zeros_creates_correct_shape() {
    let field = create_test_field();
    let data = field.data();

    // Shape should be [Nx, Ny, Nz, D, D] = [2, 2, 2, 2, 2]
    assert_eq!(data.shape(), &[2, 2, 2, 2, 2]);
}

#[test]
fn test_multifield_ones_creates_identity_matrices() {
    let shape = [1, 1, 1];
    let metric = Metric::from_signature(2, 0, 0);
    let dx = [0.1f32, 0.1, 0.1];

    let field = CausalMultiField::ones(shape, metric, dx);
    let data_vec = field.data().clone().to_vec();

    // For a single cell with 2x2 matrix, should be identity
    // [1, 0, 0, 1] in row-major order
    assert!((data_vec[0] - 1.0).abs() < 1e-6);
    assert!(data_vec[1].abs() < 1e-6);
    assert!(data_vec[2].abs() < 1e-6);
    assert!((data_vec[3] - 1.0).abs() < 1e-6);
}

#[test]
fn test_multifield_num_cells() {
    let field = create_test_field();
    assert_eq!(field.num_cells(), 8); // 2 * 2 * 2
}

#[test]
fn test_multifield_metric() {
    let field = create_test_field();
    let metric = field.metric();
    assert_eq!(metric.dimension(), 2);
}

#[test]
fn test_multifield_clone() {
    // One structural comparison covers shape, metric, dx and every coefficient. Comparing only
    // `data().shape()` and `metric()` checks two projections the compiler already guarantees a
    // derived `Clone` preserves, and never looks at the buffer.
    let field1 = CausalMultiField::<f32>::ones(
        [2, 1, 3],
        Metric::from_signature(2, 0, 0),
        [0.1f32, 0.2, 0.4],
    );
    let field2 = field1.clone();
    assert_eq!(field1, field2);
    // and the clone owns its buffer rather than aliasing the original
    let field3 = CausalMultiFieldWitness::<f32>::fmap(field2, |x: f32| x + 1.0);
    assert_ne!(field1, field3);
}

#[test]
fn test_multifield_add() {
    // `zeros + zeros == zeros` holds for `+`, `*`, `max` and for an operator that returns its
    // left argument, and the loop that checked it runs zero times on an empty grid. Use
    // operands that separate addition from the alternatives.
    let m = Metric::from_signature(2, 0, 0);
    let dx = [0.1f32, 0.2, 0.4];
    let a = CausalMultiField::<f32>::ones([1, 1, 1], m, dx);
    let z = CausalMultiField::<f32>::zeros([1, 1, 1], m, dx);

    assert_eq!(&a + &z, a, "right identity, structurally");
    let two = &a + &a;
    assert_eq!(two.data().as_slice(), &[2.0f32, 0.0, 0.0, 2.0]);
    assert_eq!(two.shape(), a.shape());
    assert_eq!(two.metric(), a.metric());
    assert_eq!(two.dx(), a.dx());
}

#[test]
fn test_multifield_sub() {
    // Subtracting zero cannot see a dropped right-hand side, and indices 1 and 2 were never
    // read. Use a non-zero subtrahend and assert every coefficient plus the geometry.
    let m = Metric::from_signature(2, 0, 0);
    let dx = [0.1f32, 0.2, 0.4];
    let a = CausalMultiField::<f32>::ones([1, 1, 1], m, dx);
    let two_a = &a + &a;

    assert_eq!(two_a - a.clone(), a, "2a - a == a, structurally");

    let z = CausalMultiField::<f32>::zeros([1, 1, 1], m, dx);
    assert_eq!(
        (z - a.clone()).data().as_slice(),
        &[-1.0f32, 0.0, 0.0, -1.0],
        "catches a sign flip"
    );
}
