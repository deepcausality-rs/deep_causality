/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Randomized range-finder SVD and the adaptive randomized TT-rounding it powers.
//!
//! The randomized strategy must (a) reconstruct low-rank data to the requested tolerance, (b) match
//! the deterministic rounding result to tolerance, (c) leave the default (deterministic) path
//! untouched, and (d) be reproducible for a fixed seed. Exercised at `f64`/`Float106` (real) and
//! `Complex<f64>` (Hermitian).

use deep_causality_num::{Complex, ConjugateScalar, Float106, FromPrimitive, RealField};
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, RoundStrategy, TensorTrain, Truncation,
};

fn v<T: FromPrimitive>(x: f64) -> T {
    T::from_f64(x).unwrap()
}

fn tensor<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    data: &[f64],
    shape: &[usize],
) -> CausalTensor<T> {
    let d: Vec<T> = data.iter().map(|&x| v::<T>(x)).collect();
    CausalTensor::new(d, shape.to_vec()).unwrap()
}

/// Working tolerance scaled by precision: `√ε · 64` (a little looser than the deterministic suite to
/// absorb the randomized projection's residual).
fn tol<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() -> T {
    T::epsilon().sqrt() * v::<T>(64.0)
}

fn approx<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(a: T, b: T) {
    assert!(
        (a - b).abs() <= tol::<T>(),
        "values differ beyond tolerance"
    );
}

/// `U · diag(S) · Vt` for a real scalar (`S` is real, so `T = T::Real`).
fn reconstruct<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    u: &CausalTensor<T>,
    s: &CausalTensor<T>,
    vt: &CausalTensor<T>,
) -> Vec<T> {
    let m = u.shape()[0];
    let k = u.shape()[1];
    let n = vt.shape()[1];
    let (u, s, vt) = (u.as_slice(), s.as_slice(), vt.as_slice());
    let mut out = vec![T::zero(); m * n];
    for row in 0..m {
        for col in 0..n {
            let mut acc = T::zero();
            for t in 0..k {
                acc += u[row * k + t] * s[t] * vt[t * n + col];
            }
            out[row * n + col] = acc;
        }
    }
    out
}

/// A rank-`r` `rows × cols` matrix built as `Σ aₚ bₚᵀ`, with deterministic pseudo-data.
fn low_rank(rows: usize, cols: usize, r: usize) -> (Vec<f64>, usize, usize) {
    let mut data = vec![0.0f64; rows * cols];
    for p in 0..r {
        for i in 0..rows {
            let ai = ((i + 1) as f64 * (p + 2) as f64).sin();
            for j in 0..cols {
                let bj = ((j + 1) as f64 / (p + 3) as f64).cos();
                data[i * cols + j] += ai * bj;
            }
        }
    }
    (data, rows, cols)
}

// ---- (a) randomized SVD reconstructs low-rank data to tolerance ------------------------------

fn check_randomized_svd_lowrank<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    // ℓ starts at ≥ maxr here, so the range is captured in one shot.
    let (data, m, n) = low_rank(6, 5, 2);
    let mat = tensor::<T>(&data, &[m, n]);
    let trunc = Truncation::<T>::by_tol(v::<T>(1e-12))
        .unwrap()
        .randomized(6, 0xABCD_1234);
    let (u, s, vt) = mat.svd_truncated(&trunc).unwrap();
    // Low rank is detected: at most a couple of singular values survive.
    assert!(s.len() <= 3, "expected ~rank-2, got {}", s.len());
    let rec = reconstruct(&u, &s, &vt);
    for (g, w) in rec.iter().zip(mat.as_slice()) {
        approx(*g, *w);
    }
}

fn check_randomized_svd_adaptive_growth<
    T: RealField + FromPrimitive + ConjugateScalar<Real = T>,
