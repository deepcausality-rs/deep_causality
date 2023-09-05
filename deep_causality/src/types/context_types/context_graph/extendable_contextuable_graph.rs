// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

impl<'l, D, S, T, ST, V> ExtendableContextuableGraph<'l, D, S, T, ST, V>
    for Context<'l, D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    fn add_extra_context(&mut self, capacity: usize, default: bool) -> usize {
        if self.extra_contexts.is_none() {
            self.extra_contexts = Some(HashMap::new());
        }

        let new_context = ultragraph::new_with_matrix_storage(capacity);

        self.number_of_extra_contexts += 1;

        let _ = self
            .extra_contexts
            .as_mut()
            .unwrap()
            .insert(self.number_of_extra_contexts, new_context);

        if default {
            self.extra_context_id = self.number_of_extra_contexts;
        }

        self.number_of_extra_contexts
    }

    fn check_extra_context_exists(&self, idx: usize) -> bool {
        if idx <= self.number_of_extra_contexts {
            true
        } else {
            false
        }
    }

    fn set_extra_default_context(&mut self, idx: usize) -> Result<(), ContextIndexError> {
        if self.check_extra_context_exists(idx) {
            return Err(ContextIndexError::new("context does not exists".into()));
        }

        self.extra_context_id = idx;

        Ok(())
    }

    fn unset_extra_default_context(&mut self) -> Result<(), ContextIndexError> {
        self.extra_context_id = 0;

        Ok(())
    }
}
