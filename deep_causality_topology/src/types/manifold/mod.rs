use alloc::string::ToString;

use crate::{ManifoldTopology, SimplicialComplex, TopologyError};
use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;

mod base_topology;
mod clone;
mod display;
mod getters;
mod manifold_topology;
mod simplicial_topology;

/// A newtype wrapper around `SimplicialComplex` that represents a Manifold.
/// Its construction enforces geometric properties essential for physics simulations.
/// The type parameter T represents data living on the manifold's simplices.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifold<T> {
    /// The underlying simplicial complex, guaranteed to satisfy manifold properties.
    pub(crate) complex: SimplicialComplex,
    /// The data associated with the manifold (e.g., scalar field values on simplices)
    pub(crate) data: CausalTensor<T>,
    /// The Focus (Cursor) for Comonadic extraction
    pub(crate) cursor: usize,
}

impl<T> Manifold<T>
where
    T: Default + Copy + Clone + PartialEq + Zero,
{
    /// Attempts to create a new `Manifold` from a `SimplicialComplex` and data.
    /// This constructor performs rigorous checks to ensure the complex satisfies manifold criteria.
    ///
    /// # Errors
    /// Returns `Err(TopologyError::ManifoldError)` if the input `SimplicialComplex`
    /// does not meet the requirements to be classified as a manifold.
    pub fn new(
        complex: SimplicialComplex,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {
        // Validation: Check data size matches complex
        let expected_size = complex.skeletons.iter().map(|s| s.simplices.len()).sum();
        if data.len() != expected_size {
            return Err(TopologyError::InvalidInput(
                "Data size must match total number of simplices in complex".to_string(),
            ));
        }

        if cursor >= data.len() {
            return Err(TopologyError::IndexOutOfBounds(
                "Initial cursor out of bounds for Manifold".to_string(),
            ));
        }

        if !Self::check_is_manifold(&complex) {
            return Err(TopologyError::ManifoldError(
                "SimplicialComplex does not satisfy manifold properties".to_string(),
            ));
        }

        Ok(Self {
            complex,
            data,
            cursor,
        })
    }

    /// Internal helper function to determine if a `SimplicialComplex` is a manifold.
    /// Uses the ManifoldTopology trait methods to perform validation.
    fn check_is_manifold(complex: &SimplicialComplex) -> bool {
        // For validation, we need to use ManifoldTopology methods
        // Create a temporary manifold with default data to access trait methods
        // This is a bit circular, but necessary for validation

        // Basic check: complex must have at least one skeleton
        if complex.skeletons.is_empty() {
            return false;
        }

        // For a proper manifold, we need non-trivial structure
        // At minimum, need vertices (0-skeleton)
        let num_vertices = complex
            .skeletons
            .first()
            .map(|s| s.simplices.len())
            .unwrap_or(0);
        if num_vertices == 0 {
            return false;
        }

        // Create temporary data for validation (just checking structure, not data)
        let temp_data = match CausalTensor::new(vec![0i8; num_vertices], vec![num_vertices]) {
            Ok(d) => d,
            Err(_) => return false,
        };

        // Create temp manifold to access trait methods
        let temp_manifold = Manifold {
            complex: complex.clone(),
            data: temp_data,
            cursor: 0,
        };

        // A manifold must be oriented
        if !temp_manifold.is_oriented() {
            return false;
        }

        // A manifold must satisfy the link condition
        if !temp_manifold.satisfies_link_condition() {
            return false;
        }

        // All checks passed
        true
    }
}
