/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The [`Marchable`] trait: a configuration that knows how to march itself to a
//! [`Report`](crate::Report) in one shot. It is the seam the campaign level's `march` verb
//! lowers onto — one case, one report — so a study never names a family-specific pipeline type.
//!
//! The three **uncoupled** config families implement `Marchable` directly here: their report
//! comes from a self-contained run (`DuctConfig`/`QttMarchConfig` from `run`, `MarchConfig` from
//! `run_owned`, which materializes a fresh geometry per case). The two **coupled** families take
//! their coupling stack as a run-time argument, absent from the config; they march through the
//! `Coupled` wrapper (which carries the stack), so `.couple(stack).march()` composes onto this
//! same trait. See `openspec/notes/cfd-dsl/04-dsl-feasibility.md` (F2).

use crate::solvers::dec::BoundaryZone;
use crate::types::CfdScalar;
use crate::types::flow::{DuctMarchRun, MarchPipeline, PhysicsStage, QttMarchRun, Report};
use crate::types::flow_config::{DuctConfig, MarchConfig, QttMarchConfig};
use deep_causality_num::ConjugateScalar;
use deep_causality_physics::PhysicsError;

/// A configuration that marches itself to a [`Report`] in one shot.
///
/// This is the campaign's one-case-one-report seam. Implemented directly by the uncoupled config
/// families; the coupled families reach it through the `Coupled` wrapper. Example-local case
/// types (a wake case carrying its own `dt`, say) implement it by delegating to their inner
/// config.
pub trait Marchable<R: CfdScalar> {
    /// Materialize (where the family needs a fresh geometry), run, and return the owned report.
    fn march(&self) -> Result<Report<R>, PhysicsError>;
}

/// The quasi-1-D duct march: a self-contained run, no geometry stage.
impl<R: CfdScalar> Marchable<R> for DuctConfig<R> {
    fn march(&self) -> Result<Report<R>, PhysicsError> {
        DuctMarchRun::new(self).run()
    }
}

/// The 2-D incompressible DEC march: materialize a fresh geometry per case, then run.
impl<const D: usize, R, Z, C> Marchable<R> for MarchConfig<D, R, Z, C>
where
    R: CfdScalar,
    Z: BoundaryZone<D, R> + Clone,
    C: PhysicsStage<D, R>,
{
    fn march(&self) -> Result<Report<R>, PhysicsError> {
        MarchPipeline::new(self).run_owned()
    }
}

/// The QTT 2-D incompressible march: a self-contained run, the tensor-train sibling of the DEC
/// march (no borrowed manifold).
impl<R> Marchable<R> for QttMarchConfig<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn march(&self) -> Result<Report<R>, PhysicsError> {
        QttMarchRun::new(self).run()
    }
}
