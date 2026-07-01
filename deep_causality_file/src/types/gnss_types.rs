/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! GNSS data type definitions and aliases.

use crate::{ClockData, DataLoadingError, OrbitData};

/// Result type for loading GNSS satellite data (clock and orbit). Errors are the crate's invariant
/// [`DataLoadingError`]; the alias reduces noise on functions that return both series.
pub type GnssDataResult<R> = Result<(Vec<ClockData<R>>, Vec<OrbitData<R>>), DataLoadingError>;
