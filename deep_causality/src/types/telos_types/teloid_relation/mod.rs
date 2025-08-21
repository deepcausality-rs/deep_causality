/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod display;

/// Defines the nature of a relationship between two Teloids in the TeloidGraph.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum TeloidRelation {
    /// Represents standard deontic inheritance, where a more general norm's
    /// modality applies to a more specific one.
    #[default]
    Inherits,
    /// Represents a defeasance relationship, where one norm (e.g., a more specific
    /// or recent one) overrides or defeats another.
    Defeats,
}
