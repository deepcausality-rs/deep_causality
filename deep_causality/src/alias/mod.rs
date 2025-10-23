/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The alias_types module provides type aliases and common types used throughout the DeepCausality framework.
//!
//! This module contains several submodules:
//! * `alias_base` - Provides basic type aliases used across the framework
//! * `alias_function` - Contains function type aliases and common function signatures
//! * `alias_lock` - Defines synchronization and locking type aliases
//! * `alias_primitives` - Contains aliases for primitive types
//! * `alias_uniform` - Provides uniform type definitions used for consistency
pub(crate) mod alias_base;
pub(crate) mod alias_csm;
pub(crate) mod alias_function;
pub(crate) mod alias_lock;
pub(crate) mod alias_primitives;
pub(crate) mod alias_tensor;
pub(crate) mod alias_uncertain;
pub(crate) mod alias_uniform;
