/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! [`EvidenceClass`]: where a gate's bound came from.
//!
//! A `[PASS]` line carries no information until the reader knows whether the bound it cleared
//! encodes knowledge from outside this codebase. Two gates can print identically while one would
//! catch a physics regression and the other only detects drift from a previous run. This type is
//! that one bit, made explicit at the gate.
//!
//! The distinction is deliberately two-valued. What a harness *verifies* (analytic solution,
//! flight-data order-of-magnitude, structural rank lever) is documented per harness; this type
//! answers the narrower question of where the *number* came from.

use core::fmt;

/// The provenance of a gate's numeric bound.
///
/// Defaults to [`Tripwire`](Self::Tripwire) wherever a gate does not declare a class: claiming
/// agreement with an external reference requires positive evidence, so the unlabeled case must be
/// the weaker one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvidenceClass {
    /// The bound is an analytic solution or a published external value, and the citation is
    /// recorded where the bound is defined. Clearing it is evidence about the physics.
    Reference,
    /// The bound is pinned from this code's own prior output. Clearing it is evidence of
    /// non-regression only, and carries no claim of external accuracy.
    Tripwire,
}

impl EvidenceClass {
    /// Whether this bound encodes knowledge from outside the codebase.
    pub fn is_reference(&self) -> bool {
        matches!(self, Self::Reference)
    }

    /// The lowercase tag used in rendered gate lines.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Tripwire => "tripwire",
        }
    }
}

impl Default for EvidenceClass {
    /// [`Tripwire`](Self::Tripwire) — the safe default, per the type-level note.
    fn default() -> Self {
        Self::Tripwire
    }
}

impl fmt::Display for EvidenceClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tag())
    }
}
