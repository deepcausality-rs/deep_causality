/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GeoSpace, Spatial};
use deep_causality_algebra::RealField;

impl<R: RealField> Spatial for GeoSpace<R> {}
