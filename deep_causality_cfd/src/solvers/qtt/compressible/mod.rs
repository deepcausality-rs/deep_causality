/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B compressible shock-capturing QTT solver family. Stage 2 provides the 1-D conservative Euler
//! flux (ideal gas + Rusanov), gated by the Sod exact-Riemann solution; later stages add the
//! body-fitted coordinate coupling, IMEX time integration, and shock fitting.

mod euler_1d;
mod fitting;
mod forcing;
mod imex;
mod marcher_2d;
mod marcher_3d;
mod marcher_3d_fitted;

pub use euler_1d::{CompressibleEuler1d, EulerState, ideal_gas_pressure};
pub use fitting::{FittedNormalShock, Park2tClosure, PostShockState, StagnationOutcome};
pub use forcing::ForcingRegion;
pub use imex::{AcousticImex1d, conservation_round, positivity_floor};
pub use marcher_2d::{CompressibleMarcher2d, EulerState2d, EulerStateTt2d, ideal_gas_pressure_2d};
pub use marcher_3d::{CompressibleMarcher3d, EulerState3d, EulerStateTt3d};
pub use marcher_3d_fitted::CompressibleMarcher3dFitted;
