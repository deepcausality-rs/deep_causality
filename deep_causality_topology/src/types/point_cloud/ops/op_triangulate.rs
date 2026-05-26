/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{PointCloud, Simplex, SimplicialComplex, Skeleton, TopologyError};
use deep_causality_sparse::CsrMatrix;
use std::collections::BTreeSet;

use deep_causality_num::{Float, Zero};
use std::iter::Sum;

// Helper function to calculate Euclidean distance between two points
fn euclidean_distance<T>(p1: &[T], p2: &[T]) -> T
where
    T: Float + Sum,
{
    p1.iter()
        .zip(p2.iter())
        .map(|(&a, &b)| (a - b).powi(2))
        .sum::<T>()
        .sqrt()
}

/// Computes the Volume (Hypervolume) of a simplex using the Cayley-Menger Determinant.
/// This is coordinate-system invariant and works for any dimension.
fn simplex_volume<T>(simplex: &Simplex, points: &[T], dim: usize) -> T
where
    T: Float + Sum + From<f64>,
{
    let k = simplex.vertices.len(); // k+1 vertices -> k-simplex
    if k == 0 {
        return T::zero();
    } // Empty
    if k == 1 {
        return <T as From<f64>>::from(1.0);
    } // 0-simplex (Point) has volume 1.0 by convention in DEC weights

    // 1-simplex (Edge)
    if k == 2 {
        let p1 = &points[simplex.vertices[0] * dim..(simplex.vertices[0] + 1) * dim];
        let p2 = &points[simplex.vertices[1] * dim..(simplex.vertices[1] + 1) * dim];
        return euclidean_distance(p1, p2);
    }

    // k-simplex (Triangle, Tet, etc.)
    // V = sqrt(det(Gram)) / k!
    let v0 = &points[simplex.vertices[0] * dim..(simplex.vertices[0] + 1) * dim];
    // Construct vectors relative to v0
    let mut vectors = Vec::new();
    for i in 1..k {
        let vi = &points[simplex.vertices[i] * dim..(simplex.vertices[i] + 1) * dim];
        let vec_i: Vec<T> = vi.iter().zip(v0.iter()).map(|(&a, &b)| a - b).collect();
        vectors.push(vec_i);
    }

    // Gram Matrix G_ij = v_i . v_j
    // Flattened for simple determinant calc
    let mut matrix_data = Vec::new();
    let n_vecs = k - 1;

    for i in 0..n_vecs {
        for j in 0..n_vecs {
            let dot: T = vectors[i]
                .iter()
                .zip(vectors[j].iter())
                .map(|(&a, &b)| a * b)
                .sum();
            matrix_data.push(dot);
        }
    }

    // Determinant (Simplified Gaussian Elimination for N dimensions)
    let det = gaussian_determinant(&mut matrix_data, n_vecs);

    if det <= T::zero() {
        return T::zero();
    }

    let mut factorial = 1.0;
    for i in 1..=n_vecs {
        factorial *= i as f64;
    }

    det.sqrt() / <T as From<f64>>::from(factorial)
}

// Simple in-place determinant for variable size
fn gaussian_determinant<T>(mat: &mut [T], n: usize) -> T
where
    T: Float + From<f64>,
{
    let mut det = T::one();
    let pivot_threshold = T::epsilon() * <T as From<f64>>::from(100.0);
    for i in 0..n {
        let pivot = i * n + i;
        if mat[pivot].abs() < pivot_threshold {
            return T::zero();
        }
        det = det * mat[pivot];

        for j in (i + 1)..n {
            let factor = mat[j * n + i] / mat[pivot];
            for k in i..n {
                let val = mat[i * n + k]; // Copy to avoid borrow issues if needed, T is Copy
                mat[j * n + k] = mat[j * n + k] - factor * val;
            }
        }
    }
    det
}

