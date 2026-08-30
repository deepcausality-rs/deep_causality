/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{PointCloud, Simplex, SimplicialComplex, Skeleton, TopologyError};
use deep_causality_linear::CsrMatrix;
use std::collections::BTreeSet;

use deep_causality_num::{Float, Zero};
use std::iter::Sum;

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

/// Returns the first duplicate-input-point pair, or `None` if every pair of
/// distinct indices has Euclidean distance at least `T::epsilon() * max_extent`,
/// where `max_extent` is the largest axis-aligned bounding-box extent of the
/// input coordinates. When `max_extent` is itself zero (all input points
/// coincide), the threshold falls back to `T::epsilon()` so that any zero-
/// distance pair is still detected.
pub(super) fn find_duplicate_points<T>(
    coords: &[T],
    num_points: usize,
    dim: usize,
) -> Option<(usize, usize)>
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

/// Returns the ambient dimension `M` of an `N x M` `points` tensor, or an
/// error naming the rank actually supplied.
///
/// `PointCloud::new` validates emptiness, matching row counts and cursor
/// range; it does not constrain the rank of the `points` tensor, and the
/// `Display` impl deliberately renders a rank-1 cloud. The `N x M` shape
/// contract is therefore established by the triangulation entry points, each
/// of which documents a `TopologyError::PointCloudError` for a precondition
/// violation. `caller` names the entry point in the message.
pub(super) fn ambient_dim(shape: &[usize], caller: &str) -> Result<usize, TopologyError> {
    if shape.len() != 2 {
        return Err(TopologyError::PointCloudError(format!(
            "{}: `points` must be a rank-2 N x M tensor, got rank {} with shape {:?}",
            caller,
            shape.len(),
            shape
        )));
    }
    Ok(shape[1])
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
    /// The returned `SimplicialComplex<T>` carries the input coordinates as
    /// geometric data for lazy Hodge ⋆ population. The Hodge ⋆ vector is not
    /// built here; the first call to
    /// [`SimplicialComplex::hodge_star_operators`] performs the lumped-mass
    /// build. Topological consumers (clique-complex, Euler characteristic,
    /// persistent homology) that never access the Hodge ⋆ surface succeed on
    /// any input geometry, including geometrically-degenerate clique complexes
    /// where the top-dimensional simplices have zero volume. The H1 top-volume
    /// degeneracy rejection surfaces only at the Hodge ⋆ access point.
    ///
    /// # Errors
    ///
    /// Returns `Err(TopologyError::PointCloudError(_))` in three cases:
    ///
    /// 1. **Empty input.** The point cloud contains no points.
    /// 2. **Coordinates that are not `N x M`.** The `points` tensor has a rank
    ///    other than 2, so it carries no ambient dimension to triangulate in.
    ///    `PointCloud::new` accepts any rank, so this entry point establishes
    ///    the shape contract; the message reports the rank it received.
    /// 3. **Duplicate input points.** A pair of input points has Euclidean
    ///    distance below `T::epsilon() * max_extent`, where `max_extent` is the
    ///    largest axis-aligned bounding-box extent. The error message contains
    ///    the substring `"duplicate point"` and references both offending
    ///    indices. Callers must deduplicate input geometry upstream.
    ///
    /// The previous H1 rejection for **degenerate top simplices** (zero-volume
    /// k-cliques) was moved to
    /// [`SimplicialComplex::hodge_star_operators`] in H4. Consumers that
    /// require a non-degenerate Hodge ⋆ surface should call that accessor and
    /// handle its `Result`; consumers that only need V/E/F-style topological
    /// counts never trigger the rejection.
    ///
    /// All numerical tolerance comparisons scale with `T::epsilon()`. No hard-
    /// coded `f64` literal appears in the rejection logic.
    ///
    /// # Precondition contract
    ///
    /// Callers should ensure input geometry is non-degenerate in the regime
    /// `coordinates ~ O(1)` magnitude. Inputs far outside that regime are
    /// accepted but the floating-point rejection thresholds become less
    /// reliable; downstream DEC consumers with strict-precision requirements
    /// should pre-normalize coordinates.
    pub fn triangulate(&self, radius: T) -> Result<SimplicialComplex<T>, TopologyError> {
        if self.is_empty() {
            return Err(TopologyError::PointCloudError("Empty Cloud".to_string()));
        }

        let num_points = self.len();
        let dim = ambient_dim(self.points.shape(), "triangulate")?;
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

        // 3. Build Higher Skeletons (Clique Expansion) capped at ambient dim.
        //
        // The cap at `k > dim` reflects that simplices of dimension exceeding
        // the ambient embedding dimension are geometrically degenerate. With
        // lazy Hodge ⋆ population (H4), this cap no longer prevents Hodge ⋆
        // collapse at construction time — the lazy accessor enforces that
        // contract at the right place. The cap is retained because clique
        // expansion beyond `dim` produces simplices that contribute nothing
        // meaningful to either TDA or DEC pipelines on a Vietoris-Rips
        // complex.
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
                #[allow(clippy::needless_range_loop)]
                for v in 0..num_points {
                    if simplex.contains_vertex(&v) {
                        continue;
                    }
                    if simplex.vertices().iter().all(|&u| adj[u][v]) {
                        let mut new_verts = simplex.vertices().clone();
                        new_verts.push(v);
                        new_verts.sort_unstable();
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
        let mut boundary_ops = Vec::new();

        for k in 0..max_dim {
            let rows = skeletons[k].simplices.len();
            let cols = skeletons[k + 1].simplices.len();
            let mut triplets = Vec::new();

            for (col, simplex) in skeletons[k + 1].simplices.iter().enumerate() {
                for i in 0..=(k + 1) {
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
        let coboundary_ops: Vec<_> = boundary_ops.iter().map(|b| b.transpose()).collect();

        // 6. Construct the complex with geometric data; Hodge ⋆ is populated
        // lazily on first access via `SimplicialComplex::hodge_star_operators`.
        Ok(SimplicialComplex::with_geometry(
            skeletons,
            boundary_ops,
            coboundary_ops,
            coords.to_vec(),
            dim,
        ))
    }
}
