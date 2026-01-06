/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
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
    for i in 0..n {
        let pivot = i * n + i;
        if mat[pivot].abs() < <T as From<f64>>::from(1e-12) {
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

impl<T, D> PointCloud<T, D>
where
    T: Float + Sum + From<f64> + Zero + PartialOrd + Copy,
{
    pub fn triangulate(&self, radius: T) -> Result<SimplicialComplex<T>, TopologyError> {
        if self.is_empty() {
            return Err(TopologyError::PointCloudError("Empty Cloud".to_string()));
        }

        let num_points = self.len();
        let dim = self.points.shape()[1];
        let coords = self.points.as_slice();

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
        let mut k = 2;
        loop {
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
                    if primal_vol > <T as From<f64>>::from(1e-12) {
                        <T as From<f64>>::from(1.0) / primal_vol
                    } else {
                        T::zero()
                    }
                } else {
                    // Intermediate dimensions (Edges in 2D/3D).
                    // Approximation: M_k = Vol(Primal) / Vol(Dual) is for hodge star *mapping*.
                    // But here we need the Mass Matrix M for the inner product.
                    // M_k [i,i] ~ Primal_Vol
                    // Wait, for 1-forms (Edges): Energy = Sum (u_i - u_j)^2 * (Dual_Area / Length)
                    // We need the "Constitutive Ratio".

                    // Since we don't have the Dual Connectivity built, we use the
                    // "Unit Weight" fallback for topology, scaled by Primal Volume for geometry.
                    // This ensures at least scaling consistency.
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
