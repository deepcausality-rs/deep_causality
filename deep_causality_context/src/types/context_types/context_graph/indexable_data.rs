/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    Context, CurrentDataIndex, DataIndexable, Datable, PreviousDataIndex, SpaceTemporal, Spatial,
    Temporal,
};

// Adds the default implementation to Context when the trait is imported. Can be overwritten to customize.
impl<D, S, T, ST> CurrentDataIndex for Context<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
}

// Adds the default implementation to Context when the trait is imported. Can be overwritten to customize.
impl<D, S, T, ST> PreviousDataIndex for Context<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
}

// Adds the set/get index methods used in CurrentDataIndex and PreviousDataIndex.
#[allow(clippy::type_complexity)]
impl<D, S, T, ST> DataIndexable for Context<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    fn get_data_index(&self, key: &usize, current: bool) -> Option<&usize> {
        if current {
            self.current_data_map.get(key)
        } else {
            self.previous_data_map.get(key)
        }
    }

    fn set_data_index(&mut self, key: usize, index: usize, current: bool) {
        if current {
            self.current_data_map.insert(key, index);
        } else {
            self.previous_data_map.insert(key, index);
        }
    }
}
