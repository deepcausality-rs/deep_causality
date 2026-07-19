/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CFD solvers: a solver uses a theory and/or physics kernels to solve one
//! designated case (lid cavity, Taylor–Green, cylinder, MMS). Each owns a
//! configuration struct + type-state builder and exposes the CfdFlow interface.

// The DEC-native incompressible Navier–Stokes solver, migrated from
// `deep_causality_physics::theories::fluid_dynamics::dec`. The B-group refactor
// splits the DEC-native rate (a `FluidTheory`) out of the solver machinery.
pub(crate) mod dec;

pub use dec::dec_config::{
    DecNs, DecNsConfig, DecNsConfigNeedsTimeStep, DecNsConfigNeedsViscosity, DecNsConfigReady,
};

// The quantized-tensor-train (QTT) rollout: a quasi-1D linear advection–diffusion marcher that
// evolves a flowfield in compressed tensor-train form (the CFD ↔ tensor-network bridge).
mod qtt;

pub use qtt::{
    AcousticImex1d, CompressibleEuler1d, CompressibleMarcher2d, CompressibleMarcher3d,
    CompressibleMarcher3dFitted, EulerState, EulerState2d, EulerState3d, EulerStateTt2d,
    EulerStateTt3d, FittedNormalShock, ForcingRegion, Park2tClosure, PostShockState, QttImmersed2d,
    QttIncompressible2d, QttLinear1d, StagnationOutcome, conservation_round, divergence_residual,
    drag_lift, ideal_gas_pressure, ideal_gas_pressure_2d, kinetic_energy, max_bond, max_speed,
    positivity_floor, preserved_drag_fraction, strip_pressure_force, wall_heat_flux,
};
