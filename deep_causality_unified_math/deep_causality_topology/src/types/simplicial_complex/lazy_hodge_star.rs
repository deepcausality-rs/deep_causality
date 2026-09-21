/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Lazy lumped-mass Hodge ⋆ population for `SimplicialComplex<T>`.
//!
//! The build is a single `pub(crate)` function the lazy accessor invokes on first read, so a
//! triangulated complex pays for the Hodge ⋆ surface only when a caller consumes it.
//!
//! That separates the two kinds of consumer. Topological ones — TDA, clique complexes, Euler
//! characteristics — never reach this path and need satisfy no geometric precondition. Geometric
//! ones — DEC, Hodge ⋆, the Laplacian — see the `"top-dimensional simplex below tolerance"` error
//! at the point of access, since the degeneracy rejection lives here rather than in
//! `PointCloud::triangulate`.

use crate::{Simplex, Skeleton, TopologyError};
use deep_causality_algebra::RealField;
use deep_causality_linear::{CsrMatrix, DenseMatrix, determinant};
use deep_causality_num::FromPrimitive;

fn euclidean_distance<T>(p1: &[T], p2: &[T]) -> T
where
    T: RealField,
{
    let mut acc = T::zero();
    for (a, b) in p1.iter().zip(p2.iter()) {
        let diff = *a - *b;
        acc += diff * diff;
    }
    acc.sqrt()
}

/// Cayley-Menger volume of a simplex against the supplied coordinate slab.
/// Returns `T::zero()` for degenerate simplices (singular Gram matrix or
/// non-positive determinant); the caller surfaces the degeneracy through the
/// top-volume threshold compare in [`build_lumped_mass_hodge_star`].
fn simplex_volume<T>(simplex: &Simplex, points: &[T], dim: usize) -> T
where
    T: RealField + FromPrimitive,
{
    let k = simplex.vertices.len();
    if k == 0 {
        return T::zero();
    }
    if k == 1 {
        return T::one();
    }

    if k == 2 {
        let p1 = &points[simplex.vertices[0] * dim..(simplex.vertices[0] + 1) * dim];
        let p2 = &points[simplex.vertices[1] * dim..(simplex.vertices[1] + 1) * dim];
        return euclidean_distance(p1, p2);
    }

    let v0 = &points[simplex.vertices[0] * dim..(simplex.vertices[0] + 1) * dim];
    let mut vectors = Vec::new();
    for i in 1..k {
        let vi = &points[simplex.vertices[i] * dim..(simplex.vertices[i] + 1) * dim];
        let vec_i: Vec<T> = vi.iter().zip(v0.iter()).map(|(&a, &b)| a - b).collect();
        vectors.push(vec_i);
    }

    let mut matrix_data = Vec::new();
    let n_vecs = k - 1;

    for i in 0..n_vecs {
        for j in 0..n_vecs {
            let mut dot = T::zero();
            for (a, b) in vectors[i].iter().zip(vectors[j].iter()) {
                dot += *a * *b;
            }
            matrix_data.push(dot);
        }
    }

    // The Gram matrix is square by construction, so the only error `determinant` can raise
    // cannot arise. It pivots by search and scales its degeneracy floor by the matrix's own
    // magnitude, so a uniformly small simplex is not read as degenerate merely for being small;
    // the caller's absolute top-volume threshold is what rejects one.
    let gram = DenseMatrix::from_vec(matrix_data, n_vecs, n_vecs)
        .expect("Gram matrix is square by construction");
    let det = determinant(&gram).expect("Gram matrix is square by construction");

    if det <= T::zero() {
        return T::zero();
    }

    let mut factorial: usize = 1;
    for i in 1..=n_vecs {
        factorial *= i;
    }
    let factorial_t = <T as FromPrimitive>::from_usize(factorial)
        .expect("factorial of n_vecs fits in every RealField");

    det.sqrt() / factorial_t
}

