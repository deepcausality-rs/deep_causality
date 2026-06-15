/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CfdScalar;

/// The set of diagnostics a march collects into its `Report`. Built fluently;
/// the scalar diagnostics (`kinetic_energy`, `divergence`, `max_speed`) sample one
/// value per step, while the immersed-body diagnostics (`drag`/`lift`, the wake
/// `probe` for Strouhal, and the final-state `centerline` profile) require an
/// immersed body / a chosen sample geometry and are opt-in by reference speed or point.
#[derive(Debug, Clone, Copy)]
pub struct Observe<const D: usize, R: CfdScalar> {
    pub(crate) kinetic_energy: bool,
    pub(crate) divergence: bool,
    pub(crate) max_speed: bool,
    /// `Some(u_ref)` enables the drag/lift coefficient series on the immersed body,
    /// nondimensionalized by the reference speed `u_ref` and the body's frontal length.
    pub(crate) drag: Option<R>,
    /// `Some(point)` enables a transverse-velocity time series sampled at the wake
    /// `point` (lattice/spacing coordinates) — the raw signal a Strouhal number is read from.
    pub(crate) probe: Option<[R; D]>,
    /// `Some(axis)` records, at the final state, the velocity profile along the domain-
    /// centered line parallel to `axis` (the Ghia centerline comparison).
    pub(crate) centerline: Option<usize>,
}

impl<const D: usize, R: CfdScalar> Default for Observe<D, R> {
    fn default() -> Self {
        Self {
            kinetic_energy: false,
            divergence: false,
            max_speed: false,
            drag: None,
            probe: None,
            centerline: None,
        }
    }
}

impl<const D: usize, R: CfdScalar> Observe<D, R> {
    /// Collect the kinetic-energy series (one sample per step, plus the seed).
    pub fn kinetic_energy(mut self) -> Self {
        self.kinetic_energy = true;
        self
    }

    /// Collect the divergence-residual series (the solver's incompressibility error).
    pub fn divergence(mut self) -> Self {
        self.divergence = true;
        self
    }

    /// Collect the maximum-speed series.
    pub fn max_speed(mut self) -> Self {
        self.max_speed = true;
        self
    }

    /// Collect the drag and lift coefficient series on the immersed body, with `u_ref`
    /// the free-stream reference speed. Requires the mesh to carry an immersed body;
    /// `run` errors otherwise.
    pub fn drag(mut self, u_ref: R) -> Self {
        self.drag = Some(u_ref);
        self
    }

    /// Collect the transverse-velocity time series at the wake `point` (in spacing
    /// units) — the signal whose dominant frequency gives the Strouhal number.
    pub fn probe(mut self, point: [R; D]) -> Self {
        self.probe = Some(point);
        self
    }

    /// Record the final-state velocity profile along the domain-centered line parallel
    /// to `axis` (the Ghia lid-cavity centerline comparison).
    pub fn centerline(mut self, axis: usize) -> Self {
        self.centerline = Some(axis);
        self
    }
}
