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
        } else if self.in_woodland(x, y) {
            Surface::Trees
        } else if x < ROAD_FROM_M {
            Surface::Grass
        } else if x < ROAD_TO_M {
            Surface::Road
        } else {
            Surface::Rock
        }
    }

    /// Where each person stands, across and along the valley, in m.
    pub fn crew_m(&self) -> &[(FloatType, FloatType)] {
        &self.crew_m
    }

    /// Distance from a point to the nearest person on the ground, in m.
    pub fn nearest_person_m(&self, x: FloatType, y: FloatType) -> FloatType {
        self.crew_m
            .iter()
            .map(|&(px, py)| Real::sqrt((x - px) * (x - px) + (y - py) * (y - py)))
            .fold(NO_ONE_NEARBY_M, |a, b| if b < a { b } else { a })
    }

    /// The centre of the tower pad at a point, if there is one.
    pub(super) fn pad_at(&self, x: FloatType, y: FloatType) -> Option<(FloatType, FloatType)> {
        let cx = self.line_across_m;
        PAD_ALONG_M
            .iter()
            .copied()
            .find(|&cy| {
                Real::abs(x - cx) <= PAD_HALF_SIDE_M && Real::abs(y - cy) <= PAD_HALF_SIDE_M
            })
            .map(|cy| (cx, cy))
    }

    /// Whether a point lies on the flat terrace above the creek.
    pub(super) fn on_terrace(&self, x: FloatType, y: FloatType) -> bool {
        (self.terrace_across_m.0..=self.terrace_across_m.1).contains(&x)
            && (self.terrace_along_m.0..=self.terrace_along_m.1).contains(&y)
    }

    /// Whether a person stands inside the square of half-side `half_m` centred on a point.
    pub fn person_within(&self, x: FloatType, y: FloatType, half_m: FloatType) -> bool {
        self.crew_m
            .iter()
            .any(|&(px, py)| Real::abs(px - x) <= half_m && Real::abs(py - y) <= half_m)
    }

    /// How high the canopy stands above the ground at a point, in m: zero outside the trees.
    pub fn canopy_m(&self, x: FloatType, y: FloatType) -> FloatType {
        if self.surface(x, y) == Surface::Trees {
            CANOPY_HEIGHT_M
        } else {
            ZERO
        }
    }

    fn in_woodland(&self, x: FloatType, y: FloatType) -> bool {
        (self.woodland_across_m.0..=self.woodland_across_m.1).contains(&x)
            && (self.woodland_along_m.0..=self.woodland_along_m.1).contains(&y)
    }

    pub(super) fn in_ravine(&self, x: FloatType, y: FloatType) -> bool {
        x >= ROAD_TO_M && (RAVINE_FROM_M..=RAVINE_TO_M).contains(&y)
    }
}
