/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{Identifiable, TimeKind};

impl Identifiable for TimeKind {
    fn id(&self) -> u64 {
        match self {
            TimeKind::Euclidean(t) => t.id(),
            TimeKind::Entropic(t) => t.id(),
            TimeKind::Discrete(t) => t.id(),
            TimeKind::Lorentzian(t) => t.id(),
            // TimeKind::Symbolic(t) => t.id(),
        }
    }
}
