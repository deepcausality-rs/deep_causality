/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::instrument::environment_reading::EnvironmentReading;
use crate::types::instrument::interferometer_configuration::InterferometerConfiguration;
use crate::types::instrument::interferometer_datum::InterferometerDatum;
use crate::types::instrument::interferometer_model::InterferometerModel;
use alloc::format;
use deep_causality_algebra::RealField;
use deep_causality_context::{
    Context, ContextId, ContextIndexError, Contextoid, ContextoidId, ContextoidType,
    ContextuableGraph, Data, DiscreteTime, NoSpace, NoSpaceTime, RelationKind, TimeScale,
};

/// An instrument's context over its payload `P`: data nodes keyed by contextoid id, discrete time
/// for when each reading was taken, and no spatial or spacetime node.
pub type InstrumentContext<P, R> = Context<Data<P>, NoSpace<R>, DiscreteTime, NoSpaceTime<R>>;

/// An atom interferometer's context.
pub type InterferometerContext<R> = InstrumentContext<InterferometerDatum<R>, R>;

/// The contextoid id of the instrument model.
pub const INSTRUMENT_MODEL: ContextoidId = 1;

/// The contextoid id of the configuration.
pub const INSTRUMENT_CONFIGURATION: ContextoidId = 2;

/// The contextoid id of the environment reading taken at `tick`, `3 + 2·tick`; `None` past
/// `u64::MAX`.
pub fn environment_id(tick: u64) -> Option<ContextoidId> {
    tick.checked_mul(2)?.checked_add(3)
}

/// The contextoid id of the `DiscreteTime` node of the reading taken at `tick`, the id after the
/// reading's; `None` past `u64::MAX`.
pub fn environment_time_id(tick: u64) -> Option<ContextoidId> {
    environment_id(tick)?.checked_add(1)
}

/// An atom interferometer's context named `name`, holding `model` under [`INSTRUMENT_MODEL`] and
/// `configuration` under [`INSTRUMENT_CONFIGURATION`], and no environment reading.
///
/// # Errors
///
/// [`QuantumError::CalculationError`] carrying the context's refusal of a node.
pub fn interferometer_context<R>(
    id: ContextId,
    name: &str,
    model: InterferometerModel<R>,
    configuration: InterferometerConfiguration<R>,
) -> Result<InterferometerContext<R>, QuantumError>
where
    R: RealField,
{
    let mut context = Context::with_capacity(id, name, 2);
    for (node, datum) in [
        (INSTRUMENT_MODEL, InterferometerDatum::Model(model)),
        (
            INSTRUMENT_CONFIGURATION,
            InterferometerDatum::Configuration(configuration),
        ),
    ] {
        let datoid = ContextoidType::Datoid(Data::new(node, datum));
        context
            .add_node(Contextoid::new(node, datoid))
            .map_err(refused)?;
    }
    Ok(context)
}

/// `reading` recorded as taken at `tick` on `scale`: the reading under [`environment_id`], a
/// `DiscreteTime` node under [`environment_time_id`], and a temporal edge from the reading to its
/// time.
///
/// # Errors
///
/// [`QuantumError::CalculationError`] when `tick` has no id below `u64::MAX`, and carrying the
/// context's refusal of a node or the edge: a tick already recorded, or a frozen context.
pub fn record_environment<R>(
    context: &mut InterferometerContext<R>,
    tick: u64,
    scale: TimeScale,
    reading: EnvironmentReading<R>,
) -> Result<(), QuantumError>
where
    R: RealField,
{
    let (Some(reading_id), Some(time_id)) = (environment_id(tick), environment_time_id(tick))
    else {
        return Err(QuantumError::CalculationError(format!(
            "an environment reading at tick {tick} has no contextoid id below u64::MAX"
        )));
    };
    let datoid = ContextoidType::Datoid(Data::new(
        reading_id,
        InterferometerDatum::Environment(reading),
    ));
    let reading_index = context
        .add_node(Contextoid::new(reading_id, datoid))
        .map_err(refused)?;
    let tempoid = ContextoidType::Tempoid(DiscreteTime::new(time_id, scale, tick));
    let time_index = context
        .add_node(Contextoid::new(time_id, tempoid))
        .map_err(refused)?;
    context
        .add_edge(reading_index, time_index, RelationKind::Temporal)
        .map_err(refused)
}

/// The instrument model under [`INSTRUMENT_MODEL`].
///
/// # Errors
///
/// [`QuantumError::CalculationError`] when no model is held there.
pub fn instrument_model<R>(
    context: &InterferometerContext<R>,
) -> Result<InterferometerModel<R>, QuantumError>
where
    R: RealField,
{
    context
        .get_data_by_id(INSTRUMENT_MODEL)
        .and_then(|datum| datum.model().copied())
        .ok_or_else(|| missing("model", INSTRUMENT_MODEL))
}

/// The configuration under [`INSTRUMENT_CONFIGURATION`].
///
/// # Errors
///
/// [`QuantumError::CalculationError`] when no configuration is held there.
pub fn instrument_configuration<R>(
    context: &InterferometerContext<R>,
) -> Result<InterferometerConfiguration<R>, QuantumError>
where
    R: RealField,
{
    context
        .get_data_by_id(INSTRUMENT_CONFIGURATION)
        .and_then(|datum| datum.configuration().copied())
        .ok_or_else(|| missing("configuration", INSTRUMENT_CONFIGURATION))
}

/// The environment reading taken at `tick`, or `None` when none was recorded there.
pub fn environment_at<R>(
    context: &InterferometerContext<R>,
    tick: u64,
) -> Option<EnvironmentReading<R>>
where
    R: RealField,
{
    environment_id(tick)
        .and_then(|id| context.get_data_by_id(id))
        .and_then(|datum| datum.environment().copied())
}

fn refused(error: ContextIndexError) -> QuantumError {
    QuantumError::CalculationError(format!("the instrument context refused a node: {error}"))
}

fn missing(what: &str, id: ContextoidId) -> QuantumError {
    QuantumError::CalculationError(format!(
        "the instrument context holds no {what} under contextoid id {id}"
    ))
}
