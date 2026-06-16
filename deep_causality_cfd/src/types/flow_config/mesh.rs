/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CfdScalar;
use crate::types::flow_config::Body;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCellRegistry, LatticeComplex, Manifold, Primitive,
};

/// The product of [`Mesh::materialize`]: the metric-bearing manifold and, when an
/// immersed body is present, its cut-cell registry for the surface-force diagnostics.
type Materialized<const D: usize, R> = (
    Manifold<LatticeComplex<D, R>, R>,
    Option<CutCellRegistry<D, R>>,
);

/// An owned mesh specification — lattice shape, per-axis periodicity, and uniform
/// spacing. It carries no borrow; `materialize` builds the manifold inside `run`.
///
/// Convenience constructors cover the example geometries: `periodic_cube` (fully
/// periodic, Taylor–Green), `box_domain` (all walls, the lid cavity / open cylinder),
/// and `channel` (periodic streamwise, walls cross-stream). Graded metrics and
/// immersed cut-cell bodies are layered on in later increments.
#[derive(Debug, Clone)]
pub struct Mesh<const D: usize, R: CfdScalar> {
    shape: [usize; D],
    periodic: [bool; D],
    spacing: R,
    body: Option<Body<D, R>>,
    grading: Option<Grading<R>>,
}

/// A smooth metric grading on one axis: a `PerEdge` `CubicalReggeGeometry` whose edge lengths vary
/// along `axis`, leaving `d`, the discrete Stokes theorem, and divergence-freeness exact (they are
/// combinatorial) while resolving walls cheaply. The structure is unchanged; only accuracy order is
/// at stake.
#[derive(Debug, Clone, Copy)]
pub enum Grading<R: CfdScalar> {
    /// Periodic cosine modulation `ℓ(pos) = h·(1 + amp·cos(2π·pos/N))` on `axis` (smooth across the
    /// seam, sums to `N·h`, so the wavenumber is unchanged).
    Cosine { axis: usize, amp: R },
}

impl<R: CfdScalar> Grading<R> {
    /// A periodic cosine grading on `axis` with relative amplitude `amp`.
    pub fn cosine(axis: usize, amp: R) -> Self {
        Grading::Cosine { axis, amp }
    }
}

impl<const D: usize, R: CfdScalar> Mesh<D, R> {
    /// A mesh with explicit shape and per-axis periodicity, unit spacing.
    pub fn new(shape: [usize; D], periodic: [bool; D]) -> Self {
        Self {
            shape,
            periodic,
            spacing: R::one(),
            body: None,
            grading: None,
        }
    }

    /// A fully periodic `n^D` cube (the Taylor–Green torus).
    pub fn periodic_cube(n: usize) -> Self {
        Self::new([n; D], [true; D])
    }

    /// A fully periodic `n^D` torus (alias of [`Mesh::periodic_cube`], for grading studies).
    pub fn torus(n: usize) -> Self {
        Self::periodic_cube(n)
    }

    /// Apply a metric [`Grading`] (a `PerEdge` geometry that varies edge lengths along one axis).
    pub fn graded(mut self, grading: Grading<R>) -> Self {
        self.grading = Some(grading);
        self
    }

    /// An all-wall box of the given shape (lid cavity, open cylinder domain).
    pub fn box_domain(shape: [usize; D]) -> Self {
        Self::new(shape, [false; D])
    }

    /// A channel: periodic on the streamwise axis (0), walls cross-stream.
    pub fn channel(shape: [usize; D]) -> Self {
        let mut periodic = [false; D];
        if D > 0 {
            periodic[0] = true;
        }
        Self::new(shape, periodic)
    }

    /// Set the uniform edge spacing (default unit).
    pub fn spacing(mut self, h: R) -> Self {
        self.spacing = h;
        self
    }

    /// Attach an immersed cut-cell body (e.g. a cylinder).
    pub fn immersed(mut self, body: Body<D, R>) -> Self {
        self.body = Some(body);
        self
    }

