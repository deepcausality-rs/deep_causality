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

use deep_causality_algebra::{ConjugateScalar, RealField};
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_num_complex::Complex;
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

/// Exercises the randomize-then-orthogonalize `round` at a meaningful interior bond: a sum of `k`
/// copies of a bond-3 train (input bond 3k, output rank 3) must round to the same tensor as the
/// deterministic round, to tolerance.
fn check_randomized_round_highbond<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let x0 = CausalTensorTrain::<T>::random_seeded(&[6, 6, 6, 6], 3, 0x33);
    let mut x = x0.clone();
    for _ in 1..6 {
        x = x.add(&x0).unwrap();
    }
    // Interior bond is now 18; numerical rank stays 3 (the sum of copies is a scalar multiple).
    assert!(x.cores()[1].shape()[2] >= 12);

    let rnd = Truncation::<T>::by_tol(v::<T>(1e-9))
        .unwrap()
        .randomized(6, 0x0ACE);
    let yr = x.round(&rnd).unwrap();

    // The randomize-then-orthogonalize round compresses the bond-18 train back to its true rank…
    assert!(
        yr.cores()[1].shape()[2] <= 4,
        "expected compression to ~rank 3, got bond {}",
        yr.cores()[1].shape()[2]
    );
    // …and reproduces the original tensor to the rounding tolerance. (Compared against the original,
    // not the deterministic round — whose rank cutoff for this exactly-rank-3 data is borderline at
    // Float106, independent of the randomized path under test.)
    let round_tol = v::<T>(1e-6);
    let (orig, got) = (x.to_dense().unwrap(), yr.to_dense().unwrap());
    for (a, b) in got.as_slice().iter().zip(orig.as_slice()) {
        assert!(
            (*a - *b).abs() <= round_tol,
            "randomized round does not reproduce the original tensor"
        );
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
#[cfg_attr(miri, ignore)] // Miri's non-deterministic float emulation makes two identical SVD runs differ bit-for-bit, breaking the reproducibility assert; test is correct under normal CI.
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

/// TT-level analogue of `svd_crossover_study`: round a **sum of `k` copies** of a low-rank train —
/// the literature's "rounding a sum of TT-tensors" regime, where the input bond is large (`k·r0`) but
/// the numerical rank stays `r0`. This is where randomized rounding is expected to pay off, unlike the
/// tiny-bond `tt_round_highbond` criterion bench. Ignored by default; run with:
///   cargo test -p deep_causality_tensor --test mod --release -- --ignored --nocapture tt_round_compressing
#[test]
#[ignore]
fn tt_round_compressing_study() {
    use std::time::Instant;
    let n = 16usize;
    let r0 = 6usize;
    println!(
        "\n{:>4} {:>10} {:>14} {:>14} {:>8}",
        "k", "in-bond", "deterministic", "randomized", "speedup"
    );
    for &k in &[4usize, 8, 16, 24] {
        let x0 = CausalTensorTrain::<f64>::random_seeded(&[n, n, n, n], r0, 0x77);
        let mut x = x0.clone();
        for _ in 1..k {
            x = x.add(&x0).unwrap();
        }
        let in_bond = x.cores()[1].shape()[2]; // interior bond after summation
        let det = Truncation::<f64>::by_tol(1e-8).unwrap();
        let rnd = det.randomized(8, 0x9119);

        let mut td = f64::INFINITY;
        let mut tr = f64::INFINITY;
        for _ in 0..3 {
            let t = Instant::now();
            let yd = x.round(&det).unwrap();
            td = td.min(t.elapsed().as_secs_f64());
            let t = Instant::now();
            let yr = x.round(&rnd).unwrap();
            tr = tr.min(t.elapsed().as_secs_f64());
            // Both must recover the same tensor to tolerance (spot-check one entry).
            let idx = [1usize, 2, 3, 0];
            assert!((yd.eval(&idx).unwrap() - yr.eval(&idx).unwrap()).abs() < 1e-6);
        }
        println!(
            "{:>4} {:>10} {:>12.3} ms {:>12.3} ms {:>7.1}x",
            k,
            in_bond,
            td * 1e3,
            tr * 1e3,
            td / tr
        );
    }
    println!();
}

/// Component-level breakdown of why TT `round()` shows only ~1.1× from randomized SVD. Times the
/// deterministic left-canonicalization, the single SVD on the *actual* largest round unfolding
/// (det vs rand), and the full round (det vs rand). Ignored; run with:
///   cargo test -p deep_causality_tensor --test mod --release -- --ignored --nocapture round_breakdown
#[test]
#[ignore]
fn round_breakdown_study() {
    use std::time::Instant;
    let n = 16usize;
    let r0 = 6usize;
    let k = 24usize;
    let x0 = CausalTensorTrain::<f64>::random_seeded(&[n, n, n, n], r0, 0x77);
    let mut x = x0.clone();
    for _ in 1..k {
        x = x.add(&x0).unwrap();
    }
    let bonds: Vec<usize> = x.cores().iter().map(|c| c.shape()[2]).collect();
    println!("\nbonds (right) per core: {:?}", bonds);

    let det = Truncation::<f64>::by_tol(1e-8).unwrap();
    let rnd = det.randomized(8, 0x9119);

    let best = |mut f: Box<dyn FnMut() -> usize>| -> f64 {
        let mut t = f64::INFINITY;
        for _ in 0..3 {
            let s = Instant::now();
            let _ = f();
            t = t.min(s.elapsed().as_secs_f64());
        }
        t
    };

    // 1. Deterministic left-canonicalization (strategy-independent part of round).
    let xc = x.clone();
    let t_lcanon = best(Box::new(move || xc.left_canonicalize().unwrap().order()));
    println!("left_canonicalize:        {:>10.3} ms", t_lcanon * 1e3);

    // 2. Single SVD on the actual largest round unfolding, extracted after canonicalization.
    let xl = x.left_canonicalize().unwrap();
    // Pick the core with the largest left bond (the compressing SVD target).
    let (kc, _) = xl
        .cores()
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| c.shape()[0])
        .unwrap();
    let c = &xl.cores()[kc];
    let (rl, nn, rr) = (c.shape()[0], c.shape()[1], c.shape()[2]);
    // Row-major [rl, nn, rr] reshaped to [rl, nn*rr] is the same buffer with a 2D shape.
    let unfold = CausalTensor::new(c.as_slice().to_vec(), vec![rl, nn * rr]).unwrap();
    println!("largest unfolding: core {kc} shape [{rl}, {}]", nn * rr);
    let u1 = unfold.clone();
    let det1 = det;
    let t_svd_det = best(Box::new(move || u1.svd_truncated(&det1).unwrap().1.len()));
    let u2 = unfold.clone();
    let rnd1 = rnd;
    let t_svd_rnd = best(Box::new(move || u2.svd_truncated(&rnd1).unwrap().1.len()));
    let kept_det = unfold.svd_truncated(&det).unwrap().1.len();
    let kept_rnd = unfold.svd_truncated(&rnd).unwrap().1.len();
    println!(
        "single SVD det:           {:>10.3} ms  (kept rank {kept_det})",
        t_svd_det * 1e3
    );
    println!(
        "single SVD rnd:           {:>10.3} ms  (kept rank {kept_rnd})",
        t_svd_rnd * 1e3
    );

    // 3. Full round, det vs rand.
    let xd = x.clone();
    let dd = det;
    let t_round_det = best(Box::new(move || xd.round(&dd).unwrap().order()));
    let xr = x.clone();
    let rr2 = rnd;
    let t_round_rnd = best(Box::new(move || xr.round(&rr2).unwrap().order()));
    println!("round det:                {:>10.3} ms", t_round_det * 1e3);
    println!("round rnd:                {:>10.3} ms", t_round_rnd * 1e3);
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
#[test]
fn randomized_round_highbond_f64() {
    check_randomized_round_highbond::<f64>();
}
#[test]
fn randomized_round_highbond_float106() {
    check_randomized_round_highbond::<Float106>();
}

/// Regression: the Jacobi SVD and Householder QR must stay finite on rank-deficient matrices, even at
/// double-double precision where a near-zero column previously overflowed `ζ²`/`β=2/(vᴴv)` to `±∞`
/// and produced NaN singular values (which then defeated the rank gate, blocking compression).
fn check_rank_deficient_svd_finite<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    // [6,3] of exact rank 2 (col3 = col1 + col2), with redundant rows (two stacked copies).
    let blk = [1.0, 2.0, 3.0, 0.5, 1.0, 1.5, 4.0, 1.0, 5.0];
    let data: Vec<f64> = blk.iter().chain(blk.iter()).copied().collect();
    let mat = tensor::<T>(&data, &[6, 3]);
    let (u, s, vt) = mat
        .svd_truncated(&Truncation::<T>::by_bond(99).unwrap())
        .unwrap();
    for x in s.as_slice() {
        assert!(x.is_finite(), "singular value is non-finite");
    }
    // Rank 2: the third singular value is negligible relative to the first.
    let sv = s.as_slice();
    assert!(sv.len() == 3);
    assert!(sv[2] <= sv[0] * v::<T>(1e-12), "expected rank-2 spectrum");
    // QR of the same matrix is finite and reconstructs A.
    let (q, r) = mat.qr().unwrap();
    for x in q.as_slice().iter().chain(r.as_slice()) {
        assert!(x.is_finite(), "QR factor is non-finite");
    }
    // U·diag(S)·Vt reproduces A.
    let rec = reconstruct(
        &u,
        &CausalTensor::new(sv.to_vec(), vec![sv.len()]).unwrap(),
        &vt,
    );
    for (g, w) in rec.iter().zip(mat.as_slice()) {
        approx(*g, *w);
    }
}

