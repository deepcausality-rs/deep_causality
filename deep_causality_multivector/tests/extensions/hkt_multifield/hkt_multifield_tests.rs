/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for HKT operations on CausalMultiField.

use deep_causality_haft::{Applicative, CoMonad, Functor, Monad};
use deep_causality_multivector::{
    CausalMultiField, CausalMultiFieldWitness, CausalMultiVector, DefaultMultivectorBackend, Metric,
};

// =============================================================================
// Helper
// =============================================================================

fn create_singleton_field(value: f32) -> CausalMultiField<DefaultMultivectorBackend, f32> {
    let metric = Metric::Euclidean(0);
    let mv = CausalMultiVector::scalar(value, metric);
    CausalMultiField::<DefaultMultivectorBackend, f32>::from_coefficients(
        &[mv],
        [1, 1, 1],
        [1.0, 1.0, 1.0],
    )
}

// =============================================================================
// fmap tests
// =============================================================================

#[test]
fn test_fmap_doubles_values() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<DefaultMultivectorBackend, f32>::from_coefficients(
        &mvs,
        [2, 2, 2],
        [1.0, 1.0, 1.0],
    );

    let doubled = CausalMultiFieldWitness::fmap(field.clone(), |x| x * 2.0);

    let orig_coeffs = field.to_coefficients();
    let doubled_coeffs = doubled.to_coefficients();

    for (orig, doubled_mv) in orig_coeffs.iter().zip(doubled_coeffs.iter()) {
        for (o, d) in orig.data().iter().zip(doubled_mv.data().iter()) {
            let o_f32: f32 = *o;
            let d_f32: f32 = *d;
            assert!(
                (o_f32 * 2.0 - d_f32).abs() < 1e-5,
                "fmap failed: {} * 2 != {}",
                o_f32,
                d_f32
            );
        }
    }
}

#[test]
fn test_fmap_type_conversion() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<DefaultMultivectorBackend, f32>::from_coefficients(
        &mvs,
        [2, 2, 2],
        [1.0, 1.0, 1.0],
    );

    // This tests that fmap can change types (f32 -> f32 in this case)
    let negated = CausalMultiFieldWitness::fmap(field, |x| -x);

    let coeffs = negated.to_coefficients();
    assert_eq!(coeffs.len(), 8);
    assert!(coeffs[0].data()[0] < 0.0, "First value should be negated");
}

#[test]
fn test_fmap_preserves_shape() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(27);
    for i in 0..27 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<DefaultMultivectorBackend, f32>::from_coefficients(
        &mvs,
        [3, 3, 3],
        [1.0, 1.0, 1.0],
    );

    let mapped = CausalMultiFieldWitness::fmap(field, |x| x + 1.0);

    assert_eq!(*mapped.shape(), [3, 3, 3]);
}

// =============================================================================
// pure tests - REPLACED WITH FACTORY TESTS
// =============================================================================

#[test]
fn test_factory_creates_singleton_field() {
    // Replaces test_pure_creates_singleton_field
    let field = create_singleton_field(42.0f32); // Use factory instead of pure

    assert_eq!(*field.shape(), [1, 1, 1]);
    assert_eq!(field.num_cells(), 1);
}

#[test]
fn test_factory_contains_value() {
    // Replaces test_pure_contains_value
    let field = create_singleton_field(3.5f32);

    let coeffs = field.to_coefficients();
    assert_eq!(coeffs.len(), 1);
    assert!((coeffs[0].data()[0] - 3.5).abs() < 1e-5);
}

#[test]
#[should_panic(expected = "Applicative::pure for CausalMultiField requires context")]
fn test_pure_panics_as_expected() {
    // Verify that calling pure actually panics with the documented message
    let _ = CausalMultiFieldWitness::<DefaultMultivectorBackend>::pure(42.0);
}

// =============================================================================
// bind tests
// =============================================================================

#[test]
fn test_bind_basic() {
    // Test that bind can process a simple transformation
    // Note: bind naturally expands the field as each scalar produces a new field
    let field = create_singleton_field(2.0f32); // Use factory

    // Bind each scalar to produce a field with that value doubled
    // The closure also uses create_singleton_field
    let bound = CausalMultiFieldWitness::<DefaultMultivectorBackend>::bind(field, |x| {
        create_singleton_field(x * 2.0)
    });

    // Extract should give us 4.0 (2.0 * 2.0)
    let result = CausalMultiFieldWitness::extract(&bound);
    assert!(
        (result - 4.0).abs() < 1e-5,
        "Bind should transform: expected 4.0, got {}",
        result
    );
}

// =============================================================================
// extract tests
// =============================================================================

#[test]
fn test_extract_gets_first_scalar() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32; // First cell has scalar = 1.0
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<DefaultMultivectorBackend, f32>::from_coefficients(
        &mvs,
        [2, 2, 2],
        [1.0, 1.0, 1.0],
    );

    let focus = CausalMultiFieldWitness::extract(&field);

    assert!(
        (focus - 1.0).abs() < 1e-5,
        "Extract should return 1.0, got {}",
        focus
    );
}

