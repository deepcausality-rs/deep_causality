/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coherent regime evaluators assembling the pointwise fluid-dynamics kernels into
//! the four classical Navier-Stokes regimes: incompressible Newtonian, compressible
//! Newtonian, Euler (inviscid limit), and Stokes (creeping-flow limit).
//!
//! Each regime is a free function composing kernels from `crate::kernels::fluids`.
//! Regime functions are stateless, side-effect-free, generic over `R: RealField`,
//! and return the pointwise RHS of `∂u/∂t = ...` in Eulerian acceleration form.

pub(crate) mod compressible_ns;
pub(crate) mod euler;
pub(crate) mod incompressible_ns;
pub(crate) mod stokes;

// Group `pub use` re-exports are commented out until each regime ships.
// pub use compressible_ns::*;
// pub use euler::*;
pub use incompressible_ns::*;
// pub use stokes::*;
