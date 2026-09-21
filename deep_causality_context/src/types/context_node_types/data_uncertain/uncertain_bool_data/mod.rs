/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ContextoidId;
mod adjustable;
mod datable;
mod display;
mod identifiable;

use deep_causality_uncertain::{RandScalar, UncertainBool};

/// A context node holding an uncertain truth value.
///
/// Carries `R` for the same reason the [`UncertainBool<R>`] carrier does: a Boolean node reads a
/// truth value off a graph whose leaves are real, so it has the graph's scalar rather than none.
/// As with the real node, the parameter costs `Context` nothing.
#[derive(Debug, Clone)]
pub struct UncertainBoolData<R: RandScalar> {
    id: ContextoidId,
    data: UncertainBool<R>,
}

impl<R: RandScalar> UncertainBoolData<R> {
    pub fn new(id: ContextoidId, data: UncertainBool<R>) -> Self {
        Self { id, data }
    }
}
