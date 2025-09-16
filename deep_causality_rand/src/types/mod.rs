/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod distr;
pub(crate) mod misc;
pub(crate) mod rand;

#[cfg(feature = "os-random")]
pub use rand::os_random_rng::OsRandomRng;

pub use rand::siphash13_rng::SipHash13Rng;
