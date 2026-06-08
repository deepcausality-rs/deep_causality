/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared support code for the causal-discovery examples. The example `main`s pull
//! their test-data writers from here so each stays focused on its pipeline.

#[path = "shared/cdl_data.rs"]
pub mod cdl_data;
