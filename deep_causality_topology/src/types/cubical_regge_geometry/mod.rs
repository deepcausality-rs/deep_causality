/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cubical Regge geometry — the cubical analogue of `ReggeGeometry<T>`.
//!
//! `CubicalReggeGeometry<D>` encodes the discrete geometric data on a D-dimensional
//! cubical (lattice) complex. It is the `Metric` associated type chosen by the
//! `ChainComplex` impl on `LatticeComplex<D>` and pairs structurally with the simplicial
//! `ReggeGeometry<T>`: where Regge calculus stores edge lengths on a simplicial complex
//! and derives curvature via deficit angles around codimension-2 hinges, the cubical
//! Regge formulation does the same on a hypercubic lattice.
//!
//! ## Naming
//!
//! "Cubical Regge calculus" / "hypercubic Regge calculus" appears in the lattice quantum
//! gravity literature as the natural extension of Regge's original simplicial construction
//! to cubical cells. The type name follows this established (if minor) usage and parallels
//! `ReggeGeometry<T>` in the simplicial path.
//!
//! ## Edge-length representation
//!
//! Cubical complexes admit three useful levels of metric uniformity:
//!
//! | Level | Storage | Use case |
//! |-------|---------|----------|
//! | Unit edge | nothing | Standard voxel grids, the Stage C / issue #487 fast path. |
//! | Uniform | `f64` | Isotropic lattice with a single spacing `a`. The lattice-spacing case in physics. |
//! | Per-axis | `[f64; D]` | Anisotropic but axis-aligned lattices. |
//! | Per-edge | `Vec<f64>` | Fully general, edge-by-edge length assignment. Required for curved cubical metrics. |
//!
//! The four levels are unioned in the private `EdgeLengths<D>` enum. Constructors exist
//! for each level so callers state intent at construction time and the type-level uniformity
//! information is preserved for downstream optimization.
//!
//! ## Forward-looking scope (not yet implemented)
//!
//! The Stage C scope ships **edge-length storage and access only**. The full cubical Regge
//! geometry includes the following derived quantities; the API is shaped to receive them:
//!
//! 1. **`cube_volume(cell_id)`** — the Euclidean volume of a top-dimensional cube as a
//!    product of incident edge lengths (or the appropriate determinant in the per-edge
//!    case). Straightforward once edge-to-axis lookup is added.
//! 2. **`face_area(cell_id)`** — areas of (D−1)-cells, derived from edge lengths.
//! 3. **`deficit_angle(bone_id)`** — the discrete curvature concentrated on codimension-2
//!    cells ("bones"), summing the angles of the cubes incident to each bone. This is the
//!    direct cubical analog of Regge's deficit-angle curvature. Requires a cubical
//!    coordinate / dihedral-angle helper.
//! 4. **`hodge_star_matrix(k)`** — the Hodge ⋆ operator on k-forms, derived from edge
//!    lengths and the cubical duality between k-cells and (D−k)-cells. Cubical Hodge ⋆
//!    is diagonal in the regular case (each primal cell has a unique dual cell of the
//!    complementary grade), which makes it cheaper than the simplicial version.
//! 5. **`metric_at(complex, grade, index)`** — the local metric signature at a specific
//!    cell, derived from the surrounding edge lengths. Cubical complexes admit a fast path:
//!    on a regular grid the signature is constant Euclidean (D, 0, 0) for spacelike
//!    lattices and Lorentzian (D−1, 1, 0) when a time axis is distinguished.
//! 6. **Causal / Lorentzian split** — a `is_timelike_axis: [bool; D]` field would let the
//!    type carry both spacelike and timelike axes, mirroring the East-Coast / West-Coast
//!    metric conventions already in `deep_causality_metric`.
//!
//! Each of these methods would land in its own submodule (`volumes.rs`, `curvature.rs`,
//! `hodge.rs`, ...) mirroring the layout of `regge_geometry/`. The struct's current
//! fields are sufficient inputs for all of them.

use deep_causality_metric::Metric;
use deep_causality_num::RealField;

