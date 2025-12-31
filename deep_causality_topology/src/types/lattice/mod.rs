/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod dual_lattice;
pub mod lattice_cell;
pub mod specialized;

pub use lattice_cell::LatticeCell;

use crate::traits::cw_complex::{CWComplex, Cell};
use deep_causality_sparse::CsrMatrix;
use std::collections::HashMap;

/// A D-dimensional regular lattice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lattice<const D: usize> {
    /// Dimensions of the lattice [L₀, L₁, ..., L_{D-1}]
    shape: [usize; D],
    /// Periodic boundary conditions per dimension
    periodic: [bool; D],
}

impl<const D: usize> Lattice<D> {
    // --- Constructors ---

    /// Create a new lattice with given shape and boundary conditions.
    pub fn new(shape: [usize; D], periodic: [bool; D]) -> Self {
        Self { shape, periodic }
    }

    /// Create a fully periodic (toroidal) lattice.
    pub fn torus(shape: [usize; D]) -> Self {
        Self::new(shape, [true; D])
    }

    /// Create an open-boundary lattice.
    pub fn open(shape: [usize; D]) -> Self {
        Self::new(shape, [false; D])
    }

    // --- Getters ---

    /// Shape of the lattice [L₀, L₁, ..., L_{D-1}].
    pub fn shape(&self) -> &[usize; D] {
        &self.shape
    }

    /// Periodic boundary conditions per dimension.
    pub fn periodic(&self) -> &[bool; D] {
        &self.periodic
    }

    /// Dimension of the lattice (always D).
    pub const fn dim(&self) -> usize {
        D
    }

    // --- Cell Access ---

    /// Get the boundary of a k-cell as a chain of (k-1)-cells.
    /// Handles periodic boundary conditions.
    pub fn boundary(&self, cell: &LatticeCell<D>) -> Vec<(LatticeCell<D>, i8)> {
        let raw_boundary = cell.boundary();
        let mut periodic_boundary = Vec::new();

        for (term_cell, coeff) in raw_boundary {
            if let Some(wrapped_cell) = self.wrap_cell(&term_cell) {
                periodic_boundary.push((wrapped_cell, coeff));
            }
        }

        periodic_boundary
    }

    /// Wrap a cell into the lattice boundaries.
    /// Returns None if the cell is out of bounds (for open boundaries).
    fn wrap_cell(&self, cell: &LatticeCell<D>) -> Option<LatticeCell<D>> {
        let mut pos = *cell.position();

        for (i, p) in pos.iter_mut().enumerate() {
            let dim_len = self.shape[i];

            if *p >= dim_len {
                if self.periodic[i] {
                    *p %= dim_len;
                } else {
                    return None;
                }
            }
        }

        Some(LatticeCell::new(pos, cell.orientation()))
    }

    /// Iterator over all cells of a given dimension k.
    pub fn iter_cells(&self, k: usize) -> LatticeCellIterator<'_, D> {
        LatticeCellIterator::new(self, k)
    }
}

// --- Iterator ---

pub struct LatticeCellIterator<'a, const D: usize> {
    lattice: &'a Lattice<D>,
    // k is used in new() but not stored
    orientations: Vec<u32>,
    current_orientation_idx: usize,
    current_pos: [usize; D],
    done: bool,
}

impl<'a, const D: usize> LatticeCellIterator<'a, D> {
    fn new(lattice: &'a Lattice<D>, k: usize) -> Self {
        // Generate all k-bit patterns in D bits
        let mut orientations = Vec::new();
        let limit: usize = 1 << D;
        for i in 0..limit {
            if i.count_ones() as usize == k {
                orientations.push(i as u32);
            }
        }

        let done: bool = orientations.is_empty();

        Self {
            lattice,
            orientations,
            current_orientation_idx: 0,
            current_pos: [0; D],
            done,
        }
    }
}

