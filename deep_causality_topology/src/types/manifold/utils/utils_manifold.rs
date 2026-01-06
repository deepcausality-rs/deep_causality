/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Utility functions for Manifold validation and analysis.

use crate::SimplicialComplex;

/// Checks if the simplicial complex is oriented.
pub(crate) fn is_oriented<T>(complex: &SimplicialComplex<T>) -> bool {
    let max_dim = complex.max_simplex_dimension();
    if max_dim == 0 {
        return true; // Points are oriented
    }

    // Convention: boundary_operators[k] maps (k+1)-simplices to k-simplices
    // So boundary_operators[max_dim - 1] maps max_dim-simplices to (max_dim-1)-simplices
    if let Some(boundary_op) = complex.boundary_operators.get(max_dim - 1) {
        let rows = boundary_op.shape().0;
        let row_indices = boundary_op.row_indices();
        let values = boundary_op.values();

        for r in 0..rows {
            let start = row_indices[r];
            let end = row_indices[r + 1];
            let mut sum: i8 = 0;
            for &val in &values[start..end] {
                sum += val;
            }

            if sum.abs() > 1 {
                return false;
            }
        }
    }
    true
}

/// Checks if the simplicial complex satisfies the link condition for manifolds.
pub(crate) fn satisfies_link_condition<T>(complex: &SimplicialComplex<T>) -> bool {
    let max_dim = complex.max_simplex_dimension();
    if max_dim == 0 {
        return true; // 0-manifold (points) satisfies link condition trivially
    }

    let num_vertices = complex
        .skeletons
        .first()
        .map(|s| s.simplices.len())
        .unwrap_or(0);
    let sphere_chi = 1 + if (max_dim - 1).is_multiple_of(2) {
        1
    } else {
        -1
    };
    let disk_chi = 1;

    for v_idx in 0..num_vertices {
        let mut link_chi: isize = 0;

        for skeleton in &complex.skeletons {
            let dim = skeleton.dim;
            if dim == 0 {
                continue;
            }

            for simplex in &skeleton.simplices {
                if simplex.vertices.contains(&v_idx) {
                    let term = if (dim - 1) % 2 == 0 { 1 } else { -1 };
                    link_chi += term;
                }
            }
        }

        if link_chi != sphere_chi && link_chi != disk_chi {
            return false;
        }
    }

    true
}

/// Computes the Euler characteristic of the simplicial complex.
#[allow(dead_code)]
pub(crate) fn euler_characteristic<T>(complex: &SimplicialComplex<T>) -> isize {
    let mut chi: isize = 0;
    for skeleton in &complex.skeletons {
        let count = skeleton.simplices.len() as isize;
        if skeleton.dim % 2 == 0 {
            chi += count;
        } else {
            chi -= count;
        }
    }
    chi
}

/// Checks if the simplicial complex has a boundary.
#[allow(dead_code)]
pub(crate) fn has_boundary<T>(complex: &SimplicialComplex<T>) -> bool {
    let max_dim = complex.max_simplex_dimension();
    if max_dim == 0 {
        return false; // Points don't have boundary in this context
    }

    // Convention: boundary_operators[k] maps (k+1)-simplices to k-simplices
    if let Some(boundary_op) = complex.boundary_operators.get(max_dim - 1) {
        let rows = boundary_op.shape().0;
        let row_indices = boundary_op.row_indices();

        for r in 0..rows {
            let start = row_indices[r];
            let end = row_indices[r + 1];
            let count = end - start;

            if count == 1 {
                return true;
            }
        }
    }
    false
}