/// Returns the first duplicate-input-point pair, or `None` if every pair of
/// distinct indices has Euclidean distance at least `T::epsilon() * max_extent`,
/// where `max_extent` is the largest axis-aligned bounding-box extent of the
/// input coordinates. When `max_extent` is itself zero (all input points
/// coincide), the threshold falls back to `T::epsilon()` so that any zero-
/// distance pair is still detected.
fn find_duplicate_points<T>(coords: &[T], num_points: usize, dim: usize) -> Option<(usize, usize)>
where
    T: Float + Sum + From<f64> + PartialOrd + Copy,
{
    if num_points < 2 {
        return None;
    }

    let mut max_extent = T::zero();
    for axis in 0..dim {
        let mut lo = coords[axis];
        let mut hi = coords[axis];
        for i in 1..num_points {
            let v = coords[i * dim + axis];
            if v < lo {
                lo = v;
            }
            if v > hi {
                hi = v;
            }
        }
        let span = hi - lo;
        if span > max_extent {
            max_extent = span;
        }
    }

    let threshold = if max_extent > T::zero() {
        T::epsilon() * max_extent
    } else {
        T::epsilon()
    };

    for i in 0..num_points {
        for j in (i + 1)..num_points {
            let p1 = &coords[i * dim..(i + 1) * dim];
            let p2 = &coords[j * dim..(j + 1) * dim];
            if euclidean_distance(p1, p2) < threshold {
                return Some((i, j));
            }
        }
    }
    None
}

