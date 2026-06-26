/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Alternating-least-squares (ALS) tensor-train solvers: `fit` (TT completion from samples) and
//! `linear` (`A x = b` in tensor-train form). Both sweep the cores of the unknown train, solving a
//! small local least-squares system per site, sharing the dense `solve_local` helper.

mod local;

pub use local::{fit, linear};
