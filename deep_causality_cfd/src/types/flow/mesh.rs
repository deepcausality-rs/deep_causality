/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CfdScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

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
}

impl<const D: usize, R: CfdScalar> Mesh<D, R> {
    /// A mesh with explicit shape and per-axis periodicity, unit spacing.
    pub fn new(shape: [usize; D], periodic: [bool; D]) -> Self {
        Self {
            shape,
            periodic,
            spacing: R::one(),
        }
    }

    /// A fully periodic `n^D` cube (the Taylor–Green torus).
    pub fn periodic_cube(n: usize) -> Self {
        Self::new([n; D], [true; D])
    }

    /// An all-wall box of the given shape (lid cavity, open cylinder domain).
    pub fn box_domain(shape: [usize; D]) -> Self {
        Self::new(shape, [false; D])
    }

    /// Set the uniform edge spacing (default unit).
    pub fn spacing(mut self, h: R) -> Self {
        self.spacing = h;
        self
    }

    /// Materialize the metric-bearing lattice manifold. Called inside `run`; the
    /// returned manifold is a local that the marcher borrows for the run's duration.
    pub(crate) fn materialize(&self) -> Result<Manifold<LatticeComplex<D, R>, R>, PhysicsError> {
        let lattice = LatticeComplex::<D, R>::new(self.shape, self.periodic);
        let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
        let data = CausalTensor::new(vec![R::zero(); total], vec![total])
            .map_err(|e| PhysicsError::DimensionMismatch(format!("mesh data tensor: {e}")))?;
        let metric = CubicalReggeGeometry::<D, R>::uniform(self.spacing);
        Ok(Manifold::from_cubical_with_metric(lattice, data, metric, 0))
    }
}
