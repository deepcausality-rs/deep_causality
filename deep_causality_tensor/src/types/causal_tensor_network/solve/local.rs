/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TensorTrain;
use crate::traits::tensor_train_operator::TensorTrainOperator;
use crate::types::causal_tensor_network::canonical_form::CanonicalForm;
use crate::types::causal_tensor_network::causal_tensor_train::CausalTensorTrain;
use crate::types::causal_tensor_network::causal_tensor_train_operator::CausalTensorTrainOperator;
use crate::types::causal_tensor_network::solve_config::SolveConfig;
use crate::types::causal_tensor_network::truncation::Truncation;
use crate::{CausalTensor, CausalTensorError};
use deep_causality_num::Scalar;

/// Fits a tensor train of bond dimension up to `max_rank` to a set of `(index, value)` samples by
/// alternating least squares (TT completion / regression).
///
/// At each site the local objective is least-squares in that core and **block-diagonal over the
/// physical index** (a sample only touches its own physical slice), so each slice is a small
/// `(r_k·r_{k+1})` ridge-regularized normal-equation solve.
///
/// # Errors
/// - [`CausalTensorError::EmptyTensor`] if `shape` is empty / has a zero dimension or there are no
///   samples.
/// - [`CausalTensorError::SweepDidNotConverge`] if the RMS residual stays above `tol` after
///   `max_sweeps`.
pub fn fit<T: Scalar>(
    shape: &[usize],
    max_rank: usize,
    samples: &[(Vec<usize>, T)],
    config: &SolveConfig<T>,
) -> Result<CausalTensorTrain<T>, CausalTensorError> {
    let d = shape.len();
    if d == 0 || shape.contains(&0) || samples.is_empty() {
        return Err(CausalTensorError::EmptyTensor);
    }
    let ranks = bond_ranks(shape, max_rank);
    let mut cores = init_random(shape, &ranks, 0xF17_F17);

    let mut residual = rms_residual(&cores, samples)?;
    for _sweep in 0..config.max_sweeps() {
        // Forward then backward over the sites (Gauss–Seidel coordinate descent).
        for k in forward_then_back(d) {
            fit_site(&mut cores, shape, k, samples, config.ridge());
        }
        residual = rms_residual(&cores, samples)?;
        if residual <= config.tol() {
            return CausalTensorTrain::from_cores(cores);
        }
    }
    let _ = residual;
    Err(CausalTensorError::SweepDidNotConverge)
}

/// Solves `A x = b` for `x` (bond dimension up to `max_rank`) in tensor-train form by one-site ALS
/// on the normal equations `(AᵀA) x = Aᵀb`.
///
/// `G = Aᵀ∘A` (an MPO on the input space) and `c = Aᵀ·b` (a state) are formed once; each site then
/// solves the local effective system `G_local · vec(core_k) = c_local` built from the running
/// environments.
///
/// # Errors
/// - [`CausalTensorError::ShapeMismatch`] if `b`'s physical dimensions differ from `A`'s output.
/// - [`CausalTensorError::SweepDidNotConverge`] if the residual stays above `tol`.
pub fn linear<T: Scalar>(
    a: &CausalTensorTrainOperator<T>,
    b: &CausalTensorTrain<T>,
    max_rank: usize,
    config: &SolveConfig<T>,
) -> Result<CausalTensorTrain<T>, CausalTensorError> {
    if b.phys_dims() != a.out_dims() {
        return Err(CausalTensorError::ShapeMismatch);
    }
    let exact = Truncation::by_bond(usize::MAX)?;
    // Normal-equation operator G = Aᵀ ∘ A (maps the input space to itself) and rhs c = Aᵀ b.
    let at = a.transpose();
    let g = at.compose(a, &exact)?;
    let c = at.apply(b, &exact)?;

    let shape = a.in_dims().to_vec();
    let d = shape.len();
    let ranks = bond_ranks(&shape, max_rank);
    let mut x = init_random(&shape, &ranks, 0x5017_5017);

    let mut residual = linear_residual(&g, &c, &x)?;
    for _sweep in 0..config.max_sweeps() {
        for k in forward_then_back(d) {
            linear_site(&mut x, &g, &c, k, config.ridge())?;
        }
        residual = linear_residual(&g, &c, &x)?;
        if residual <= config.tol() {
            return CausalTensorTrain::from_cores(x);
        }
    }
    let _ = residual;
    Err(CausalTensorError::SweepDidNotConverge)
}

// ============================================================================
// fit internals
// ============================================================================

