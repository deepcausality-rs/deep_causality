/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Temporal, TimeKind};
use core::fmt;
use deep_causality_algebra::RealField;
use deep_causality_core::Identifiable;
use deep_causality_num::FromPrimitive;

impl<R: RealField + FromPrimitive + fmt::Display> fmt::Display for TimeKind<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeKind::Newtonian(t) => {
                write!(f, "NewtonianTime(id: {}, τ: {})", t.id(), t.time_unit())
            }
            TimeKind::Entropic(t) => {
                write!(f, "EntropicTime(id: {}, t: {})", t.id(), t.time_unit())
            }
            TimeKind::Discrete(t) => {
                write!(f, "DiscreteTime(id: {}, tick: {})", t.id(), t.time_unit())
            }
            TimeKind::Minkowski(t) => {
                write!(f, "MinkowskiTime(id: {}, t: {})", t.id(), t.time_unit())
            } // TimeKind::Symbolic(t) => write!(f, "SymbolicTime(id: {}, {})", t.id(), t),
        }
    }
}
