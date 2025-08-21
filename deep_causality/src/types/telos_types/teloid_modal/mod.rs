/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod display;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TeloidModal {
    #[default]
    Obligatory,
    Impermissible,
    Optional(i64), // Optional must be associated with a cost value.
}
