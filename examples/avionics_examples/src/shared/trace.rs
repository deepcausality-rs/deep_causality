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
pub const PLASMA_FREQ: usize = 4;
pub const GNSS_DENIED: usize = 5;
pub const NAV_ERR: usize = 8;

/// Appends one row of [`TRACE_COLUMNS`] values per coupled step to [`TRACE_FIELD`]. Composed last,
/// after the safety gate, so the recorded bank is the clamped command the lift stage flies next.
#[derive(Debug, Clone, Copy)]
pub struct TraceRecorder;

impl PhysicsStage<2, FloatType> for TraceRecorder {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, FloatType>,
        field: &mut CoupledField<FloatType>,
    ) -> Result<(), PhysicsError> {
        let zero = lift::<FloatType>(0.0);
        let regime = field.regime();
        let truth = field.scalar("truth_state").unwrap_or(&[]);
        let position: [FloatType; 3] =
            core::array::from_fn(|i| truth.get(i).copied().unwrap_or(zero));
        let (nav_err, nav_var) = match field.nav() {
            Some(engine) if truth.len() >= 3 => {
                let p = engine.position();
                let d: [FloatType; 3] = core::array::from_fn(|i| p[i] - position[i]);
                (utils::norm3(d), engine.position_variance())
            }
            _ => (zero, zero),
        };
        let row: [FloatType; TRACE_COLUMNS] = [
            utils::scalar0(field, "flight_altitude"),
            utils::scalar0(field, "flight_mach"),
            utils::scalar0(field, "flight_speed"),
            field.scalar("n_e").map(utils::peak).unwrap_or(zero),
            regime.map(|r| r.plasma_frequency).unwrap_or(zero),
            lift(if regime.is_some_and(|r| r.gnss_denied) {
                1.0
            } else {
                0.0
            }),
            regime.map(|r| r.knudsen).unwrap_or(zero),
            regime.map(|r| model_code(r.model)).unwrap_or(zero),
            nav_err,
            nav_var,
            utils::scalar0(field, "heat_flux"),
            utils::scalar0(field, "g_load"),
            field.control_action().unwrap_or(zero),
            position[0],
            position[1],
            position[2],
        ];
        match field.scalar_mut(TRACE_FIELD) {
            Some(trace) => trace.extend_from_slice(&row),
            None => field.set_scalar(TRACE_FIELD, row.to_vec()),
        }
        Ok(())
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

impl TableRow for TraceRow {
    type Scalar = FloatType;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[
        ("leg", "-"),
        ("t", "s"),
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
    fn cells(&self) -> Vec<FloatType> {
        let mut cells = Vec::with_capacity(Self::SCHEMA.len());
        cells.push(self.leg);
        cells.push(self.t);
        cells.extend_from_slice(&self.values);
        cells
    }
}

/// The rows of `trace` from index `from` on, labelled with their leg. `leg_ends` holds the row
/// count at the end of each leg in flight order; a row past the last end takes the last leg.
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
