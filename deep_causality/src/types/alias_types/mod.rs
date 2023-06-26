// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use petgraph::Directed;
use petgraph::matrix_graph::MatrixGraph as MGraph;

use crate::prelude::{CausalityError, Node, RelationKind};

// Type aliases
pub type IdentificationValue = u64;
pub type NumericalValue = f64;
pub type DescriptionValue = String;

// Fn aliases for assumable, assumption, & assumption collection
pub type EvalFn = fn(&[NumericalValue]) -> bool;
pub type CausalFn = fn(NumericalValue) -> Result<bool, CausalityError>;
pub type ContextMatrixGraph<D, S, T, ST> = MGraph<Node<D, S, T, ST>, RelationKind, Directed, Option<RelationKind>, u32>;
//pub type MatrixGraph<N, E> = petgraph::matrix_graph::MatrixGraph<N, E>;
