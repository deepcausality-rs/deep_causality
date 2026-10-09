/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The instrument context: what an instrument is, how it is set, and what its surroundings read,
//! held in a `deep_causality_context::Context` and keyed by contextoid id.
//!
//! [`InstrumentContext`] is generic over the payload, so every instrument, an atom interferometer
//! or a QPU, stores its own payload type in the same alias. The atom interferometer's payload is
//! [`InterferometerDatum`]: its [`InterferometerModel`] and [`InterferometerConfiguration`] under
//! fixed ids, and each [`EnvironmentReading`] under an id derived from the tick it was taken at,
//! beside a `DiscreteTime` node for that tick. Every payload type is `Storable`, so the context
//! round-trips through `snapshot` and `restore`.
//!
//! An interferometer's [`Fringe`] carries a published value with its standard error onto the
//! read-out as [`EffectiveDraws`], the form the pipeline's published evidence takes.
//!
//! The module is behind the `qcm` feature, with the pipeline that reads it.

pub(crate) mod effective_draws;
pub(crate) mod environment_reading;
pub(crate) mod instrument_context;
pub(crate) mod interferometer_configuration;
pub(crate) mod interferometer_datum;
pub(crate) mod interferometer_model;
pub(crate) mod record_fields;
pub(crate) mod wave_vector;

pub use effective_draws::*;
pub use environment_reading::*;
pub use instrument_context::*;
pub use interferometer_configuration::*;
pub use interferometer_datum::*;
pub use interferometer_model::*;
pub use wave_vector::*;
