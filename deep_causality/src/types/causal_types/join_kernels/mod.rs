/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Built-in fan-in join kernels for reconvergence nodes.
//!
//! A join kernel is a concrete [`ContextualJoinFn`](crate::ContextualJoinFn) plus the
//! configuration it reads from a node's context channel. The engine invokes it at a
//! multi-fired reconvergence to reduce the labeled parent effects to the node's input.

pub mod linear_join;
