/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::instrument::effective_draws::EffectiveDraws;
use crate::types::qcm::hypothesis::Hypothesis;
use crate::types::qpu::histogram::CountHistogram;
use crate::types::qpu::shot_estimate::ShotEstimate;
use alloc::format;
use alloc::string::String;
use deep_causality_algebra::RealField;
use deep_causality_context::{Context, Datable, SpaceTemporal, Spatial, Temporal};
use deep_causality_context_store::{
    ContextSnapshot, DataRecord, Recordable, SpaceRecord, SpaceTimeRecord, TimeRecord,
};
use deep_causality_num::FromPrimitive;

/// Where an observed experiment's evidence comes from.
#[derive(Debug, Clone, PartialEq)]
pub enum EvidenceSource<R: RealField> {
    /// Draws from the Born sampler at the read-out `truth` predicts for the experiment: the
    /// generating cause, in simulation. `truth` need not be among the candidates.
    Simulated(Hypothesis<R>),
    /// Counts measured on the device, outcome `1` accepting.
    Recorded(CountHistogram),
    /// A published value with its standard error, as effective draws on the instrument's fringe.
    Published(EffectiveDraws<R>),
}

/// One observed experiment: its name, the read-out with the shots behind it, the counts when the
/// evidence came as counts, and the context the configuration stood in at observation time.
#[derive(Debug, Clone, PartialEq)]
pub struct Observation<R> {
    experiment: String,
    read_out: ShotEstimate<R>,
    counts: Option<CountHistogram>,
    context: Option<ContextSnapshot>,
}

impl<R: RealField + FromPrimitive> Observation<R> {
    pub(crate) fn new(
        experiment: String,
        read_out: ShotEstimate<R>,
        counts: Option<CountHistogram>,
        context: Option<ContextSnapshot>,
    ) -> Self {
        Self {
            experiment,
            read_out,
            counts,
            context,
        }
    }

    /// The experiment observed.
    pub fn experiment(&self) -> &str {
        &self.experiment
    }

    /// The read-out.
    pub fn read_out(&self) -> &ShotEstimate<R> {
        &self.read_out
    }

    /// The shots behind the read-out, the effective draws rounded for a published value.
    pub fn shots(&self) -> u64 {
        self.read_out.shots()
    }

    /// The counts, for simulated and recorded evidence.
    pub fn counts(&self) -> Option<&CountHistogram> {
        self.counts.as_ref()
    }

    /// The configuration's context at observation time, when the configuration is a context.
    pub fn context(&self) -> Option<&ContextSnapshot> {
        self.context.as_ref()
    }
}

/// What an observation records of the configuration it ran under.
pub trait ObservedContext {
    /// The configuration as a context snapshot, or `None` when it is not a context.
    ///
    /// # Errors
    ///
    /// The context's refusal to project onto a snapshot.
    fn context_snapshot(&self) -> Result<Option<ContextSnapshot>, QuantumError>;
}

/// A context records itself.
impl<D, S, T, ST> ObservedContext for Context<D, S, T, ST>
where
    D: Datable + Clone + Recordable<DataRecord>,
    S: Spatial + Clone + Recordable<SpaceRecord>,
    T: Temporal + Clone + Recordable<TimeRecord>,
    ST: SpaceTemporal + Clone + Recordable<SpaceTimeRecord>,
{
    fn context_snapshot(&self) -> Result<Option<ContextSnapshot>, QuantumError> {
        self.snapshot().map(Some).map_err(|e| {
            QuantumError::CalculationError(format!("the configuration's context: {e}"))
        })
    }
}
