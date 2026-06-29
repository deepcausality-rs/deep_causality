/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B compressible shock-capturing QTT solver family. Stage 2 provides the 1-D conservative Euler
//! flux (ideal gas + Rusanov), gated by the Sod exact-Riemann solution; later stages add the
//! body-fitted coordinate coupling, IMEX time integration, and shock fitting.

mod euler_1d;

pub use euler_1d::{CompressibleEuler1d, EulerState, ideal_gas_pressure};
