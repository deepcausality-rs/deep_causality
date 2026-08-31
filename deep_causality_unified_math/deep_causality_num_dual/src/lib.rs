/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]
extern crate core;

// Private re-exports so the moved `dual` module keeps resolving these through
// `crate::` paths.
use deep_causality_algebra::{Annihilating, Associative, Commutative, Distributive, Real};
use deep_causality_num::{FromPrimitive, One, Zero};

mod dual;
mod extensions;

// Dual number type (forward-mode automatic differentiation; the differentiating *number*).
// The differentiation/integration *operators* live in `deep_causality_calculus`.
pub use crate::dual::dual_number::Dual;

// HKT witness for `Dual` (Functor, Foldable, and the lax monoidal stack). `CoMonad` is
// deliberately deferred; see the module docs.
pub use crate::extensions::DualWitness;