/// Cubical Regge geometry: discrete metric data on a D-dimensional cubical complex.
///
/// Parallels `ReggeGeometry<R>` for the simplicial case. See the module-level doc for
/// the four supported levels of edge-length uniformity and the forward-looking scope of
/// derived geometric quantities. Parameterized over `R: RealField` so the precision of
/// stored edge lengths is a choice at construction time (`f32`, `f64`, `Float106`, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct CubicalReggeGeometry<const D: usize, R: RealField> {
    edge_lengths: EdgeLengths<D, R>,
    /// Optional per-axis flag marking timelike axes for Lorentzian / Minkowski lattices.
    /// `None` ⇒ all axes spacelike (Euclidean metric). `Some([..])` ⇒ flagged axes are
    /// timelike. Forward-looking: drives the metric-signature methods listed in the
    /// module doc; the Stage C constructors leave it `None`.
    timelike_axes: Option<[bool; D]>,
}

/// Private union of the four edge-length representations. Kept private so callers can't
/// inadvertently depend on the variant layout; access is through the public constructors
/// and the `axis_length` / `edge_length` getters.
#[derive(Debug, Clone, PartialEq)]
enum EdgeLengths<const D: usize, R: RealField> {
    /// Every edge has length `1.0`. The Stage C / voxel-grid fast path. Carries no `R`-typed
    /// storage; the `R: RealField` parameter exists only to satisfy the type-level binding.
    UnitEdge,
    /// Every edge has length `length`. Isotropic lattice with a single spacing.
    Uniform { length: R },
    /// Edges within each axis are uniform; lengths may differ between axes.
    PerAxis { lengths: [R; D] },
    /// Fully general per-edge lengths, indexed by edge cell_id in the lattice's
    /// `iter_cells(1)` ordering. Required for curved cubical metrics.
    PerEdge { lengths: Vec<R> },
}

impl<const D: usize, R: RealField> CubicalReggeGeometry<D, R> {
    // -- Constructors -----------------------------------------------------------------

    /// The unit-edge cubical Regge geometry: every edge has length `1.0`.
    ///
    /// The canonical metric for voxel grids and the Stage C / issue #487 fast path.
    /// Returns `R::one()` for uniform / axis / edge queries.
    pub fn unit() -> Self {
        Self {
            edge_lengths: EdgeLengths::UnitEdge,
            timelike_axes: None,
        }
    }

    /// Isotropic lattice with a single spacing `length` on every edge.
    ///
    /// Equivalent to the lattice-spacing scalar `a` used throughout lattice gauge theory.
    pub fn uniform(length: R) -> Self {
        Self {
            edge_lengths: EdgeLengths::Uniform { length },
            timelike_axes: None,
        }
    }

    /// Anisotropic axis-aligned lattice: each axis has its own uniform edge length.
    ///
    /// All edges in dimension `i` have length `lengths[i]`. Useful for stretched lattices,
    /// asymmetric voxel grids, and anisotropic-coupling lattice gauge studies.
    pub fn per_axis(lengths: [R; D]) -> Self {
        Self {
            edge_lengths: EdgeLengths::PerAxis { lengths },
            timelike_axes: None,
        }
    }

    /// Fully general per-edge length assignment.
    ///
    /// `lengths[i]` is the length of the i-th edge in the lattice complex's
    /// `iter_cells(1)` enumeration order. Required for curved cubical metrics
    /// (non-flat lattices, dynamic spacetimes, deficit-angle-bearing geometries).
    pub fn from_edge_lengths(lengths: Vec<R>) -> Self {
        Self {
            edge_lengths: EdgeLengths::PerEdge { lengths },
            timelike_axes: None,
        }
    }

    /// Attach a Lorentzian axis pattern to this geometry.
    ///
    /// Each `true` entry in `timelike` marks the corresponding axis as timelike (negative
    /// signature). When all entries are `false` the geometry remains Euclidean.
    ///
    /// Forward-looking: consumed by the metric-signature methods in the module doc.
    pub fn with_timelike_axes(mut self, timelike: [bool; D]) -> Self {
        self.timelike_axes = Some(timelike);
        self
    }

    // -- Getters ----------------------------------------------------------------------

    /// `true` iff this geometry is the unit-edge case (every edge has length `1.0`).
    pub fn is_unit_edge(&self) -> bool {
        matches!(self.edge_lengths, EdgeLengths::UnitEdge)
    }

    /// `true` iff every edge has the same length (either UnitEdge or Uniform).
    pub fn is_isotropic(&self) -> bool {
        matches!(
            self.edge_lengths,
            EdgeLengths::UnitEdge | EdgeLengths::Uniform { .. }
        )
    }

    /// `true` iff edges within each axis are uniform (UnitEdge, Uniform, or PerAxis).
    pub fn is_axis_aligned(&self) -> bool {
        !matches!(self.edge_lengths, EdgeLengths::PerEdge { .. })
    }

