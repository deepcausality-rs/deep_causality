/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Test utilities: an in-memory backend for both storage traits, an in-memory substrate, and a
//! runtime-free way to drive the asynchronous contract.

mod block_on;
mod memory_events;
mod memory_storage;
mod memory_substrate;

pub use block_on::block_on;
pub use memory_events::MemoryEvents;
pub use memory_storage::MemoryStorage;
pub use memory_substrate::MemorySubstrate;
