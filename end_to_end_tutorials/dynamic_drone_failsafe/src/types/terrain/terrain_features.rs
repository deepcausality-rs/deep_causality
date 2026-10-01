/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::{FloatType, Surface, Terrain};
use deep_causality_algebra::Real;

impl Terrain {
    /// What covers the ground at a point.
    pub fn surface(&self, x: FloatType, y: FloatType) -> Surface {
        if self.pad_at(x, y).is_some() {
            Surface::Pad
        } else if self.in_ravine(x, y) {
            Surface::Ravine
        } else if x < CREEK_EDGE_M {
            Surface::Water
        } else if x < ROAD_FROM_M {
            Surface::Grass
        } else if x < ROAD_TO_M {
            Surface::Road
        } else {
            Surface::Rock
        }
    }

    /// Distance from a point to the nearest person on the ground, in m.
    pub fn nearest_person_m(&self, x: FloatType, y: FloatType) -> FloatType {
        CREW_M
            .iter()
            .map(|&(px, py)| Real::sqrt((x - px) * (x - px) + (y - py) * (y - py)))
            .fold(NO_ONE_NEARBY_M, |a, b| if b < a { b } else { a })
    }

    /// The centre of the tower pad at a point, if there is one.
    pub(super) fn pad_at(&self, x: FloatType, y: FloatType) -> Option<(FloatType, FloatType)> {
        PAD_CENTRES_M.iter().copied().find(|&(cx, cy)| {
            Real::abs(x - cx) <= PAD_HALF_SIDE_M && Real::abs(y - cy) <= PAD_HALF_SIDE_M
        })
    }

    pub(super) fn in_ravine(&self, x: FloatType, y: FloatType) -> bool {
        x >= ROAD_TO_M && (RAVINE_FROM_M..=RAVINE_TO_M).contains(&y)
    }
}
