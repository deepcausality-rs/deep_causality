/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The tower memberships for this container are pinned at compile time in
//! `src/traits/tower_pins.rs`, not tested here.
//!
//! A test that calls a function bounded on `Ring` passes by compiling. If the impl were missing the
//! crate would not build, so running such a test checks nothing the build has not. What cannot be
//! checked by the build succeeding — that a matrix is *refused* by `CommutativeRing` — is a
//! `compile_fail` doctest on the algebra module itself.
