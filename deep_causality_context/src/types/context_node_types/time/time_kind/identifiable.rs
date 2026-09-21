/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use crate::TimeKind;
use deep_causality_algebra::RealField;
use deep_causality_core::Identifiable;

impl<R: RealField> Identifiable for TimeKind<R> {
    fn id(&self) -> ContextoidId {
        match self {
            TimeKind::Euclidean(t) => t.id(),
            TimeKind::Entropic(t) => t.id(),
            TimeKind::Discrete(t) => t.id(),
            TimeKind::Lorentzian(t) => t.id(),
            // TimeKind::Symbolic(t) => t.id(),
        }
    }
}
