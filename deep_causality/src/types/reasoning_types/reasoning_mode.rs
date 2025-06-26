/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ReasoningMode {
    Deterministic,
    Probabilistic,
    Symbolic,
}
