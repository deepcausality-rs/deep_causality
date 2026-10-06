/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the closed-loop insulin-pump example.

#![allow(dead_code)] // Domain fields kept for narrative clarity even if not all are read.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NewtonianTime, NoSpace, NoSpaceTime, Temporal, TimeScale,
};
use deep_causality_core::{CausalityError, PropagatingProcess};
use deep_causality_num::Lift;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need lifting
/// through `deep_causality_num::lift` to switch away from `f64`.
pub type FloatType = f64;

/// One tick is `TICK_MINUTES` minutes. The 24-tick run covers six hours of
/// monitoring, long enough for both the hyperglycemic and the ketoacidotic
/// threshold to be crossed in the open-loop trajectory.
pub const N_TICKS: u32 = 24;

/// Tick duration in minutes: the time step of the glucose simulation.
pub const TICK_MINUTES: u32 = 15;

/// Deterministic perturbation over the tick starting at `tick`: the hepatic
/// glucose output over one tick plus the rise of every meal eaten within it.
/// Tick `k` covers the minutes `[k * TICK_MINUTES, (k + 1) * TICK_MINUTES)`.
/// The hepatic output and the meals are read from `pump`.
pub fn perturbation_at(tick: u32, pump: &PumpContext) -> Result<FloatType, CausalityError> {
    let start = (tick * TICK_MINUTES).lift::<FloatType>();
    let end = ((tick + 1) * TICK_MINUTES).lift::<FloatType>();
    let tick_hours = TICK_MINUTES.lift::<FloatType>() / 60.0;
    let baseline = read(pump, HEPATIC_GLUCOSE_OUTPUT)? * tick_hours;
    let mut meal: FloatType = 0.0;
    for (at, rise) in MEALS {
        if (start..end).contains(&read_minutes(pump, at)?) {
            meal += read(pump, rise)?;
        }
    }
    Ok(baseline + meal)
}

/// Accumulated trajectory and clinical statistics.
#[derive(Debug, Default, Clone)]
pub struct PatientState {
    pub tick: u32,
    pub trajectory: Vec<FloatType>,
    pub bolus_count: u32,
    pub total_insulin_units: FloatType,
    pub max_glucose_observed: FloatType,
    pub ketoacidosis_at: Option<u32>,
}

/// The thresholds, calibration, patient parameters and meals the steps read. Each quantity is a
/// `Data` contextoid; each meal time is a `NewtonianTime` contextoid, an instant of classical time
/// in minutes from the start of the monitoring window. The context holds no position or spacetime
/// event, so its spatial and spacetime slots are empty.
pub type PumpContext =
    Context<Data<FloatType>, NoSpace<FloatType>, NewtonianTime<FloatType>, NoSpaceTime<FloatType>>;

/// Node index: normal fasting glucose target (mg/dL).
pub const TARGET_GLUCOSE: usize = 0;
/// Node index: hyperglycemic alarm (mg/dL). The monitor fires a corrective bolus when glucose
/// climbs above this level.
pub const HYPERGLYCEMIC_THRESHOLD: usize = 1;
/// Node index: catastrophic threshold (mg/dL). Crossing this enters diabetic ketoacidosis
/// territory and is recorded against the trajectory.
pub const KETOACIDOSIS_THRESHOLD: usize = 2;
/// Node index: insulin sensitivity factor, mg/dL of glucose reduction per unit of fast-acting
/// insulin.
pub const INSULIN_SENSITIVITY: usize = 3;
/// Node index: hepatic glucose output, the rise in blood glucose without meals or insulin
/// (mg/dL per hour).
pub const HEPATIC_GLUCOSE_OUTPUT: usize = 4;
/// Node index: time of the small meal (`NewtonianTime`, minutes).
pub const SMALL_MEAL_AT: usize = 5;
/// Node index: glucose rise from the small meal (mg/dL).
pub const SMALL_MEAL_RISE: usize = 6;
/// Node index: time of the larger meal (`NewtonianTime`, minutes).
pub const LARGE_MEAL_AT: usize = 7;
/// Node index: glucose rise from the larger meal (mg/dL).
pub const LARGE_MEAL_RISE: usize = 8;
/// Node index: time of the snack (`NewtonianTime`, minutes).
pub const SNACK_AT: usize = 9;
/// Node index: glucose rise from the snack (mg/dL).
pub const SNACK_RISE: usize = 10;

/// The meals as (time node, rise node) pairs.
pub const MEALS: [(usize, usize); 3] = [
    (SMALL_MEAL_AT, SMALL_MEAL_RISE),
    (LARGE_MEAL_AT, LARGE_MEAL_RISE),
    (SNACK_AT, SNACK_RISE),
];

/// The nominal patient, pump and meal schedule, added in node-index order: node `i` holds
/// contextoid id `i + 1`. The three meals fall every 90 minutes.
pub fn nominal_pump_context() -> Result<PumpContext, ContextIndexError> {
    let data = |id, value| Contextoid::new(id, ContextoidType::Datoid(Data::new(id, value)));
    let minute = |id, at| {
        Contextoid::new(
            id,
            ContextoidType::Tempoid(NewtonianTime::new(id, TimeScale::Minute, at)),
        )
    };
    let mut context = Context::with_capacity(1, "pump", 11);
    for node in [
        data(1, 100.0),    // TARGET_GLUCOSE
        data(2, 180.0),    // HYPERGLYCEMIC_THRESHOLD
        data(3, 300.0),    // KETOACIDOSIS_THRESHOLD
        data(4, 50.0),     // INSULIN_SENSITIVITY
        data(5, 24.0),     // HEPATIC_GLUCOSE_OUTPUT: 6 mg/dL per 15-minute tick
        minute(6, 30.0),   // SMALL_MEAL_AT
        data(7, 70.0),     // SMALL_MEAL_RISE
        minute(8, 120.0),  // LARGE_MEAL_AT
        data(9, 90.0),     // LARGE_MEAL_RISE
        minute(10, 210.0), // SNACK_AT
        data(11, 60.0),    // SNACK_RISE
    ] {
        context.add_node(node)?;
    }
    Ok(context)
}

/// Read one `Data` contextoid's payload out of the pump context, or name the node it lacks.
pub fn read(context: &PumpContext, index: usize) -> Result<FloatType, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| {
            CausalityError::MissingParameter(format!("pump context Datoid at node {index}"))
        })
}

/// Read one meal time, in minutes from the start of the monitoring window, out of the pump
/// context, or name the node it lacks.
pub fn read_minutes(context: &PumpContext, index: usize) -> Result<FloatType, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().tempoid())
        .filter(|instant| instant.time_scale() == TimeScale::Minute)
        .map(|instant| instant.time_unit())
        .ok_or_else(|| {
            CausalityError::MissingParameter(format!(
                "pump context NewtonianTime in minutes at node {index}"
            ))
        })
}

pub type PumpProcess<T> = PropagatingProcess<T, PatientState, PumpContext>;
