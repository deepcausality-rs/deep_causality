use deep_causality_topology::Cell;
use deep_causality_topology::{CWComplex, CellComplex};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct MockCell {
    id: usize,
    dim: usize,
}

impl Cell for MockCell {
    fn dim(&self) -> usize {
        self.dim
    }
    fn boundary(&self) -> Vec<(Self, i8)> {
        Vec::new()
    }
}

#[test]
fn test_cell_complex_construction() {
    let cells = vec![
        MockCell { id: 0, dim: 0 },
        MockCell { id: 1, dim: 0 },
        MockCell { id: 2, dim: 1 }, // Edge
    ];

    let complex = CellComplex::from_cells(cells);
    assert_eq!(complex.num_cells(0), 2);
    assert_eq!(complex.num_cells(1), 1);
    assert_eq!(complex.dimension(), 1);
}
