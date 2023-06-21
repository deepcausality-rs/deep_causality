/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */


use crate::prelude::{Causable, NodeIndex, NumericalValue};

pub trait CausableGraph<T>
    where
        T: Causable + PartialEq,
{
    // Root Node
    fn add_root_causaloid(&mut self, value: T) -> NodeIndex;
    fn contains_root_causaloid(&self) -> bool;
    fn get_root_causaloid(&self) -> Option<&T>;
    fn get_root_index(&self) -> Option<NodeIndex>;

    // Nodes
    fn add_causaloid(&mut self, value: T) -> NodeIndex;
    fn contains_causaloid(&self, index: NodeIndex) -> bool;
    fn get_causaloid(&self, index: NodeIndex) -> Option<&T>;
    fn remove_causaloid(&mut self, index: NodeIndex);

    // Edges
    fn add_edge(&mut self, a: NodeIndex, b: NodeIndex);
    fn add_edg_with_weight(&mut self, a: NodeIndex, b: NodeIndex, weight: u64);

    fn contains_edge(&self, a: NodeIndex, b: NodeIndex) -> bool;
    fn remove_edge(&mut self, a: NodeIndex, b: NodeIndex);

    // Utils
    fn all_active(&self) -> bool;
    fn number_active(&self) -> NumericalValue;
    fn percent_active(&self) -> NumericalValue;
    fn size(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn clear(&mut self);
    fn count_edges(&self) -> usize;
    fn count_nodes(&self) -> usize;
}
