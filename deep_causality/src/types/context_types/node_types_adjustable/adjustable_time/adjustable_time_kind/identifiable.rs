// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{Identifiable, AdjustableTimeKind};

impl Identifiable for AdjustableTimeKind {
    fn id(&self) -> u64 {
        match self {
            AdjustableTimeKind::Euclidean(t) => t.id(),
            AdjustableTimeKind::Entropic(t) => t.id(),
            AdjustableTimeKind::Discrete(t) => t.id(),
            AdjustableTimeKind::Lorentzian(t) => t.id(),
            // AdjustableTimeKind::Symbolic(t) => t.id(),
        }
    }
}
