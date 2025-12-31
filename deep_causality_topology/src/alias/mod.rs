/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::DefaultBackend;
use deep_causality_tensor::TensorBackend;

use crate::types::backend::ManifoldView;
use crate::types::backend::TopologyView;

/// Adjacency Matrix mapped to backend (Shape: [N, N])
pub type AdjacencyMatrix<B, T> = <B as TensorBackend>::Tensor<T>;

/// Incidence Matrix mapped to backend (Shape: [Nodes, Edges])
pub type IncidenceMatrix<B, T> = <B as TensorBackend>::Tensor<T>;

/// Laplacian Matrix (L = D - A)
pub type LaplacianMatrix<B, T> = <B as TensorBackend>::Tensor<T>;

/// Manifold Metric Tensor (g_{mn})
pub type MetricTensor<B, T> = <B as TensorBackend>::Tensor<T>;

/// Convenience alias: Backend-agnostic TopologyView
pub type GraphView<T = f32> = TopologyView<DefaultBackend, T>;

/// Convenience alias: Backend-agnostic ManifoldView
pub type ChartView<T = f64> = ManifoldView<DefaultBackend, T>;
