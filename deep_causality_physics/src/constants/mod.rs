/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod atomic;
pub mod electromagnetic;
pub mod thermodynamics;
pub mod universal;

pub use atomic::*;
pub use electromagnetic::*;
pub use thermodynamics::*;
pub use universal::*;

/// Common physical constants used throughout the library.
///
/// Values are taken from the [CODATA 2022](https://physics.nist.gov/cuu/Constants/) recommended values.
///
/// Note:
/// * "Exact" values are defined by international agreement (SI definition).
/// * Other values are experimental measurements with associated uncertainties (ignored here for standard f64 precision).