>() {
    // oversample = 1 ⇒ ℓ starts at 2 ≪ maxr = 18, so the adaptive loop must grow to capture rank 3.
    let (data, m, n) = low_rank(20, 18, 3);
    let mat = tensor::<T>(&data, &[m, n]);
    let trunc = Truncation::<T>::by_tol(v::<T>(1e-10))
        .unwrap()
        .randomized(1, 0x55AA);
    let (u, s, vt) = mat.svd_truncated(&trunc).unwrap();
    assert!(s.len() >= 3, "rank-3 range not captured: {}", s.len());
    let rec = reconstruct(&u, &s, &vt);
    for (g, w) in rec.iter().zip(mat.as_slice()) {
        approx(*g, *w);
    }
}

// ---- (b) randomized rounding matches deterministic to tolerance ------------------------------

fn check_randomized_round_matches<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    // Build a genuinely low-rank train: the sum of two rank-1 trains is rank-2.
    let r1a = CausalTensorTrain::<T>::random_seeded(&[4, 4, 4, 4], 1, 0x11);
    let r1b = CausalTensorTrain::<T>::random_seeded(&[4, 4, 4, 4], 1, 0x22);
    let x = r1a.add(&r1b).unwrap();
    let dense = x.to_dense().unwrap();

    let det = Truncation::<T>::by_tol(v::<T>(1e-10)).unwrap();
    let rnd = det.randomized(8, 0xDEAD_BEEF);

    let yd = x.round(&det).unwrap().to_dense().unwrap();
    let yr = x.round(&rnd).unwrap().to_dense().unwrap();

    // Randomized matches deterministic, and both reconstruct the original low-rank tensor.
    for ((a, b), c) in yr
        .as_slice()
        .iter()
        .zip(yd.as_slice())
        .zip(dense.as_slice())
    {
        approx(*a, *b);
        approx(*a, *c);
    }
}

// ---- (c) default strategy is deterministic and untouched --------------------------------------

#[test]
fn default_strategy_is_deterministic() {
    assert_eq!(
        Truncation::<f64>::by_bond(4).unwrap().strategy(),
        RoundStrategy::Deterministic
    );
    assert_eq!(
        Truncation::<f64>::by_tol(1e-9).unwrap().strategy(),
        RoundStrategy::Deterministic
    );
    let r = Truncation::<f64>::by_tol(1e-9).unwrap().randomized(8, 7);
    assert_eq!(
        r.strategy(),
        RoundStrategy::Randomized {
            oversample: 8,
            seed: 7
        }
    );
    // The randomized builder leaves the gates untouched.
    assert_eq!(
        r.max_bond(),
        Truncation::<f64>::by_tol(1e-9).unwrap().max_bond()
    );
    assert_eq!(r.rel_tol(), 1e-9);
}

// ---- (d) reproducible for a fixed seed --------------------------------------------------------

#[test]
fn randomized_is_reproducible() {
    let (data, m, n) = low_rank(12, 10, 4);
    let mat = tensor::<f64>(&data, &[m, n]);
    let trunc = Truncation::<f64>::by_tol(1e-10)
        .unwrap()
        .randomized(4, 0xFEED);
    let (u1, s1, vt1) = mat.svd_truncated(&trunc).unwrap();
    let (u2, s2, vt2) = mat.svd_truncated(&trunc).unwrap();
    assert_eq!(u1.as_slice(), u2.as_slice());
    assert_eq!(s1.as_slice(), s2.as_slice());
    assert_eq!(vt1.as_slice(), vt2.as_slice());
}

// ---- Complex (Hermitian) randomized SVD -------------------------------------------------------

