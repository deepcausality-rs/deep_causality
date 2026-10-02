/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::{Drone, FloatType, PatchReading, Quantity, Terrain};
use deep_causality_algebra::Real;
use deep_causality_num::{lift_i64, lift_usize, lower};
use deep_causality_uncertain::{SampleSession, Uncertain, UncertainError};

impl Drone {
    /// Local solar time, in hours since midnight of the launch day.
    pub fn local_hour(&self) -> FloatType {
        self.mission.start_hour() + lift_usize::<FloatType>(self.time_s) / SECONDS_PER_HOUR
    }

    /// The grid index of the ground patch directly below the drone.
    pub fn patch_below(&self) -> (i64, i64) {
        (patch_index(self.x), patch_index(self.y))
    }

    /// One frame of the downward thermal camera and LiDAR: a reading of every ground patch inside
    /// the footprint, which shrinks as the drone descends. The noise grows with height and is drawn
    /// from seeded sessions, so every run sees the same frames.
    pub fn scan(&self, terrain: &Terrain) -> Result<Vec<PatchReading>, UncertainError> {
        let reach = self.agl_m * HALF_FOV_TAN;
        let half = if reach > MIN_FOOTPRINT_HALF_M {
            reach
        } else {
            MIN_FOOTPRINT_HALF_M
        };
        let hour = self.local_hour();
        let sigmas = [
            TEMPERATURE_SIGMA_C.0 + TEMPERATURE_SIGMA_C.1 * self.agl_m,
            TEMPERATURE_SIGMA_C.0 + TEMPERATURE_SIGMA_C.1 * self.agl_m,
            SLOPE_SIGMA_DEG.0 + SLOPE_SIGMA_DEG.1 * self.agl_m,
            RETURNS_SIGMA.0 + RETURNS_SIGMA.1 * self.agl_m,
            PROTRUSION_SIGMA_M.0 + PROTRUSION_SIGMA_M.1 * self.agl_m,
        ];
        let mut readings = Vec::new();
        for i in patch_index(self.x - half)..=patch_index(self.x + half) {
            for j in patch_index(self.y - half)..=patch_index(self.y + half) {
                let centre = (patch_centre(i), patch_centre(j));
                let truth = truth(terrain, centre, hour);
                let mut values = [ZERO; 5];
                for (q, quantity) in Quantity::ALL.iter().enumerate() {
                    let session = SampleSession::seeded(self.mission.seed() + q as u64);
                    let draw = Uncertain::normal(truth[q], sigmas[q])
                        .sample_at(&session, noise_index(self.time_s, i, j))?;
                    values[q] = if *quantity == Quantity::Returns {
                        Real::clamp(draw, ZERO, ONE)
                    } else {
                        draw
                    };
                }
                readings.push(PatchReading::new((i, j), centre, values, sigmas));
            }
        }
        Ok(readings)
    }
}

/// The true value of each quantity on the patch centred at `centre`.
fn truth(terrain: &Terrain, centre: (FloatType, FloatType), hour: FloatType) -> [FloatType; 5] {
    let (x, y) = centre;
    let ground = terrain.ground_temperature_c(x, y, hour);
    let canopy = terrain.canopy_m(x, y);
    // A person under the canopy is hidden from the thermal camera and the LiDAR alike.
    let person = canopy == ZERO && terrain.person_within(x, y, PATCH_SIDE_M * HALF);
    let returns = if terrain.surface(x, y) == crate::Surface::Water {
        WATER_RETURNS
    } else {
        LAND_RETURNS
    };
    [
        ground,
        if person { PERSON_TEMPERATURE_C } else { ground },
        terrain.slope_deg(x, y),
        returns,
        if person { PERSON_HEIGHT_M } else { canopy },
    ]
}

/// The grid index of the patch containing a coordinate, in m.
fn patch_index(m: FloatType) -> i64 {
    lower(Real::floor(m / PATCH_SIDE_M)) as i64
}

/// The coordinate of the centre of the patch with grid index `i`, in m.
fn patch_centre(i: i64) -> FloatType {
    (lift_i64::<FloatType>(i) + HALF) * PATCH_SIDE_M
}

/// A draw index unique to the second and the patch, so each frame's noise is fresh.
fn noise_index(time_s: usize, i: i64, j: i64) -> u64 {
    (time_s as u64) * 1_000_000 + ((i + 500) as u64) * 1_000 + (j + 500) as u64
}
