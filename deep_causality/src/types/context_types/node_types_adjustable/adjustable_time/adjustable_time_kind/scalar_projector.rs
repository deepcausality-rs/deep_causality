// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableTimeKind, ScalarProjector};

impl ScalarProjector for AdjustableTimeKind {
    type Scalar = f64;

    fn project(&self) -> Self::Scalar {
        match self {
            AdjustableTimeKind::Discrete(t) => t.project() as f64,
            AdjustableTimeKind::Euclidean(t) => t.project(),
            AdjustableTimeKind::Entropic(t) => t.project() as f64,
            AdjustableTimeKind::Lorentzian(t) => t.project(),
            // AdjustableTimeKind::Symbolic(t) => t.project(),
        }
    }
}
