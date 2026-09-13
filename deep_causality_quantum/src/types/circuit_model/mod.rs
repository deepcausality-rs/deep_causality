/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Compositional models in QC, the category of controlled quantum instruments (Lorenz & Tull,
//! arXiv:2602.16612, Example 57), with their two semantics functors.

pub(crate) mod circuit_box;
pub(crate) mod exact_semantics;
pub(crate) mod gate_unitary;
pub(crate) mod induced_dag;
pub(crate) mod model;
pub(crate) mod numeric_semantics;
pub(crate) mod qc_morphism;
pub(crate) mod queries;
pub(crate) mod semantics_path;
pub(crate) mod wire;

pub use circuit_box::*;
pub use exact_semantics::*;
pub use gate_unitary::*;
pub use induced_dag::*;
pub use model::*;
pub use qc_morphism::*;
pub use queries::*;
pub use semantics_path::*;
pub use wire::*;