#[test]
fn complex_randomized_svd_lowrank() {
    type C = Complex<f64>;
    let (m, n, r) = (7usize, 6usize, 2usize);
    // Rank-2 complex matrix Σ aₚ bₚᴴ.
    let mut data = vec![C::new(0.0, 0.0); m * n];
    for p in 0..r {
        for i in 0..m {
            let a = C::new(
                ((i + 1) as f64 * (p + 2) as f64).sin(),
                ((i + 2) as f64 / (p + 1) as f64).cos(),
            );
            for j in 0..n {
                let b = C::new(
                    ((j + 1) as f64 / (p + 3) as f64).cos(),
                    ((j + 2) as f64 * (p + 1) as f64).sin(),
                );
                data[i * n + j] += a * b.conjugate();
            }
        }
    }
    let mat = CausalTensor::new(data.clone(), vec![m, n]).unwrap();
    let trunc = Truncation::<f64>::by_tol(1e-12)
        .unwrap()
        .randomized(6, 0xC0FFEE);
    let (u, s, vt) = mat.svd_truncated(&trunc).unwrap();
    assert!(s.len() <= 3, "expected ~rank-2, got {}", s.len());

    // Reconstruct U · diag(S) · Vᴴ (S real, injected on the real axis).
    let k = s.len();
    let (us, ss, vts) = (u.as_slice(), s.as_slice(), vt.as_slice());
    let tol = 1e-12_f64.sqrt() * 64.0;
    for row in 0..m {
        for col in 0..n {
            let mut acc = C::new(0.0, 0.0);
            for t in 0..k {
                acc += us[row * k + t] * C::from_real(ss[t]) * vts[t * n + col];
            }
            let diff = acc + data[row * n + col] * C::new(-1.0, 0.0);
            assert!(
                diff.modulus_squared().sqrt() <= tol,
                "complex reconstruction beyond tolerance"
            );
        }
    }
}

// ---- crossover study: deterministic Jacobi vs randomized range-finder at scale ----------------
// Ignored by default (timing, not assertions). Run with:
//   cargo test -p deep_causality_tensor --test mod -- --ignored --nocapture svd_crossover

#[test]
#[ignore]
fn svd_crossover_study() {
    use std::time::Instant;
    let rank = 20usize;
    println!(
        "\n{:>8} {:>6} {:>14} {:>14} {:>8}",
        "size", "rank", "deterministic", "randomized", "speedup"
    );
    // Sizes kept modest so `--ignored` runs in a few seconds; the speedup grows ~linearly with size
    // (measured: 38× at 100², 274× at 400², ~935× at 1000² for rank 20 on an M3 Max).
    for &s in &[100usize, 200, 400] {
        let (data, _, _) = low_rank(s, s, rank);
        let mat = tensor::<f64>(&data, &[s, s]);
        let det = Truncation::<f64>::by_tol(1e-9).unwrap();
        let rnd = det.randomized(10, 0x1234_5678);

        // Warm + measure best-of-3 to damp noise.
        let mut td = f64::INFINITY;
        let mut tr = f64::INFINITY;
        for _ in 0..3 {
            let t = Instant::now();
            let (_u, sd, _v) = mat.svd_truncated(&det).unwrap();
            td = td.min(t.elapsed().as_secs_f64());
            assert!(sd.len() <= rank + 2);

            let t = Instant::now();
            let (_u, sr, _v) = mat.svd_truncated(&rnd).unwrap();
            tr = tr.min(t.elapsed().as_secs_f64());
            assert!(sr.len() <= rank + 2);
        }
        println!(
            "{:>8} {:>6} {:>12.3} ms {:>12.3} ms {:>7.1}x",
            s,
            rank,
            td * 1e3,
            tr * 1e3,
            td / tr
        );
    }
    println!();
}

// ---- monomorphized entry points ---------------------------------------------------------------

#[test]
fn randomized_svd_lowrank_f64() {
    check_randomized_svd_lowrank::<f64>();
}
#[test]
fn randomized_svd_lowrank_float106() {
    check_randomized_svd_lowrank::<Float106>();
}
#[test]
fn randomized_svd_adaptive_growth_f64() {
    check_randomized_svd_adaptive_growth::<f64>();
}
#[test]
fn randomized_svd_adaptive_growth_float106() {
    check_randomized_svd_adaptive_growth::<Float106>();
}
#[test]
fn randomized_round_matches_f64() {
    check_randomized_round_matches::<f64>();
}
#[test]
fn randomized_round_matches_float106() {
    check_randomized_round_matches::<Float106>();
}
