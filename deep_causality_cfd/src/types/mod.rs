/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CFD value types: the `CfdScalar` bound, the per-step `Ambient`, the marching
//! state, configuration structs, and the Flow DSL surface types.

mod ambient;
mod cfd_scalar;
pub mod flow;

pub use ambient::Ambient;
pub use cfd_scalar::CfdScalar;
