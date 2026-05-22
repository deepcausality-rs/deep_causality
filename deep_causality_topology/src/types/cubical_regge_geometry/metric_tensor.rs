/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Per-cell metric tensor for `CubicalReggeGeometry<D, R, S>` — Phase R5.3.
//!
//! On an axis-aligned cubical lattice, the local metric tensor at every cell
//! is diagonal. The diagonal entries are the squared edge lengths along each
//! axis, with sign determined by the signature marker `S`:
//!
//! - [`Euclidean`]: every diagonal entry is positive (`+L_axis²`).
//! - [`Lorentzian`]: each entry flagged timelike in `timelike_axes` carries a
//!   negative sign (`−L_axis²`), all others positive — East-Coast convention.
//!
//! The returned `CausalTensor<R>` has shape `[D, D]` and zero off-diagonal
//! entries.

use super::{CubicalReggeGeometry, SignatureMarker};
use crate::types::lattice_complex::{LatticeCell, LatticeComplex};
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;

impl<const D: usize, R, S> CubicalReggeGeometry<D, R, S>
where
    R: RealField + FromPrimitive,
    S: SignatureMarker,
{
    /// Local metric tensor `g_{μν}` at the given cell, encoded as a `CausalTensor`
    /// of shape `[D, D]`.
    ///
    /// Diagonal entry `g_{ii} = ±L_i²` where:
    /// - `L_i` is the edge length along axis `i` at the cell's position.
    /// - The sign is `−` iff axis `i` is timelike (per `timelike_axes` and the
    ///   `Lorentzian` marker); `+` otherwise.
    ///
    /// Off-diagonal entries are zero (axis-aligned cubical assumption).
    pub fn metric_tensor_at(
        &self,
        complex: &LatticeComplex<D, R>,
        cell: &LatticeCell<D>,
    ) -> CausalTensor<R> {
        let position = *cell.position();
        let timelike_axes = self.timelike_axes.as_ref();

        let mut data = vec![R::zero(); D * D];
        for axis in 0..D {
            // Resolve the edge length along this axis at the cell's position.
            let length = self.edge_length_along_axis_at(complex, position, axis);
            let l_sq = length * length;
            // Apply the sign: negative iff this axis is flagged timelike on a
            // Lorentzian signature. The runtime check on `timelike_axes` is
            // sufficient because `Euclidean` constructors guarantee
            // `timelike_axes = None`.
            let is_timelike = timelike_axes.is_some_and(|axes| axes[axis]);
            let entry = if is_timelike { -l_sq } else { l_sq };
            data[axis * D + axis] = entry;
        }
        CausalTensor::new(data, vec![D, D]).expect("D × D metric tensor allocation")
    }

    /// Resolve the edge length along `axis` at the given vertex position,
    /// dispatching on the geometry's edge-length representation. Mirrors the
    /// dispatch used by `cell_volume`.
    fn edge_length_along_axis_at(
        &self,
        complex: &LatticeComplex<D, R>,
        position: [usize; D],
        axis: usize,
    ) -> R {
        use super::EdgeLengths;
        match &self.edge_lengths {
            EdgeLengths::UnitEdge => R::one(),
            EdgeLengths::Uniform { length } => *length,
            EdgeLengths::PerAxis { lengths } => lengths[axis],
            EdgeLengths::PerEdge { lengths } => {
                // For `PerEdge`, look up the edge starting at `position` going
                // positive along `axis`. If that position is past the lattice
                // bound (open lattice), fall back to the edge ending at
                // `position` from the negative direction.
                let shape = complex.shape();
                let is_periodic = complex.periodic()[axis];
                let max_edge_pos = if is_periodic {
                    shape[axis]
                } else if shape[axis] == 0 {
                    return R::one();
                } else {
                    shape[axis] - 1
                };
                let mut probe = position;
                if position[axis] < max_edge_pos {
                    probe[axis] = position[axis];
                } else if position[axis] > 0 {
                    probe[axis] = position[axis] - 1;
                } else {
                    return R::one();
                }
                let idx = complex.edge_index(probe, axis);
                lengths.get(idx).copied().unwrap_or_else(R::one)
            }
        }
    }
}
