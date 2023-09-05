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

    fn extra_ctx_add_node(
        &mut self,
        value: Contextoid<D, S, T, ST, V>,
    ) -> Result<usize, ContextIndexError> {
        return match self.get_extra_default_context() {
            Ok(ctx) => Ok(ctx.add_node(value)),
            Err(e) => Err(e),
        };
    }

    fn extra_ctx_contains_node(&mut self, index: usize) -> Result<bool, ContextIndexError> {
        return match self.get_extra_default_context() {
            Ok(ctx) => Ok(ctx.contains_node(index)),
            Err(e) => Err(e),
        };
    }

    // Fix
    // fn extra_ctx_get_node(
    //     &mut self,
    //     index: usize,
    // ) -> Result<&Contextoid<D, S, T, ST, V>, ContextIndexError> {
    //     return match self.get_extra_default_context()
    //     {
    //         Ok(ctx) => match ctx.get_node(index) {
    //             Some(node) => Ok(node),
    //             None => Err(ContextIndexError::new(format!(
    //                 "node {} does not exist",
    //                 index
    //             ))),
    //         },
    //         Err(e) => Err(e),
    //     };
    // }

    fn extra_ctx_remove_node(&mut self, index: usize) -> Result<(), ContextIndexError> {
        return match self.get_extra_default_context() {
            Ok(ctx) => match ctx.remove_node(index) {
                Ok(()) => Ok(()),
                Err(e) => Err(ContextIndexError::new(e.to_string())),
            },
            Err(e) => Err(e),
        };
    }
}

impl<'l, D, S, T, ST, V> Context<'l, D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default + Add<V, Output = V> + Sub<V, Output = V> + Mul<V, Output = V>,
{
    fn get_extra_default_context(
        &mut self,
    ) -> Result<&mut UltraGraph<Contextoid<D, S, T, ST, V>>, ContextIndexError> {
        if self.extra_context_id == 0 {
            return Err(ContextIndexError::new("context ID not set".into()));
        }

        if self.check_extra_context_exists(self.extra_context_id) {
            return Err(ContextIndexError::new("context does not exists".into()));
        }

        let ctx = self
            .extra_contexts
            .as_mut()
            .unwrap()
            .get_mut(&self.extra_context_id);

        return match ctx {
            None => Err(ContextIndexError::new("context does not exists".into())),
            Some(ctx) => Ok(ctx),
        };
    }
}
