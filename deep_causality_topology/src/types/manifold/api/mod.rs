/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Public API for Manifold operations.
//!
//! This module contains the public interface that dispatches to either
//! CPU or MLX implementations based on feature flags and heuristics.

mod constructors;
mod covariance;
mod geometry;
