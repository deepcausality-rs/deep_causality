// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ReasoningMode {
    Deterministic,
    Probabilistic,
    Symbolic,
}
