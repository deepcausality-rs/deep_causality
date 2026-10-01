/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{FaultTimeline, FloatType, Mission};

impl Mission {
    /// The line's distance across the slope, in m.
    pub fn line_across_m(&self) -> FloatType {
        self.line_across_m
    }

    /// Local solar time at launch, in hours.
    pub fn start_hour(&self) -> FloatType {
        self.start_hour
    }

    /// The wind across and along the valley, in m/s.
    pub fn wind_m_s(&self) -> (FloatType, FloatType) {
        self.wind_m_s
    }

    /// How far a gust moves the drone in its last second before touchdown, across and along, in m.
    pub fn touchdown_gust_m(&self) -> (FloatType, FloatType) {
        self.touchdown_gust_m
    }

    pub fn faults(&self) -> FaultTimeline {
        self.faults
    }

    /// The seed of the sensor noise.
    pub fn seed(&self) -> u64 {
        self.seed
    }
}
