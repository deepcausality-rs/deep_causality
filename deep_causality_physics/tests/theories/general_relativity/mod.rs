/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the general_relativity module.
//! Test files mirror the source structure:
//! - adm_state_tests.rs -> adm_state.rs
//! - gr_lie_mapping_tests.rs -> gr_lie_mapping.rs
//! - gr_ops_impl_tests.rs -> gr_ops_impl.rs
//! - metrics_tests.rs -> metrics.rs

#[cfg(test)]
pub mod adm_state_tests;
#[cfg(test)]
pub mod gr_lie_mapping_tests;
#[cfg(test)]
pub mod gr_ops_impl_tests;
#[cfg(test)]
pub mod metrics_tests;
