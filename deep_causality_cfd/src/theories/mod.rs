/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fluid-dynamics theories: the Navier–Stokes regimes (incompressible,
//! compressible, Euler, Stokes) and the DEC-native incompressible rate.
//!
//! What lives where, precisely:
//!
//! * **The four pointwise regime evaluators live here**, as free functions
//!   (`incompressible_ns_rhs`, `compressible_ns_*_rhs`, `euler_*_rhs`,
//!   `stokes_rhs`), each with a causal `PropagatingEffect` wrapper
//!   (`*_rhs_effect`) in [`wrappers`].
//! * **The shared sub-kernels stay in `deep_causality_physics`**:
//!   `convective_acceleration_kernel`, `pressure_gradient_force_kernel`,
//!   `viscous_diffusion_kernel` and `continuity_rhs_kernel`. The evaluators here
//!   compose those kernels rather than reimplementing them.
//! * **`DecIncompressible` is the only `FluidTheory` realization in this module**;
//!   it wraps the DEC rate kernel. The pointwise evaluators are free functions and
//!   do not implement that trait.

mod compressible_ns;
mod euler;
mod incompressible_dec;
mod incompressible_ns;
mod stokes;
mod wrappers;

// The pointwise NS regime evaluators (`*_rhs`) and their causal `PropagatingEffect`
// wrappers (`*_rhs_effect`), plus the DEC-native `FluidTheory` realization.
pub use compressible_ns::{
    compressible_ns_continuity_rhs, compressible_ns_energy_rhs, compressible_ns_momentum_rhs,
};
pub use euler::euler_momentum_rhs;
pub use incompressible_dec::DecIncompressible;
pub use incompressible_ns::incompressible_ns_rhs;
pub use stokes::stokes_momentum_rhs;
pub use wrappers::{
    compressible_ns_continuity_rhs_effect, compressible_ns_energy_rhs_effect,
    compressible_ns_momentum_rhs_effect, euler_momentum_rhs_effect, incompressible_ns_rhs_effect,
    stokes_momentum_rhs_effect,
};