impl<T, D> PointCloud<T, D>
where
    T: Float + Sum + From<f64> + Zero + PartialOrd + Copy,
{
    /// Builds a Vietoris-Rips simplicial complex from the point cloud at the
    /// given connectivity radius. Two vertices form an edge iff their Euclidean
    /// distance is at most `radius`; higher simplices are built by clique
    /// expansion, capped at the ambient dimension.
    ///
    /// # Errors
    ///
    /// Returns `Err(TopologyError::PointCloudError(_))` in three cases:
    ///
    /// 1. **Empty input.** The point cloud contains no points.
    /// 2. **Duplicate input points.** A pair of input points has Euclidean
    ///    distance below `T::epsilon() * max_extent`, where `max_extent` is the
    ///    largest axis-aligned bounding-box extent. The error message contains
    ///    the substring `"duplicate point"` and references both offending
    ///    indices. Callers must deduplicate input geometry upstream.
    /// 3. **Degenerate top simplex.** A top-dimensional simplex in the
    ///    constructed complex has computed volume below `T::epsilon() * 100`.
    ///    This indicates geometrically-degenerate input (collinear vertices in
    ///    2D ambient, coplanar vertices in 3D ambient, etc.). The error
    ///    message contains the substrings `"top-dimensional simplex"` and
    ///    `"below tolerance"`. Callers must either tighten the input geometry
    ///    or pick a different triangulation strategy.
    ///
    /// All numerical tolerance comparisons scale with `T::epsilon()`. The
    /// `T::epsilon() * 100` constant is the workspace-standard near-zero
    /// scaling for generic float types and applies uniformly across the
    /// `RealField` family (`f32`, `f64`, `Float106`). No hard-coded `f64`
    /// literal appears in the rejection logic.
    ///
    /// # Precondition contract
    ///
    /// Callers should ensure input geometry is non-degenerate in the regime
    /// `coordinates ~ O(1)` magnitude. Inputs far outside that regime are
    /// accepted but the floating-point rejection thresholds become less
    /// reliable; downstream consumers with strict-precision requirements
    /// should pre-normalize coordinates.
    pub fn triangulate(&self, radius: T) -> Result<SimplicialComplex<T>, TopologyError> {
        if self.is_empty() {
            return Err(TopologyError::PointCloudError("Empty Cloud".to_string()));
        }

        let num_points = self.len();
        let dim = self.points.shape()[1];
        let coords = self.points.as_slice();

        if let Some((i, j)) = find_duplicate_points(coords, num_points, dim) {
            return Err(TopologyError::PointCloudError(format!(
                "triangulate: duplicate point at index {} matches index {} (distance below T::epsilon() * max_extent)",
                i, j
            )));
        }

        // 1. Build 0-Skeleton
        let mut zero_simplices = Vec::with_capacity(num_points);
        for i in 0..num_points {
            zero_simplices.push(Simplex::new(vec![i]));
        }
        let zero_skeleton = Skeleton::new(0, zero_simplices);

        // 2. Build 1-Skeleton (Edges)
        let mut one_simplices = BTreeSet::new();
        let mut adj = vec![vec![false; num_points]; num_points];

        for i in 0..num_points {
            for j in (i + 1)..num_points {
                let p1 = &coords[i * dim..(i + 1) * dim];
                let p2 = &coords[j * dim..(j + 1) * dim];
                if euclidean_distance(p1, p2) <= radius {
                    adj[i][j] = true;
                    adj[j][i] = true;
                    one_simplices.insert(Simplex::new(vec![i, j]));
                }
            }
        }
        let one_skeleton = Skeleton::new(1, one_simplices.into_iter().collect());

        let mut skeletons = vec![zero_skeleton, one_skeleton];

        // 3. Build Higher Skeletons (Clique Expansion)
        //
        // The clique-expansion loop is capped at the ambient dimension `dim`.
        // A k-simplex with k > ambient_dim has at least k+1 vertices lying in
        // an ambient_dim-dimensional space, so it is necessarily degenerate
        // (zero signed volume). Building such a simplex produces a complex
        // whose lumped-mass Hodge ⋆ at grade 0 collapses to zero, which in
        // turn makes the simplicial codifferential return identically zero
        // for any field on it. The cap is therefore both mathematically
        // correct (degenerate simplices carry no geometric information) and
        // necessary for downstream consumers such as `Manifold::laplacian`,
        // `Manifold::codifferential`, and `Manifold::hodge_decompose` to
        // produce non-trivial output.
        //
        // For non-degenerate point clouds where `num_points <= ambient_dim + 1`,
        // the cap never fires because the clique expansion naturally
        // terminates before reaching grade `dim + 1`.
        let mut k = 2;
        loop {
            if k > dim {
                break;
            }
            let prev_skel = &skeletons[k - 1];
            if prev_skel.simplices.is_empty() {
                break;
            }

            let mut next_simplices = BTreeSet::new();

            for simplex in &prev_skel.simplices {
                // Try extending with every vertex
                #[allow(clippy::needless_range_loop)]
                for v in 0..num_points {
                    if simplex.contains_vertex(&v) {
                        continue;
                    }

                    // Check connectivity to all existing vertices in simplex
                    if simplex.vertices().iter().all(|&u| adj[u][v]) {
                        let mut new_verts = simplex.vertices().clone();
                        new_verts.push(v);
                        new_verts.sort_unstable(); // Canonical
                        next_simplices.insert(Simplex::new(new_verts));
                    }
                }
            }

            if next_simplices.is_empty() {
                break;
            }
            skeletons.push(Skeleton::new(k, next_simplices.into_iter().collect()));
            k += 1;
        }

        let max_dim = skeletons.len() - 1;

        // 4. Build Boundary Operators
        // Convention: boundary_operators[k] maps (k+1)-simplices to k-simplices
        // boundary_operators[k] has shape (N_k x N_{k+1})
        let mut boundary_ops = Vec::new();

        for k in 0..max_dim {
            let rows = skeletons[k].simplices.len();
            let cols = skeletons[k + 1].simplices.len();
            let mut triplets = Vec::new();

            for (col, simplex) in skeletons[k + 1].simplices.iter().enumerate() {
                for i in 0..=(k + 1) {
                    // Create face by removing vertex i
                    let mut face_verts = simplex.vertices.clone();
                    face_verts.remove(i);
                    let face = Simplex::new(face_verts);

                    if let Some(row) = skeletons[k].get_index(&face) {
                        let sign = if i % 2 == 0 { 1 } else { -1 };
                        triplets.push((row, col, sign));
                    }
                }
            }
            boundary_ops.push(CsrMatrix::from_triplets(rows, cols, &triplets).unwrap());
        }

        // 5. Build Coboundary Operators (Transpose)
        // Convention: coboundary_operators[k] = boundary_operators[k].transpose()
        let coboundary_ops: Vec<_> = boundary_ops.iter().map(|b| b.transpose()).collect();

        // 6. Build Mass Matrices (Diagonal Hodge Star) using Barycentric Dual Volumes
        // This is the scientifically critical part for diffusion.
        // Mass_k [i,i] = Vol(Dual_i) / Vol(Primal_i)

        // First, compute Primal Volumes for EVERYTHING.
        let mut primal_volumes: Vec<Vec<T>> = Vec::new();
        for skel in &skeletons {
            let vols = skel
                .simplices
                .iter()
                .map(|s| simplex_volume(s, coords, dim))
                .collect();
            primal_volumes.push(vols);
        }

        // Compute Dual Volumes (Barycentric approximation)
        // For a k-simplex sigma, V_dual(sigma) = Sum_{tau > sigma} coeff * Vol(tau)
        // The rigorous Barycentric dual volume is complex.
        // We use the "Lumped Mass" approximation which is standard for diagonal matrices.
        // Mass_0 (Vertex): Sum of (Vol(incident n-simplices) / (n+1))
        // Mass_n (Top): 1.0 / Vol(n-simplex)

        let mut hodge_ops = Vec::new();

        for k_dim in 0..=max_dim {
            let count = skeletons[k_dim].simplices.len();
            let mut triplets = Vec::new();

            for i in 0..count {
                let primal_vol = primal_volumes[k_dim][i];

                let mass_val = if k_dim == 0 {
                    // Vertex Mass: "Lumped" volume around the vertex.
                    // V_dual = Sum(Vol(n-simplices containing v)) / (n+1)
                    // This distributes the total volume equally to vertices.
                    let mut dual_vol = T::zero();
                    let n_skel = &skeletons[max_dim];
                    let n_vols = &primal_volumes[max_dim];

                    // Note: This loop is O(Vertices * Cells), optimization would use an incidence map.
                    // For small-medium point clouds, this is acceptable.
                    for (cell_idx, cell) in n_skel.simplices.iter().enumerate() {
                        if cell.contains_vertex(&i) {
                            dual_vol = dual_vol + n_vols[cell_idx];
                        }
                    }
                    dual_vol / <T as From<f64>>::from((max_dim + 1) as f64)
                } else if k_dim == max_dim {
                    // Top-dimension form.
                    // Inner product <a, b> = Integral (a * b) dV
                    // Since top-forms are densities per volume, and we store integrated values,
                    // the metric term is 1/Volume.
                    // (Standard DEC derivation for diagonal star_n)
                    let top_threshold = T::epsilon() * <T as From<f64>>::from(100.0);
                    if primal_vol > top_threshold {
                        <T as From<f64>>::from(1.0) / primal_vol
                    } else {
                        return Err(TopologyError::PointCloudError(format!(
                            "triangulate: top-dimensional simplex at index {} has volume below tolerance (T::epsilon() * 100), indicating degenerate input geometry",
                            i
                        )));
                    }
                } else {
                    // Intermediate dimensions (Edges in 2D/3D).
                    // Approximation: M_k = Vol(Primal) / Vol(Dual) is for hodge star *mapping*.
                    // But here we need the Mass Matrix M for the inner product.
                    // M_k [i,i] ~ Primal_Vol
                    //
                    // For non-degenerate top simplices (guaranteed by the top-mass branch),
                    // every sub-simplex is non-degenerate by linear-algebra transitivity.
                    // The duplicate-point check at the top of triangulate guarantees positive
                    // edge lengths. The assertion is therefore unreachable in release builds;
                    // it catches future regressions where the transitivity argument breaks.
                    debug_assert!(
                        primal_vol > T::zero(),
                        "intermediate-grade simplex has non-positive primal volume; upstream duplicate-point and top-volume checks should have caught this"
                    );
                    primal_vol
                };

                triplets.push((i, i, mass_val));
            }

            hodge_ops.push(CsrMatrix::from_triplets(count, count, &triplets).unwrap());
        }

        Ok(SimplicialComplex::new(
            skeletons,
            boundary_ops,
            coboundary_ops,
            hodge_ops,
        ))
    }
}