    /// Materialize the metric-bearing lattice manifold. Called inside `run`; the
    /// returned manifold is a local that the marcher borrows for the run's duration.
    /// The cut-cell registry (when an immersed body is present) is returned alongside
    /// for the surface-force diagnostics — a `None` for a body-free domain.
    pub(crate) fn materialize(&self) -> Result<Materialized<D, R>, PhysicsError> {
        let lattice = LatticeComplex::<D, R>::new(self.shape, self.periodic);
        let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
        let data = CausalTensor::new(vec![R::zero(); total], vec![total])
            .map_err(|e| PhysicsError::DimensionMismatch(format!("mesh data tensor: {e}")))?;
        let base = self.base_geometry(&lattice);
        let (metric, registry) = match &self.body {
            Some(body) => {
                let primitive = Primitive::<D, R>::ball(body.center(), body.radius());
                let registry = CutCellRegistry::from_primitive(&lattice, &base, &primitive)
                    .map_err(|e| PhysicsError::TopologyError(format!("cut-cell registry: {e}")))?
                    .with_cell_merging(body.merge_floor_value());
                // The metric folds in its own copy; the registry copy stays for the
                // read-only surface-force diagnostics (no per-step cost).
                (base.with_cut_cells(registry.clone()), Some(registry))
            }
            None => (base, None),
        };
        Ok((
            Manifold::from_cubical_with_metric(lattice, data, metric, 0),
            registry,
        ))
    }

    /// Materialize just the geometry — the metric-bearing manifold the caller owns (B1) for a
    /// geometry-only study (e.g. a DEC operator-accuracy sweep) that does not march. Equivalent to
    /// [`MarchConfig::materialize`](crate::MarchConfig::materialize) without a solver case.
    ///
    /// # Errors
    /// Any failure building the lattice, metric, or cut-cell geometry.
    pub fn manifold(&self) -> Result<Manifold<LatticeComplex<D, R>, R>, PhysicsError> {
        self.materialize().map(|(manifold, _registry)| manifold)
    }

    /// The base Regge geometry: a uniform metric, or a `PerEdge` graded metric when a [`Grading`]
    /// is set. Built from the lattice so the graded edge lengths follow `iter_cells(1)` order.
    fn base_geometry(&self, lattice: &LatticeComplex<D, R>) -> CubicalReggeGeometry<D, R> {
        match &self.grading {
            Some(Grading::Cosine {
                axis: graded_axis,
                amp,
            }) => {
                let two_pi = R::from_f64(2.0 * core::f64::consts::PI)
                    .expect("2π lifts into every real field");
                let n =
                    R::from_usize(self.shape[*graded_axis]).expect("a lattice extent lifts into R");
                let edge_lengths: Vec<R> = lattice
                    .iter_cells(1)
                    .map(|cell| {
                        let axis = cell.orientation().trailing_zeros() as usize;
                        if axis == *graded_axis {
                            let pos = R::from_usize(cell.position()[axis])
                                .expect("a lattice index lifts into R");
                            self.spacing * (R::one() + *amp * (two_pi * pos / n).cos())
                        } else {
                            self.spacing
                        }
                    })
                    .collect();
                CubicalReggeGeometry::from_edge_lengths(edge_lengths)
            }
            None => CubicalReggeGeometry::uniform(self.spacing),
        }
    }

    /// Rebuild the cut-cell registry alone (when an immersed body is present). The geometry the
    /// caller owns (B1) is borrowed read-only by the marcher and does not surrender its registry,
    /// so the surface-force diagnostics rebuild it from the body spec — deterministic and one-time.
    pub(crate) fn cut_registry(&self) -> Result<Option<CutCellRegistry<D, R>>, PhysicsError> {
        match &self.body {
            Some(body) => {
                let lattice = LatticeComplex::<D, R>::new(self.shape, self.periodic);
                let base = CubicalReggeGeometry::<D, R>::uniform(self.spacing);
                let primitive = Primitive::<D, R>::ball(body.center(), body.radius());
                let registry = CutCellRegistry::from_primitive(&lattice, &base, &primitive)
                    .map_err(|e| PhysicsError::TopologyError(format!("cut-cell registry: {e}")))?
                    .with_cell_merging(body.merge_floor_value());
                Ok(Some(registry))
            }
            None => Ok(None),
        }
    }

    /// The immersed body's reference frontal length (the diameter `2r`), used to
    /// nondimensionalize the surface force into a drag/lift coefficient. `None` for a
    /// body-free domain.
    pub(crate) fn frontal_length(&self) -> Option<R> {
        self.body.as_ref().map(|b| {
            let two = R::from_f64(2.0).expect("2.0 lifts into every real field");
            two * b.radius()
        })
    }
}
