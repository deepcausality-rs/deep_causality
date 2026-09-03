/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Sensor traces: time-stamped, per-channel samples where absence is honest.
//!
//! A trace file is a CSV table whose first column is the timestamp and whose remaining
//! columns are channels. An **empty cell is a missing sample** and loads as `None`; no
//! sentinel value (zero, NaN, or otherwise) ever stands in for a measurement that was not
//! taken. The consumer lifts presence into `MaybeUncertain` and noise into `Uncertain`;
//! this crate stays free of that dependency by design.

/// One channel of a trace: its name, unit, and one optional sample per timestamp row.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorChannel<R> {
    name: String,
    unit: String,
    samples: Vec<Option<R>>,
}

impl<R> SensorChannel<R> {
    pub(crate) fn new(name: String, unit: String, samples: Vec<Option<R>>) -> Self {
        Self {
            name,
            unit,
            samples,
        }
    }

    /// The channel name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The channel unit; empty when the file declared none.
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// One entry per timestamp row; `None` exactly where the file has no sample.
    pub fn samples(&self) -> &[Option<R>] {
        &self.samples
    }
}

/// A loaded trace set: shared timestamps (seconds, exact `f64` as written in the file) and
/// the channels aligned to them.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorTraceSet<R> {
    timestamps: Vec<f64>,
    channels: Vec<SensorChannel<R>>,
}

impl<R> SensorTraceSet<R> {
    pub(crate) fn new(timestamps: Vec<f64>, channels: Vec<SensorChannel<R>>) -> Self {
        Self {
            timestamps,
            channels,
        }
    }

    /// The shared timestamp axis, in file order.
    pub fn timestamps(&self) -> &[f64] {
        &self.timestamps
    }

    /// The channels, each aligned to [`timestamps`](Self::timestamps).
    pub fn channels(&self) -> &[SensorChannel<R>] {
        &self.channels
    }

    /// A channel by name, when present.
    pub fn channel(&self, name: &str) -> Option<&SensorChannel<R>> {
        self.channels.iter().find(|c| c.name() == name)
    }
}