fn fit_site<T: Scalar>(
    cores: &mut [CausalTensor<T>],
    shape: &[usize],
    k: usize,
    samples: &[(Vec<usize>, T)],
    ridge: T,
) {
    let rk = cores[k].shape()[0];
    let nk = shape[k];
    let rkp = cores[k].shape()[2];
    let p = rk * rkp;

    // One block per physical slice i: M_i (p×p), y_i (p).
    let mut blocks_m = vec![vec![T::zero(); p * p]; nk];
    let mut blocks_y = vec![vec![T::zero(); p]; nk];

    for (idx, val) in samples {
        let i0 = idx[k];
        let ls = left_product(cores, idx, k); // length rk
        let rs = right_product(cores, idx, k); // length rkp
        // feature φ[(a,b)] = ls[a]·rs[b]
        let m = &mut blocks_m[i0];
        let y = &mut blocks_y[i0];
        for a in 0..rk {
            for b in 0..rkp {
                let phi = ls[a] * rs[b];
                y[a * rkp + b] += *val * phi;
                for a2 in 0..rk {
                    for b2 in 0..rkp {
                        let phi2 = ls[a2] * rs[b2];
                        m[(a * rkp + b) * p + (a2 * rkp + b2)] += phi * phi2;
                    }
                }
            }
        }
    }

    // Solve each slice and write the new core.
    let mut data = vec![T::zero(); rk * nk * rkp];
    for i in 0..nk {
        let mut m = blocks_m[i].clone();
        for j in 0..p {
            m[j * p + j] += ridge;
        }
        let mut y = blocks_y[i].clone();
        solve_dense(&mut m, &mut y, p);
        for a in 0..rk {
            for b in 0..rkp {
                data[a * (nk * rkp) + i * rkp + b] = y[a * rkp + b];
            }
        }
    }
    cores[k] = CausalTensor::new(data, vec![rk, nk, rkp]).unwrap();
}

/// Left partial product `∏_{j<k} core_j[:, idx_j, :]`, a length-`r_k` row vector.
fn left_product<T: Scalar>(cores: &[CausalTensor<T>], idx: &[usize], k: usize) -> Vec<T> {
    let mut v = vec![T::one()]; // r_0 == 1
    for (j, core) in cores.iter().enumerate().take(k) {
        let (rl, n, rr) = (core.shape()[0], core.shape()[1], core.shape()[2]);
        let cd = core.as_slice();
        let i = idx[j];
        let mut nv = vec![T::zero(); rr];
        for (a, &va) in v.iter().enumerate().take(rl) {
            if va == T::zero() {
                continue;
            }
            let base = a * (n * rr) + i * rr;
            for (b, nvb) in nv.iter_mut().enumerate() {
                *nvb += va * cd[base + b];
            }
        }
        v = nv;
    }
    v
}

/// Right partial product `∏_{j>k} core_j[:, idx_j, :]`, a length-`r_{k+1}` column vector.
fn right_product<T: Scalar>(cores: &[CausalTensor<T>], idx: &[usize], k: usize) -> Vec<T> {
    let d = cores.len();
    let mut v = vec![T::one()]; // r_d == 1
    for j in (k + 1..d).rev() {
        let core = &cores[j];
        let (rl, n, rr) = (core.shape()[0], core.shape()[1], core.shape()[2]);
        let cd = core.as_slice();
        let i = idx[j];
        let mut nv = vec![T::zero(); rl];
        for (a, nva) in nv.iter_mut().enumerate() {
            let base = a * (n * rr) + i * rr;
            let mut acc = T::zero();
            for (b, &vb) in v.iter().enumerate().take(rr) {
                acc += cd[base + b] * vb;
            }
            *nva = acc;
        }
        v = nv;
    }
    v
}

fn rms_residual<T: Scalar>(
    cores: &[CausalTensor<T>],
    samples: &[(Vec<usize>, T)],
) -> Result<T, CausalTensorError> {
    let train = CausalTensorTrain::from_cores_raw(cores.to_vec(), CanonicalForm::None);
    let mut sum = T::zero();
    for (idx, val) in samples {
        let got = train.eval(idx)?;
        let e = got - *val;
        sum += e * e;
    }
    let n = <T as deep_causality_num::FromPrimitive>::from_usize(samples.len()).ok_or(
        CausalTensorError::InvalidParameter("sample count".to_string()),
    )?;
    Ok((sum / n).sqrt())
}

// ============================================================================
// linear internals
// ============================================================================

