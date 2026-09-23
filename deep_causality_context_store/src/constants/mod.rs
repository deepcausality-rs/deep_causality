/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// The shape version of the records.
///
/// Moves whenever a record changes shape. A snapshot written under a newer version than a reader
/// supports is refused with [`ProjectionError::Version`](crate::ProjectionError::Version).
pub const RECORD_VERSION: u16 = 1;
