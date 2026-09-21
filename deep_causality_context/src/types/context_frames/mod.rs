/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The frames this crate ships.
//!
//! # Which one to reach for
//!
//! | Frame | Members | Reach for it when |
//! |---|---|---|
//! | [`UniformFrame`](crate::UniformFrame) | the variant `Kind` enums | the world is not known at compile time — this is the ordinary case |
//! | [`BaseFrame`](crate::BaseFrame) | concrete Euclidean types | the world is fixed and known at compile time |
//! | [`ClockFrame`](crate::ClockFrame) | [`NoSpaceTime`](crate::NoSpaceTime) for space and spacetime | the context holds a root, some data and a clock, and nothing spatial |
//!
//! [`UniformFrame`] is first because a model can change regime within one run, and because a
//! context assembled from stored records carries its frame per record, so a read spanning several
//! can return more than one. A fixed frame cannot hold either result.
//!
//! [`ClockFrame`] is not an edge case. Across the workspace the contextoid variants are used
//! Root 45, Datoid 47, Tempoid 15, Spaceoid 5, SpaceTempoid 5, so most contexts hold no spatial
//! node at all.

mod base_frame;
mod clock_frame;
mod uniform_frame;

pub use base_frame::BaseFrame;
pub use clock_frame::ClockFrame;
pub use uniform_frame::UniformFrame;
