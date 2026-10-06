/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::RealField;
use deep_causality_context::{
    MetricSignature, MetricTensor4D, MetricTensorError, MetricTensorErrorEnum, TangentSpacetime,
};
use deep_causality_metric::{Metric, detect_convention, is_lorentzian};
use deep_causality_num::{Float106, FromPrimitive};

/// An event at the origin with tangent vector `(1, 0, 0, 0)` and the default tensor
/// `diag(−c², 1, 1, 1)`.
fn event<R: RealField + FromPrimitive>() -> TangentSpacetime<R> {
    let (z, one) = (R::zero(), R::one());
    TangentSpacetime::new(1, z, z, z, z, one, z, z, z)
}

#[test]
fn test_reports_a_four_dimensional_lorentzian_signature() {
    let t = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(t.metric(), Metric::Lorentzian(4));
    assert!(is_lorentzian(&t.metric()));
    assert_eq!(detect_convention(&t.metric()), Some(true));
}

#[test]
fn test_the_signature_survives_replacing_every_tensor_component() {
    // This is the case the type exists for: a numerically evolved metric whose components all
    // change while the signature, which is invariant under continuous evolution, does not.
    let mut t = event();
    let before = t.metric();

    t.update_metric_tensor([
        [-9.0, 0.5, 0.25, 0.125],
        [0.5, 3.0, 0.75, 0.5],
        [0.25, 0.75, 3.0, 0.25],
        [0.125, 0.5, 0.25, 3.0],
    ])
    .expect("a symmetric tensor of signature (−, +, +, +)");

    assert_ne!(t.metric_tensor()[0][0], -1.0);
    assert_eq!(t.metric(), before);
    assert_eq!(t.metric(), Metric::Lorentzian(4));
}

fn diag(d: [f64; 4]) -> [[f64; 4]; 4] {
    let mut m = [[0.0; 4]; 4];
    for (i, v) in d.into_iter().enumerate() {
        m[i][i] = v;
    }
    m
}

#[test]
fn test_a_tensor_that_is_not_lorentzian_is_refused_and_the_tensor_kept() {
    // Positive definite, degenerate, two negative axes, and asymmetric: none is a metric of
    // signature (−, +, +, +), so each is refused and the node keeps its default tensor.
    let mut asymmetric = diag([-1.0, 1.0, 1.0, 1.0]);
    asymmetric[1][2] = 0.5;
    let bad = [
        (
            diag([1.0, 1.0, 1.0, 1.0]),
            MetricTensorError::Signature(4, 0, 0),
        ),
        (
            diag([-1.0, 1.0, 1.0, 0.0]),
            MetricTensorError::Signature(2, 1, 1),
        ),
        (
            diag([-1.0, -1.0, 1.0, 1.0]),
            MetricTensorError::Signature(2, 2, 0),
        ),
        (asymmetric, MetricTensorError::Asymmetric(1, 2)),
    ];
    for (tensor, expected) in bad {
        let mut t = event();
        let before = t.metric_tensor();
        assert_eq!(t.update_metric_tensor(tensor), Err(expected));
        assert_eq!(t.metric_tensor(), before);
    }
}

#[test]
fn test_a_lorentzian_tensor_with_a_zero_diagonal_is_accepted() {
    // [[0, 1], [1, 0]] in the t-x block has eigenvalues ±1, so with two positive spatial axes the
    // signature is (−, +, +, +) although no diagonal entry is negative. The inertia count must
    // read it through the 2×2 block, not the diagonal.
    let mut tensor = diag([0.0, 0.0, 1.0, 1.0]);
    tensor[0][1] = 1.0;
    tensor[1][0] = 1.0;
    let mut t = event();
    assert!(t.update_metric_tensor(tensor).is_ok());
    assert_eq!(t.metric_tensor(), tensor);
}

#[test]
fn test_the_signature_check_at_every_precision() {
    // Row K. The default tensor, a Lorentzian tensor read through a 2×2 block, and a
    // positive-definite tensor, at f32 and Float106.
    fn check<R>(lift: impl Fn(f64) -> R)
    where
        R: RealField + FromPrimitive + core::fmt::Debug,
    {
        let z = lift(0.0);
        let mut t = event::<R>();
        // The default tensor, c² ≈ 9·10¹⁶ beside 1, passes the check it is held to.
        let default = t.metric_tensor();
        assert!(t.update_metric_tensor(default).is_ok(), "{default:?}");
        // Its negation, diag(c², −1, −1, −1), has one positive and three negative eigenvalues.
        let negated = default.map(|row| row.map(|v| -v));
        assert!(t.update_metric_tensor(negated).is_err(), "{negated:?}");
        let mut block = [[z; 4]; 4];
        block[0][1] = lift(1.0);
        block[1][0] = lift(1.0);
        block[2][2] = lift(1.0);
        block[3][3] = lift(1.0);
        assert!(t.update_metric_tensor(block).is_ok());
        let mut riemannian = [[z; 4]; 4];
        for (i, row) in riemannian.iter_mut().enumerate() {
            row[i] = lift(1.0);
        }
        assert!(t.update_metric_tensor(riemannian).is_err());
        assert_eq!(t.metric_tensor(), block);
    }
    check(|v| v as f32);
    check(Float106::from);
}

