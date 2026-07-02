/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared machinery of the plasma-blackout CFD examples (`plasma_blackout_corridor`,
//! `plasma_blackout_weather`): the working precision, the specification constants both examples
//! fly, the small numeric helpers, the example-local physics stages, and the descent-world
//! builder plus the composed coupling stack.
//!
//! The examples stay the stories; this module is the cast they share. Every tuned number keeps
//! its label next to its definition, exactly as the per-example `constants.rs` files did before
//! the split.

pub mod constants;
pub mod stages;
pub mod support;
pub mod world;

/// The working precision of the corridor and the weather table (flow, plasma, navigation,
/// control are all generic over it). Switch between `f64` and `deep_causality_num::Float106`
/// (106-bit double-double); the specification constants stay `f64` literals, which either type
/// represents exactly, and every derived number is computed in this type, so this alias is the
/// only line that changes. Measured (2026-07-02): a `Float106` corridor run reproduces every
/// gate and event step exactly at ~11x the wall-clock; `f32` fails on exponent range.
pub type FloatType = f64;
