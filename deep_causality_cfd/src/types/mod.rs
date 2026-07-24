/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CFD value types: the `CfdScalar` bound, the per-step `Ambient`, the marching
//! state, configuration structs, and the CfdFlow DSL surface types.

mod ambient;
mod evidence_class;
pub mod flow;
pub mod flow_config;
mod keyed_table;
mod ladder_outcome;

pub use ambient::Ambient;
pub use evidence_class::EvidenceClass;
pub use keyed_table::{KeyedInterpolation, KeyedTable};
pub use ladder_outcome::LadderOutcome;