/// `P[a][b] = ∇λ_a · ∇λ_b` for one simplex, row-major over `(n+1)²`.
///
/// Only the inner products are ever needed, never the gradient vectors, and they are read
/// straight off the inverse edge-Gram: with `e_i = p_i − p_0`, the barycentric coordinates
/// satisfy `λ = (EᵀE)⁻¹ Eᵀ (x − p_0)`, so `∇λ_i · ∇λ_j = (EᵀE)⁻¹[i][j]` for `i, j ≥ 1`. The
/// row and column for `λ_0` follow from `Σ_a ∇λ_a = 0`, since the barycentric coordinates sum
/// to one everywhere.
///
/// Returns `None` when the edge-Gram is singular, which is a degenerate cell.
fn barycentric_gradient_gram<T>(simplex: &Simplex, points: &[T], dim: usize) -> Option<Vec<T>>
where
    T: RealField + FromPrimitive,
{
    let vs = &simplex.vertices;
    let n = vs.len().checked_sub(1)?;
    if n == 0 {
        return Some(vec![T::zero()]);
    }

    let coord = |v: usize| &points[v * dim..(v + 1) * dim];
    let p0 = coord(vs[0]);
    let edges: Vec<Vec<T>> = (1..=n)
        .map(|i| {
            let pi = coord(vs[i]);
            (0..dim).map(|d| pi[d] - p0[d]).collect()
        })
        .collect();

    let mut gram = vec![T::zero(); n * n];
    for i in 0..n {
        for j in 0..n {
            let mut acc = T::zero();
            for (a, b) in edges[i].iter().zip(edges[j].iter()) {
                acc += *a * *b;
            }
            gram[i * n + j] = acc;
        }
    }

    let gm = DenseMatrix::from_vec(gram, n, n).ok()?;
    let ginv = deep_causality_linear::inverse(&gm).ok()?;
    let gi = ginv.as_slice();

    let m = n + 1;
    let mut p = vec![T::zero(); m * m];
    let mut total = T::zero();
    for i in 0..n {
        for j in 0..n {
            p[(i + 1) * m + (j + 1)] = gi[i * n + j];
            total += gi[i * n + j];
        }
    }
    for j in 0..n {
        let mut col = T::zero();
        for i in 0..n {
            col += gi[i * n + j];
        }
        p[j + 1] = T::zero() - col;
        p[(j + 1) * m] = T::zero() - col;
    }
    p[0] = total;
    Some(p)
}

/// `∫_T W_σ · W_σ dV`, the self-mass of the Whitney k-form of a k-simplex on the cell that
/// carries it.
///
/// With `W_σ = k! Σ_l (−1)^l λ_{i_l} dλ_{i_0} ∧ … ∧ ^dλ_{i_l} ∧ … ∧ dλ_{i_k}` and
/// `∫_T λ_a λ_b dV = |T| (1 + δ_ab) / ((n+1)(n+2))`, expanding the product gives
///
/// ```text
/// (k!)^2 Σ_{l,m} (−1)^{l+m} ∫λ_{i_l}λ_{i_m} · det[ ∇λ_a · ∇λ_b ]
/// ```
///
/// where the determinant is the Gram of the two index lists with positions `l` and `m` removed
/// — the inner product of two decomposable k-covectors. `local` gives σ's vertices as positions
/// within the cell's own vertex list.
fn whitney_self_mass<T>(
    local: &[usize],
    grad_gram: &[T],
    m: usize,
    volume: T,
    n: usize,
) -> Option<T>
where
    T: RealField + FromPrimitive,
{
    let k = local.len().checked_sub(1)?;
    let denom = <T as FromPrimitive>::from_usize((n + 1) * (n + 2))?;
    let two = <T as FromPrimitive>::from_f64(2.0)?;

    let mut total = T::zero();
    for l in 0..=k {
        for mm in 0..=k {
            let rows: Vec<usize> = (0..=k).filter(|t| *t != l).map(|t| local[t]).collect();
            let cols: Vec<usize> = (0..=k).filter(|t| *t != mm).map(|t| local[t]).collect();

            let det = if k == 0 {
                T::one()
            } else {
                let mut sub = vec![T::zero(); k * k];
                for (a, &ra) in rows.iter().enumerate() {
                    for (b, &cb) in cols.iter().enumerate() {
                        sub[a * k + b] = grad_gram[ra * m + cb];
                    }
                }
                determinant(&DenseMatrix::from_vec(sub, k, k).ok()?).ok()?
            };

            // ∫ λ_a λ_b carries the extra factor of two on the diagonal.
            let coincide = if local[l] == local[mm] { two } else { T::one() };
            let integral = volume * coincide / denom;
            let term = integral * det;
            if (l + mm) % 2 == 0 {
                total += term;
            } else {
                total -= term;
            }
        }
    }

    let k_fact = <T as FromPrimitive>::from_usize((1..=k).product::<usize>().max(1))?;
    Some(total * k_fact * k_fact)
}

