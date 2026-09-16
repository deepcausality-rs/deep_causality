/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Mesh construction for the thermalization stage.

use crate::FloatType;
use deep_causality_linear::CsrMatrix;
use deep_causality_num::const_scalar_from_int;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    Manifold, ReggeGeometry, Simplex, SimplicialComplex, SimplicialComplexBuilder,
    SimplicialManifold, TopologyError,
};

/// A 1D chain of `data.len()` vertices joined by `data.len() - 1` edges, carrying `data` on the
/// vertices and zero on the edges.
///
/// The codifferential reads the Hodge star out of the complex's cache, so the complex is rebuilt
/// with identity mass matrices at both grades; the unit-edge Regge metric supplies the metric
/// instance the Laplacian requires without contributing data of its own.
/// Small whole numbers, at the working type.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);

pub(crate) fn make_1d_manifold(
    data: Vec<FloatType>,
) -> Result<SimplicialManifold<FloatType, FloatType>, TopologyError> {
    let n = data.len();
    let mut builder = SimplicialComplexBuilder::new(1);
    for i in 0..n - 1 {
        builder.add_simplex(Simplex::new(vec![i, i + 1]))?;
    }
    let complex: SimplicialComplex<FloatType> = builder.build()?;

    let skeletons = complex.skeletons().clone();
    let boundaries = complex.boundary_operators().clone();
    let coboundaries = complex.coboundary_operators().clone();

    let num_vertices = skeletons[0].simplices().len();
    let num_edges = skeletons[1].simplices().len();

    let hodge = vec![identity_matrix(num_vertices)?, identity_matrix(num_edges)?];
    let complex_with_hodge = SimplicialComplex::new(skeletons, boundaries, coboundaries, hodge);

    // The data tensor spans every simplex: the vertex values, then a zero for each edge.
    let mut full_data = data;
    full_data.resize(num_vertices + num_edges, ZERO);
    let len = full_data.len();
    let tensor = CausalTensor::new(full_data, vec![len])?;

    let edge_lengths = CausalTensor::new(vec![ONE; num_edges], vec![num_edges])?;
    let metric = ReggeGeometry::new(edge_lengths);

    Manifold::with_metric(complex_with_hodge, tensor, Some(metric), 0)
}

fn identity_matrix(n: usize) -> Result<CsrMatrix<FloatType>, TopologyError> {
    let triplets: Vec<(usize, usize, FloatType)> = (0..n).map(|i| (i, i, ONE)).collect();
    CsrMatrix::from_triplets(n, n, &triplets)
        .map_err(|e| TopologyError::InvalidInput(format!("identity mass matrix: {e:?}")))
}
