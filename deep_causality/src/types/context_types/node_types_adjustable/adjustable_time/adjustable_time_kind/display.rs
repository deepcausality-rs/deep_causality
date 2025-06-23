// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use std::fmt;
use crate::prelude::{Identifiable, Temporal, AdjustableTimeKind};

impl fmt::Display for AdjustableTimeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdjustableTimeKind::Euclidean(t) => write!(f, "AdjustableEuclideanTime(id: {}, τ: {})", t.id(), t.time_unit()),
            AdjustableTimeKind::Entropic(t) => write!(f, "AdjustableEntropicTime(id: {}, t: {})", t.id(), t.time_unit()),
            AdjustableTimeKind::Discrete(t) => write!(f, "AdjustableDiscreteTime(id: {}, tick: {})", t.id(), t.time_unit()),
            AdjustableTimeKind::Lorentzian(t) => write!(f, "AdjustableLorentzianTime(id: {}, t: {})", t.id(), t.time_unit()),
            // AdjustableTimeKind::Symbolic(t) => write!(f, "AdjustableSymbolicTime(id: {}, {})", t.id(), t),
        }
    }
}