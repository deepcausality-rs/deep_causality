/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod alias;
mod errors;
mod traits;
mod types;
pub mod utils_test;

// Alias
pub use crate::alias::{BaseTeloidStore, TeloidID, TeloidTag};

// Errors
pub use errors::deontic_error::DeonticError;

// Traits
pub use traits::deontic_explainable::DeonticExplainable;
pub use traits::deontic_inferable::DeonticInferable;
pub use traits::teloid_storable::TeloidStorable;
pub use traits::teloidable::Teloidable;

// Types
pub use crate::types::effect_ethos::EffectEthos;
pub use crate::types::tag_index::TagIndex;
pub use crate::types::teloid::Teloid;
pub use crate::types::teloid_graph::TeloidGraph;
pub use crate::types::teloid_modal::TeloidModal;
pub use crate::types::teloid_relation::TeloidRelation;
pub use crate::types::teloid_store::TeloidStore;
pub use crate::types::teloid_verdict::Verdict;
