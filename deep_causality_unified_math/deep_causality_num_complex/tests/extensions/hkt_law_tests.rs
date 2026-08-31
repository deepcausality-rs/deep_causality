/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Functor and Foldable laws for the Cayley-Dickson witnesses.
//!
//! These are fixed-arity products, so the functor laws hold structurally: `fmap` cannot lose or
//! reorder a slot. The tests assert that anyway, because "structurally obvious" is what the rest of
//! this workspace's law findings were also assumed to be.

use deep_causality_haft::{Foldable, Functor};
use deep_causality_num_complex::utils_tests::utils_hkt_law_tests::LawRng;
use deep_causality_num_complex::{
    Complex, ComplexWitness, Octonion, OctonionWitness, Quaternion, QuaternionWitness,
};

fn cx(rng: &mut LawRng) -> Complex<f64> {
    Complex {
        re: rng.scalar(),
        im: rng.scalar(),
    }
}
fn quat(rng: &mut LawRng) -> Quaternion<f64> {
    Quaternion {
        w: rng.scalar(),
        x: rng.scalar(),
        y: rng.scalar(),
        z: rng.scalar(),
    }
}
fn oct(rng: &mut LawRng) -> Octonion<f64> {
    Octonion {
        s: rng.scalar(),
        e1: rng.scalar(),
        e2: rng.scalar(),
        e3: rng.scalar(),
        e4: rng.scalar(),
        e5: rng.scalar(),
        e6: rng.scalar(),
        e7: rng.scalar(),
    }
}

// ---------------------------------------------------------------------------
// Functor identity
// ---------------------------------------------------------------------------

#[test]
fn functor_identity() {
    let mut rng = LawRng::new(0xC0FFEE);
    for _ in 0..32 {
        let c = cx(&mut rng);
        assert_eq!(
            <ComplexWitness as Functor<ComplexWitness>>::fmap(c, |x| x),
            c
        );

        let q = quat(&mut rng);
        assert_eq!(
            <QuaternionWitness as Functor<QuaternionWitness>>::fmap(q, |x| x),
            q
        );

        let o = oct(&mut rng);
        assert_eq!(
            <OctonionWitness as Functor<OctonionWitness>>::fmap(o, |x| x),
            o
        );
    }
}

#[test]
fn functor_composition() {
    let mut rng = LawRng::new(0xC0FFEE ^ 1);
    for _ in 0..32 {
        let (p, q) = (rng.scalar(), rng.scalar());
        let f = move |x: f64| x * p;
        let g = move |x: f64| x + q;

        let c = cx(&mut rng);
        let lhs = <ComplexWitness as Functor<ComplexWitness>>::fmap(
            <ComplexWitness as Functor<ComplexWitness>>::fmap(c, f),
            g,
        );
        let rhs = <ComplexWitness as Functor<ComplexWitness>>::fmap(c, move |x| g(f(x)));
        assert_eq!(lhs, rhs, "Complex: fmap(g) . fmap(f) != fmap(g . f)");

        let o = oct(&mut rng);
        let lhs = <OctonionWitness as Functor<OctonionWitness>>::fmap(
            <OctonionWitness as Functor<OctonionWitness>>::fmap(o, f),
            g,
        );
        let rhs = <OctonionWitness as Functor<OctonionWitness>>::fmap(o, move |x| g(f(x)));
        assert_eq!(lhs, rhs, "Octonion: fmap(g) . fmap(f) != fmap(g . f)");
    }
}

// ---------------------------------------------------------------------------
// fmap changes the component type, which is what NoConstraint buys
// ---------------------------------------------------------------------------

#[test]
fn fmap_changes_the_component_type() {
    let c = Complex {
        re: 1.5f64,
        im: -2.5,
    };
    let labels = <ComplexWitness as Functor<ComplexWitness>>::fmap(c, |x| format!("{x:.1}"));
    assert_eq!(labels.re, "1.5");
    assert_eq!(labels.im, "-2.5");

    let q = Quaternion {
        w: 1.0f64,
        x: 2.0,
        y: 3.0,
        z: 4.0,
    };
    let flags = <QuaternionWitness as Functor<QuaternionWitness>>::fmap(q, |x| x > 2.5);
    assert_eq!(
        (flags.w, flags.x, flags.y, flags.z),
        (false, false, true, true)
    );
}

#[test]
fn fmap_reaches_a_nested_component() {
    // Dropping the struct-level bound is what makes this type well formed. The outer `Complex`
    // maps its components without needing any arithmetic on them.
    let inner = Quaternion {
        w: 1.0f64,
        x: 2.0,
        y: 3.0,
        z: 4.0,
    };
    let nested: Complex<Quaternion<f64>> = Complex {
        re: inner,
        im: inner,
    };

    let sums = <ComplexWitness as Functor<ComplexWitness>>::fmap(nested, |q| {
        <QuaternionWitness as Foldable<QuaternionWitness>>::fold(q, 0.0, |acc, x| acc + x)
    });
    assert_eq!(sums.re, 10.0);
    assert_eq!(sums.im, 10.0);
}

// ---------------------------------------------------------------------------
// Foldable
// ---------------------------------------------------------------------------

#[test]
fn fold_visits_every_component_in_order() {
    let c = Complex {
        re: 1.0f64,
        im: 2.0,
    };
    let order = <ComplexWitness as Foldable<ComplexWitness>>::fold(c, Vec::new(), |mut acc, x| {
        acc.push(x);
        acc
    });
    assert_eq!(order, vec![1.0, 2.0], "Complex folds re then im");

    let q = Quaternion {
        w: 1.0f64,
        x: 2.0,
        y: 3.0,
        z: 4.0,
    };
    let order =
        <QuaternionWitness as Foldable<QuaternionWitness>>::fold(q, Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
    assert_eq!(
        order,
        vec![1.0, 2.0, 3.0, 4.0],
        "Quaternion folds w, x, y, z"
    );

    let o = Octonion {
        s: 1.0f64,
        e1: 2.0,
        e2: 3.0,
        e3: 4.0,
        e4: 5.0,
        e5: 6.0,
        e6: 7.0,
        e7: 8.0,
    };
    let order =
        <OctonionWitness as Foldable<OctonionWitness>>::fold(o, Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
    assert_eq!(
        order,
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
        "Octonion folds s, e1..e7"
    );
}

#[test]
fn fold_agrees_with_fmap_on_element_count() {
    // A functor over a fixed-arity product cannot lose a slot; the fold count pins that.
    let mut rng = LawRng::new(0xC0FFEE ^ 2);
    for _ in 0..16 {
        let o = oct(&mut rng);
        let mapped = <OctonionWitness as Functor<OctonionWitness>>::fmap(o, |x| x * 2.0);
        let n =
            <OctonionWitness as Foldable<OctonionWitness>>::fold(mapped, 0usize, |acc, _| acc + 1);
        assert_eq!(n, 8, "Octonion must always fold exactly eight components");
    }
}
