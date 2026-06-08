/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt;

/// Failure cases for reading or writing a CPDAG CSV file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpdagError {
    /// The CPDAG file could not be found.
    FileNotFound(String),
    /// An OS / IO error occurred while reading or writing the file.
    Io(String),
    /// The `# … vertices=N` header line was missing or malformed.
    MissingHeader,
    /// A row could not be parsed (wrong field count, non-numeric index, or an
    /// unknown mark token).
    Parse(String),
    /// A vertex index in the file is outside `0..num_vertices`.
    VertexOutOfRange {
        /// The offending index.
        index: usize,
        /// The declared vertex count.
        num_vertices: usize,
    },
    /// Building the graph failed (carries the underlying topology error message).
    Graph(String),
}

impl fmt::Display for CpdagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpdagError::FileNotFound(p) => write!(f, "CPDAG file not found: {}", p),
            CpdagError::Io(e) => write!(f, "CPDAG file IO error: {}", e),
            CpdagError::MissingHeader => {
                write!(f, "CPDAG file is missing the '# … vertices=N' header line")
            }
            CpdagError::Parse(e) => write!(f, "CPDAG file parse error: {}", e),
            CpdagError::VertexOutOfRange {
                index,
                num_vertices,
            } => write!(
                f,
                "CPDAG vertex index {} is out of range for {} vertices",
                index, num_vertices
            ),
            CpdagError::Graph(e) => write!(f, "CPDAG graph construction failed: {}", e),
        }
    }
}

impl std::error::Error for CpdagError {}
