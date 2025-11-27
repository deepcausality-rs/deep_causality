#![no_std]
extern crate alloc;

mod errors;
mod extensions;
mod traits;
mod types;

// Re-export extensions
pub use extensions::bounded_adjunction::CausalTopologyWitness as BoundedAdjacencyWitness;
pub use extensions::bounded_adjunction::ChainWitness;
pub use extensions::bounded_comonad::CausalTopologyWitness as BoundedComonadWitness;

// Re-export types
pub use types::chain::Chain;
pub use types::regge_geometry::ReggeGeometry;
pub use types::simplex::Simplex;
pub use types::simplicial_complex::SimplicialComplex;
pub use types::skeleton::Skeleton;
pub use types::topology::Topology;
