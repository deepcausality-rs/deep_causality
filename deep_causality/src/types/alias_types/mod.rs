// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use petgraph::Directed;

use crate::prelude::{CausalityError, Contextoid, RelationKind};

// Type aliases
pub type IdentificationValue = u64;
pub type NumericalValue = f64;
pub type DescriptionValue = String;

// Fn aliases for assumable, assumption, & assumption collection
pub type EvalFn = fn(&[NumericalValue]) -> bool;
pub type CausalFn = fn(NumericalValue) -> Result<bool, CausalityError>;