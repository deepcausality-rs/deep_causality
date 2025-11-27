use alloc::vec::Vec;
use core::slice::SliceIndex;

mod display;
mod getters;

/// A combinatorial simplex defined by sorted vertex indices.
/// Order is strictly increasing to ensure canonical representation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Simplex {
    pub(crate) vertices: Vec<usize>,
}

impl Simplex {
    pub fn new(vertices: Vec<usize>) -> Self {
        Self { vertices }
    }
}

impl Simplex {
    /// Returns a sub-simplex defined by the given range of vertices.
    pub fn subsimplex<R>(&self, range: R) -> Self
    where
        R: SliceIndex<[usize], Output = [usize]>,
    {
        Simplex {
            vertices: self.vertices[range].to_vec(),
        }
    }
}
