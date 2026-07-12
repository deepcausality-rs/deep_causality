/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration for the **uncertain-inflow march** (the sensor-fed causal-monad march): an owned
//! bundle of the solver config, the [`UncertainInflowZone`], the per-step `MaybeUncertain` sensor
//! stream, the seed, and the horizon. The geometry is **not** held here — the caller owns it and
//! lends it at run time via `.on(&manifold)` (B1), exactly like [`MarchConfig`](crate::MarchConfig).
//! Run by [`CfdFlow::march`](crate::CfdFlow).

use crate::CfdScalar;
use crate::solvers::DecNsConfig;
use crate::solvers::dec::uncertain_inflow::UncertainInflowZone;
use crate::types::flow_config::Seed;
use deep_causality_physics::PhysicsError;
use deep_causality_uncertain::{MaybeUncertain, ProbabilisticType};

/// An owned uncertain-inflow march configuration. The dimension is fixed by the geometry at
/// `.on(&manifold)`, not here.
pub struct UncertainMarchConfig<R: CfdScalar + ProbabilisticType> {
    pub(crate) name: String,
    pub(crate) solver: DecNsConfig<R>,
    pub(crate) zone: UncertainInflowZone<R>,
    pub(crate) stream: Vec<MaybeUncertain<R>>,
    pub(crate) steps: usize,
    pub(crate) seed: Seed,
}

/// Fluent builder for an [`UncertainMarchConfig`]. Required: `solver`, `inflow_zone`,
/// `sensor_stream`, `march_for`. The seed defaults to [`Seed::Rest`].
pub struct UncertainMarchConfigBuilder<R: CfdScalar + ProbabilisticType> {
    name: String,
    solver: Option<DecNsConfig<R>>,
    zone: Option<UncertainInflowZone<R>>,
    stream: Option<Vec<MaybeUncertain<R>>>,
    steps: Option<usize>,
    seed: Seed,
}

impl<R: CfdScalar + ProbabilisticType> UncertainMarchConfigBuilder<R> {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            solver: None,
            zone: None,
            stream: None,
            steps: None,
            seed: Seed::Rest,
        }
    }

    /// The DEC incompressible solver configuration (built via [`CfdConfigBuilder::dec_ns`](crate::CfdConfigBuilder)).
    pub fn solver(mut self, config: DecNsConfig<R>) -> Self {
        self.solver = Some(config);
        self
    }

    /// The sensor-fed inflow boundary patch (presence gate, collapse, dropout policy).
    pub fn inflow_zone(mut self, zone: UncertainInflowZone<R>) -> Self {
        self.zone = Some(zone);
        self
    }

    /// The per-step sensor stream: `stream[i]` feeds march step `i`. Must cover the horizon.
    pub fn sensor_stream(mut self, stream: Vec<MaybeUncertain<R>>) -> Self {
        self.stream = Some(stream);
        self
    }

    /// The initial condition (default [`Seed::Rest`]).
    pub fn seed(mut self, seed: Seed) -> Self {
        self.seed = seed;
        self
    }

    /// March the given number of steps.
    pub fn march_for(mut self, steps: usize) -> Self {
        self.steps = Some(steps);
        self
    }

    /// Finalize.
    ///
    /// # Errors
    /// `PhysicsError::DimensionMismatch` when a required field is missing or the sensor stream is
    /// shorter than the horizon.
    pub fn build(self) -> Result<UncertainMarchConfig<R>, PhysicsError> {
        let solver = self.solver.ok_or_else(|| {
            PhysicsError::DimensionMismatch("uncertain_march: solver config is required".into())
        })?;
        let zone = self.zone.ok_or_else(|| {
            PhysicsError::DimensionMismatch("uncertain_march: inflow_zone is required".into())
        })?;
        let stream = self.stream.ok_or_else(|| {
            PhysicsError::DimensionMismatch("uncertain_march: sensor_stream is required".into())
        })?;
        let steps = self.steps.ok_or_else(|| {
            PhysicsError::DimensionMismatch("uncertain_march: march_for is required".into())
        })?;
        if stream.len() < steps {
            return Err(PhysicsError::DimensionMismatch(format!(
                "uncertain_march: sensor stream has {} samples but the horizon is {steps} steps",
                stream.len()
            )));
        }
        Ok(UncertainMarchConfig {
            name: self.name,
            solver,
            zone,
            stream,
            steps,
            seed: self.seed,
        })
    }
}
