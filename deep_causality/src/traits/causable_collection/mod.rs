/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::traits::causable_collection::collection_explaining::CausableCollectionExplaining;
use crate::{Causable, CausableCollectionReasoning};

pub mod collection_accessor;
pub mod collection_explaining;
pub mod collection_reasoning;

pub trait CausableCollection<T>:
    CausableCollectionExplaining<T> + CausableCollectionReasoning<T>
where
    T: Causable,
{
}