impl<'a, const D: usize> Iterator for LatticeCellIterator<'a, D> {
    type Item = LatticeCell<D>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let orientation = self.orientations[self.current_orientation_idx];
        let cell = LatticeCell::new(self.current_pos, orientation);

        // Advance position logic...
        let mut carry = true;
        for i in 0..D {
            if carry {
                self.current_pos[i] += 1;

                let limit = self.lattice.shape[i];
                let is_active_dim = (orientation & (1 << i)) != 0;
                let is_periodic = self.lattice.periodic[i];

                let max_idx = if is_active_dim && !is_periodic {
                    if limit == 0 { 0 } else { limit - 1 }
                } else {
                    limit
                };

                if self.current_pos[i] >= max_idx {
                    self.current_pos[i] = 0;
                    carry = true;
                } else {
                    carry = false;
                }
            }
        }

        if carry {
            self.current_orientation_idx += 1;
            if self.current_orientation_idx >= self.orientations.len() {
                self.done = true;
            } else {
                self.current_pos = [0; D];
            }
        }

        Some(cell)
    }
}

impl<const D: usize> CWComplex for Lattice<D> {
    type CellType = LatticeCell<D>;

    fn cells(&self, k: usize) -> Box<dyn Iterator<Item = Self::CellType> + '_> {
        Box::new(self.iter_cells(k))
    }

    fn num_cells(&self, k: usize) -> usize {
        let mut total = 0;
        let limit: usize = 1 << D;
        for i in 0..limit {
            if i.count_ones() as usize == k {
                let orientation = i as u32;
                let mut count = 1;
                for d in 0..D {
                    let dim_len = self.shape[d];
                    let is_active = (orientation & (1 << d)) != 0;
                    let is_periodic = self.periodic[d];

                    let valid_positions = if is_active && !is_periodic {
                        if dim_len == 0 { 0 } else { dim_len - 1 }
                    } else {
                        dim_len
                    };
                    count *= valid_positions;
                }
                total += count;
            }
        }
        total
    }

    fn max_dim(&self) -> usize {
        D
    }

    fn boundary_matrix(&self, k: usize) -> CsrMatrix<i8> {
        let rows = self.num_cells(k - 1);
        let cols = self.num_cells(k);

        let mut row_map = HashMap::new();
        for (i, cell) in self.cells(k - 1).enumerate() {
            row_map.insert(cell, i);
        }

        let mut triplets = Vec::new();

        for (j, cell) in self.cells(k).enumerate() {
            let boundary = self.boundary(&cell);
            for (term_cell, coeff) in boundary {
                if let Some(&i) = row_map.get(&term_cell) {
                    triplets.push((i, j, coeff));
                }
            }
        }

        CsrMatrix::from_triplets(rows, cols, &triplets).unwrap_or_else(|_| CsrMatrix::new())
    }

    fn betti_number(&self, k: usize) -> usize {
        let all_periodic = self.periodic.iter().all(|&p| p);
        if all_periodic {
            if k > D {
                return 0;
            }
            let mut res = 1;
            for i in 0..k {
                res = res * (D - i) / (i + 1);
            }
            res
        } else {
            let p_dims = self.periodic.iter().filter(|&&p| p).count();
            if k > p_dims {
                return 0;
            }
            let mut res = 1;
            for i in 0..k {
                res = res * (p_dims - i) / (i + 1);
            }
            res
        }
    }
}

// --- Specialized Constructors ---

impl Lattice<2> {
    pub fn square_torus(l: usize) -> Self {
        Self::new([l, l], [true, true])
    }
    pub fn square_open(l: usize) -> Self {
        Self::new([l, l], [false, false])
    }
}

impl Lattice<3> {
    pub fn cubic_torus(l: usize) -> Self {
        Self::new([l, l, l], [true, true, true])
    }
    pub fn cubic_open(l: usize) -> Self {
        Self::new([l, l, l], [false, false, false])
    }
}

impl Lattice<4> {
    pub fn hypercubic_torus(l: usize) -> Self {
        Self::new([l, l, l, l], [true, true, true, true])
    }
}