    /// The uniform edge length, if every edge has the same length.
    /// `Some(R::one())` for UnitEdge, `Some(length)` for Uniform, `None` otherwise.
    pub fn uniform_length(&self) -> Option<R> {
        match &self.edge_lengths {
            EdgeLengths::UnitEdge => Some(R::one()),
            EdgeLengths::Uniform { length } => Some(*length),
            _ => None,
        }
    }

    /// Per-axis uniform lengths, if the geometry is axis-aligned.
    /// `Some([R::one(); D])` for UnitEdge, `Some([length; D])` for Uniform, `Some(lengths)`
    /// for PerAxis, `None` for PerEdge.
    pub fn axis_lengths(&self) -> Option<[R; D]> {
        match &self.edge_lengths {
            EdgeLengths::UnitEdge => Some([R::one(); D]),
            EdgeLengths::Uniform { length } => Some([*length; D]),
            EdgeLengths::PerAxis { lengths } => Some(*lengths),
            EdgeLengths::PerEdge { .. } => None,
        }
    }

    /// Edge length for an edge along axis `axis`. Returns `None` if `axis >= D` or if the
    /// geometry is per-edge (in which case `edge_length_at(edge_id)` is the right call).
    pub fn axis_length(&self, axis: usize) -> Option<R> {
        if axis >= D {
            return None;
        }
        match &self.edge_lengths {
            EdgeLengths::UnitEdge => Some(R::one()),
            EdgeLengths::Uniform { length } => Some(*length),
            EdgeLengths::PerAxis { lengths } => Some(lengths[axis]),
            EdgeLengths::PerEdge { .. } => None,
        }
    }

    /// Edge length at a specific edge `edge_id` (index in the lattice's `iter_cells(1)`
    /// ordering). Defined for all four representations: returns the uniform value for the
    /// degenerate cases, the per-edge value for the general case. `None` if `edge_id` is
    /// out of range for a per-edge geometry.
    pub fn edge_length_at(&self, edge_id: usize) -> Option<R> {
        match &self.edge_lengths {
            EdgeLengths::UnitEdge => Some(R::one()),
            EdgeLengths::Uniform { length } => Some(*length),
            EdgeLengths::PerAxis { .. } => {
                // For per-axis representation, edge_id alone is not enough — the axis must
                // be supplied separately via `axis_length`. Returning `None` here signals
                // that the caller should use `axis_length(axis)` for this representation.
                None
            }
            EdgeLengths::PerEdge { lengths } => lengths.get(edge_id).copied(),
        }
    }

    /// All edge lengths as a flat slice, if the geometry is per-edge.
    ///
    /// `Some(slice)` for PerEdge, `None` for the three uniform cases (where there is no
    /// materialized per-edge vector to slice).
    pub fn edge_lengths(&self) -> Option<&[R]> {
        match &self.edge_lengths {
            EdgeLengths::PerEdge { lengths } => Some(lengths.as_slice()),
            _ => None,
        }
    }

    /// Per-axis timelike flag pattern, if one was attached. `None` ⇒ all axes spacelike.
    pub fn timelike_axes(&self) -> Option<&[bool; D]> {
        self.timelike_axes.as_ref()
    }

    /// `true` iff at least one axis is flagged timelike.
    pub fn is_lorentzian(&self) -> bool {
        self.timelike_axes
            .as_ref()
            .is_some_and(|axes| axes.iter().any(|&t| t))
    }

    /// Local metric signature for the cubical complex.
    ///
    /// Stage C fast path: on a regular cubical complex with axis-aligned edge lengths,
    /// the metric signature is constant across the lattice and is determined entirely by
    /// `timelike_axes`. All spacelike axes contribute a `+` (Euclidean) signature; each
    /// flagged timelike axis contributes a `−` (Lorentzian) signature.
    ///
    /// Forward-looking: for per-edge / curved cubical metrics, the signature can vary by
    /// cell and the right entry point will be a per-cell `metric_at(cell_id)` method. The
    /// current method assumes axis-aligned uniformity and returns the global signature.
    pub fn signature(&self) -> Metric {
        let p_plus_q = D;
        let q = self
            .timelike_axes
            .as_ref()
            .map(|axes| axes.iter().filter(|&&t| t).count())
            .unwrap_or(0);
        let p = p_plus_q.saturating_sub(q);
        Metric::from_signature(p, q, 0)
    }
}
