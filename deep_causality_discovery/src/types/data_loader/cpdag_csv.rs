/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CSV (de)serialization for a [`MixedGraph`] CPDAG, faithful to the typed-endpoint
//! model.
//!
//! Each edge is stored as its canonical pair plus both endpoint marks, so the
//! format round-trips any mixed graph (DAG / CPDAG / MAG / PAG), not only
//! arc/undirected CPDAGs. The vertex count rides in a `# … vertices=N` comment
//! header so isolated vertices survive a round trip. The IO lives here in the
//! discovery crate (which already depends on `csv`), keeping the topology
//! data-structure crate free of filesystem code.
//!
//! ```text
//! # deep_causality MixedGraph v1; vertices=5
//! src,dst,mark_src,mark_dst
//! 0,1,Tail,Arrow      (an arc 0 -> 1)
//! 1,2,Tail,Tail       (an undirected edge 1 -- 2)
//! ```

use crate::CpdagError;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Mark, MixedGraph};
use std::io::Write;

/// Writes `graph` to `path` as a CPDAG CSV file.
///
/// # Errors
/// Returns [`CpdagError::Io`] if the file cannot be written.
pub fn save_cpdag_csv<T>(graph: &MixedGraph<T>, path: &str) -> Result<(), CpdagError> {
    let mut out = String::new();
    out.push_str(&format!(
        "# deep_causality MixedGraph v1; vertices={}\n",
        graph.num_vertices()
    ));
    out.push_str("src,dst,mark_src,mark_dst\n");
    for (&(lo, hi), edge) in graph.edges() {
        out.push_str(&format!(
            "{},{},{},{}\n",
            lo,
            hi,
            mark_str(edge.lo),
            mark_str(edge.hi)
        ));
    }

    let mut file = std::fs::File::create(path).map_err(|e| CpdagError::Io(e.to_string()))?;
    file.write_all(out.as_bytes())
        .map_err(|e| CpdagError::Io(e.to_string()))?;
    Ok(())
}

/// Reads a CPDAG CSV file from `path` into a unit-payload [`MixedGraph`].
///
/// # Errors
/// * [`CpdagError::FileNotFound`] / [`CpdagError::Io`] on read failure.
/// * [`CpdagError::MissingHeader`] if the `vertices=N` header line is absent.
/// * [`CpdagError::Parse`] for a malformed row or an unknown mark token.
/// * [`CpdagError::VertexOutOfRange`] if a vertex index is `>= N`.
/// * [`CpdagError::Graph`] if graph construction rejects the edge set.
pub fn load_cpdag_csv(path: &str) -> Result<MixedGraph<()>, CpdagError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CpdagError::FileNotFound(path.to_string())
        } else {
            CpdagError::Io(e.to_string())
        }
    })?;

    let num_vertices = parse_vertices_header(&content)?;

    let data = CausalTensor::new(vec![(); num_vertices], vec![num_vertices])
        .map_err(|e| CpdagError::Graph(e.to_string()))?;
    let mut graph =
        MixedGraph::new(num_vertices, data, 0).map_err(|e| CpdagError::Graph(e.to_string()))?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .comment(Some(b'#'))
        .from_reader(content.as_bytes());

    for result in rdr.records() {
        let record = result.map_err(|e| CpdagError::Parse(e.to_string()))?;
        if record.len() != 4 {
            return Err(CpdagError::Parse(format!(
                "expected 4 fields (src,dst,mark_src,mark_dst), got {}",
                record.len()
            )));
        }
        let src = parse_index(&record[0], num_vertices)?;
        let dst = parse_index(&record[1], num_vertices)?;
        let mark_src = parse_mark(&record[2])?;
        let mark_dst = parse_mark(&record[3])?;
        graph
            .add_edge(src, dst, mark_src, mark_dst)
            .map_err(|e| CpdagError::Graph(e.to_string()))?;
    }

    Ok(graph)
}

/// Renders a [`Mark`] as its full-word token.
fn mark_str(mark: Mark) -> &'static str {
    match mark {
        Mark::Tail => "Tail",
        Mark::Arrow => "Arrow",
        Mark::Circle => "Circle",
    }
}

/// Parses a full-word mark token into a [`Mark`].
fn parse_mark(token: &str) -> Result<Mark, CpdagError> {
    match token.trim() {
        "Tail" => Ok(Mark::Tail),
        "Arrow" => Ok(Mark::Arrow),
        "Circle" => Ok(Mark::Circle),
        other => Err(CpdagError::Parse(format!("unknown mark token '{}'", other))),
    }
}

/// Parses a vertex index, rejecting non-numeric values and out-of-range indices.
fn parse_index(field: &str, num_vertices: usize) -> Result<usize, CpdagError> {
    let index = field
        .trim()
        .parse::<usize>()
        .map_err(|_| CpdagError::Parse(format!("non-numeric vertex index '{}'", field)))?;
    if index >= num_vertices {
        return Err(CpdagError::VertexOutOfRange {
            index,
            num_vertices,
        });
    }
    Ok(index)
}

/// Extracts the vertex count from the `# … vertices=N` comment header line.
fn parse_vertices_header(content: &str) -> Result<usize, CpdagError> {
    for line in content.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }
        if let Some(idx) = trimmed.find("vertices=") {
            let rest = &trimmed[idx + "vertices=".len()..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            return digits
                .parse::<usize>()
                .map_err(|_| CpdagError::MissingHeader);
        }
    }
    Err(CpdagError::MissingHeader)
}
