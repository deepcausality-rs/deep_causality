/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Result writers, expressed as lazy [`deep_causality_haft::IoAction`]s.

pub mod write_rows;
pub mod write_table;

pub use write_rows::{WriteRows, write_rows};
pub use write_table::{WriteTable, write_table};
