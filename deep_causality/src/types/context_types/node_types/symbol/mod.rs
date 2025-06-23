use deep_causality_macros::Constructor;
use crate::prelude::{Identifiable, Symbolic, SymbolicRepresentation};

#[derive(Constructor, Debug, Clone, PartialEq)]
pub struct BaseSymbol {
    id: u64,
    repr: SymbolicRepresentation,
}

impl Identifiable for BaseSymbol {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Symbolic for BaseSymbol {
    type Repr = SymbolicRepresentation;

    fn symbol(&self) -> &Self::Repr {
        &self.repr
    }
}

impl Default for BaseSymbol {
    /// Returns a default null symbol, typically representing a root or empty node.
    fn default() -> Self {
        Self {
            id: 0,
            repr: SymbolicRepresentation::new_atom("Default".to_string()),
        }
    }
}