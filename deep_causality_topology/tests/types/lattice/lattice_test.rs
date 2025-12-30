use deep_causality_topology::{CWComplex, Lattice};

#[test]
fn test_lattice_construction() {
    let lattice = Lattice::<2>::new([10, 10], [true, true]); // Torus
    assert_eq!(lattice.shape(), &[10, 10]);
    assert_eq!(lattice.dim(), 2);
    assert!(lattice.periodic().iter().all(|&p| p));
}

#[test]
fn test_lattice_cell_counting() {
    // 3x3 square grid
    // Vertices: 3*3 = 9
    // Edges:
    //  Horizontal: 3 rows * 2 edges (open) or 3 edges (periodic)
    //  Vertical: same

    // Open
    let open = Lattice::<2>::new([3, 3], [false, false]);
    assert_eq!(open.num_cells(0), 9); // vertices
    // Edges: 3*(3-1) + 3*(3-1) = 6 + 6 = 12
    assert_eq!(open.num_cells(1), 12);
    // Faces: (3-1)*(3-1) = 4
    assert_eq!(open.num_cells(2), 4);

    // Periodic (Torus)
    let periodic = Lattice::<2>::new([3, 3], [true, true]);
    assert_eq!(periodic.num_cells(0), 9);
    // Edges: 3*3 + 3*3 = 18
    assert_eq!(periodic.num_cells(1), 18);
    // Faces: 3*3 = 9
    assert_eq!(periodic.num_cells(2), 9);
}

#[test]
fn test_lattice_iterators() {
    // 2x2 Open
    let lat = Lattice::<2>::new([2, 2], [false, false]);

    // Vertices: (0,0), (1,0), (0,1), (1,1) -> 4
    assert_eq!(lat.cells(0).count(), 4);

    // Edges: (0,0)-h, (0,1)-h, (0,0)-v, (1,0)-v -> 4
    // Vertices are 0..L-1.
    // Horizontal edges connect (i, j) to (i+1, j). Exist if i < L-1.
    // L=2. i=0 exists. i=1 No.
    // Rows j=0, 1. So (0,0) and (0,1) horizontal. (2 edges)
    // Vert same. Total 4.
    assert_eq!(lat.cells(1).count(), 4);

    // Faces: (0,0) -> 1
    assert_eq!(lat.cells(2).count(), 1);
}
