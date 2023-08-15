use ultragraph::prelude::*;

const SIZE: usize = 10;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data {
    x: u8,
}

fn get_ultra_graph() -> UltraGraph<StorageCSRGraph<Data>, Data> {
    ultragraph::new_with_csr_storage::<Data>(SIZE)
}

#[test]
fn test_size() {
    let g = get_ultra_graph();

    let expected = 0;
    let actual = g.size();
    assert_eq!(expected, actual);
}

#[test]
fn test_is_empty() {
    let g = get_ultra_graph();

    assert!(g.is_empty());
}

#[test]
fn test_number_nodes() {
    let g = get_ultra_graph();

    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}

#[test]
fn test_number_edges() {
    let g = get_ultra_graph();

    let expected = 0;
    let actual = g.number_edges();
    assert_eq!(expected, actual);
}

#[test]
fn test_clear() {
    let mut g = get_ultra_graph();
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);

    assert_eq!(root_index, 10);

    assert!(!g.is_empty());

    g.clear();

    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = 0;
    let actual = g.number_edges();
    assert_eq!(expected, actual);
}

#[test]
fn test_add_root_node() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 10);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}


#[test]
fn test_get_root_index() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
    assert!(!g.contains_root_node());

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert!(g.contains_root_node());

    let result = g.get_root_index().unwrap();
    assert_eq!(root_index, 10);
    assert_eq!(result, 10);
    assert_eq!(root_index, result);
    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_last_index() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
    assert!(!g.contains_root_node());

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert!(g.contains_root_node());
    assert_eq!(root_index, 10);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = 0;
    let actual = g.get_last_index().unwrap();
    assert_eq!(expected, actual);
}


#[test]
fn test_add_node() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 10);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let index = g.add_node(d);
    assert_eq!(index, 11);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}


#[test]
fn test_contains_node() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 10);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(11);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let index = g.add_node(d);
    assert_eq!(index, 11);

    let expected = true;
    let actual = g.contains_node(11);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_node() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 10);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let index = g.add_node(d);
    assert_eq!(index, 11);

    let data = g.get_node(index).unwrap();
    assert_eq!(*data, d);
    assert_eq!(data.x, d.x);
    assert_eq!(data.x, 42);

    let expected = true;
    let actual = g.contains_node(index);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);
}


#[test]
fn test_remove_node() {
    //
    // CSR does not support removing nodes.
    //
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 10);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    //
    // remove_node always returns an error
    //
    let res = g.remove_node(root_index);
    assert!(res.is_err());
}

#[test]
fn test_add_edge() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 10);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(1);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let node_a_index = g.add_node(d);
    assert_eq!(node_a_index, 11);

    let expected = true;
    let actual = g.contains_node(node_a_index);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());
}


#[test]
fn test_add_edge_with_weight() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 10);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(11);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let node_a_index = g.add_node(d);
    assert_eq!(node_a_index, 11);

    let expected = true;
    let actual = g.contains_node(node_a_index);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let res = g.add_edge_with_weight(root_index, node_a_index, 42);
    assert!(res.is_ok());
}

#[test]
fn test_contains_edge() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 10);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(11);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let node_a_index = g.add_node(d);
    assert_eq!(node_a_index, 11);

    let expected = true;
    let actual = g.contains_node(node_a_index);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);

    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());

    let expected = true;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);
}


#[test]
fn test_remove_edge() {
    let mut g = get_ultra_graph();
    assert!(g.is_empty());
    let expected = 0;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let d = Data { x: 1 };
    let root_index = g.add_root_node(d);
    assert_eq!(root_index, 10);

    let expected = 1;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_node(11);
    assert_eq!(expected, actual);

    let d = Data { x: 42 };
    let node_a_index = g.add_node(d);
    assert_eq!(node_a_index, 11);

    let expected = true;
    let actual = g.contains_node(node_a_index);
    assert_eq!(expected, actual);

    let expected = 2;
    let actual = g.number_nodes();
    assert_eq!(expected, actual);

    let expected = false;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);

    let res = g.add_edge(root_index, node_a_index);
    assert!(res.is_ok());

    let expected = true;
    let actual = g.contains_edge(root_index, node_a_index);
    assert_eq!(expected, actual);

    let res = g.remove_edge(root_index, node_a_index);
    assert!(res.is_err());
}