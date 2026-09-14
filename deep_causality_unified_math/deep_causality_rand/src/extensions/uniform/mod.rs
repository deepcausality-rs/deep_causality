/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Type-specific uniform-sampler extensions: the `SampleUniform` / `RandFloat` /
//! `UniformSampler` impls for the concrete scalar types, segregated from the core
//! `Uniform` / `UniformFloat` machinery in `types::distr::uniform`.

pub mod uniform_float;
pub mod uniform_u32;
mod uniform_u64;
mod uniform_usize;