/// Regression (end-to-end): the *deterministic* round must compress an exactly-rank-3 train (a sum of
/// six copies of a bond-3 train, input bond 18) back to rank 3 — at both precisions. Before the
/// near-zero-column NaN fix, `Float106` left the bond uncompressed because NaN singular values slipped
/// past the tolerance gate.
fn check_deterministic_round_rank_deficient<
    T: RealField + FromPrimitive + ConjugateScalar<Real = T>,
>() {
    let x0 = CausalTensorTrain::<T>::random_seeded(&[6, 6, 6, 6], 3, 0x33);
    let mut x = x0.clone();
    for _ in 1..6 {
        x = x.add(&x0).unwrap();
    }
    assert!(x.cores()[1].shape()[2] >= 12);
    let yd = x
        .round(&Truncation::<T>::by_tol(v::<T>(1e-9)).unwrap())
        .unwrap();
    for c in yd.cores() {
        assert!(
            c.shape()[2] <= 4,
            "deterministic round failed to compress rank-3 train: bond {}",
            c.shape()[2]
        );
    }
    let (orig, got) = (x.to_dense().unwrap(), yd.to_dense().unwrap());
    for (a, b) in got.as_slice().iter().zip(orig.as_slice()) {
        assert!((*a - *b).abs() <= v::<T>(1e-6));
    }
}

#[test]
fn rank_deficient_svd_finite_f64() {
    check_rank_deficient_svd_finite::<f64>();
}
#[test]
fn rank_deficient_svd_finite_float106() {
    check_rank_deficient_svd_finite::<Float106>();
}
#[test]
fn deterministic_round_rank_deficient_f64() {
    check_deterministic_round_rank_deficient::<f64>();
}
#[test]
fn deterministic_round_rank_deficient_float106() {
    check_deterministic_round_rank_deficient::<Float106>();
}