#[test]
fn test_a_lorentzian_tensor_that_couples_t_and_x_is_accepted() {
    // The t–x block [[1, 2], [2, 1]] has eigenvalues 1 ± 2, that is 3 and −1, so with two
    // positive spatial axes the tensor is Lorentzian although its time diagonal is positive.
    // Eliminating t leaves 1 − 2·2/1 = −3 in the x slot: the elimination must subtract.
    let mut tensor = diag([1.0, 1.0, 1.0, 1.0]);
    tensor[0][1] = 2.0;
    tensor[1][0] = 2.0;
    let mut t = event();
    assert!(t.update_metric_tensor(tensor).is_ok());
    assert_eq!(t.metric_tensor(), tensor);
}

#[test]
fn test_the_refusal_reports_the_inertia_it_found() {
    // diag(−1, 1, 1, 0) has eigenvalues −1, 1, 1 and 0, read off the diagonal.
    let mut t = event();
    let err = t
        .update_metric_tensor(diag([-1.0, 1.0, 1.0, 0.0]))
        .expect_err("a degenerate tensor is refused");
    assert_eq!(
        err.0,
        MetricTensorErrorEnum::Signature {
            positive: 2,
            negative: 1,
            zero: 1
        }
    );
}

/// `Sᵀ D S` for the diagonal `d` and a unimodular integer `S` built from `steps` elementary row
/// additions drawn from a fixed-seed generator. Every entry stays a small integer, exact in `f64`.
fn congruent_to(d: [f64; 4], seed: u64, steps: usize) -> [[f64; 4]; 4] {
    let mut state = seed;
    let mut next = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) as usize
    };
    let mut s = [[0.0; 4]; 4];
    for (i, row) in s.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    for _ in 0..steps {
        let (i, j) = (next() % 4, next() % 4);
        if i == j {
            continue;
        }
        let k = [-2.0, -1.0, 1.0, 2.0][next() % 4];
        // Row i += k · row j keeps det S = 1.
        let source = s[j];
        for (cell, v) in s[i].iter_mut().zip(source) {
            *cell += k * v;
        }
    }
    let mut g = [[0.0; 4]; 4];
    for (a, row) in g.iter_mut().enumerate() {
        for (b, cell) in row.iter_mut().enumerate() {
            *cell = (0..4).map(|m| s[m][a] * d[m] * s[m][b]).sum();
        }
    }
    g
}

#[test]
fn test_the_signature_check_is_invariant_under_congruence() {
    // Sylvester's law of inertia (Horn & Johnson 2013, Theorem 4.5.8): Sᵀ D S has the inertia of
    // D for every invertible S. So every congruence of diag(−1, 1, 1, 1) is Lorentzian and is
    // accepted, and every congruence of a definite or (2, 2) diagonal is refused, whatever
    // couplings S introduces. The expected answer comes from D, not from the code under test.
    for seed in 0..200 {
        let mut t = event();
        let lorentzian = congruent_to([-1.0, 1.0, 1.0, 1.0], seed, 6);
        assert!(
            t.update_metric_tensor(lorentzian).is_ok(),
            "seed {seed}: {lorentzian:?}"
        );
        for d in [
            [1.0, 1.0, 1.0, 1.0],
            [-1.0, -1.0, 1.0, 1.0],
            [-1.0, -1.0, -1.0, 1.0],
        ] {
            let other = congruent_to(d, seed, 6);
            assert!(
                t.update_metric_tensor(other).is_err(),
                "seed {seed}, D = {d:?}: {other:?}"
            );
        }
    }
}

/// A metric tensor in `f64`, indexed t, x, y, z.
type Tensor = [[f64; 4]; 4];

