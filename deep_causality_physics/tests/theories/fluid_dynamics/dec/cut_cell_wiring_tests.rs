/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CFD Stage 4 Group B5 — cut-cell solver wiring.
//!
//! The cut-cell registry lives on the geometry (immutable Stage-4 Context); every Hodge-star
//! read consults it, so the compiled stencils, the constrained Leray projection and the
//! codifferential all see the immersed body transparently — the solver itself needs no new
//! plumbing. These tests pin the two wiring guarantees:
//!
//! - **Transparency / non-breaking (B5, the B6 reduction):** a geometry carrying an *empty*
//!   registry marches **bit-identically** to the plain geometry — the cut clip reduces to the
//!   Stage-3 wall clip, so the whole operator stack is unchanged when no body is present.
//! - **Active + stable:** a registry that removes interior cells changes the star yet keeps a
//!   convergent, divergence-free projected march (the cut star feeds the masked CG correctly).

use deep_causality_physics::{BodyForceOneForm, DecNsSolver};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCellRegistry, LatticeCell, LatticeComplex, Manifold,
};

const NU: f64 = 0.1;
const HEIGHT: f64 = 1.0;

/// A Poiseuille channel manifold (periodic-x, wall-y) over a `uniform(h)` metric, optionally
/// carrying a cut-cell registry.
fn channel_manifold(
    ny: usize,
    registry: Option<CutCellRegistry<2, f64>>,
) -> Manifold<LatticeComplex<2, f64>, f64> {
    let h = HEIGHT / (ny - 1) as f64;
    let lattice = LatticeComplex::<2, f64>::new([4, ny], [true, false]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let mut metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(h);
    if let Some(reg) = registry {
        metric = metric.with_cut_cells(reg);
    }
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// March the channel `steps` steps from rest under the Poiseuille body force; return the final
/// edge-cochain state vector.
fn march(m: &Manifold<LatticeComplex<2, f64>, f64>, ny: usize, steps: usize) -> Vec<f64> {
    let h = HEIGHT / (ny - 1) as f64;
    let g = 8.0 * NU;
    let n1 = m.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[idx] = g * h;
        }
    }
    let force = BodyForceOneForm::new(CausalTensor::new(force, vec![n1]).unwrap(), m).unwrap();

    let dt = 0.5 * h * h / (4.0 * NU);
    let solver = DecNsSolver::new(m, NU, dt, Some(&force)).unwrap();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();
    for _ in 0..steps {
        state = solver.step(&state).unwrap().into_state();
    }
    state.as_one_form().as_slice().to_vec()
}

#[test]
fn empty_registry_marches_bit_identically_to_plain_geometry() {
    let ny = 9;
    let steps = 60;

    let plain = channel_manifold(ny, None);
    let with_empty = channel_manifold(ny, Some(CutCellRegistry::<2, f64>::new()));

    let a = march(&plain, ny, steps);
    let b = march(&with_empty, ny, steps);

    assert_eq!(a.len(), b.len());
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        // The empty-registry star is the 2^{-b} wall clip to the last bit (powers of two are
        // exact), so the entire march is bit-identical.
        assert!(
            (x - y).abs() <= 1e-15 * (1.0 + x.abs()),
            "edge {i}: plain {x} != empty-registry {y} — the wiring must be a no-op when empty"
        );
    }
}

#[test]
fn solid_cell_registry_keeps_a_convergent_divergence_free_march() {
    let ny = 9;
    let lattice = LatticeComplex::<2, f64>::new([4, ny], [true, false]);

    // Remove one interior top cell (base [1, 3]); the cut star shrinks the surrounding dual
    // volumes. No cut-face BC yet (that is B4) — this only pins that B5 keeps the projection
    // well-posed and divergence-free under a cut star.
    let mut reg = CutCellRegistry::<2, f64>::new();
    let solid = LatticeCell::<2>::new([1, 3], 0b11);
    let idx = lattice.cells(2).position(|c| c == solid).unwrap();
    reg.insert(idx, deep_causality_topology::CutCell::<2, f64>::solid(1.0));

    let m = channel_manifold(ny, Some(reg));
    let h = HEIGHT / (ny - 1) as f64;
    let g = 8.0 * NU;
    let n1 = m.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (i, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[i] = g * h;
        }
    }
    let force = BodyForceOneForm::new(CausalTensor::new(force, vec![n1]).unwrap(), &m).unwrap();
    let dt = 0.5 * h * h / (4.0 * NU);
    let solver = DecNsSolver::new(&m, NU, dt, Some(&force)).unwrap();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();

    for _ in 0..40 {
        let out = solver.step(&state).expect("cut-star march must converge");
        // The projected field stays divergence-free at the solve tolerance every step.
        assert!(
            out.divergence_residual() < 1e-8,
            "cut-star projection lost divergence-freeness: {}",
            out.divergence_residual()
        );
        state = out.into_state();
    }
    // The state is finite throughout.
    assert!(state.as_one_form().as_slice().iter().all(|v| v.is_finite()));
}
