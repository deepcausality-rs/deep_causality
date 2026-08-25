/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::{Compose, Hom, Injective, RingHom, Surjective};
use deep_causality_num_complex::{Complex, ComplexToReal, RealToComplex};

fn assert_ring_hom<H: RingHom>() {}
fn assert_injective<H: Injective>() {}
fn assert_surjective<H: Surjective>() {}

#[test]
fn test_embeds_with_zero_imaginary_part() {
    let e = RealToComplex::<f64>::new();
    let z = e.apply(2.5);
    assert_eq!(z.re, 2.5);
    assert_eq!(z.im, 0.0);
}

#[test]
fn test_embedding_is_an_injective_ring_hom() {
    assert_ring_hom::<RealToComplex<f64>>();
    assert_injective::<RealToComplex<f64>>();
    let e = RealToComplex::<f64>::new();
    assert_ne!(e.apply(1.0), e.apply(2.0));
}

#[test]
fn test_embedding_laws() {
    let e = RealToComplex::<f64>::new();
    let (a, b) = (3.0_f64, 4.0_f64);
    assert_eq!(e.apply(a + b), e.apply(a) + e.apply(b));
    assert_eq!(e.apply(a * b), e.apply(a) * e.apply(b));
    assert_eq!(e.apply(1.0), Complex::new(1.0, 0.0));
}

#[test]
fn test_projection_is_surjective_but_not_injective() {
    assert_surjective::<ComplexToReal<f64>>();
    let p = ComplexToReal::<f64>::new();
    assert_eq!(p.apply(Complex::new(3.0, 4.0)), 3.0);
    // two distinct complexes share a real part
    assert_eq!(
        p.apply(Complex::new(1.0, 1.0)),
        p.apply(Complex::new(1.0, 2.0))
    );
}

#[test]
fn test_projection_is_not_multiplicative() {
    // The reason `ComplexToReal` carries no `RingHom` impl:
    //   re(i · i) = re(-1) = -1   but   re(i) · re(i) = 0
    let p = ComplexToReal::<f64>::new();
    let i = Complex::new(0.0_f64, 1.0);
    assert_eq!(p.apply(i * i), -1.0);
    assert_eq!(p.apply(i) * p.apply(i), 0.0);
    assert_ne!(p.apply(i * i), p.apply(i) * p.apply(i));
}

#[test]
fn test_projection_is_additive_even_though_not_a_ring_hom() {
    let p = ComplexToReal::<f64>::new();
    let (z, w) = (Complex::new(1.0, 5.0), Complex::new(2.0, -3.0));
    assert_eq!(p.apply(z + w), p.apply(z) + p.apply(w));
}

#[test]
fn test_round_trip_composite_is_the_identity_on_the_reals() {
    let round = Compose::new(RealToComplex::<f64>::new(), ComplexToReal::<f64>::new());
    for x in [-2.5_f64, 0.0, 1.0, 7.25] {
        assert_eq!(round.apply(x), x);
    }
}

#[test]
fn test_composite_through_the_projection_is_only_a_hom() {
    // `ComplexToReal` is not a `RingHom`, so the composite is not one either. Only `Hom` resolves.
    fn only_hom<H: Hom>() {}
    only_hom::<Compose<RealToComplex<f64>, ComplexToReal<f64>>>();
}

#[test]
fn test_default_and_new_agree() {
    assert_eq!(RealToComplex::<f64>::new(), RealToComplex::<f64>::default());
    assert_eq!(ComplexToReal::<f64>::new(), ComplexToReal::<f64>::default());
}

#[test]
fn test_works_at_f32() {
    let z = RealToComplex::<f32>::new().apply(1.5_f32);
    assert_eq!(z, Complex::new(1.5_f32, 0.0));
    assert_eq!(ComplexToReal::<f32>::new().apply(z), 1.5_f32);
}