/// One ALS site update for `G x = c`: build the local system from the environments and solve.
fn linear_site<T: Scalar>(
    x: &mut [CausalTensor<T>],
    g: &CausalTensorTrainOperator<T>,
    c: &CausalTensorTrain<T>,
    k: usize,
    ridge: T,
) -> Result<(), CausalTensorError> {
    let rk = x[k].shape()[0];
    let nk = x[k].shape()[1];
    let rkp = x[k].shape()[2];
    let n = rk * nk * rkp;

    // Left/right environments of <x|G|x> (3-index) and <x|c> (2-index).
    let (gl, grl) = env_g_left(x, g, k); // gl: [rk, gk, rk]
    let (gr, grr) = env_g_right(x, g, k); // gr: [rkp, gkp, rkp]
    let cl = env_c_left(x, c, k); // [rk, ck]
    let cr = env_c_right(x, c, k); // [rkp, ckp]
    let gcore = &g.cores()[k]; // [gk, nk, nk, gkp]
    let ccore = &c.cores()[k]; // [ck, nk, ckp]
    let gk = grl;
    let gkp = grr;
    let ck = ccore.shape()[0];
    let ckp = ccore.shape()[2];
    let gd = gcore.as_slice();
    let cd = ccore.as_slice();

    // Local operator H[(a,i,b),(a',i',b')] = Σ gl[a,gl_,a']·G[gl_,i,i',gr_]·gr[b,gr_,b'].
    let mut hmat = vec![T::zero(); n * n];
    for a in 0..rk {
        for ap in 0..rk {
            for gl_ in 0..gk {
                let lval = gl[(a * gk + gl_) * rk + ap];
                if lval == T::zero() {
                    continue;
                }
                for b in 0..rkp {
                    for bp in 0..rkp {
                        for gr_ in 0..gkp {
                            let rval = gr[(b * gkp + gr_) * rkp + bp];
                            if rval == T::zero() {
                                continue;
                            }
                            let lr = lval * rval;
                            for i in 0..nk {
                                for ip in 0..nk {
                                    let gval = gd[(((gl_ * nk) + i) * nk + ip) * gkp + gr_];
                                    if gval == T::zero() {
                                        continue;
                                    }
                                    let row = (a * nk + i) * rkp + b;
                                    let col = (ap * nk + ip) * rkp + bp;
                                    hmat[row * n + col] += lr * gval;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Local rhs y[(a,i,b)] = Σ cl[a,cl_]·C[cl_,i,cr_]·cr[b,cr_].
    let mut y = vec![T::zero(); n];
    for a in 0..rk {
        for cl_ in 0..ck {
            let lval = cl[a * ck + cl_];
            if lval == T::zero() {
                continue;
            }
            for b in 0..rkp {
                for cr_ in 0..ckp {
                    let rval = cr[b * ckp + cr_];
                    if rval == T::zero() {
                        continue;
                    }
                    for i in 0..nk {
                        let cval = cd[(cl_ * nk + i) * ckp + cr_];
                        y[(a * nk + i) * rkp + b] += lval * cval * rval;
                    }
                }
            }
        }
    }

    for j in 0..n {
        hmat[j * n + j] += ridge;
    }
    solve_dense(&mut hmat, &mut y, n);
    x[k] = CausalTensor::new(y, vec![rk, nk, rkp])?;
    Ok(())
}

/// Left environment of `<x|G|x>` up to (not including) site `k`: shape `[r_k, gG_k, r_k]`.
fn env_g_left<T: Scalar>(
    x: &[CausalTensor<T>],
    g: &CausalTensorTrainOperator<T>,
    k: usize,
) -> (Vec<T>, usize) {
    // E[a, gl, a'] starting at the boundary [1,1,1] = 1.
    let mut e = vec![T::one()];
    let mut rl = 1usize;
    let mut gl = 1usize;
    for (xc, gc) in x.iter().zip(g.cores().iter()).take(k) {
        let (n, xrr) = (xc.shape()[1], xc.shape()[2]);
        let grr = gc.shape()[3];
        let xd = xc.as_slice();
        let gd = gc.as_slice();
        let mut ne = vec![T::zero(); xrr * grr * xrr];
        for a in 0..rl {
            for gli in 0..gl {
                for ap in 0..rl {
                    let ev = e[(a * gl + gli) * rl + ap];
                    if ev == T::zero() {
                        continue;
                    }
                    for i in 0..n {
                        let xai =
                            xd[a * (n * xrr) + i * xrr..a * (n * xrr) + i * xrr + xrr].to_vec();
                        for ip in 0..n {
                            let gv_base = (((gli * n) + i) * n + ip) * grr;
                            for c in 0..xrr {
                                let xv = xai[c];
                                if xv == T::zero() {
                                    continue;
                                }
                                let evx = ev * xv;
                                for grj in 0..grr {
                                    let gv = gd[gv_base + grj];
                                    if gv == T::zero() {
                                        continue;
                                    }
                                    let evxg = evx * gv;
                                    let xpbase = ap * (n * xrr) + ip * xrr;
                                    for cp in 0..xrr {
                                        ne[(c * grr + grj) * xrr + cp] += evxg * xd[xpbase + cp];
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        e = ne;
        rl = xrr;
        gl = grr;
    }
    (e, gl)
}

/// Right environment of `<x|G|x>` from site `k+1`: shape `[r_{k+1}, gG_{k+1}, r_{k+1}]`.
fn env_g_right<T: Scalar>(
    x: &[CausalTensor<T>],
    g: &CausalTensorTrainOperator<T>,
    k: usize,
) -> (Vec<T>, usize) {
    let d = x.len();
    let mut e = vec![T::one()];
    let mut rr = 1usize;
    let mut gr = 1usize;
    for (xc, gc) in x[k + 1..d].iter().zip(g.cores()[k + 1..d].iter()).rev() {
        let (xrl, n, xrr) = (xc.shape()[0], xc.shape()[1], xc.shape()[2]);
        let grl_ = gc.shape()[0];
        let grr = gc.shape()[3];
        let xd = xc.as_slice();
        let gd = gc.as_slice();
        let mut ne = vec![T::zero(); xrl * grl_ * xrl];
        for a in 0..xrl {
            for gli in 0..grl_ {
                for ap in 0..xrl {
                    let mut acc = T::zero();
                    for i in 0..n {
                        for ip in 0..n {
                            for c in 0..rr {
                                let xv = xd[a * (n * xrr) + i * xrr + c];
                                if xv == T::zero() {
                                    continue;
                                }
                                for grj in 0..gr {
                                    let gv = gd[(((gli * n) + i) * n + ip) * grr + grj];
                                    if gv == T::zero() {
                                        continue;
                                    }
                                    for cp in 0..rr {
                                        let ev2 = e[(c * gr + grj) * rr + cp];
                                        acc += xv * gv * ev2 * xd[ap * (n * xrr) + ip * xrr + cp];
                                    }
                                }
                            }
                        }
                    }
                    ne[(a * grl_ + gli) * xrl + ap] = acc;
                }
            }
        }
        e = ne;
        rr = xrl;
        gr = grl_;
    }
    (e, gr)
}

/// Left environment of `<x|c>` up to site `k`: shape `[r_k, c_k]`.
fn env_c_left<T: Scalar>(x: &[CausalTensor<T>], c: &CausalTensorTrain<T>, k: usize) -> Vec<T> {
    let mut e = vec![T::one()];
    let mut rl = 1usize;
    let mut cl = 1usize;
    for (xc, cc) in x.iter().zip(c.cores().iter()).take(k) {
        let (n, xrr) = (xc.shape()[1], xc.shape()[2]);
        let crr = cc.shape()[2];
        let xd = xc.as_slice();
        let cd = cc.as_slice();
        let mut ne = vec![T::zero(); xrr * crr];
        for a in 0..rl {
            for ci in 0..cl {
                let ev = e[a * cl + ci];
                if ev == T::zero() {
                    continue;
                }
                for i in 0..n {
                    for cpp in 0..xrr {
                        let xv = xd[a * (n * xrr) + i * xrr + cpp];
                        if xv == T::zero() {
                            continue;
                        }
                        for cj in 0..crr {
                            ne[cpp * crr + cj] += ev * xv * cd[(ci * n + i) * crr + cj];
                        }
                    }
                }
            }
        }
        e = ne;
        rl = xrr;
        cl = crr;
    }
    e
}

/// Right environment of `<x|c>` from site `k+1`: shape `[r_{k+1}, c_{k+1}]`.
fn env_c_right<T: Scalar>(x: &[CausalTensor<T>], c: &CausalTensorTrain<T>, k: usize) -> Vec<T> {
    let d = x.len();
    let mut e = vec![T::one()];
    let mut rr = 1usize;
    let mut cr = 1usize;
    for (xc, cc) in x[k + 1..d].iter().zip(c.cores()[k + 1..d].iter()).rev() {
        let (xrl, n, xrr) = (xc.shape()[0], xc.shape()[1], xc.shape()[2]);
        let (crl, crr) = (cc.shape()[0], cc.shape()[2]);
        let xd = xc.as_slice();
        let cd = cc.as_slice();
        let mut ne = vec![T::zero(); xrl * crl];
        for a in 0..xrl {
            for ci in 0..crl {
                let mut acc = T::zero();
                for i in 0..n {
                    for cpp in 0..rr {
                        let xv = xd[a * (n * xrr) + i * xrr + cpp];
                        if xv == T::zero() {
                            continue;
                        }
                        for cj in 0..cr {
                            acc += xv * cd[(ci * n + i) * crr + cj] * e[cpp * cr + cj];
                        }
                    }
                }
                ne[a * crl + ci] = acc;
            }
        }
        e = ne;
        rr = xrl;
        cr = crl;
    }
    e
}

fn linear_residual<T: Scalar>(
    g: &CausalTensorTrainOperator<T>,
    c: &CausalTensorTrain<T>,
    x: &[CausalTensor<T>],
) -> Result<T, CausalTensorError> {
    // ‖G x − c‖ relative to ‖c‖.
    let exact = Truncation::by_bond(usize::MAX)?;
    let train = CausalTensorTrain::from_cores_raw(x.to_vec(), CanonicalForm::None);
    let gx = g.apply(&train, &exact)?;
    let diff = gx.add(&c.scale(-T::one()))?;
    let nc = c.norm()?;
    Ok(diff.norm()? / (nc + T::epsilon()))
}

// ============================================================================
// shared helpers
// ============================================================================

/// Per-bond ranks: `r_0 = r_d = 1`, interior `r_k = min(max_rank, ∏ left dims, ∏ right dims)`.
fn bond_ranks(shape: &[usize], max_rank: usize) -> Vec<usize> {
    let d = shape.len();
    let mut left = vec![1usize; d + 1];
    for k in 1..=d {
        left[k] = left[k - 1].saturating_mul(shape[k - 1]);
    }
    let mut right = vec![1usize; d + 1];
    for k in (0..d).rev() {
        right[k] = right[k + 1].saturating_mul(shape[k]);
    }
    let mut r = vec![1usize; d + 1];
    for (k, slot) in r.iter_mut().enumerate().take(d).skip(1) {
        *slot = max_rank.min(left[k]).min(right[k]).max(1);
    }
    r
}

/// A deterministically-seeded random train with the given per-bond ranks.
fn init_random<T: Scalar>(shape: &[usize], ranks: &[usize], seed: u64) -> Vec<CausalTensor<T>> {
    let d = shape.len();
    let mut state = seed;
    (0..d)
        .map(|k| {
            let (rl, n, rr) = (ranks[k], shape[k], ranks[k + 1]);
            let data = (0..rl * n * rr).map(|_| rand_unit(&mut state)).collect();
            CausalTensor::new(data, vec![rl, n, rr]).unwrap()
        })
        .collect()
}

/// Forward `0..d` then backward `d-2..=1` site order for one sweep.
fn forward_then_back(d: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..d).collect();
    if d >= 2 {
        order.extend((1..d - 1).rev());
    }
    order
}

/// Solves `m·z = y` (row-major `n×n`) by Gaussian elimination with partial pivoting; on return `y`
/// holds `z`. A (near-)singular pivot leaves that component at zero.
fn solve_dense<T: Scalar>(m: &mut [T], y: &mut [T], n: usize) {
    for col in 0..n {
        let mut piv = col;
        let mut best = m[col * n + col].abs();
        for r in (col + 1)..n {
            let v = m[r * n + col].abs();
            if v > best {
                best = v;
                piv = r;
            }
        }
        if best <= T::zero() {
            continue;
        }
        if piv != col {
            for cc in 0..n {
                m.swap(col * n + cc, piv * n + cc);
            }
            y.swap(col, piv);
        }
        let d = m[col * n + col];
        for r in 0..n {
            if r == col {
                continue;
            }
            let factor = m[r * n + col] / d;
            if factor == T::zero() {
                continue;
            }
            for cc in col..n {
                let sub = factor * m[col * n + cc];
                m[r * n + cc] -= sub;
            }
            let sy = factor * y[col];
            y[r] -= sy;
        }
    }
    for col in 0..n {
        let d = m[col * n + col];
        if d != T::zero() {
            y[col] = y[col] / d;
        } else {
            y[col] = T::zero();
        }
    }
}

fn rand_unit<T: Scalar>(state: &mut u64) -> T {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    let unit = (z >> 11) as f64 / (1u64 << 53) as f64;
    <T as deep_causality_num::FromPrimitive>::from_f64(unit * 2.0 - 1.0).unwrap()
}
