/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The orthomodular quantum-logic verdict carrier and the Born-rule extraction
//! that reads a classical verdict out of a state at the measurement boundary.

pub(crate) mod born;
pub(crate) mod projection;

pub use born::*;
pub use projection::*;
