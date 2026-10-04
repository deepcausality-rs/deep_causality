/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The per-step flight trace: a recorder stage that appends one row per coupled step to the
//! field scalar [`TRACE_FIELD`], and the typed rows a program writes that trace out as.
//!
//! The coupled field crosses every leg boundary and every fork, so the trace accumulates over the
//! whole descent: a pause's field holds every row flown up to it, and a branch report's
//! `"final_trace"` series holds the trunk's rows followed by the branch's own. The recorder only
//! reads the field and writes its own scalar, so composing it changes no other quantity.

use super::FloatType;
use super::constants::DT_FLIGHT;
use super::utils;
use deep_causality_cfd::{
    CoupledField, Coupling, GoverningModel, PhysicsError, PhysicsStage, StepContext, TableRow,
};
use deep_causality_num::{Lift, lift};

/// The field scalar the recorder appends to.
pub const TRACE_FIELD: &str = "trace";

/// Values the recorder appends per step, in [`TraceRow`] field order (excluding `leg` and `t`).
pub const TRACE_COLUMNS: usize = 16;

/// Positions of the recorded values within [`TraceRow::values`].
pub const ALTITUDE: usize = 0;
pub const MACH: usize = 1;
pub const SPEED: usize = 2;
pub const NE_PEAK: usize = 3;
pub const PLASMA_FREQ: usize = 4;
pub const GNSS_DENIED: usize = 5;
pub const NAV_ERR: usize = 8;
pub const HEAT_FLUX: usize = 10;

/// Appends one row of [`TRACE_COLUMNS`] values per coupled step to [`TRACE_FIELD`]. Composed last,
/// after the safety gate, so the recorded bank is the clamped command the lift stage flies next.
/// Fails the step when the field lacks any value the row records.
#[derive(Debug, Clone, Copy)]
pub struct TraceRecorder;

impl PhysicsStage<2, FloatType> for TraceRecorder {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        const STAGE: &str = "trace recorder";
        let regime = field.regime().ok_or_else(|| absent(STAGE, "regime"))?;
        let position: [FloatType; 3] = *field
            .scalar("truth_state")
            .and_then(|truth| truth.first_chunk::<3>())
            .ok_or_else(|| absent(STAGE, "truth_state"))?;
        let engine = field.nav().ok_or_else(|| absent(STAGE, "nav"))?;
        let p = engine.position();
        let d: [FloatType; 3] = core::array::from_fn(|i| p[i] - position[i]);
        let row: [FloatType; TRACE_COLUMNS] = [
            required(field, STAGE, "flight_altitude")?,
            required(field, STAGE, "flight_mach")?,
            required(field, STAGE, "flight_speed")?,
            field
                .scalar("n_e")
                .filter(|ne| !ne.is_empty())
                .map(utils::peak)
                .ok_or_else(|| absent(STAGE, "n_e"))?,
            regime.plasma_frequency,
            lift(if regime.gnss_denied { 1.0 } else { 0.0 }),
            regime.knudsen,
            model_code(regime.model),
            utils::norm3(d),
            engine.position_variance(),
            required(field, STAGE, "heat_flux")?,
            required(field, STAGE, "g_load")?,
            field
                .control_action()
                .ok_or_else(|| absent(STAGE, "control_action"))?,
            position[0],
            position[1],
            position[2],
        ];
        append_row(field, TRACE_FIELD, &row);
        Ok(())
    }
}

/// The error the recorder `stage` returns when the field lacks `name`.
fn absent(stage: &str, name: &str) -> PhysicsError {
    PhysicsError::CalculationError(format!("{stage}: the field carries no '{name}'"))
}

/// The first cell of the field scalar `name`; the recorder `stage` fails if the field has none.
fn required(
    field: &CoupledField<FloatType>,
    stage: &str,
    name: &str,
) -> Result<FloatType, PhysicsError> {
    field
        .scalar(name)
        .and_then(|s| s.first().copied())
        .ok_or_else(|| absent(stage, name))
}

/// Appends `row` to the field scalar `name`, creating it on the first step.
fn append_row(field: &mut CoupledField<FloatType>, name: &str, row: &[FloatType]) {
    match field.scalar_mut(name) {
        Some(trace) => trace.extend_from_slice(row),
        None => field.set_scalar(name, row.to_vec()),
    }
}

/// The governing model as a table code: 0 continuum, 1 slip, 2 transitional, 3 free-molecular.
fn model_code(model: GoverningModel) -> FloatType {
    lift(match model {
        GoverningModel::Continuum => 0.0,
        GoverningModel::Slip => 1.0,
        GoverningModel::Transitional => 2.0,
        GoverningModel::FreeMolecular => 3.0,
    })
}

/// `stage` followed by the [`TraceRecorder`].
pub fn traced<S: PhysicsStage<2, FloatType>>(stage: S) -> impl PhysicsStage<2, FloatType> {
    Coupling::between_steps()
        .then(stage)
        .then(TraceRecorder)
        .build()
}

/// The number of complete rows in a recorded trace.
pub fn row_count(trace: &[FloatType]) -> usize {
    trace.len() / TRACE_COLUMNS
}

/// The field scalar the burn recorder appends to.
pub const BURN_TRACE_FIELD: &str = "burn_trace";

/// Values the burn recorder appends per step.
pub const BURN_COLUMNS: usize = 7;