#[test]
fn test_extract_from_factory() {
    let value = 99.5f32;
    let field = create_singleton_field(value);

    let extracted = CausalMultiFieldWitness::extract(&field);

    assert!((extracted - value).abs() < 1e-5);
}

// =============================================================================
// extend tests
// =============================================================================

#[test]
fn test_extend_preserves_shape() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<DefaultMultivectorBackend, f32>::from_coefficients(
        &mvs,
        [2, 2, 2],
        [1.0, 1.0, 1.0],
    );

    // Dummy extend that returns 1.0
    // We assume extend will create field with same metric/shape but values mapped
    let extended = CausalMultiFieldWitness::extend(&field, |_fa| 1.0f32);

    // Note: our extend implementation currently reuses fa structure.
    // If output is f32 (scalar), dx/metric might be problematic if not carefully handled in extend impl.
    // But let's check shape preservation first.

    assert_eq!(*extended.shape(), [2, 2, 2]);
}

#[test]
fn test_extend_applies_function() {
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<DefaultMultivectorBackend, f32>::from_coefficients(
        &mvs,
        [2, 2, 2],
        [1.0, 1.0, 1.0],
    );

    // Apply a function that extracts and squares the focus
    let extended = CausalMultiFieldWitness::extend(&field, |fa| {
        let focus = CausalMultiFieldWitness::extract(fa);
        focus * focus
    });

    let coeffs = extended.to_coefficients();
    // Each cell should have the same value (focus^2 = (i+1)^2).
    // Wait, test logic:
    // data[0] = (i+1).
    // extract(fa) where fa is a shifted view centered at i.
    // So extract(fa) returns value at i.
    // So we expect (i+1)^2.
    // Previous failing test expected 1.0 constant.
    // Why?
    // "let val: f32 = mv.data()[0]; assert!((val - 1.0).abs() < 1e-4);"
    // Oh, because original test iterated mvs in a way that presumably was 1.0?
    // No, loop i=0..8. data[0] = i+1. So 1, 2, 3...
    // The test assertion `val - 1.0` was simply incorrect for this setup unless extend does something else.
    // Or maybe the loop sets ALL to 1?
    // "mvs.push(...)" inside loop.
    // Yes, values are 1, 2, 3...
    // So assertion `val - 1.0` would fail for i>0.
    // I will fix the assertion to match the data logic.
    // Since extend preserves order (impl details: loop i=0..n, push f(view)).
    // view is shifted. extract(view) -> value at center -> original[i].
    // So result[i] = original[i]^2.

    for (i, mv) in coeffs.iter().enumerate() {
        let val: f32 = mv.data()[0];
        let original_val = (i + 1) as f32;
        let expected = original_val * original_val;
        assert!(
            (val - expected).abs() < 1e-4,
            "At index {}, expected {}, got {}",
            i,
            expected,
            val
        );
    }
}

// =============================================================================
// Functor law tests
// =============================================================================

#[test]
fn test_functor_identity_law() {
    // fmap(id, fa) == fa
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<DefaultMultivectorBackend, f32>::from_coefficients(
        &mvs,
        [2, 2, 2],
        [1.0, 1.0, 1.0],
    );

    let mapped = CausalMultiFieldWitness::fmap(field.clone(), |x| x); // identity

    let orig_coeffs = field.to_coefficients();
    let mapped_coeffs = mapped.to_coefficients();

    for (orig, mapped_mv) in orig_coeffs.iter().zip(mapped_coeffs.iter()) {
        for (o, m) in orig.data().iter().zip(mapped_mv.data().iter()) {
            let o_f32: f32 = *o;
            let m_f32: f32 = *m;
            assert!((o_f32 - m_f32).abs() < 1e-6, "Identity law violated");
        }
    }
}

#[test]
fn test_functor_composition_law() {
    // fmap(f . g, fa) == fmap(f, fmap(g, fa))
    let metric = Metric::from_signature(2, 0, 0);
    let num_blades = 4;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = (i + 1) as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<DefaultMultivectorBackend, f32>::from_coefficients(
        &mvs,
        [2, 2, 2],
        [1.0, 1.0, 1.0],
    );

    let f = |x: f32| x + 1.0;
    let g = |x: f32| x * 2.0;

    // Compose: (f . g)(x) = f(g(x)) = (x * 2) + 1
    let composed = CausalMultiFieldWitness::fmap(field.clone(), |x| f(g(x)));
    let sequential = CausalMultiFieldWitness::fmap(CausalMultiFieldWitness::fmap(field, g), f);

    let composed_coeffs = composed.to_coefficients();
    let sequential_coeffs = sequential.to_coefficients();

    for (c, s) in composed_coeffs.iter().zip(sequential_coeffs.iter()) {
        for (cv, sv) in c.data().iter().zip(s.data().iter()) {
            let cv_f32: f32 = *cv;
            let sv_f32: f32 = *sv;
            assert!((cv_f32 - sv_f32).abs() < 1e-5, "Composition law violated");
        }
    }
}
