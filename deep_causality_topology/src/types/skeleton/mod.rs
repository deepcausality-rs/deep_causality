use crate::types::simplex::Simplex;
use alloc::vec::Vec;

/// A collection of all simplices of dimension K.
pub struct Skeleton {
    pub(crate) dim: usize,
    /// Canonical list of simplices. The index in this vector is the "Global ID".
    pub(crate) simplices: Vec<Simplex>,
}

impl Skeleton {
    pub fn new(dim: usize, simplices: Vec<Simplex>) -> Self {
        Self { dim, simplices }
    }
}

impl Skeleton {
    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn simplices(&self) -> &Vec<Simplex> {
        &self.simplices
    }
}

impl Skeleton {
    /// Find the global index of a simplex via binary search.
    pub fn get_index(&self, simplex: &Simplex) -> Option<usize> {
        self.simplices.binary_search(simplex).ok()
    }
}
