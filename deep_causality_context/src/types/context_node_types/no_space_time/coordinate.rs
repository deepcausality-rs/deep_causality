/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::IndexError;
use crate::{Coordinate, NoSpaceTime};
use deep_causality_algebra::RealField;

impl<R: RealField> Coordinate for NoSpaceTime<R> {
    /// The scalar a frame naming this type works in. No value of this type is ever produced,
    /// because the coordinate system is empty; the parameter exists so the frame stays coherent.
    type Coord = R;

    /// Zero. There are no axes.
    fn dimension(&self) -> usize {
        0
    }

    /// Always an error: with no axes, every index is out of bounds.
    fn coordinate(&self, index: usize) -> Result<&R, IndexError> {
        Err(IndexError(format!(
            "Coordinate index out of bounds: {index}. NoSpaceTime has no axes"
        )))
    }
}
