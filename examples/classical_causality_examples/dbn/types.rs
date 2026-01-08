/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Simple state struct to pass rain probability between causaloids
#[derive(Debug, Clone, Copy, Default)]
pub struct WeatherState {
    pub rain_probability: f64,
    pub current_day: f64,
}
