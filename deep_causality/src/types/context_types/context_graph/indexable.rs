// SPDX-License-Identifier: MIT
// Copyright (c) "2024" . The DeepCausality Authors. All Rights Reserved.

use std::ops::{Add, Mul, Sub};

use crate::prelude::{Context, Datable, Indexable, SpaceTemporal, Spatial, Temporable};

impl<'l, D, S, T, ST, V> Indexable for Context<'l, D, S, T, ST, V>
    where
        D: Datable,
        S: Spatial<V>,
        T: Temporable<V>,
        ST: SpaceTemporal<V>,
        V: Default + Add<V, Output=V> + Sub<V, Output=V> + Mul<V, Output=V>,
{
    fn get_index(&self, key: &usize, current: bool) -> Option<&usize> {
        {
            if current {
                self.current_index_map.get(&key)
            } else {
                self.previous_index_map.get(&key)
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
