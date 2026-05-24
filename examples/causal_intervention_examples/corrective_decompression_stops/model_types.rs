/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types for the corrective-decompression-stops example.

#![allow(dead_code)] // Domain fields kept for narrative clarity even if not all are read.

use deep_causality_core::PropagatingProcess;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this crate would need wrapping
/// in `FloatType::from(…)` to switch away from `f64`.
pub type FloatType = f64;

/// One tick is 0.5 minutes (30 seconds) of dive time.
/// `N_TICKS = 30` covers 15 minutes, long enough for the open-loop run to
/// surface in five minutes and for the closed loop to insert several
/// decompression stops before finishing.
pub const N_TICKS: u32 = 30;

/// Inspired N2 partial pressure (bar) at depth `d` (m). Atmospheric N2
/// fraction is 79%; absolute pressure climbs by 1 bar per 10 m of depth.
pub fn inspired_n2_pp(depth: FloatType) -> FloatType {
    let ambient = 1.0 + depth / 10.0;
    0.79 * ambient
}

pub fn ambient_pressure(depth: FloatType) -> FloatType {
    1.0 + depth / 10.0
}

/// Single-compartment dive state. The original `diving_decompression`
/// example tracks 16 compartments per Bühlmann ZH-L16C; this retrofit
/// uses one mid-range compartment because the intervention pattern is
/// the same regardless of how many compartments you carry.
#[derive(Debug, Default, Clone)]
pub struct DiveState {
    pub tick: u32,
    pub depth_m: FloatType,
    pub tissue_n2_bar: FloatType,
    pub depth_trajectory: Vec<FloatType>,
    pub tissue_trajectory: Vec<FloatType>,
    pub ratio_trajectory: Vec<FloatType>,
    pub last_ratio: FloatType,
    pub stop_count: u32,
    pub max_ratio_observed: FloatType,
    pub dcs_at: Option<u32>,
}

/// Read-only dive plan and decompression thresholds.
#[derive(Debug, Clone)]
pub struct DiveConfig {
    /// Starting depth at the bottom phase (m).
    pub starting_depth_m: FloatType,
    /// Tissue N2 partial pressure at the bottom, fully saturated to depth.
    pub starting_tissue_n2_bar: FloatType,
    /// Metres of ascent per tick under a continuous-ascent plan.
    pub normal_ascent_m_per_tick: FloatType,
    /// Tick duration in minutes.
    pub tick_minutes: FloatType,
    /// Tissue half-time in minutes. The compartment here is a mid-range
    /// 10-minute half-time, comparable to Bühlmann compartment 2.
    pub half_time_min: FloatType,
    /// Supersaturation ratio at which DCS risk is assumed certain.
    pub dcs_ratio_threshold: FloatType,
    /// Monitor threshold. The closed loop fires a corrective stop the
    /// moment the post-tick ratio crosses this.
    pub safety_ratio_threshold: FloatType,
}

pub fn nominal_dive_config() -> DiveConfig {
    DiveConfig {
        starting_depth_m: 30.0,
        starting_tissue_n2_bar: inspired_n2_pp(30.0),
        normal_ascent_m_per_tick: 3.0,
        tick_minutes: 0.5,
        half_time_min: 10.0,
        dcs_ratio_threshold: 1.6,
        // A single ascent tick can swing the ratio by roughly +0.25 at
        // this physics. The safety threshold is set well below the DCS
        // line so a stop fires before the next ascent could overshoot.
        safety_ratio_threshold: 1.15,
    }
}

pub type DiveProcess<T> = PropagatingProcess<T, DiveState, DiveConfig>;
