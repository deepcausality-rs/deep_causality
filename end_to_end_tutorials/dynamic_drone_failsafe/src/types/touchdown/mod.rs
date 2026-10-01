/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod touchdown_getters;

use crate::constants::*;
use crate::{Drone, FloatType, Outcome, Surface, Terrain};

/// Where the drone came down, and what became of it: the ground truth no controller sees.
#[derive(Debug, Clone, Copy)]
pub struct Touchdown {
    x: FloatType,
    y: FloatType,
    surface: Surface,
    slope_deg: FloatType,
    nearest_person_m: FloatType,
    outcome: Outcome,
}

impl Touchdown {
    /// Judges where `drone` came down on `terrain`: a fall where its battery died, or a touchdown.
    pub fn of(terrain: &Terrain, drone: &Drone) -> Self {
        let (x, y) = drone.position();
        if drone.fell() {
            Self::fall(terrain, x, y, Outcome::Fell)
        } else if drone.hit_trees() {
            Self::fall(terrain, x, y, Outcome::HitTrees)
        } else {
            Self::assess(terrain, x, y)
        }
    }

    /// Judges an uncontrolled descent at a point on `terrain`, a fall or a crash into the trees: the
    /// drone is lost, as `lost` says, and anyone within the clearance is at risk.
    pub fn fall(terrain: &Terrain, x: FloatType, y: FloatType, lost: Outcome) -> Self {
        let nearest_person_m = terrain.nearest_person_m(x, y);
        Self {
            x,
            y,
            surface: terrain.surface(x, y),
            slope_deg: terrain.slope_deg(x, y),
            nearest_person_m,
            outcome: if nearest_person_m < PERSON_CLEARANCE_M {
                Outcome::AmongPeople
            } else {
                lost
            },
        }
    }

    /// Judges a touchdown at a point on `terrain`.
    pub fn assess(terrain: &Terrain, x: FloatType, y: FloatType) -> Self {
        let surface = terrain.surface(x, y);
        let slope_deg = terrain.slope_deg(x, y);
        let nearest_person_m = terrain.nearest_person_m(x, y);
        let outcome = if surface == Surface::Water {
            Outcome::Ditched
        } else if surface == Surface::Ravine {
            Outcome::IntoRavine
        } else if surface == Surface::Trees {
            Outcome::HitTrees
        } else if nearest_person_m < PERSON_CLEARANCE_M {
            Outcome::AmongPeople
        } else if slope_deg > TIP_OVER_DEG {
            roll(terrain, x, y)
        } else {
            Outcome::Safe
        };
        Self {
            x,
            y,
            surface,
            slope_deg,
            nearest_person_m,
            outcome,
        }
    }
}

/// Tumbles a tipped drone downhill in half-metre steps until the ground flattens or it reaches
/// water.
fn roll(terrain: &Terrain, mut x: FloatType, mut y: FloatType) -> Outcome {
    let mut distance_m = ZERO;
    for _ in 0..10_000 {
        if terrain.surface(x, y) == Surface::Water || terrain.slope_deg(x, y) < ROLL_STOP_DEG {
            break;
        }
        let (dx, dy) = terrain.downhill(x, y);
        x += dx * HALF;
        y += dy * HALF;
        distance_m += HALF;
    }
    Outcome::TippedAndRolled {
        distance_m,
        came_to_rest_on: terrain.surface(x, y),
    }
}
