/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `RiemannMap` against a small concrete implementation.
//!
//! The previous witness stored four unrelated types in a tuple and returned the fourth from
//! `curvature`, so it asserted nothing about curvature; `scatter` panicked. The trait now names one
//! vector space, so a witness can compute a real contraction and the tests can assert the
//! properties a curvature operator has: linearity in each slot and antisymmetry in the first two.

use deep_causality_haft::RiemannMap;

const DIM: usize = 3;

#[derive(Debug, Clone, PartialEq)]
struct Vec3([f64; DIM]);

impl Vec3 {
    fn scale(&self, k: f64) -> Self {
        Vec3([self.0[0] * k, self.0[1] * k, self.0[2] * k])
    }
    fn add(&self, o: &Self) -> Self {
        Vec3([self.0[0] + o.0[0], self.0[1] + o.0[1], self.0[2] + o.0[2]])
    }
    fn close(&self, o: &Self) -> bool {
        self.0
            .iter()
            .zip(o.0.iter())
            .all(|(a, b)| (a - b).abs() < 1e-9)
    }
}

/// `R^d_abc`, stored flat and antisymmetric in `(a, b)` by construction.
#[derive(Debug, Clone)]
struct Riemann3(Vec<f64>);

/// Flat index for `R^d_abc`.
fn at(d: usize, a: usize, b: usize, c: usize) -> usize {
    ((d * DIM + a) * DIM + b) * DIM + c
}

impl Riemann3 {
    /// Builds a tensor antisymmetric in its middle pair, which is what makes
    /// `R(u,v)w == -R(v,u)w` a consequence rather than a coincidence.
    fn antisymmetric(seed: f64) -> Self {
        let mut t = vec![0.0f64; DIM * DIM * DIM * DIM];
        let mut n = seed;
        for d in 0..DIM {
            for a in 0..DIM {
                for b in 0..a {
                    for c in 0..DIM {
                        n = (n * 7.13 + 1.7).fract();
                        let x = n * 2.0 - 1.0;
                        t[at(d, a, b, c)] = x;
                        t[at(d, b, a, c)] = -x;
                    }
                }
            }
        }
        let out = Self(t);
        // The generator establishes the precondition the antisymmetry test relies on, so it is
        // checked here rather than assumed.
        for d in 0..DIM {
            for a in 0..DIM {
                for b in 0..DIM {
                    for c in 0..DIM {
                        let s = out.0[at(d, a, b, c)] + out.0[at(d, b, a, c)];
                        assert!(s.abs() < 1e-12, "generator is not antisymmetric in (a, b)");
                    }
                }
            }
        }
        out
    }
}

struct Curvature3;

impl RiemannMap for Curvature3 {
    type Tensor = Riemann3;
    type Vector = Vec3;

    fn curvature(t: &Riemann3, u: &Vec3, v: &Vec3, w: &Vec3) -> Vec3 {
        let mut out = [0.0f64; DIM];
        for (d, o) in out.iter_mut().enumerate() {
            for a in 0..DIM {
                for b in 0..DIM {
                    for c in 0..DIM {
                        *o += t.0[at(d, a, b, c)] * u.0[a] * v.0[b] * w.0[c];
                    }
                }
            }
        }
        Vec3(out)
    }

    fn scatter(t: &Riemann3, in_1: &Vec3, in_2: &Vec3) -> (Vec3, Vec3) {
        let half = Self::curvature(t, in_1, in_2, in_2).scale(0.5);
        let other = Self::curvature(t, in_2, in_1, in_1).scale(0.5);
        (half, other)
    }
}

fn basis(i: usize) -> Vec3 {
    let mut d = [0.0; DIM];
    d[i] = 1.0;
    Vec3(d)
}

#[test]
fn curvature_is_antisymmetric_in_the_first_two_slots() {
    // R(u,v)w == -R(v,u)w, swept over several tensors and every basis triple.
    for s in 0..6 {
        let t = Riemann3::antisymmetric(0.13 + s as f64 * 0.17);
        for i in 0..DIM {
            for j in 0..DIM {
                for k in 0..DIM {
                    let (u, v, w) = (basis(i), basis(j), basis(k));
                    let uvw = Curvature3::curvature(&t, &u, &v, &w);
                    let vuw = Curvature3::curvature(&t, &v, &u, &w);
                    assert!(
                        uvw.close(&vuw.scale(-1.0)),
                        "R(e{i},e{j})e{k} != -R(e{j},e{i})e{k} for tensor {s}: {uvw:?} vs {vuw:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn curvature_vanishes_when_the_first_two_arguments_agree() {
    // A direct consequence of antisymmetry, and the case a single-fixture test would miss.
    let t = Riemann3::antisymmetric(0.31);
    for i in 0..DIM {
        for k in 0..DIM {
            let out = Curvature3::curvature(&t, &basis(i), &basis(i), &basis(k));
            assert!(
                out.close(&Vec3([0.0; DIM])),
                "R(u,u)w should vanish, got {out:?}"
            );
        }
    }
}

#[test]
fn curvature_is_linear_in_each_slot() {
    let t = Riemann3::antisymmetric(0.47);
    let (u, v, w) = (basis(0), basis(1), basis(2));
    let k = 2.5;

    // Homogeneity, one slot at a time.
    let scaled_u = Curvature3::curvature(&t, &u.scale(k), &v, &w);
    let scaled_v = Curvature3::curvature(&t, &u, &v.scale(k), &w);
    let scaled_w = Curvature3::curvature(&t, &u, &v, &w.scale(k));
    let base = Curvature3::curvature(&t, &u, &v, &w);

    for (label, got) in [("u", scaled_u), ("v", scaled_v), ("w", scaled_w)] {
        assert!(
            got.close(&base.scale(k)),
            "R is not homogeneous in {label}: {got:?} vs {:?}",
            base.scale(k)
        );
    }

    // Additivity in the third slot.
    let w2 = basis(0);
    let sum = Curvature3::curvature(&t, &u, &v, &w.add(&w2));
    let parts = Curvature3::curvature(&t, &u, &v, &w).add(&Curvature3::curvature(&t, &u, &v, &w2));
    assert!(
        sum.close(&parts),
        "R is not additive in w: {sum:?} vs {parts:?}"
    );
}

#[test]
fn scatter_returns_two_states_in_the_same_space() {
    let t = Riemann3::antisymmetric(0.59);
    let (a, b) = Curvature3::scatter(&t, &basis(0), &basis(1));
    // Both outputs are Vec3; the contraction is finite for finite input.
    assert!(
        a.0.iter().all(|x| x.is_finite()),
        "out-state 1 not finite: {a:?}"
    );
    assert!(
        b.0.iter().all(|x| x.is_finite()),
        "out-state 2 not finite: {b:?}"
    );
}
