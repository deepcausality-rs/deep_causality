/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A scalar the crate never names still draws from a range.

use deep_causality_num::{BFloat16, Float106, ToPrimitive};
use deep_causality_rand::{Distribution, Rng, Uniform, Xoshiro256};

/// One generic function, every float the tower carries. Returns (all in `[low, high]`, how many
/// landed exactly on `high`).
fn in_range<T>(low: f64, high: f64) -> (bool, usize)
where
    T: deep_causality_algebra::RealField + deep_causality_num::FromPrimitive + ToPrimitive,
{
    let mut g = Xoshiro256::from_seed(0x5EED_2026);
    let lo = T::from_f64(low).unwrap();
    let hi = T::from_f64(high).unwrap();
    let d = Uniform::new(lo, hi).unwrap();
    let mut on_high = 0;
    let ok = (0..2000).all(|_| {
        let v: T = d.sample(&mut g);
        let v = v.to_f64().unwrap();
        if v == high {
            on_high += 1;
        }
        v >= low && v < high
    });
    (ok, on_high)
}

#[test]
fn every_float_in_the_tower_draws_from_a_range() {
    // `Uniform::new`'s contract is `[low, high)`, strictly, at every scalar. It did not hold at
    // `BFloat16`: the affine map `u * scale + low` rounded up onto `high` in 14 draws of 2 000,
    // while `f32` and wider never reached it. The sampler now rejects a result that lands on the
    // bound, so the count below is zero for every scalar and stays zero for one added later.
    for (name, (ok, on_high)) in [
        ("f32", in_range::<f32>(10.0, 20.0)),
        ("f64", in_range::<f64>(10.0, 20.0)),
        ("Float106", in_range::<Float106>(10.0, 20.0)),
        // Named nowhere in deep_causality_rand. It draws because it satisfies the algebra.
        ("BFloat16", in_range::<BFloat16>(10.0, 20.0)),
    ] {
        assert!(ok, "{name}: a draw left [low, high)");
        assert_eq!(on_high, 0, "{name}: {on_high} draws reached the exclusive bound");
    }
}

#[test]
fn every_unsigned_in_the_tower_draws_from_a_range() {
    let mut g = Xoshiro256::from_seed(7);
    // u8, u16 and u128 had no sampler at all before; none is named in the crate now.
    assert!((0..200).all(|_| { let v: u8 = g.random_range(10u8..20); (10..20).contains(&v) }));
    assert!((0..200).all(|_| { let v: u16 = g.random_range(10u16..20); (10..20).contains(&v) }));
    assert!((0..200).all(|_| { let v: u32 = g.random_range(10u32..20); (10..20).contains(&v) }));
    assert!((0..200).all(|_| { let v: u64 = g.random_range(10u64..20); (10..20).contains(&v) }));
    assert!((0..200).all(|_| { let v: u128 = g.random_range(10u128..20); (10..20).contains(&v) }));
    assert!((0..200).all(|_| { let v: usize = g.random_range(10usize..20); (10..20).contains(&v) }));
}

#[test]
fn the_usize_defect_is_gone() {
    // The old hand-written usize sampler drew from `next_u32`, so a range wider than 2^32
    // returned only its bottom 2^32. Measured then: 200 000 draws never exceeded 4 294 942 982.
    let mut g = Xoshiro256::from_seed(7);
    let hi = 1usize << 40;
    let max = (0..50_000).map(|_| g.random_range(0usize..hi)).max().unwrap();
    assert!(
        max > (1usize << 32),
        "the widest of 50 000 draws over 2^40 was {max}, still inside 2^32"
    );
}
