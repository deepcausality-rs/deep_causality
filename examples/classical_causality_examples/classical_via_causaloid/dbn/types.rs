/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::Verdict;

/// Simple state struct to pass rain probability between causaloids
#[derive(Debug, Clone, Copy, Default)]
pub struct WeatherState {
    pub rain_probability: f64,
    pub current_day: f64,
}

/// Graph reasoning requires the wire carrier to be a `Verdict` (the Stage-4 join bound:
/// reconvergent values fuse with the commutative `∇ = Verdict::join`). `WeatherState` is the
/// **product lattice** of its fields over the extended-real min/max lattice (bounds `±∞`,
/// complement `1 − x`), lawful for arbitrary values — `current_day` is not a probability. This
/// chain-shaped DBN has no reconvergent join, so the instance is a carrier bound, never
/// exercised as a merge.
impl Verdict for WeatherState {
    fn bottom() -> Self {
        WeatherState {
            rain_probability: f64::NEG_INFINITY,
            current_day: f64::NEG_INFINITY,
        }
    }
    fn top() -> Self {
        WeatherState {
            rain_probability: f64::INFINITY,
            current_day: f64::INFINITY,
        }
    }
    fn meet(self, other: Self) -> Self {
        WeatherState {
            rain_probability: self.rain_probability.min(other.rain_probability),
            current_day: self.current_day.min(other.current_day),
        }
    }
    fn join(self, other: Self) -> Self {
        WeatherState {
            rain_probability: self.rain_probability.max(other.rain_probability),
            current_day: self.current_day.max(other.current_day),
        }
    }
    fn complement(self) -> Self {
        WeatherState {
            rain_probability: 1.0 - self.rain_probability,
            current_day: 1.0 - self.current_day,
        }
    }
}
