/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod mission_getters;

use crate::constants::*;
use crate::{FaultTimeline, FloatType};

/// One flight's conditions: the line's distance across the slope, the local hour at launch, the
/// wind, the gust at touchdown, when the faults strike, and the seed of the sensor noise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mission {
    line_across_m: FloatType,
    start_hour: FloatType,
    wind_m_s: (FloatType, FloatType),
    touchdown_gust_m: (FloatType, FloatType),
    faults: FaultTimeline,
    seed: u64,
}

impl Mission {
    /// `wind_m_s` is the wind across and along the valley; `touchdown_gust_m` is how far a gust
    /// moves the drone, across and along, in its last second before touchdown.
    pub fn new(
        line_across_m: FloatType,
        start_hour: FloatType,
        wind_m_s: (FloatType, FloatType),
        touchdown_gust_m: (FloatType, FloatType),
        faults: FaultTimeline,
        seed: u64,
    ) -> Self {
        Self {
            line_across_m,
            start_hour,
            wind_m_s,
            touchdown_gust_m,
            faults,
            seed,
        }
    }
}

impl Default for Mission {
    /// The night parts 1 to 4 fly: the line 60 m across, launch at 22:00, a steady 2 m/s wind down
    /// the slope with no gust, the fix degrading at 50 s and lost at 55 s, the link lost at 65 s, a
    /// cell failing at 85 s.
    fn default() -> Self {
        Self::new(
            LINE_ACROSS_M,
            FLIGHT_START_HOUR,
            (DOWNSLOPE_WIND_M_S, ZERO),
            (ZERO, ZERO),
            FaultTimeline::new(
                GNSS_DEGRADED_AT_S,
                GNSS_LOST_AT_S,
                LINK_LOST_AT_S,
                Some(BATTERY_FAULT_AT_S),
            ),
            SCAN_SEED,
        )
    }
}
