/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Properties of the curvature contraction on `CurvatureTensorWitness`.
//!
//! The previous suite asserted point values on hand-placed tensors. Its main case used
//! `CurvatureTensor::flat`, whose components are all zero, and asserted the result was zero: an
//! oracle that any implementation returning zeros passes, including one that ignores its arguments.
//! Its second case set a single component and passed `u == w`, so transposing those indices changed
//! nothing.
//!
//! These tests assert the properties a curvature operator has, over generated tensors and generated
//! vectors, with `u`, `v` and `w` distinct.

use deep_causality_haft::RiemannMap;
use deep_causality_metric::Metric;
use deep_causality_topology::utils_tests::LawRng;
use deep_causality_topology::{
    CurvatureSymmetry, CurvatureTensor, CurvatureTensorWitness, TensorVector,
};

type W = CurvatureTensorWitness<f64>;
const TOL: f64 = 1e-9;

/// Components antisymmetric in the `(a, b)` pair.
///
/// This is what makes `R(u,v)w == -R(v,u)w` a consequence rather than an accident. A generator that
/// does not establish it turns the antisymmetry check into a test of nothing, so the result is
/// verified before it is used.
fn antisymmetric_tensor(dim: usize, rng: &mut LawRng) -> CurvatureTensor<f64> {
    let mut comps = vec![0.0f64; dim * dim * dim * dim];
    let at = |d: usize, a: usize, b: usize, c: usize| ((d * dim + a) * dim + b) * dim + c;

    for d in 0..dim {
        for a in 0..dim {
            for b in 0..a {
                for c in 0..dim {
                    let x = rng.well_scaled(1.0);
                    comps[at(d, a, b, c)] = x;
                    comps[at(d, b, a, c)] = -x;
                }
            }
        }
    }

    // Self-check: a property test is only as good as the precondition its generator establishes.
    for d in 0..dim {
        for a in 0..dim {
            for b in 0..dim {
                for c in 0..dim {
                    let s = comps[at(d, a, b, c)] + comps[at(d, b, a, c)];
                    assert!(s.abs() < 1e-12, "generator failed to be antisymmetric in (a,b)");
                }
            }
        }
    }

    CurvatureTensor::from_generator(dim, Metric::Euclidean(dim), CurvatureSymmetry::None, {
        let comps = comps.clone();
        move |d, a, b, c| comps[at(d, a, b, c)]
    })
}

fn vector(dim: usize, rng: &mut LawRng) -> TensorVector<f64> {
    TensorVector::new(&rng.well_scaled_vec(dim, 3.0))
}

fn close(a: &TensorVector<f64>, b: &TensorVector<f64>) -> bool {
    a.dim() == b.dim()
        && a.as_slice()
            .iter()
            .zip(b.as_slice())
            .all(|(x, y)| (x - y).abs() <= TOL * (1.0 + x.abs().max(y.abs())))
}

fn scaled(v: &TensorVector<f64>, k: f64) -> TensorVector<f64> {
    TensorVector::new(&v.as_slice().iter().map(|x| x * k).collect::<Vec<_>>())
}

fn added(a: &TensorVector<f64>, b: &TensorVector<f64>) -> TensorVector<f64> {
    TensorVector::new(
        &a.as_slice()
            .iter()
            .zip(b.as_slice())
            .map(|(x, y)| x + y)
            .collect::<Vec<_>>(),
    )
}

#[test]
fn curvature_is_antisymmetric_in_the_first_two_slots() {
    let mut rng = LawRng::new(0xA117_5EED);
    for dim in [2usize, 3, 4] {
        for case in 0..8 {
            let t = antisymmetric_tensor(dim, &mut rng);
            // u, v and w are distinct and are not basis vectors, so an index transposition shows.
            let (u, v, w) = (
                vector(dim, &mut rng),
                vector(dim, &mut rng),
                vector(dim, &mut rng),
            );

            let uvw = W::curvature(&t, &u, &v, &w);
            let vuw = W::curvature(&t, &v, &u, &w);

            assert!(
                close(&uvw, &scaled(&vuw, -1.0)),
                "R(u,v)w != -R(v,u)w at dim={dim} case={case}: {:?} vs {:?}",
                uvw.as_slice(),
                vuw.as_slice()
            );
        }
    }
}

