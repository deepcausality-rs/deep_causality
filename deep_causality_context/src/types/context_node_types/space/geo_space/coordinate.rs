/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::IndexError;
use crate::{Coordinate, GeoSpace};
use deep_causality_algebra::RealField;

impl<R: RealField> Coordinate for GeoSpace<R> {
    type Coord = R;
    fn dimension(&self) -> usize {
        3
    }

    fn coordinate(&self, index: usize) -> Result<&R, IndexError> {
        match index {
            0 => Ok(&self.lat),
            1 => Ok(&self.lon),
            2 => Ok(&self.alt),
            _ => Err(IndexError(format!(
                "Coordinate index out of bounds: {}",
                index
            ))),
        }
    }
}
