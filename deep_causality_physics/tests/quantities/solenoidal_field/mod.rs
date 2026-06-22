/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Type-state tests for `SolenoidalField`: both construction paths produce
//! discretely divergence-free fields at every precision backend; every
//! rejection branch is covered. The unconstructibility guarantees (no public
//! constructor, no `Add`/`Mul`) are enforced by `compile_fail` doctests on the
//! type itself in `src/units/fluid_dynamics/solenoidal_field/mod.rs`.
//!
//! Shared fixtures (`unit_manifold`, `random_cochain`, `divergence`,
//! `sup_norm`) live in `deep_causality_physics::utils_tests` so each leaf
//! compiles standalone under Bazel.

mod divergence_free_tests;
mod open_boundary_tests;
mod rejection_tests;
mod wall_bounded_tests;
