// SPDX-License-Identifier: MIT
// Copyright (c) "2024" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{Context, Datable, Indexable, Symbolic};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;

impl<D, S, T, ST, SYM, V> Indexable for Context<D, S, T, ST, SYM, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporal<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    fn get_index(&self, key: &usize, current: bool) -> Option<&usize> {
        {
            if current {
                self.current_index_map.get(key)
            } else {
                self.previous_index_map.get(key)
            }
        }
    }

    fn set_index(&mut self, key: usize, index: usize, current: bool) {
        {
            if current {
                self.current_index_map.insert(key, index);
            } else {
                self.previous_index_map.insert(key, index);
            }
        }
    }
}
