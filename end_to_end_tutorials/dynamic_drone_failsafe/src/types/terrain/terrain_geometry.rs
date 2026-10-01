/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::{FloatType, Terrain};
use deep_causality_algebra::Real;

impl Terrain {
    /// Elevation of the ground above the creek, in m.
    pub fn elevation_m(&self, x: FloatType, y: FloatType) -> FloatType {
        if let Some((cx, _)) = self.pad_at(x, y) {
            return cross_section_m(cx);
        }
        let base = cross_section_m(x);
        if self.in_ravine(x, y) {
            base - RAVINE_DEPTH_M
        } else {
            base
        }
    }

    /// Slope of the ground at a point, in degrees, from the elevation half a metre either way.
    pub fn slope_deg(&self, x: FloatType, y: FloatType) -> FloatType {
        let (gx, gy) = self.gradient(x, y);
        Real::atan(Real::sqrt(gx * gx + gy * gy)) * DEGREES_PER_HALF_TURN
            / <FloatType as Real>::pi()
    }

    /// The ground's downhill direction at a point, as a unit vector, or zero where it is flat.
    pub fn downhill(&self, x: FloatType, y: FloatType) -> (FloatType, FloatType) {
        let (gx, gy) = self.gradient(x, y);
        let norm = Real::sqrt(gx * gx + gy * gy);
        if norm > ZERO {
            (-gx / norm, -gy / norm)
        } else {
            (ZERO, ZERO)
        }
    }

    fn gradient(&self, x: FloatType, y: FloatType) -> (FloatType, FloatType) {
        let gx = (self.elevation_m(x + HALF, y) - self.elevation_m(x - HALF, y)) / (HALF + HALF);
        let gy = (self.elevation_m(x, y + HALF) - self.elevation_m(x, y - HALF)) / (HALF + HALF);
        (gx, gy)
    }
}

/// Elevation of the valley cross-section at `x` across the slope, in m.
fn cross_section_m(x: FloatType) -> FloatType {
    let grass = Real::tan(GRASS_SLOPE_DEG * <FloatType as Real>::pi() / DEGREES_PER_HALF_TURN);
    let rock = Real::tan(ROCK_SLOPE_DEG * <FloatType as Real>::pi() / DEGREES_PER_HALF_TURN);
    if x < CREEK_EDGE_M {
        ZERO
    } else if x < ROAD_FROM_M {
        (x - CREEK_EDGE_M) * grass
    } else if x < ROAD_TO_M {
        (ROAD_FROM_M - CREEK_EDGE_M) * grass
    } else {
        (ROAD_FROM_M - CREEK_EDGE_M) * grass + (x - ROAD_TO_M) * rock
    }
}
