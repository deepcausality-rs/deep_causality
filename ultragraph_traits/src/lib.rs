/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The [`EdgeKey`] trait of `ultragraph`, separated so that a crate declaring an edge payload
//! can implement it without depending on the graph.
//!
//! The crate builds on `core` alone. Its `std` and `no-std` features change nothing; they let
//! the workspace's bare-metal checks treat it like every other crate.

#![no_std]

mod extensions;
mod traits;

pub use crate::traits::edge_key::EdgeKey;
