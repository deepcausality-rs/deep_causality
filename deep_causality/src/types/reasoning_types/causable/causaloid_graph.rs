// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::collections::HashMap;
use std::fmt::Debug;

use petgraph::algo::astar;
use petgraph::Directed;
use petgraph::graph::NodeIndex as GraphNodeIndex;
use petgraph::matrix_graph::MatrixGraph;
use petgraph::prelude::EdgeRef;

use crate::prelude::*;
use crate::protocols::causable::Causable;
use crate::protocols::causable_graph::CausableGraph;

type DefaultIx = u32;

pub type NodeIndex<Ix = DefaultIx> = GraphNodeIndex<Ix>;


#[derive(Clone)]
pub struct CausaloidGraph<T>
    where
        T: Causable + Clone + PartialEq,
{
    root_index: NodeIndex,
    // Edge weights need to be numerical (u64) to make shortest path algo work.
    causaloid_graph: MatrixGraph<T, u64, Directed, Option<u64>, u32>,
    causes_map: HashMap<NodeIndex, T>,
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
            causaloid_graph: MatrixGraph::default(),
            causes_map: HashMap::new(),
        }
    }

    pub fn new_with_capacity(
        capacity: usize
    )
        -> Self
    {
        Self {
            root_index: NodeIndex::new(0),
            causaloid_graph: MatrixGraph::default(),
            causes_map: HashMap::with_capacity(capacity),
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
        -> NodeIndex
    {
        let root_index = self.add_causaloid(value);
        self.root_index = root_index;
        root_index
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

    fn get_root_index(&self) -> Option<NodeIndex> {
        if self.contains_root_causaloid() {
            Some(self.root_index)
        } else {
            None
        }
    }

    fn add_causaloid(
        &mut self,
        value: T,
    )
        -> NodeIndex
    {
        let node_index = self.causaloid_graph.add_node(value.clone());

        self.causes_map.insert(node_index, value);

        node_index
    }

    fn contains_causaloid(
        &self,
        index: NodeIndex,
    )
        -> bool
    {
        self.causes_map.contains_key(&index)
    }

    fn get_causaloid(
        &self,
        index: NodeIndex,
    )
        -> Option<&T>
    {
        self.causes_map.get(&index)
    }

    fn remove_causaloid(
        &mut self,
        index: NodeIndex,
    )
    {
        self.causaloid_graph.remove_node(index);
        self.causes_map.remove(&index);
    }

    fn add_edge(
        &mut self,
        a: NodeIndex,
        b: NodeIndex,
    )
    {
        self.causaloid_graph.add_edge(a, b, 0);
    }

    fn add_edg_with_weight(
        &mut self,
        a: NodeIndex,
        b: NodeIndex,
        weight: u64)
    {
        self.causaloid_graph.add_edge(a, b, weight);
    }

    fn contains_edge(
        &self,
        a: NodeIndex,
        b: NodeIndex,
    )
        -> bool
    {
        self.causaloid_graph.has_edge(a, b)
    }

    fn remove_edge(
        &mut self,
        a: NodeIndex,
        b: NodeIndex,
    )
    {
        self.causaloid_graph.remove_edge(a, b);
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
        self.causaloid_graph.clear();
        self.causes_map.clear();
    }

    fn edge_count(
        &self
    )
        -> usize
    {
        self.causaloid_graph.edge_count()
    }

    fn node_count(
        &self
    )
        -> usize
    {
        self.causaloid_graph.node_count()
    }
}


impl<T> CausableGraphReasoning<T> for CausaloidGraph<T>
    where
        T: Causable + Clone + PartialEq,
{
    fn explain_all_causes(
        &self
    )
        -> Result<String, CausalityGraphError>
    {
        let start_index = self.root_index;
        let stop_index = NodeIndex::new(self.causes_map.len());

        match self.explain_from_to_cause(start_index, stop_index) {
            Ok(explanation) => Ok(explanation),
            Err(e) => Err(e)
        }
    }

    fn explain_subgraph_from_cause(
        &self,
        start_index: NodeIndex,
    )
        -> Result<String, CausalityGraphError>
    {
        let stop_index = NodeIndex::new(self.causes_map.len());
        match self.explain_from_to_cause(start_index, stop_index) {
            Ok(explanation) => Ok(explanation),
            Err(e) => Err(e)
        }
    }

    fn explain_shortest_path_between_causes(
        &self,
        start_index: NodeIndex,
        stop_index: NodeIndex,
    )
        -> Result<String, CausalityGraphError>
    {
        match self.explain_shortest_path_from_to_cause(start_index, stop_index) {
            Ok(explanation) => Ok(explanation),
            Err(e) => Err(e)
        }
    }

    fn reason_all_causes(
        &self,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    )
        -> Result<bool, CausalityGraphError>
    {
        if self.contains_root_causaloid()
        {
            let start_index = self.root_index;
            let stop_index = NodeIndex::new(self.causes_map.len());

            match self.reason_from_to_cause(start_index, stop_index, data, data_index) {
                Ok(result) => Ok(result),
                Err(e) => Err(e)
            }
        } else {
            Err(CausalityGraphError("Graph does not contains root causaloid".into()))
        }
    }

    fn reason_subgraph_from_cause(
        &self,
        start_index: NodeIndex,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    )
        -> Result<bool, CausalityGraphError>
    {
        let stop_index = NodeIndex::new(self.causes_map.len());
        match self.reason_from_to_cause(start_index, stop_index, data, data_index) {
            Ok(result) => Ok(result),
            Err(e) => Err(e)
        }
    }

    fn reason_shortest_path_between_causes(
        &self,
        start_index: NodeIndex,
        stop_index: NodeIndex, data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    )
        -> Result<bool, CausalityGraphError>
    {
        match self.reason_shortest_path_from_to_cause(
            start_index,
            stop_index,
            data,
            data_index)
        {
            Ok(result) => Ok(result),
            Err(e) => Err(e)
        }
    }

    fn reason_single_cause(
        &self,
        index: NodeIndex,
        data: &[NumericalValue],
    )
        -> Result<bool, CausalityGraphError>
    {
        if self.is_empty()
        {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        if !self.contains_causaloid(index)
        {
            return Err(CausalityGraphError("Graph does not contain causaloid".to_string()));
        }

        if data.is_empty()
        {
            return Err(CausalityGraphError("Data are empty (len ==0).".into()));
        }

        let causaloid = self.get_causaloid(index)
            .expect("Failed to get causaloid");

        self.verify_cause(causaloid, data)
    }
}


impl<T> CausaloidGraph<T>
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

        if !self.contains_causaloid(start_index) {
            return Err(CausalityGraphError("Graph does not contains start causaloid".into()));
        }

        if !self.contains_causaloid(stop_index) {
            return Err(CausalityGraphError("Graph does not contains stop causaloid".into()));
        }

        let mut stack = Vec::with_capacity(self.causes_map.len());
        let mut explanation = String::new();

        let cause = self.get_causaloid(start_index).expect("Failed to get causaloid");

        append_string(&mut explanation, &cause.explain().unwrap());

        stack.push(self.causaloid_graph.neighbors(start_index));

        while let Some(children) = stack.last_mut() {
            if let Some(child) = children.next() {
                let cause = self.get_causaloid(child)
                    .expect("Failed to get causaloid");

                append_string(&mut explanation, &cause.explain().unwrap());

                if child == stop_index {
                    return Ok(explanation);
                } else {
                    stack.push(self.causaloid_graph.neighbors(child));
                }
            } else {
                stack.pop();
            }
        }

        Ok(explanation)
    }

    fn explain_shortest_path_from_to_cause(
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

        if !self.contains_causaloid(start_index) {
            return Err(CausalityGraphError("Graph does not contains start causaloid".into()));
        }

        if !self.contains_causaloid(stop_index) {
            return Err(CausalityGraphError("Graph does not contains stop causaloid".into()));
        }

        let shortest_path = match self.get_shortest_path(start_index, stop_index) {
            Ok(shortest_path) => shortest_path,
            Err(e) => return Err(e)
        };

        let mut explanation = String::new();

        for index in shortest_path {
            let cause = self.get_causaloid(index)
                .expect("Failed to get causaloid");

            append_string(&mut explanation, &cause.explain().unwrap());
        }

        Ok(explanation)
    }

    // Algo inspired by simple path
    // https://github.com/petgraph/petgraph/blob/master/src/algo/simple_paths.rs
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

        if !self.contains_causaloid(start_index)
        {
            return Err(CausalityGraphError("Graph does not contains start causaloid".into()));
        }

        if data.is_empty()
        {
            return Err(CausalityGraphError("Data are empty (len ==0).".into()));
        }

        let cause = self.get_causaloid(start_index).expect("Failed to get causaloid");

        let obs = self.get_obs(cause.id(), data, &data_index);

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
        stack.push(self.causaloid_graph.neighbors(start_index));

        while let Some(children) = stack.last_mut()
        {
            if let Some(child) = children.next()
            {
                let cause = self.get_causaloid(child)
                    .expect("Failed to get causaloid");

                let obs = self.get_obs(cause.id(), data, &data_index);

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
                    stack.push(self.causaloid_graph.neighbors(child));
                }
            } else {
                stack.pop();
            }
        }

        // If all of the previous nodes evaluated to true,
        // then all nodes must be true, hence return true.
        Ok(true)
    }

    fn reason_shortest_path_from_to_cause(
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

        if !self.contains_causaloid(start_index) {
            return Err(CausalityGraphError("Graph does not contains start causaloid".into()));
        }

        if !self.contains_causaloid(stop_index) {
            return Err(CausalityGraphError("Graph does not contains stop causaloid".into()));
        }

        let shortest_path = match self.get_shortest_path(start_index, stop_index) {
            Ok(shortest_path) => shortest_path,
            Err(e) => return Err(e)
        };

        for index in shortest_path {
            let cause = self.get_causaloid(index)
                .expect("Failed to get causaloid");

            let obs = self.get_obs(cause.id(), data, &data_index);

            let res = match cause.verify_single_cause(&obs)
            {
                Ok(res) => res,
                Err(e) => return Err(CausalityGraphError(e.0)),
            };

            if !res
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn get_shortest_path(
        &self,
        start_index: NodeIndex,
        stop_index: NodeIndex,
    )
        -> Result<Vec<NodeIndex>, CausalityGraphError>
    {
        // A* algorithm
        // https://docs.rs/petgraph/latest/petgraph/algo/astar/fn.astar.html
        let (_, path) = astar(&self.causaloid_graph,
                              start_index,
                              |finish| finish == stop_index,
                              |e| *e.weight(),
                              |_| 0)
            .expect("Could not find shortest path");

        Ok(path)
    }

    fn verify_cause(
        &self,
        causaloid: &impl Causable,
        data: &[NumericalValue],
    )
        -> Result<bool, CausalityGraphError>
    {
        if data.is_empty()
        {
            return Err(CausalityGraphError("Data are empty (len=0)".into()));
        }

        if data.len() == 1
        {
            let obs = data.first()
                .expect("Failed to get data");

            return match causaloid.verify_single_cause(obs) {
                Ok(res) => Ok(res),
                Err(e) => Err(CausalityGraphError(e.0)),
            };
        }

        if data.len() > 1
        {
            for obs in data.iter() {
                if !causaloid.verify_single_cause(obs)
                    .expect("Failed to verify data") {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    fn get_obs<'a>(
        &self,
        cause_id: IdentificationValue,
        data: &'a [NumericalValue],
        data_index: &'a Option<&HashMap<IdentificationValue, IdentificationValue>>,
    )
        -> NumericalValue
    {
        let obs = if data_index.is_some()
        {
            let idx = data_index.unwrap().get(&cause_id)
                .expect("Failed to get data index");

            let index = idx.to_owned() as usize;
            data.get(index)
                .expect("Failed to get data")
        } else {
            let index = cause_id as usize;
            data.get(index)
                .expect("Failed to get data")
        };

        obs.to_owned()
    }
}


fn append_string<'l>(
    s1: &'l mut String,
    s2: &'l str,
)
    -> &'l str
{
    s1.push('\n');
    s1.push_str(format!(" * {}", s2).as_str());
    s1.push('\n');

    s1
}
