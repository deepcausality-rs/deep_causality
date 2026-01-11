/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
extern crate core;

mod errors;
mod extensions;
mod traits;
mod types;

pub use crate::types::backend;
pub mod alias;
pub mod utils_tests;

// Re-export errors
pub use crate::errors::link_variable_error::LinkVariableError;
pub use crate::errors::topology_error::{TopologyError, TopologyErrorEnum};

// Re-export extensions
pub use crate::extensions::hkt_cell_complex::CellComplexWitness;
pub use crate::extensions::hkt_graph::GraphWitness;
pub use crate::extensions::hkt_hypergraph::HypergraphWitness;
pub use crate::extensions::hkt_lattice::LatticeWitness;
pub use crate::extensions::hkt_manifold::ManifoldWitness;
pub use crate::extensions::hkt_point_cloud::PointCloudWitness;
pub use crate::extensions::hkt_simplicial_complex::ChainWitness;
pub use crate::extensions::hkt_topology::TopologyWitness;

// Re-export gauge field HKT extensions
pub use crate::extensions::hkt_gauge::hkt_curvature::{CurvatureTensorWitness, TensorVector};
pub use crate::extensions::hkt_gauge::hkt_gauge_witness::{GaugeFieldHKT, GaugeFieldWitness};
pub use crate::extensions::hkt_gauge::hkt_lattice_gauge::LatticeGaugeFieldWitness;
pub use extensions::hkt_gauge::hkt_adjunction_stokes::{
    BoundaryWitness, ExteriorDerivativeWitness, StokesAdjunction, StokesContext,
};
// Re-export traits
pub use crate::traits::base_topology::BaseTopology;
pub use crate::traits::cw_complex::{CWComplex, Cell};
pub use crate::traits::gauge_group::GaugeGroup;
pub use crate::traits::graph_topology::GraphTopology;
pub use crate::traits::hypergraph_topology::HypergraphTopology;
pub use crate::traits::manifold_topology::ManifoldTopology;
pub use crate::traits::simplicial_topology::SimplicialTopology;

// Re-export types
pub use crate::types::cell_complex::{BoundaryOperator, CellComplex};
pub use crate::types::chain::Chain;
pub use crate::types::graph::Graph;
pub use crate::types::hypergraph::Hypergraph;
pub use crate::types::lattice::dual_lattice::DualLattice;
pub use crate::types::lattice::specialized::{
    HeavyHexLattice, HoneycombLattice, KagomeLattice, TriangularLattice,
};
pub use crate::types::lattice::{Lattice, LatticeCell};
pub use crate::types::manifold::Manifold;
pub use crate::types::point_cloud::PointCloud;
pub use crate::types::regge_geometry::ReggeGeometry;
pub use crate::types::simplex::Simplex;
pub use crate::types::simplicial_complex::{SimplicialComplex, SimplicialComplexBuilder};
pub use crate::types::skeleton::Skeleton;
pub use crate::types::topology::Topology;

// Re-export gauge field types
pub use crate::types::curvature_tensor::{
    CurvatureSymmetry, CurvatureTensor, CurvatureTensorVector,
};
pub use crate::types::differential_form::DifferentialForm;
pub use crate::types::gauge::gauge_groups::{Electroweak, Lorentz, SU2, SU3, StandardModel, U1};
pub use crate::types::gauge::{GaugeField, LatticeGaugeField, LinkVariable};
