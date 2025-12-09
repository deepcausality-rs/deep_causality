/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Common physical constants used throughout the library.
///
/// Values are taken from the [CODATA 2022](https://physics.nist.gov/cuu/Constants/) recommended values.
///
/// Note:
/// * "Exact" values are defined by international agreement (SI definition).
/// * Other values are experimental measurements with associated uncertainties (ignored here for standard f64 precision).
pub(crate) mod atomic;
pub(crate) mod electromagnetic;
pub(crate) mod thermodynamics;
pub(crate) mod universal;

pub use atomic::*;
pub use electromagnetic::*;
pub use thermodynamics::*;
pub use universal::*;
