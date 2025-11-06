/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::traits::causable_collection::collection_explaining::CausableCollectionExplaining;
use crate::{Causable, CausalMonad, MonadicCausable};

pub mod collection_accessor;
pub mod collection_explaining;
pub mod monadic_collection_reasoning;

pub trait CausableCollection<T>: CausableCollectionExplaining<T>
where
    T: Causable + MonadicCausable<CausalMonad>,
{
}
