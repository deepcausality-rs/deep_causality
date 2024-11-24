// SPDX-License-Identifier: MIT
// Copyright (c) "2023" The DeepCausality Authors. All Rights Reserved.

use super::*;

impl<D, S, T, ST, V> ExtendableContextuableGraph<D, S, T, ST, V> for Context<D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    fn extra_ctx_add_new(&mut self, capacity: usize, default: bool) -> u64 {
        if self.extra_contexts.is_none() {
            self.extra_contexts = Some(HashMap::new());
        }

        let new_context = ultragraph::new_with_matrix_storage(capacity);

        self.number_of_extra_contexts += 1;

        self.extra_contexts
            .as_mut()
            .expect("Failed get a mutable reference to extra_contexts")
            .insert(self.number_of_extra_contexts, new_context);

        if default {
            self.extra_context_id = self.number_of_extra_contexts;
        }

        self.number_of_extra_contexts
    }

    fn extra_ctx_check_exists(&self, idx: u64) -> bool {
        idx <= self.number_of_extra_contexts
    }

    fn extra_ctx_get_current_id(&self) -> u64 {
        self.extra_context_id
    }

    fn extra_ctx_set_current_id(&mut self, idx: u64) -> Result<(), ContextIndexError> {
        if !self.extra_ctx_check_exists(idx) {
            return Err(ContextIndexError::new("context does not exists".into()));
        }

        self.extra_context_id = idx;

        Ok(())
    }

    fn extra_ctx_unset_current_id(&mut self) -> Result<(), ContextIndexError> {
        self.extra_context_id = 0;

        Ok(())
    }

    fn extra_ctx_add_node(
        &mut self,
        value: Contextoid<D, S, T, ST, V>,
    ) -> Result<usize, ContextIndexError> {
        return match self.get_current_extra_context_mut() {
            Ok(ctx) => Ok(ctx.add_node(value)),
            Err(e) => Err(e),
        };
    }

    fn extra_ctx_contains_node(&self, index: usize) -> bool {
        return match self.get_current_extra_context() {
            Ok(ctx) => ctx.contains_node(index),
            Err(_) => false,
        };
    }

    fn extra_ctx_get_node(
        &self,
        index: usize,
    ) -> Result<&Contextoid<D, S, T, ST, V>, ContextIndexError> {
        return match self.get_current_extra_context() {
            Ok(ctx) => match ctx.get_node(index) {
                Some(node) => Ok(node),
                None => Err(ContextIndexError::new(format!(
                    "node {} does not exist",
                    index
                ))),
            },
            Err(e) => Err(e),
        };
    }

    fn extra_ctx_remove_node(&mut self, index: usize) -> Result<(), ContextIndexError> {
        return match self.get_current_extra_context_mut() {
            Ok(ctx) => match ctx.remove_node(index) {
                Ok(()) => Ok(()),
                Err(e) => Err(ContextIndexError::new(e.to_string())),
            },
            Err(e) => Err(e),
        };
    }

    fn extra_ctx_add_edge(
        &mut self,
        a: usize,
        b: usize,
        weight: RelationKind,
    ) -> Result<(), ContextIndexError> {
        if !self.extra_ctx_contains_node(a) {
            return Err(ContextIndexError(format!("index a {} not found", a)));
        };

        if !self.extra_ctx_contains_node(b) {
            return Err(ContextIndexError(format!("index b {} not found", b)));
        };

        return match self.get_current_extra_context_mut() {
            Ok(ctx) => match ctx.add_edge_with_weight(a, b, weight as u64) {
                Ok(()) => Ok(()),
                Err(e) => Err(ContextIndexError::new(e.to_string())),
            },
            Err(e) => Err(e),
        };
    }

    fn extra_ctx_contains_edge(&self, a: usize, b: usize) -> bool {
        if !self.extra_ctx_contains_node(a) {
            return false;
        };

        if !self.extra_ctx_contains_node(b) {
            return false;
        };

        return match self.get_current_extra_context() {
            Ok(ctx) => ctx.contains_edge(a, b),
            Err(_) => false,
        };
    }

    fn extra_ctx_remove_edge(&mut self, a: usize, b: usize) -> Result<(), ContextIndexError> {
        if !self.extra_ctx_contains_node(a) {
            return Err(ContextIndexError("index a not found".into()));
        };

        if !self.extra_ctx_contains_node(b) {
            return Err(ContextIndexError("index b not found".into()));
        };

        return match self.get_current_extra_context_mut() {
            Ok(ctx) => match ctx.remove_edge(a, b) {
                Ok(()) => Ok(()),
                Err(e) => Err(ContextIndexError::new(e.to_string())),
            },
            Err(e) => Err(e),
        };
    }

    fn extra_ctx_size(&self) -> Result<usize, ContextIndexError> {
        return match self.get_current_extra_context() {
            Ok(ctx) => Ok(ctx.size()),
            Err(e) => Err(e),
        };
    }

    fn extra_ctx_is_empty(&self) -> Result<bool, ContextIndexError> {
        return match self.get_current_extra_context() {
            Ok(ctx) => Ok(ctx.is_empty()),
            Err(e) => Err(e),
        };
    }

    fn extra_ctx_node_count(&self) -> Result<usize, ContextIndexError> {
        return match self.get_current_extra_context() {
            Ok(ctx) => Ok(ctx.number_nodes()),
            Err(e) => Err(e),
        };
    }

    fn extra_ctx_edge_count(&self) -> Result<usize, ContextIndexError> {
        return match self.get_current_extra_context() {
            Ok(ctx) => Ok(ctx.number_edges()),
            Err(e) => Err(e),
        };
    }
}

impl<'l, D, S, T, ST, V> Context<D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    fn get_current_extra_context(
        &self,
    ) -> Result<&ExtraContext<D, S, T, ST, V>, ContextIndexError> {
        if self.extra_context_id == 0 {
            return Err(ContextIndexError::new("context ID not set".into()));
        }

        if !self.extra_ctx_check_exists(self.extra_context_id) {
            return Err(ContextIndexError::new("context does not exists".into()));
        }

        let ctx = self
            .extra_contexts
            .as_ref()
            .expect("Failed to get a reference to extra_contexts")
            .get(&self.extra_context_id);

        match ctx {
            None => Err(ContextIndexError::new("context does not exists".into())),
            Some(ctx) => Ok(ctx),
        }
    }

    fn get_current_extra_context_mut(
        &mut self,
    ) -> Result<&mut ExtraContext<D, S, T, ST, V>, ContextIndexError> {
        if self.extra_context_id == 0 {
            return Err(ContextIndexError::new("context ID not set".into()));
        }

        if !self.extra_ctx_check_exists(self.extra_context_id) {
            return Err(ContextIndexError::new("context does not exists".into()));
        }

        let ctx = self
            .extra_contexts
            .as_mut()
            .expect("Failed to get a reference to extra_contexts")
            .get_mut(&self.extra_context_id);

        match ctx {
            None => Err(ContextIndexError::new("context does not exists".into())),
            Some(ctx) => Ok(ctx),
        }
    }
}
