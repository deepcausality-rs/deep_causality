// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

// Alias type renames the container type to UltraGraph
pub use crate::alias::UltraGraph;
// Errors
pub use crate::errors::UltraGraphError;
// Protocols
pub use crate::protocols::graph_like::GraphLike;
pub use crate::protocols::graph_root::GraphRoot;
pub use crate::protocols::graph_storage::GraphStorage;
// Storage implementation
pub use crate::storage::matrix_graph::UltraMatrixGraph;
// Main type
pub use crate::types::ultra_graph::UltraGraphContainer;

