// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::collections::HashMap;
use std::fmt::Debug;
use petgraph::matrix_graph::MatrixGraph;
use crate::prelude::*;
use crate::utils::reasoning_utils;


#[derive(Clone)]
pub struct CausaloidGraph<T>
    where
        T: Causable + Clone + PartialEq,
{
    root_index: NodeIndex,
    graph: CausalGraph<T>,
    causes_map: HashMap<NodeIndex, T>,
    index_map: IndexMap,
}


impl<T> CausaloidGraph<T>
    where
        T: Causable + Clone + PartialEq,
{
    pub fn new()
        -> Self
    {
        Self {
            root_index: NodeIndex::new(0),
            graph: MatrixGraph::default(),
            causes_map: HashMap::new(),
            index_map: HashMap::new(),
        }
    }

    pub fn new_with_capacity(
        capacity: usize
    )
        -> Self
    {
        Self {
            root_index: NodeIndex::new(0),
            graph: MatrixGraph::with_capacity(capacity),
            causes_map: HashMap::with_capacity(capacity),
            index_map: HashMap::with_capacity(capacity),
        }
    }
}

impl<T> Default for CausaloidGraph<T>
    where
        T: Debug + Causable + Clone + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CausableGraph<T> for CausaloidGraph<T>
    where
        T: Causable + Clone + PartialEq,
{
    fn add_root_causaloid(
        &mut self,
        value: T,
    )
        -> usize
    {
        let idx = self.add_causaloid(value);
        let root_index = NodeIndex::new(idx);
        self.root_index = root_index;
        self.index_map.insert(root_index.index(), root_index);

        root_index.index()
    }

    fn contains_root_causaloid(
        &self
    )
        -> bool
    {
        self.causes_map.contains_key(&self.root_index)
    }

    fn get_root_causaloid(&self) -> Option<&T> {
        self.causes_map.get(&self.root_index)
    }

    fn get_root_index(&self) -> Option<usize> {
        if self.contains_root_causaloid() {
            Some(self.root_index.index())
        } else {
            None
        }
    }

    fn get_last_index(&self)
                      -> Result<usize, CausalityGraphError>
    {
        if !self.is_empty() {
            Ok(self.causes_map.len() - 1)
        } else {
            Err(CausalityGraphError("Graph is empty".to_string()))
        }
    }

    fn add_causaloid(
        &mut self,
        value: T,
    )
        -> usize
    {
        let node_index = self.graph.add_node(value.clone());

        self.causes_map.insert(node_index, value);
        self.index_map.insert(node_index.index(), node_index);

        node_index.index()
    }

    fn contains_causaloid(
        &self,
        index: usize,
    )
        -> bool
    {
        self.index_map.get(&index).is_some()
    }

    fn get_causaloid(
        &self,
        index: usize,
    )
        -> Option<&T>
    {
        if !self.contains_causaloid(index) {
            None
        } else {
            let k = self.index_map.get(&index).expect("index not found");
            self.causes_map.get(k)
        }
    }

    fn remove_causaloid(
        &mut self,
        index: usize,
    )
        -> Result<(), CausalGraphIndexError>
    {
        if !self.contains_causaloid(index) {
            return Err(CausalGraphIndexError(format!("index not found: {}", index)));
        };

        let k = self.index_map.get(&index).unwrap();
        self.graph.remove_node(*k);
        self.causes_map.remove(k);

        self.index_map.remove(&index);

        Ok(())
    }

    fn add_edge(
        &mut self,
        a: usize,
        b: usize,
    )
        -> Result<(), CausalGraphIndexError>
    {
        if !self.contains_causaloid(a) {
            return Err(CausalGraphIndexError(format!("index a {} not found", a)));
        };

        if !self.contains_causaloid(b) {
            return Err(CausalGraphIndexError(format!("index b {} not found", b)));
        };

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");

        self.graph.add_edge(*k, *l, 0);

        Ok(())
    }

    fn add_edg_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    )
        -> Result<(), CausalGraphIndexError>
    {
        if !self.contains_causaloid(a) {
            return Err(CausalGraphIndexError(format!("index a {} not found", a)));
        };

        if !self.contains_causaloid(b) {
            return Err(CausalGraphIndexError(format!("index b {} not found", b)));
        };

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");

        self.graph.add_edge(*k, *l, weight);

        Ok(())
    }

    fn contains_edge(
        &self,
        a: usize,
        b: usize,
    )
        -> bool
    {
        if !self.contains_causaloid(a) {
            return false;
        };

        if !self.contains_causaloid(b) {
            return false;
        };

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");

        self.graph.has_edge(*k, *l)
    }

    fn remove_edge(
        &mut self,
        a: usize,
        b: usize,
    )
        -> Result<(), CausalGraphIndexError>
    {
        if !self.contains_causaloid(a) {
            return Err(CausalGraphIndexError("index a not found".into()));
        };

        if !self.contains_causaloid(b) {
            return Err(CausalGraphIndexError("index b not found".into()));
        };

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");

        self.graph.remove_edge(*k, *l);

        Ok(())
    }

    fn all_active(
        &self
    )
        -> bool
    {
        for (_, cause) in self.causes_map.iter() {
            if !cause.is_active() {
                return false;
            }
        }

        true
    }

    fn number_active(
        &self
    )
        -> NumericalValue
    {
        self.causes_map.iter().filter(|(_, c)| c.is_active()).count() as NumericalValue
    }

    fn percent_active(
        &self
    )
        -> NumericalValue
    {
        let count = self.number_active();
        let total = self.size() as NumericalValue;
        (count / total) * (100 as NumericalValue)
    }

    fn size(
        &self
    )
        -> usize
    {
        self.causes_map.len()
    }

    fn is_empty(
        &self
    )
        -> bool
    {
        self.causes_map.is_empty()
    }

    fn clear(
        &mut self
    )
    {
        self.graph.clear();
        self.causes_map.clear();
    }

    fn number_edges(
        &self
    )
        -> usize
    {
        self.graph.edge_count()
    }

    fn number_nodes(
        &self
    )
        -> usize
    {
        self.graph.node_count()
    }

    fn get_graph(&self) -> &CausalGraph<T> {
        &self.graph
    }
}

