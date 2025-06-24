// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{TimeKind, ScalarProjector};

impl ScalarProjector for TimeKind {
    type Scalar = f64;

    fn project(&self) -> Self::Scalar {
        match self {
            TimeKind::Discrete(t) => t.project() as f64,
            TimeKind::Euclidean(t) => t.project(),
            TimeKind::Entropic(t) => t.project() as f64,
            TimeKind::Lorentzian(t) => t.project(),
            // TimeKind::Symbolic(t) => t.project(),
        }
    }
}
