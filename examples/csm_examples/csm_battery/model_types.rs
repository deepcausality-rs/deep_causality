/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! What the controller reads each minute, and what a charge session ends with.

use crate::FloatType;

/// One minute of sensor data: the cell temperature now and a minute ago, in °C, and the current
/// that flowed in between, in A.
#[derive(Debug, Clone, Copy, Default)]
pub struct Reading {
    pub cell_c: FloatType,
    pub previous_c: FloatType,
    pub amps: FloatType,
}

/// How a charge session ended: its length, the charge reached, the hottest the cell got, and how
/// many minutes it charged above the ageing threshold.
#[derive(Debug, Clone, Copy)]
pub struct Summary {
    pub minutes: usize,
    pub charge_pct: FloatType,
    pub peak_c: FloatType,
    pub minutes_hot: usize,
}