/// Builds the lumped-mass Hodge ⋆ operators for a simplicial complex from its
/// skeletons and the originating geometric data (coordinates + ambient
/// dimension).
///
/// Returns `Err(TopologyError::PointCloudError)` when the complex contains a
/// top-dimensional simplex of volume below `T::epsilon() * 100`. The error
/// message contains the substrings `"top-dimensional simplex"` and
/// `"below tolerance"` plus the offending simplex index.
///
/// Empty complexes (no skeletons) return `Ok(Vec::new())`.
pub(crate) fn build_lumped_mass_hodge_star<T>(
    skeletons: &[Skeleton],
    coords: &[T],
    dim: usize,
) -> Result<Vec<CsrMatrix<T>>, TopologyError>
where
    T: RealField + FromPrimitive,
{
    if skeletons.is_empty() {
        return Ok(Vec::new());
    }

    let max_dim = skeletons.len() - 1;

    let mut primal_volumes: Vec<Vec<T>> = Vec::with_capacity(skeletons.len());
    for skel in skeletons {
        let vols: Vec<T> = skel
            .simplices
            .iter()
            .map(|s| simplex_volume(s, coords, dim))
            .collect();
        primal_volumes.push(vols);
    }

    let hundred = <T as FromPrimitive>::from_f64(100.0).expect("100.0 fits in every RealField");
    let top_threshold = T::epsilon() * hundred;
    let max_dim_plus_one =
        <T as FromPrimitive>::from_usize(max_dim + 1).expect("max_dim + 1 fits in every RealField");

    // Every top-dimensional volume is checked before any grade is built. The intermediate grades
    // read the barycentric gradients of these same cells, and those do not exist for a degenerate
    // one, so without this the caller would see a failure to invert an edge-Gram at grade 1
    // instead of the degeneracy named as such at grade n.
    // Only where there are intermediate grades, which is where the gradients are read. Below
    // that the per-grade branches keep the behaviour they had: at `max_dim == 0` the `k == 0`
    // arm answers first and a vertexless cell is legitimately given no mass, which this check
    // would otherwise reject.
    if max_dim >= 2 {
        for (i, v) in primal_volumes[max_dim].iter().enumerate() {
            if *v <= top_threshold {
                return Err(TopologyError::PointCloudError(format!(
                    "hodge_star_operators: top-dimensional simplex at index {} has volume below tolerance (T::epsilon() * 100), indicating degenerate input geometry",
                    i
                )));
            }
        }
    }

    let mut hodge_ops = Vec::with_capacity(skeletons.len());

    for k_dim in 0..=max_dim {
        let count = skeletons[k_dim].simplices.len();
        let mut triplets = Vec::new();

        for i in 0..count {
            let primal_vol = primal_volumes[k_dim][i];

            let mass_val = if k_dim == 0 {
                let mut dual_vol = T::zero();
                let n_skel = &skeletons[max_dim];
                let n_vols = &primal_volumes[max_dim];
                for (cell_idx, cell) in n_skel.simplices.iter().enumerate() {
                    if cell.contains_vertex(&i) {
                        dual_vol += n_vols[cell_idx];
                    }
                }
                dual_vol / max_dim_plus_one
            } else if k_dim == max_dim {
                if primal_vol > top_threshold {
                    T::one() / primal_vol
                } else {
                    return Err(TopologyError::PointCloudError(format!(
                        "hodge_star_operators: top-dimensional simplex at index {} has volume below tolerance (T::epsilon() * 100), indicating degenerate input geometry",
                        i
                    )));
                }
            } else {
                debug_assert!(
                    primal_vol > T::zero(),
                    "intermediate-grade simplex has non-positive primal volume; upstream duplicate-point check and top-volume rejection should have caught this"
                );

                // The lumped Whitney mass, summed over the star of the simplex. This is the
                // same quantity the endpoint grades carry: `∫λ_i` over the vertex star at
                // `k = 0`, `1/|T|` at `k = n`.
                //
                // It scales as `h^(n-2k)`. At `k = 1` in three dimensions that exponent is 1,
                // which a primal volume also has, so a refinement study cannot separate the two
                // at that grade and the tests pin closed forms instead.
                let mut mass = T::zero();
                for (cell_idx, cell) in skeletons[max_dim].simplices.iter().enumerate() {
                    let sigma = &skeletons[k_dim].simplices[i];
                    let local: Option<Vec<usize>> = sigma
                        .vertices
                        .iter()
                        .map(|v| cell.vertices.iter().position(|w| w == v))
                        .collect();
                    let Some(local) = local else {
                        continue; // this cell does not carry the simplex
                    };

                    let gram = barycentric_gradient_gram(cell, coords, dim).ok_or_else(|| {
                        TopologyError::PointCloudError(format!(
                            "hodge_star_operators: top-dimensional simplex at index {cell_idx} is \
                             degenerate, so the barycentric gradients it needs do not exist"
                        ))
                    })?;
                    let m = cell.vertices.len();
                    mass += whitney_self_mass(
                        &local,
                        &gram,
                        m,
                        primal_volumes[max_dim][cell_idx],
                        max_dim,
                    )
                    .ok_or_else(|| {
                        TopologyError::PointCloudError(format!(
                            "hodge_star_operators: Whitney mass unavailable for grade {k_dim} \
                             simplex {i}"
                        ))
                    })?;
                }
                mass
            };

            triplets.push((i, i, mass_val));
        }

        hodge_ops.push(CsrMatrix::from_triplets(count, count, &triplets).unwrap());
    }

    Ok(hodge_ops)
}
