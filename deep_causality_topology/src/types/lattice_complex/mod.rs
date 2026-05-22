/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod dual_lattice_complex;
pub mod lattice_cell;
pub mod specialized;

pub use lattice_cell::LatticeCell;

use crate::traits::cell::Cell;
use crate::traits::chain_complex::ChainComplex;
use deep_causality_num::RealField;
use deep_causality_sparse::CsrMatrix;
use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::OnceLock;

/// A combinatorial cubical lattice in `D` dimensions, paired with a metric precision `R`.
///
/// `D` is the lattice dimensionality (always a compile-time constant); `R` is the
/// precision of the associated metric, threaded through to `Metric = CubicalReggeGeometry<D, R>`
/// via the `ChainComplex` impl. The combinatorial caches (`shape`, `periodic`, coboundary
/// memo) are `R`-independent — the `PhantomData<R>` field exists only to anchor the
/// metric-precision parameter. See `design.md` Decision 1 of the
/// `generalize-topology-over-realfield` change set for the rationale.
#[derive(Debug)]
pub struct LatticeComplex<const D: usize, R: RealField> {
    /// Dimensions of the lattice [L₀, L₁, ..., L_{D-1}]
    shape: [usize; D],
    /// Periodic boundary conditions per dimension
    periodic: [bool; D],
    /// Lazy memo of coboundary matrices: δ_k = (∂_{k+1})ᵀ.
    /// One `OnceLock` per grade in `0..=D`; populated on first call to `coboundary_matrix(k)`;
    /// ignored for equality. `OnceLock` (not `Mutex<HashMap>`) keeps reads lock-free after first
    /// init and lets `coboundary_matrix` return `Cow::Borrowed`, eliminating the per-call CSR
    /// clone. `Box<[OnceLock<...>]>` is `Send + Sync` for the existing
    /// `Arc<LatticeComplex<D, R>>` consumers in `gauge_field_lattice`.
    coboundary_cache: Box<[OnceLock<CsrMatrix<i8>>]>,
    /// Anchors the metric-precision parameter `R`. Zero-sized.
    _precision: PhantomData<R>,
}

impl<const D: usize, R: RealField> Clone for LatticeComplex<D, R> {
    fn clone(&self) -> Self {
        Self {
            shape: self.shape,
            periodic: self.periodic,
            // Cache intentionally not cloned: cheaper to recompute lazily on the clone.
            coboundary_cache: Self::fresh_cache(),
            _precision: PhantomData,
        }
    }
}

impl<const D: usize, R: RealField> PartialEq for LatticeComplex<D, R> {
    fn eq(&self, other: &Self) -> bool {
        self.shape == other.shape && self.periodic == other.periodic
    }
}

impl<const D: usize, R: RealField> Eq for LatticeComplex<D, R> {}

impl<const D: usize, R: RealField> LatticeComplex<D, R> {
    // --- Constructors ---

    /// Create a new lattice with given shape and boundary conditions.
    pub fn new(shape: [usize; D], periodic: [bool; D]) -> Self {
        Self {
            shape,
            periodic,
            coboundary_cache: Self::fresh_cache(),
            _precision: PhantomData,
        }
    }

    /// Build an empty per-grade coboundary cache with `D + 1` slots (one per grade `0..=D`).
    fn fresh_cache() -> Box<[OnceLock<CsrMatrix<i8>>]> {
        (0..=D)
            .map(|_| OnceLock::new())
            .collect::<Vec<_>>()
            .into_boxed_slice()
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
    pub fn iter_cells(&self, k: usize) -> LatticeCellIterator<'_, D, R> {
        LatticeCellIterator::new(self, k)
    }
}

// --- Iterator ---

pub struct LatticeCellIterator<'a, const D: usize, R: RealField> {
    lattice: &'a LatticeComplex<D, R>,
    // k is used in new() but not stored
    orientations: Vec<u32>,
    current_orientation_idx: usize,
    current_pos: [usize; D],
    done: bool,
}

impl<'a, const D: usize, R: RealField> LatticeCellIterator<'a, D, R> {
    fn new(lattice: &'a LatticeComplex<D, R>, k: usize) -> Self {
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

impl<'a, const D: usize, R: RealField> Iterator for LatticeCellIterator<'a, D, R> {
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

impl<const D: usize, R: RealField> ChainComplex for LatticeComplex<D, R> {
    type CellType = LatticeCell<D>;
    type CellIter<'a>
        = LatticeCellIterator<'a, D, R>
    where
        Self: 'a;
    type Metric = crate::CubicalReggeGeometry<D, R>;

    fn cells(&self, k: usize) -> Self::CellIter<'_> {
        self.iter_cells(k)
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

    fn boundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>> {
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

        Cow::Owned(
            CsrMatrix::from_triplets(rows, cols, &triplets).unwrap_or_else(|_| CsrMatrix::new()),
        )
    }

    fn coboundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>> {
        // Lazy memo: δ_k = (∂_{k+1})ᵀ. One OnceLock per grade in 0..=D.
        let slot = &self.coboundary_cache[k];
        let m = slot.get_or_init(|| self.boundary_matrix(k + 1).into_owned().transpose());
        Cow::Borrowed(m)
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

impl<R: RealField> LatticeComplex<2, R> {
    pub fn square_torus(l: usize) -> Self {
        Self::new([l, l], [true, true])
    }
    pub fn square_open(l: usize) -> Self {
        Self::new([l, l], [false, false])
    }
}

impl<R: RealField> LatticeComplex<3, R> {
    pub fn cubic_torus(l: usize) -> Self {
        Self::new([l, l, l], [true, true, true])
    }
    pub fn cubic_open(l: usize) -> Self {
        Self::new([l, l, l], [false, false, false])
    }
}

impl<R: RealField> LatticeComplex<4, R> {
    pub fn hypercubic_torus(l: usize) -> Self {
        Self::new([l, l, l, l], [true, true, true, true])
    }
}
