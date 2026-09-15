/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reach-throughs for the `tests/` tree.
//!
//! A few guards defend against node shapes no public builder produces, so no public API can put a
//! graph into the state they reject. These expose the crate-internal constructors that can, so the
//! guards are covered by a behavioural test in `tests/` rather than by an in-`src` one.

pub mod raw_graph;
