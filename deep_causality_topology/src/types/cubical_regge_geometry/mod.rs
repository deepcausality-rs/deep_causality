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
//! ## Shipped (Phases R1–R3 of `add-cubical-regge-calculus-core`)
//!
//! The geometric core of cubical Regge calculus is implemented across two sibling submodules:
//!
//! - `volumes` — R1: `cell_volume(complex, cell)` and `top_cell_volume(complex, cell)`,
//!   dispatching on the four `EdgeLengths` variants under the axis-aligned cubical assumption.
//! - `curvature` — R2 + R3: `dihedral_angle(complex, top_cube, hinge)` (returns π/2
//!   uniformly under axis-alignment), `deficit_angle(complex, hinge_id)` (depends only on
//!   hinge incidence count), and `regge_action(complex)` (sums `cell_volume(h) ·
//!   deficit_angle(h)` over every (D−2)-hinge).
//!
//! Curvature on an axis-aligned cubical lattice arises from two sources:
//!
//! 1. **Hinge incidence count.** Interior hinges on a periodic lattice have 4 incident
//!    top cubes, sum dihedrals = 2π, deficit = 0 (flat). Boundary hinges on open
//!    lattices have fewer incident cubes (e.g. 1 at a 2D corner, deficit 3π/2 —
//!    intrinsic boundary curvature).
//! 2. **Hinge volumes.** The action's edge-length sensitivity flows through the
//!    `cell_volume(h)` factor — vertex hinges in 2D have volume 1 (empty product),
//!    edge hinges in 3D and square hinges in 4D scale with the metric.
//!
//! ## Forward-looking scope (deferred to follow-up change sets)
//!
//! The following derived quantities are designed-for but not yet implemented; the struct's
//! current fields and submodule layout are sufficient inputs for all of them:
//!
//! - **`hodge_star_matrix(k)`** — Hodge ⋆ on k-forms, diagonal under axis-alignment.
//!   Deferred to `add-cubical-regge-calculus-analytical` (R4). The keystone that promotes
//!   `manifold/differential/{hodge,laplacian}.rs` to be generic over `ChainComplex`.
//! - **`metric_at(complex, cell)`** and a type-level Lorentzian marker — local metric
//!   signature, light-cone enforcement. Deferred to R5. Reads the `timelike_axes` field
//!   which is currently stored but ignored by `regge_action` and `deficit_angle`.
//! - **`regge_gradient(complex)`** and `metropolis_update` — action gradient and Markov
//!   chain dynamics. Deferred to R6.
//!
//! See [`openspec/notes/CubicalReggeCalculus.md`](../../../../openspec/notes/CubicalReggeCalculus.md)
//! for the full R1–R6 design note.

pub mod curvature;
pub mod volumes;

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
    pub(super) edge_lengths: EdgeLengths<D, R>,
    /// Optional per-axis flag marking timelike axes for Lorentzian / Minkowski lattices.
    /// `None` ⇒ all axes spacelike (Euclidean metric). `Some([..])` ⇒ flagged axes are
    /// timelike. Forward-looking: drives the metric-signature methods listed in the
    /// module doc; the Stage C constructors leave it `None`.
    timelike_axes: Option<[bool; D]>,
}

/// Module-private union of the four edge-length representations. `pub(super)` so sibling
/// submodules (`volumes`, `curvature`) can match on the variant for closed-form fast paths;
/// crate-level callers still go through the public constructors and the `axis_length` /
/// `edge_length` getters.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum EdgeLengths<const D: usize, R: RealField> {
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
