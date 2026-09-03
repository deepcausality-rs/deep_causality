/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the geometric-QEC example.

/// The side of the square torus. `L = 4` gives the `[[32, 2]]` toric code.
pub const LATTICE_SIDE: usize = 4;

/// The LDPC bound the code is checked against, and the one it fails.
pub const LDPC_BOUND: usize = 4;
pub const LDPC_BOUND_TOO_TIGHT: usize = 3;
