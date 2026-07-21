/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `BoundaryZone` trait: a first-class, composable boundary condition with layered dispatch.

use alloc::vec::Vec;

use deep_causality_topology::{LatticeComplex, Manifold};

use crate::solvers::dec::DecNsScalar;

/// A composable boundary condition for the DEC Navier–Stokes solver.
///
/// A zone declares **which solver stage(s) it affects** through one `collect_*` hook per stage,
/// each defaulting to a no-op — so a zone overrides only the layers it touches and is inert on the
/// rest (design `add-boundary-zone-abstraction` D1). The solver folds every zone's contribution at
/// the matching stage:
///
/// * `collect_rate_source` — adds a forcing term to the rate right-hand side (e.g. a body force).
/// * `collect_constrained_edges` — edges pinned to zero in the constrained projection, unioned with
///   the structural no-slip set.
/// * `collect_lift` — prescribed (inhomogeneous, possibly step-dependent) edge values, e.g. a
///   moving-wall tangential velocity.
/// * `collect_prescribed_edges` — inflow edges fixed at their field value, their flux counted in
///   the open-boundary projection's divergence (`leray_project_open_opts`).
/// * `collect_reference_vertices` — outflow pressure-reference vertices for the open-boundary
///   projection.
/// * `collect_slip_edges` — edges *removed* from the structural no-slip set, freeing them for a
///   free-slip wall.
///
/// The list above is the whole set, and every entry is folded by
/// [`DecNsSolver::with_zones`](crate::solvers::dec::DecNsSolver::with_zones) — a hook documented
/// here but never read would be a promise the solver does not keep.
///
/// Each hook *collects into* an accumulator, so a composite zone simply sequences its members'
/// calls — composition is **static** (a typed tuple, no `dyn`; see the `()` and `(A, B)` impls),
/// resolved at compile time on the HKT foundation.
///
/// Structural boundaries — no-slip on lattice walls (non-periodic axes) and immersed bodies (a
/// `CutCellRegistry` on the geometry) — are derived automatically at the topology/metric layer and
/// are not part of the zone set; the zone set carries the explicit boundary **actuators**.
pub trait BoundaryZone<const D: usize, R: DecNsScalar> {
    /// Add this zone's forcing term to the rate source accumulator (edge cochain).
    fn collect_rate_source(&self, _manifold: &Manifold<LatticeComplex<D, R>, R>, _acc: &mut [R]) {}

    /// Add this zone's zero-constrained edges.
    ///
    /// Composed by **union** with the structural no-slip set and the inflow edges: a constrained
    /// edge is one pinned to zero rate, so pinning it twice is idempotent. The union is taken after
    /// `collect_slip_edges` has un-pinned its edges, so a constraint supplied here outranks a
    /// free-slip relaxation.
    ///
    /// No shipped zone implements this hook. Its intended consumer is the `aperture-resolved-noslip`
    /// capability — fragment-accurate cut-cell no-slip is precisely a zone that supplies its own
    /// constrained edges rather than accepting the staircase set derived from the lattice.
    fn collect_constrained_edges(
        &self,
        _manifold: &Manifold<LatticeComplex<D, R>, R>,
        _out: &mut Vec<usize>,
    ) {
    }

    /// Add this zone's prescribed (inhomogeneous) edge values for march step `step`.
    fn collect_lift(
        &self,
        _manifold: &Manifold<LatticeComplex<D, R>, R>,
        _step: usize,
        _out: &mut Vec<(usize, R)>,
    ) {
    }

    /// Add this zone's prescribed (inflow) edges, fixed at their field value.
    fn collect_prescribed_edges(
        &self,
        _manifold: &Manifold<LatticeComplex<D, R>, R>,
        _out: &mut Vec<usize>,
    ) {
    }

    /// Add this zone's outflow pressure-reference vertices.
    fn collect_reference_vertices(
        &self,
        _manifold: &Manifold<LatticeComplex<D, R>, R>,
        _out: &mut Vec<usize>,
    ) {
    }

    /// Add this zone's **un-pinned** (free-slip) edges: edges the auto-derived no-slip set would
    /// pin, but which this zone frees (a free-slip wall's tangential edges, zero shear). The solver
    /// removes them from the no-slip set.
    fn collect_slip_edges(
        &self,
        _manifold: &Manifold<LatticeComplex<D, R>, R>,
        _out: &mut Vec<usize>,
    ) {
    }
}

/// The identity zone: no boundary contribution (the closed-domain default).
impl<const D: usize, R: DecNsScalar> BoundaryZone<D, R> for () {}

/// Static composition: a pair of zones is itself a zone that folds each stage over both members.
impl<const D: usize, R: DecNsScalar, A, B> BoundaryZone<D, R> for (A, B)
where
    A: BoundaryZone<D, R>,
    B: BoundaryZone<D, R>,
{
    fn collect_rate_source(&self, manifold: &Manifold<LatticeComplex<D, R>, R>, acc: &mut [R]) {
        self.0.collect_rate_source(manifold, acc);
        self.1.collect_rate_source(manifold, acc);
    }

    fn collect_constrained_edges(
        &self,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
        out: &mut Vec<usize>,
    ) {
        self.0.collect_constrained_edges(manifold, out);
        self.1.collect_constrained_edges(manifold, out);
    }

    fn collect_lift(
        &self,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
        step: usize,
        out: &mut Vec<(usize, R)>,
    ) {
        self.0.collect_lift(manifold, step, out);
        self.1.collect_lift(manifold, step, out);
    }

    fn collect_prescribed_edges(
        &self,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
        out: &mut Vec<usize>,
    ) {
        self.0.collect_prescribed_edges(manifold, out);
        self.1.collect_prescribed_edges(manifold, out);
    }

    fn collect_reference_vertices(
        &self,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
        out: &mut Vec<usize>,
    ) {
        self.0.collect_reference_vertices(manifold, out);
        self.1.collect_reference_vertices(manifold, out);
    }

    fn collect_slip_edges(
        &self,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
        out: &mut Vec<usize>,
    ) {
        self.0.collect_slip_edges(manifold, out);
        self.1.collect_slip_edges(manifold, out);
    }
}
