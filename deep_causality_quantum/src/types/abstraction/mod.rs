/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Abstractions between compositional models: the object of Lorenz & Tull, arXiv:2602.16612,
//! Definitions 14 and 15, its naturality check, and the bound its residuals place on the diamond
//! distance.

#[allow(clippy::module_inception)]
pub(crate) mod abstraction;
pub(crate) mod alignment_structure;
pub(crate) mod code_abstraction;
pub(crate) mod diamond_bound;
pub(crate) mod fault_set;
pub(crate) mod fault_tolerance;
pub(crate) mod ideal_recovery;
pub(crate) mod naturality;
pub(crate) mod qc_model;
pub(crate) mod query;
pub(crate) mod type_alignment;

pub use abstraction::*;
pub use alignment_structure::*;
pub use code_abstraction::*;
pub use diamond_bound::*;
pub use fault_set::*;
pub use fault_tolerance::*;
pub use ideal_recovery::*;
pub use naturality::*;
pub use qc_model::*;
pub use query::*;
pub use type_alignment::*;
