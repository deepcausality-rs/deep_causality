/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod display;

/// Represents the modal status of a teloid, indicating its obligation or permissibility.
///
/// This enum is used to define whether a teloid (a goal or an end) is obligatory,
/// impermissible, or optional, potentially with an associated cost.
///
/// For more details, see section 5 (Causality as EPP) and 8 (Teleology) in The EPP reference paper:
/// <https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf>
///
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TeloidModal {
    #[default]
    Obligatory,
    Impermissible,
    Optional(i64), // Optional must be associated with a cost value.
}