/// `[[0, 4, 1, 0], [4, 0, 0, 1], [1, 0, s, 0], [0, 1, 0, s]]`. Its largest entry, 4, couples t
/// and x and exceeds twice every diagonal entry, so the elimination takes the t–x block as a 2×2
/// pivot, and y and z couple to that block. The tensor commutes with swapping t with x and y with
/// z, so it acts on the vectors `(a, a, b, b)` as `[[4, 1], [1, s]]` and on `(a, −a, b, −b)` as
/// `[[−4, 1], [1, s]]`. For `s = 1` the first is positive definite and the second has determinant
/// −5: signature (−, +, +, +). For `s = −1` the first has determinant −5 and the second is
/// negative definite: three negative eigenvalues.
fn coupled_null_block(s: f64) -> Tensor {
    [
        [0.0, 4.0, 1.0, 0.0],
        [4.0, 0.0, 0.0, 1.0],
        [1.0, 0.0, s, 0.0],
        [0.0, 1.0, 0.0, s],
    ]
}

/// Pairs of tensors whose inertia is known without the code under test, the first of each pair of
/// signature (−, +, +, +) and the second not: a diagonal pair, whose largest entry lies on the
/// diagonal; `coupled_null_block(±1)`; and 50 congruences `SᵀDS` of `diag(−1, 1, 1, 1)` and
/// `diag(−1, −1, 1, 1)`.
fn tensors_of_known_inertia() -> Vec<(Tensor, Tensor)> {
    let mut pairs = vec![
        (diag([-1.0, 1.0, 1.0, 1.0]), diag([-1.0, -1.0, 1.0, 1.0])),
        (coupled_null_block(1.0), coupled_null_block(-1.0)),
    ];
    pairs.extend((0..50).map(|seed| {
        (
            congruent_to([-1.0, 1.0, 1.0, 1.0], seed, 6),
            congruent_to([-1.0, -1.0, 1.0, 1.0], seed, 6),
        )
    }));
    pairs
}

/// Asserts that `scale · G` is accepted for the first tensor `G` of every pair of
/// [`tensors_of_known_inertia`] and refused for the second.
fn assert_verdicts_at_scale<R>(scale: R, lift: impl Fn(f64) -> R)
where
    R: RealField + FromPrimitive + core::fmt::Debug,
{
    let scaled = |g: Tensor| g.map(|row| row.map(|v| lift(v) * scale));
    for (lorentzian, other) in tensors_of_known_inertia() {
        let mut t = event::<R>();
        assert!(
            t.update_metric_tensor(scaled(lorentzian)).is_ok(),
            "{scale:?} · {lorentzian:?}"
        );
        assert!(
            t.update_metric_tensor(scaled(other)).is_err(),
            "{scale:?} · {other:?}"
        );
    }
}

#[test]
fn test_the_signature_check_does_not_depend_on_scale() {
    // Inertia is invariant under a positive scale factor, so the verdict on λ·G is the verdict on
    // G for λ = 10⁻³, 1 and 10³: a small tensor whose largest entry is on the diagonal must not be
    // routed through a 2×2 block, and a large one must not overflow the comparison.
    for lambda in [1e-3, 1.0, 1e3] {
        assert_verdicts_at_scale(lambda, |v| v);
    }
}

#[test]
fn test_the_signature_check_holds_where_products_of_entries_leave_the_range() {
    // Row J. At 2^±665 ≈ 10^±200 the product of two entries leaves the range of f64: 2^1330
    // overflows and 2^−1330 underflows. At 2^±100 ≈ 10^±30 the same holds for f32. A power of two
    // scales every entry exactly, so 2^e · G has the inertia of G.
    assert_verdicts_at_scale(2f64.powi(665), |v| v);
    assert_verdicts_at_scale(2f64.powi(-665), |v| v);
    assert_verdicts_at_scale(2f32.powi(100), |v| v as f32);
    assert_verdicts_at_scale(2f32.powi(-100), |v| v as f32);
}

#[test]
fn test_a_tensor_with_a_non_finite_entry_is_refused() {
    // Row I. A tensor with a NaN or an infinite entry is not a real symmetric matrix, so it has no
    // signature at all; it is refused, naming the non-finite entry, and the tensor is kept.
    let mut off = diag([-1.0, 1.0, 1.0, 1.0]);
    off[0][1] = f64::NAN;
    off[1][0] = f64::NAN;
    let mut on = diag([-1.0, 1.0, 1.0, 1.0]);
    on[2][2] = f64::NAN;
    let infinite = diag([f64::NEG_INFINITY, 1.0, 1.0, 1.0]);
    let cases = [
        (off, MetricTensorError::NonFinite(0, 1)),
        (on, MetricTensorError::NonFinite(2, 2)),
        (infinite, MetricTensorError::NonFinite(0, 0)),
    ];
    for (tensor, expected) in cases {
        let mut t = event();
        let before = t.metric_tensor();
        assert_eq!(t.update_metric_tensor(tensor), Err(expected));
        assert_eq!(t.metric_tensor(), before);
    }
}
