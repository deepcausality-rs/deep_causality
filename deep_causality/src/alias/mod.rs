/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The alias_types module provides type aliases and common types used throughout the DeepCausality framework.
//!
//! This module contains several submodules:
//! * `alias_base` - The scalar `FloatType` and the model, causaloid, and graph aliases over `BaseContext`
//! * `alias_csm` - State-action pairs and the collections a causal state machine holds them in
//! * `alias_function` - Function type aliases for causal and assumption evaluation functions
//! * `alias_uncertain` - The uncertain carriers at `FloatType` and the uncertain activation predicate
//! * `alias_uniform` - The model, causaloid, and graph aliases over `UniformContext`
pub(crate) mod alias_base;
pub(crate) mod alias_csm;
pub(crate) mod alias_function;
pub(crate) mod alias_uncertain;
pub(crate) mod alias_uniform;

pub use crate::alias::alias_base::*;
pub use crate::alias::alias_csm::*;
pub use crate::alias::alias_function::*;
pub use crate::alias::alias_uncertain::*;
pub use crate::alias::alias_uniform::*;