/// Positions of the recorded values within a burn-trace row.
pub const THROTTLE: usize = 0;
pub const PROPELLANT: usize = 1;
pub const DESCENT_RATE: usize = 2;
pub const AXIAL_ACCEL: usize = 3;
pub const PRESERVED_DRAG: usize = 4;
pub const DV_ACTUAL: usize = 5;
pub const DV_FROZEN: usize = 6;

/// Appends one row of [`BURN_COLUMNS`] powered-descent values per coupled step to
/// [`BURN_TRACE_FIELD`]: the throttle the propulsion stages flew, propellant, descent rate, axial
/// deceleration, the preserved-drag fraction, and the two velocity increments `AxialWitness`
/// accumulates. The preserved-drag fraction and the frozen-drag increment are NaN on any step the
/// field carries none; every other value missing from the field fails the step.
#[derive(Debug, Clone, Copy)]
pub struct BurnRecorder;

impl PhysicsStage<2, FloatType> for BurnRecorder {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        const STAGE: &str = "burn recorder";
        let row: [FloatType; BURN_COLUMNS] = [
            required(field, STAGE, "realized_throttle")?,
            required(field, STAGE, "propellant")?,
            required(field, STAGE, "descent_rate")?,
            required(field, STAGE, "axial_accel")?,
            field
                .scalar("preserved_drag_fraction")
                .and_then(|s| s.first().copied())
                .unwrap_or(FloatType::NAN),
            required(field, STAGE, "dv_actual")?,
            // Accumulated only in a world publishing the frozen-drag fraction: the fork's branches.
            field
                .scalar("dv_frozen")
                .and_then(|s| s.first().copied())
                .unwrap_or(FloatType::NAN),
        ];
        append_row(field, BURN_TRACE_FIELD, &row);
        Ok(())
    }
}

/// `stage` followed by the [`TraceRecorder`] and the [`BurnRecorder`].
pub fn burn_traced<S: PhysicsStage<2, FloatType>>(stage: S) -> impl PhysicsStage<2, FloatType> {
    Coupling::between_steps()
        .then(stage)
        .then(TraceRecorder)
        .then(BurnRecorder)
        .build()
}

/// The complete rows of a recorded burn trace.
pub fn burn_rows(trace: &[FloatType]) -> &[[FloatType; BURN_COLUMNS]] {
    trace.as_chunks::<BURN_COLUMNS>().0
}

/// One recorded step, labelled with the leg it was flown in and its flight time.
#[derive(Debug, Clone, Copy)]
pub struct TraceRow {
    /// The leg the step belongs to, counted from 1.
    pub leg: FloatType,
    /// Flight time at the end of the step, s: the row's index from the first recorded step plus
    /// one, times the coupled step.
    pub t: FloatType,
    pub values: [FloatType; TRACE_COLUMNS],
}

/// Column names and units of the recorded values, in [`TraceRow::values`] order.
pub const TRACE_SCHEMA: [(&str, &str); TRACE_COLUMNS] = [
    ("altitude", "m"),
    ("mach", "-"),
    ("speed", "m/s"),
    ("ne_peak", "m^-3"),
    ("plasma_freq", "rad/s"),
    ("gnss_denied", "-"),
    ("knudsen", "-"),
    (
        "regime",
        "0 continuum 1 slip 2 transitional 3 free-molecular",
    ),
    ("nav_err", "m"),
    ("nav_var", "m^2"),
    ("heat_flux", "W/m2"),
    ("g_load", "g"),
    ("bank", "rad"),
    ("x", "m"),
    ("y", "m"),
    ("z", "m"),
];

/// The schema of a table whose rows hold the `P` columns of `prefix` followed by the recorded
/// values. Evaluated in a constant, an `N` other than `P + TRACE_COLUMNS` fails to compile.
pub const fn trace_schema_after<const P: usize, const N: usize>(
    prefix: [(&'static str, &'static str); P],
) -> [(&'static str, &'static str); N] {
    assert!(N == P + TRACE_COLUMNS, "N must be P + TRACE_COLUMNS");
    let mut schema = [("", ""); N];
    let mut i = 0;
    while i < N {
        schema[i] = if i < P {
            prefix[i]
        } else {
            TRACE_SCHEMA[i - P]
        };
        i += 1;
    }
    schema
}

impl TableRow for TraceRow {
    type Scalar = FloatType;
    const SCHEMA: &'static [(&'static str, &'static str)] =
        &trace_schema_after::<2, { TRACE_COLUMNS + 2 }>([("leg", "-"), ("t", "s")]);
    fn cells(&self) -> Vec<FloatType> {
        let mut cells = Vec::with_capacity(Self::SCHEMA.len());
        cells.push(self.leg);
        cells.push(self.t);
        cells.extend_from_slice(&self.values);
        cells
    }
}

/// The complete rows of `trace` from index `from` on, labelled with their leg. `leg_ends` holds the
/// row count at the end of each leg in flight order; a row past the last end takes the last leg.
pub fn trace_rows(trace: &[FloatType], from: usize, leg_ends: &[usize]) -> Vec<TraceRow> {
    let (rows, _) = trace.as_chunks::<TRACE_COLUMNS>();
    rows.iter()
        .enumerate()
        .skip(from)
        .map(|(i, values)| {
            let leg = leg_ends
                .iter()
                .position(|&end| i < end)
                .unwrap_or(leg_ends.len().saturating_sub(1))
                + 1;
            TraceRow {
                leg: leg.lift(),
                t: (i + 1).lift::<FloatType>() * lift::<FloatType>(DT_FLIGHT),
                values: *values,
            }
        })
        .collect()
}
