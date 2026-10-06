/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ScalarProjector, TimeKind};
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift_count};

impl<R: RealField + FromPrimitive + Default> ScalarProjector for TimeKind<R> {
    type Scalar = R;

    fn project(&self) -> Self::Scalar {
        match self {
            TimeKind::Discrete(t) => lift_count(t.project()),
            TimeKind::Newtonian(t) => t.project(),
            TimeKind::Entropic(t) => lift_count(t.project()),
            TimeKind::Minkowski(t) => t.project(),
            // TimeKind::Symbolic(t) => t.project(),
        }
    }
}