impl<T> CausableGraphExplaining<T> for CausaloidGraph<T>
    where
        T: Causable + Clone + PartialEq,
{
    fn explain_from_to_cause(
        &self,
        start_index: NodeIndex,
        stop_index: NodeIndex,
    )
        -> Result<String, CausalityGraphError>
    {
        if self.is_empty()
        {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        if !self.contains_causaloid(start_index.index()) {
            return Err(CausalityGraphError("Graph does not contains start causaloid".into()));
        }

        if !self.contains_causaloid(stop_index.index()) {
            return Err(CausalityGraphError("Graph does not contains stop causaloid".into()));
        }

        let mut stack = Vec::with_capacity(self.causes_map.len());
        let mut explanation = String::new();

        let cause = self.get_causaloid(start_index.index()).expect("Failed to get causaloid");

        reasoning_utils::append_string(&mut explanation, &cause.explain().unwrap());

        stack.push(self.get_graph().neighbors(start_index));

        while let Some(children) = stack.last_mut() {
            if let Some(child) = children.next() {
                let cause = self.get_causaloid(child.index())
                    .expect("Failed to get causaloid");

                reasoning_utils::append_string(&mut explanation, &cause.explain().unwrap());

                if child == stop_index {
                    return Ok(explanation);
                } else {
                    stack.push(self.get_graph().neighbors(child));
                }
            } else {
                stack.pop();
            }
        }

        Ok(explanation)
    }
}

impl<T> CausableGraphReasoning<T> for CausaloidGraph<T>
    where
        T: Causable + Clone + PartialEq,
{
    // Algo inspired by simple path https://github.com/petgraph/petgraph/blob/master/src/algo/simple_paths.rs
    fn reason_from_to_cause(
        &self,
        start_index: NodeIndex,
        stop_index: NodeIndex,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    )
        -> Result<bool, CausalityGraphError>
    {
        if self.is_empty()
        {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        if !self.contains_causaloid(start_index.index())
        {
            return Err(CausalityGraphError("Graph does not contains start causaloid".into()));
        }

        if data.is_empty()
        {
            return Err(CausalityGraphError("Data are empty (len ==0).".into()));
        }

        let cause = self.get_causaloid(start_index.index()).expect("Failed to get causaloid");

        let obs = reasoning_utils::get_obs(cause.id(), data, &data_index);

        let res = match cause.verify_single_cause(&obs)
        {
            Ok(res) => res,
            Err(e) => return Err(CausalityGraphError(e.0)),
        };

        if !res
        {
            return Ok(false);
        }

        let mut stack = Vec::with_capacity(self.causes_map.len());
        stack.push(self.get_graph().neighbors(start_index));

        while let Some(children) = stack.last_mut()
        {
            if let Some(child) = children.next()
            {
                let cause = self.get_causaloid(child.index())
                    .expect("Failed to get causaloid");

                let obs = reasoning_utils::get_obs(cause.id(), data, &data_index);

                let res = if cause.is_singleton()
                {
                    match cause.verify_single_cause(&obs)
                    {
                        Ok(res) => res,
                        Err(e) => return Err(CausalityGraphError(e.0)),
                    }
                } else {
                    match cause.verify_all_causes(data, data_index)
                    {
                        Ok(res) => res,
                        Err(e) => return Err(CausalityGraphError(e.0)),
                    }
                };

                if !res
                {
                    return Ok(false);
                }

                if child == stop_index
                {
                    return Ok(true);
                } else {
                    stack.push(self.graph.neighbors(child));
                }
            } else {
                stack.pop();
            }
        }

        // If all of the previous nodes evaluated to true,
        // then all nodes must be true, hence return true.
        Ok(true)
    }
}
