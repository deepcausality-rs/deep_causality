/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for HKT operations on CausalMultiField.

use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiField, CausalMultiVector, MultiFieldWitness};
use deep_causality_tensor::CpuBackend;

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

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let doubled = MultiFieldWitness::fmap(field.clone(), |x| x * 2.0);

    let orig_coeffs = field.to_coefficients();
    let doubled_coeffs = doubled.to_coefficients();

    for (orig, doubled_mv) in orig_coeffs.iter().zip(doubled_coeffs.iter()) {
        for (o, d) in orig.data().iter().zip(doubled_mv.data().iter()) {
            assert!(
                (o * 2.0 - d).abs() < 1e-5,
                "fmap failed: {} * 2 != {}",
                o,
                d
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

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    // This tests that fmap can change types (f32 -> f32 in this case)
    let negated = MultiFieldWitness::fmap(field, |x| -x);

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

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [3, 3, 3], [1.0, 1.0, 1.0]);

    let mapped = MultiFieldWitness::fmap(field, |x| x + 1.0);

    assert_eq!(*mapped.shape(), [3, 3, 3]);
}

// =============================================================================
// pure tests
// =============================================================================

#[test]
fn test_pure_creates_singleton_field() {
    let field = MultiFieldWitness::pure(42.0f32);

    assert_eq!(*field.shape(), [1, 1, 1]);
    assert_eq!(field.num_cells(), 1);
}

#[test]
fn test_pure_contains_value() {
    let field = MultiFieldWitness::pure(3.5f32);

    let coeffs = field.to_coefficients();
    assert_eq!(coeffs.len(), 1);
    assert!((coeffs[0].data()[0] - 3.5).abs() < 1e-5);
}

// =============================================================================
// bind tests
// =============================================================================

#[test]
fn test_bind_basic() {
    // Test that bind can process a simple transformation
    // Note: bind naturally expands the field as each scalar produces a new field
    let field = MultiFieldWitness::pure(2.0f32);

    // Bind each scalar to produce a field with that value doubled
    let bound = MultiFieldWitness::bind(field, |x| MultiFieldWitness::pure(x * 2.0));

    // Extract should give us 4.0 (2.0 * 2.0)
    let result = MultiFieldWitness::extract(&bound);
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

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let focus = MultiFieldWitness::extract(&field);

    assert!(
        (focus - 1.0).abs() < 1e-5,
        "Extract should return 1.0, got {}",
        focus
    );
}

#[test]
fn test_extract_from_pure() {
    let value = 99.5f32;
    let field = MultiFieldWitness::pure(value);

    let extracted = MultiFieldWitness::extract(&field);

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

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let extended = MultiFieldWitness::extend(&field, |_fa| 1.0f32);

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

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    // Apply a function that extracts and squares the focus
    let extended = MultiFieldWitness::extend(&field, |fa| {
        let focus = MultiFieldWitness::extract(fa);
        focus * focus
    });

    let coeffs = extended.to_coefficients();
    // Each cell should have the same value (focus^2 = 1.0)
    for mv in &coeffs {
        assert!((mv.data()[0] - 1.0).abs() < 1e-4);
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

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let mapped = MultiFieldWitness::fmap(field.clone(), |x| x); // identity

    let orig_coeffs = field.to_coefficients();
    let mapped_coeffs = mapped.to_coefficients();

    for (orig, mapped_mv) in orig_coeffs.iter().zip(mapped_coeffs.iter()) {
        for (o, m) in orig.data().iter().zip(mapped_mv.data().iter()) {
            assert!((o - m).abs() < 1e-6, "Identity law violated");
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

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    let f = |x: f32| x + 1.0;
    let g = |x: f32| x * 2.0;

    // Compose: (f . g)(x) = f(g(x)) = (x * 2) + 1
    let composed = MultiFieldWitness::fmap(field.clone(), |x| f(g(x)));
    let sequential = MultiFieldWitness::fmap(MultiFieldWitness::fmap(field, g), f);

    let composed_coeffs = composed.to_coefficients();
    let sequential_coeffs = sequential.to_coefficients();

    for (c, s) in composed_coeffs.iter().zip(sequential_coeffs.iter()) {
        for (cv, sv) in c.data().iter().zip(s.data().iter()) {
            assert!((cv - sv).abs() < 1e-5, "Composition law violated");
        }
    }
}
