/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Self-contained deterministic Gaussian sampling for the randomized tensor-network kernels.
//!
//! Randomized range-finder SVD (and the randomized TT-rounding built on it) needs standard-normal
//! sketch matrices. To stay dependency-free and reproducible, samples are drawn from the same
//! `splitmix64` stream used elsewhere in the crate and mapped to normals by the Box–Muller
//! transform. The `f64` arithmetic here is the bit-mixing/transform itself — the only scalar that
//! ever enters the generic tensor algebra is the injected `T`, so the precision invariant holds.

use deep_causality_num::{ConjugateScalar, FromPrimitive};

/// One `splitmix64` step, returning a uniform in `[0, 1)` from the high 53 bits.
fn next_unit(state: &mut u64) -> f64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

/// A row-major `len`-element vector of standard-normal samples injected into the scalar `T`.
///
/// For a real `T` the values are real normals; for a complex `T` they are injected on the real axis
/// (`im = 0`) via `from_f64`, which is exactly `from_real` of a real normal — a real Gaussian sketch
/// is sufficient to capture a column space regardless of conjugation.
pub(crate) fn gaussian_vec<T: ConjugateScalar>(len: usize, seed: u64) -> Vec<T> {
    let mut state = seed;
    let mut out = Vec::with_capacity(len);
    while out.len() < len {
        // Box–Muller: two independent uniforms → two independent normals.
        let mut u1 = next_unit(&mut state);
        if u1 <= f64::MIN_POSITIVE {
            u1 = f64::MIN_POSITIVE; // guard against ln(0)
        }
        let u2 = next_unit(&mut state);
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = core::f64::consts::TAU * u2;
        let z0 = r * theta.cos();
        out.push(<T as FromPrimitive>::from_f64(z0).unwrap());
        if out.len() < len {
            let z1 = r * theta.sin();
            out.push(<T as FromPrimitive>::from_f64(z1).unwrap());
        }
    }
    out
}
