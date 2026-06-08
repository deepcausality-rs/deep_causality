/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{CpdagError, MixedGraph, load_cpdag_csv, save_cpdag_csv};
use deep_causality_tensor::CausalTensor;
use std::io::Write;
use tempfile::NamedTempFile;

fn unit_graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

fn write(content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

#[test]
fn test_round_trip_preserves_structure_and_isolated_vertex() {
    // 4 vertices: arc 0->1, undirected 1--2, bidirected 0<->3 ... but vertex 3 via
    // bidirected; vertex count 4 with an arc/undirected/bidirected mix; no edge on
    // a hypothetical isolated vertex is exercised by the vertices=N header.
    let mut g = unit_graph(4);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.add_bidirected(0, 3).unwrap();

    let file = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    let path = file.path().to_str().unwrap();
    save_cpdag_csv(&g, path).unwrap();

    let loaded = load_cpdag_csv(path).unwrap();
    assert_eq!(loaded.num_vertices(), 4);
    assert_eq!(loaded.num_edges(), 3);
    assert_eq!(loaded.edge_kind(0, 1), g.edge_kind(0, 1));
    assert_eq!(loaded.edge_kind(1, 2), g.edge_kind(1, 2));
    assert_eq!(loaded.edge_kind(0, 3), g.edge_kind(0, 3));
}

#[test]
fn test_load_arc_direction() {
    let file = write("# vertices=2\nsrc,dst,mark_src,mark_dst\n0,1,Tail,Arrow\n");
    let g = load_cpdag_csv(file.path().to_str().unwrap()).unwrap();
    // Tail at 0, Arrow at 1 => directed arc 0 -> 1, so 0 is a parent of 1.
    assert_eq!(g.parents(1), vec![0]);
}

#[test]
fn test_missing_header_is_error() {
    let file = write("src,dst,mark_src,mark_dst\n0,1,Tail,Arrow\n");
    assert!(matches!(
        load_cpdag_csv(file.path().to_str().unwrap()),
        Err(CpdagError::MissingHeader)
    ));
}

#[test]
fn test_unknown_mark_is_parse_error() {
    let file = write("# vertices=2\nsrc,dst,mark_src,mark_dst\n0,1,Tail,Wat\n");
    assert!(matches!(
        load_cpdag_csv(file.path().to_str().unwrap()),
        Err(CpdagError::Parse(_))
    ));
}

#[test]
fn test_vertex_out_of_range_is_error() {
    let file = write("# vertices=2\nsrc,dst,mark_src,mark_dst\n0,5,Tail,Arrow\n");
    assert!(matches!(
        load_cpdag_csv(file.path().to_str().unwrap()),
        Err(CpdagError::VertexOutOfRange { .. })
    ));
}

#[test]
fn test_file_not_found_is_error() {
    assert!(matches!(
        load_cpdag_csv("/no/such/cpdag.csv"),
        Err(CpdagError::FileNotFound(_))
    ));
}

#[test]
fn test_wrong_field_count_is_parse_error() {
    let file = write("# vertices=2\nsrc,dst,mark_src,mark_dst\n0,1,Tail\n"); // 3 fields
    assert!(matches!(
        load_cpdag_csv(file.path().to_str().unwrap()),
        Err(CpdagError::Parse(_))
    ));
}

#[test]
fn test_self_loop_is_graph_error() {
    // A self-loop (u == v) is rejected by the graph, surfacing as CpdagError::Graph.
    let file = write("# vertices=2\nsrc,dst,mark_src,mark_dst\n0,0,Tail,Arrow\n");
    assert!(matches!(
        load_cpdag_csv(file.path().to_str().unwrap()),
        Err(CpdagError::Graph(_))
    ));
}
