/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod errors;
mod extensions;
mod traits;
mod types;

pub mod utils_tests;

// Re-export errors
pub use crate::errors::topology_error::TopologyError;

// Re-export extensions
pub use crate::extensions::hkt_graph::GraphWitness;
pub use crate::extensions::hkt_hypergraph::HypergraphWitness;
pub use crate::extensions::hkt_manifold::ManifoldWitness;
pub use crate::extensions::hkt_point_cloud::PointCloudWitness;
pub use crate::extensions::hkt_simplicial_complex::ChainWitness;
pub use crate::extensions::hkt_topology::TopologyWitness;

// Re-export traits
pub use crate::traits::base_topology::BaseTopology;
pub use crate::traits::graph_topology::GraphTopology;
pub use crate::traits::hypergraph_topology::HypergraphTopology;
pub use crate::traits::manifold_topology::ManifoldTopology;
pub use crate::traits::simplicial_topology::SimplicialTopology;

// Re-export types
pub use crate::types::chain::Chain;
pub use crate::types::graph::Graph;
pub use crate::types::hypergraph::Hypergraph;
pub use crate::types::manifold::Manifold;
pub use crate::types::point_cloud::PointCloud;
pub use crate::types::regge_geometry::ReggeGeometry;
pub use crate::types::simplex::Simplex;
pub use crate::types::simplicial_complex::SimplicialComplex;
pub use crate::types::skeleton::Skeleton;
pub use crate::types::topology::Topology;
