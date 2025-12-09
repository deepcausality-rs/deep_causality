/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;
extern crate core;

pub mod constants;
pub mod dynamics;
pub mod electromagnetism;
pub mod error;
pub mod fluids;
pub mod materials;
pub mod nuclear;
pub mod quantum;
pub mod relativity;
pub mod thermodynamics;
pub use constants::*;
pub use error::physics_error::{PhysicsError, PhysicsErrorEnum};
pub mod astro;
pub mod types;
pub mod waves;

pub use types::*;
