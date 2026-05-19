/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{Cell, Simplex};
use std::collections::HashMap;

#[test]
fn test_simplex_new() {
    let vertices = vec![0, 1, 2];
    let simplex = Simplex::new(vertices.clone());
    assert_eq!(simplex.vertices(), &vertices);
}

#[test]
fn test_simplex_getters() {
    let vertices = vec![5, 8, 12];
    let simplex = Simplex::new(vertices.clone());
    assert_eq!(simplex.vertices(), &vertices);
}

#[test]
fn test_simplex_subsimplex() {
    let simplex = Simplex::new(vec![0, 1, 2, 3]);

    // Subsimplex (face) with range 0..2 (vertices 0, 1)
    let sub1 = simplex.subsimplex(0..2);
    assert_eq!(sub1.vertices(), &vec![0, 1]);

    // Subsimplex (edge) with range 1..3 (vertices 1, 2)
    let sub2 = simplex.subsimplex(1..3);
    assert_eq!(sub2.vertices(), &vec![1, 2]);

    // Subsimplex (vertex) with range 2..3 (vertex 2)
    let sub3 = simplex.subsimplex(2..3);
    assert_eq!(sub3.vertices(), &vec![2]);

    // Full simplex as subsimplex
    let full = simplex.subsimplex(0..4);
    assert_eq!(full.vertices(), &vec![0, 1, 2, 3]);

    // Empty subsimplex
    let empty = simplex.subsimplex(0..0);
    assert_eq!(empty.vertices(), &Vec::<usize>::new());
}

#[test]
fn test_simplex_display() {
    let simplex = Simplex::new(vec![10, 20, 30]);
    assert_eq!(format!("{}", simplex), "Simplex(10, 20, 30)");

    let vertex_simplex = Simplex::new(vec![5]);
    assert_eq!(format!("{}", vertex_simplex), "Simplex(5)");

    let empty_simplex = Simplex::new(vec![]);
    assert_eq!(format!("{}", empty_simplex), "Simplex()");
}

#[test]
fn test_simplex_equality() {
    let s1 = Simplex::new(vec![0, 1, 2]);
    let s2 = Simplex::new(vec![0, 1, 2]);
    let s3 = Simplex::new(vec![0, 2, 1]); // Different order, but should be canonicalized

    assert_eq!(s1, s2);
    assert_eq!(s1, s2);
    // Note: Simplex::new sorts vertices internally to ensure canonical representation.
    // Even if input is unsorted, equality holds.
    assert_eq!(s1, s3);
}

#[test]
fn test_simplex_new_sorts_vertices() {
    // Unsorted input must be canonicalized via internal sort.
    let s = Simplex::new(vec![3, 1, 2, 0]);
    assert_eq!(s.vertices(), &vec![0, 1, 2, 3]);
}

#[test]
fn test_simplex_contains_vertex() {
    let s = Simplex::new(vec![10, 20, 30]);
    assert!(s.contains_vertex(&10));
    assert!(s.contains_vertex(&20));
    assert!(s.contains_vertex(&30));
    assert!(!s.contains_vertex(&0));
    assert!(!s.contains_vertex(&25));
    assert!(!s.contains_vertex(&100));

    let empty = Simplex::new(vec![]);
    assert!(!empty.contains_vertex(&0));
}

#[test]
fn test_simplex_cell_dim() {
    // dim = vertices.len() - 1
    let vertex = Simplex::new(vec![5]);
    assert_eq!(<Simplex as Cell>::dim(&vertex), 0);

    let edge = Simplex::new(vec![0, 1]);
    assert_eq!(<Simplex as Cell>::dim(&edge), 1);

    let triangle = Simplex::new(vec![0, 1, 2]);
    assert_eq!(<Simplex as Cell>::dim(&triangle), 2);

    let tet = Simplex::new(vec![0, 1, 2, 3]);
    assert_eq!(<Simplex as Cell>::dim(&tet), 3);

    // Empty simplex: saturating_sub(1) -> 0.
    let empty = Simplex::new(vec![]);
    assert_eq!(<Simplex as Cell>::dim(&empty), 0);
}

#[test]
fn test_simplex_cell_boundary_empty_and_vertex() {
    // A 0-simplex (vertex) and an empty simplex both yield an empty boundary.
    let empty = Simplex::new(vec![]);
    assert!(<Simplex as Cell>::boundary(&empty).is_empty());

    let vertex = Simplex::new(vec![7]);
    assert!(<Simplex as Cell>::boundary(&vertex).is_empty());
}

#[test]
fn test_simplex_cell_boundary_edge() {
    // ∂[v0, v1] = [v1] - [v0]
    let edge = Simplex::new(vec![0, 1]);
    let b = <Simplex as Cell>::boundary(&edge);
    assert_eq!(b.len(), 2);
    // i=0 -> remove v0 -> [v1] with sign +1
    assert_eq!(b[0].0, Simplex::new(vec![1]));
    assert_eq!(b[0].1, 1);
    // i=1 -> remove v1 -> [v0] with sign -1
    assert_eq!(b[1].0, Simplex::new(vec![0]));
    assert_eq!(b[1].1, -1);
}

#[test]
fn test_simplex_cell_boundary_triangle_signs() {
    // ∂[v0, v1, v2] = [v1,v2] - [v0,v2] + [v0,v1]
    let tri = Simplex::new(vec![0, 1, 2]);
    let b = <Simplex as Cell>::boundary(&tri);
    assert_eq!(b.len(), 3);
    assert_eq!(b[0].0, Simplex::new(vec![1, 2]));
    assert_eq!(b[0].1, 1);
    assert_eq!(b[1].0, Simplex::new(vec![0, 2]));
    assert_eq!(b[1].1, -1);
    assert_eq!(b[2].0, Simplex::new(vec![0, 1]));
    assert_eq!(b[2].1, 1);
}

#[test]
fn test_simplex_hash_canonical() {
    // Two simplices constructed from differently-ordered vertices must hash equal,
    // since `new` canonicalizes the vertex order before storage.
    let s1 = Simplex::new(vec![2, 0, 1]);
    let s2 = Simplex::new(vec![1, 2, 0]);
    let mut map: HashMap<Simplex, &str> = HashMap::new();
    map.insert(s1, "tri");
    assert_eq!(map.get(&s2), Some(&"tri"));
}

#[test]
fn test_simplex_order() {
    let s1 = Simplex::new(vec![0, 1]);
    let s2 = Simplex::new(vec![0, 1, 2]);
    let s3 = Simplex::new(vec![1, 2]);

    assert!(s1 < s2);
    assert!(s1 < s3);
    assert!(s2 < s3);
}