#[test]
fn curvature_vanishes_when_the_first_two_arguments_agree() {
    let mut rng = LawRng::new(0xA117_5EED ^ 1);
    for dim in [2usize, 3, 4] {
        let t = antisymmetric_tensor(dim, &mut rng);
        for _ in 0..8 {
            let (u, w) = (vector(dim, &mut rng), vector(dim, &mut rng));
            let out = W::curvature(&t, &u, &u, &w);
            assert!(
                close(&out, &TensorVector::zeros(dim)),
                "R(u,u)w should vanish at dim={dim}, got {:?}",
                out.as_slice()
            );
        }
    }
}

#[test]
fn curvature_is_homogeneous_in_each_slot() {
    let mut rng = LawRng::new(0xA117_5EED ^ 2);
    for dim in [2usize, 3] {
        let t = antisymmetric_tensor(dim, &mut rng);
        for _ in 0..8 {
            let (u, v, w) = (
                vector(dim, &mut rng),
                vector(dim, &mut rng),
                vector(dim, &mut rng),
            );
            let k = rng.well_scaled(4.0);
            let base = scaled(&W::curvature(&t, &u, &v, &w), k);

            for (slot, got) in [
                ("u", W::curvature(&t, &scaled(&u, k), &v, &w)),
                ("v", W::curvature(&t, &u, &scaled(&v, k), &w)),
                ("w", W::curvature(&t, &u, &v, &scaled(&w, k))),
            ] {
                assert!(
                    close(&got, &base),
                    "R is not homogeneous in {slot} at dim={dim}, k={k}"
                );
            }
        }
    }
}

#[test]
fn curvature_is_additive_in_the_transported_slot() {
    let mut rng = LawRng::new(0xA117_5EED ^ 3);
    for dim in [2usize, 3] {
        let t = antisymmetric_tensor(dim, &mut rng);
        for _ in 0..8 {
            let (u, v) = (vector(dim, &mut rng), vector(dim, &mut rng));
            let (w1, w2) = (vector(dim, &mut rng), vector(dim, &mut rng));

            let sum = W::curvature(&t, &u, &v, &added(&w1, &w2));
            let parts = added(
                &W::curvature(&t, &u, &v, &w1),
                &W::curvature(&t, &u, &v, &w2),
            );
            assert!(sum.as_slice().len() == parts.as_slice().len());
            assert!(
                close(&sum, &parts),
                "R(u,v)(w1+w2) != R(u,v)w1 + R(u,v)w2 at dim={dim}"
            );
        }
    }
}

#[test]
fn flat_spacetime_gives_zero_and_curved_spacetime_does_not() {
    // The zero result on a flat tensor is necessary but not sufficient: an implementation that
    // ignored its arguments would also pass it. The second half is what makes the pair meaningful.
    let mut rng = LawRng::new(0xA117_5EED ^ 4);
    let dim = 4;

    let flat: CurvatureTensor<f64> = CurvatureTensor::flat(dim);
    let (u, v, w) = (
        TensorVector::<f64>::basis(dim, 0),
        TensorVector::<f64>::basis(dim, 1),
        TensorVector::<f64>::new(&[1.0, 2.0, 3.0, 4.0]),
    );
    let flat_out = W::curvature(&flat, &u, &v, &w);
    assert!(
        close(&flat_out, &TensorVector::zeros(dim)),
        "flat spacetime should give zero deviation, got {:?}",
        flat_out.as_slice()
    );

    let curved = antisymmetric_tensor(dim, &mut rng);
    let curved_out = W::curvature(&curved, &u, &v, &w);
    assert!(
        !close(&curved_out, &TensorVector::zeros(dim)),
        "a curved tensor must not give zero deviation, or the flat case proves nothing"
    );
}

#[test]
fn scatter_produces_finite_states_and_respects_scaling() {
    let mut rng = LawRng::new(0xA117_5EED ^ 5);
    let dim = 3;
    let t = antisymmetric_tensor(dim, &mut rng);

    let (a, b) = (vector(dim, &mut rng), vector(dim, &mut rng));
    let (o1, o2) = W::scatter(&t, &a, &b);

    assert!(
        o1.as_slice().iter().all(|x| x.is_finite()) && o2.as_slice().iter().all(|x| x.is_finite()),
        "scattering produced a non-finite out-state"
    );
    assert_eq!(o1.dim(), dim, "out-state 1 has the wrong dimension");
    assert_eq!(o2.dim(), dim, "out-state 2 has the wrong dimension");

    // The amplitude is bilinear in the in-states, so scaling one scales both outputs.
    let k = 3.0;
    let (s1, s2) = W::scatter(&t, &scaled(&a, k), &b);
    assert!(
        close(&s1, &scaled(&o1, k)) && close(&s2, &scaled(&o2, k)),
        "scattering is not linear in the first in-state"
    );
}
