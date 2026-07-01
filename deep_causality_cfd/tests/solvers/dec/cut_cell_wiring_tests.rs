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

use deep_causality_cfd::{BodyForceOneForm, DecNsSolver};
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

/// March as [`march`], but with projection warm start enabled on the solver.
fn march_warm(m: &Manifold<LatticeComplex<2, f64>, f64>, ny: usize, steps: usize) -> Vec<f64> {
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
    let solver = DecNsSolver::new(m, NU, dt, Some(&force))
        .unwrap()
        .with_warm_start();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();
    for _ in 0..steps {
        state = solver.step(&state).unwrap().into_state();
    }
    state.as_one_form().as_slice().to_vec()
}

/// Warm start changes only the CG iteration count, so a warm-started march tracks the cold march to
/// within the projection tolerance over a wall-bounded (no-slip) channel.
#[test]
fn warm_start_matches_the_cold_march() {
    let ny = 9;
    let steps = 40;
    let m = channel_manifold(ny, None);
    let cold = march(&m, ny, steps);
    let warm = march_warm(&m, ny, steps);
    assert_eq!(cold.len(), warm.len());
    let max_diff = cold
        .iter()
        .zip(warm.iter())
        .fold(0.0_f64, |acc, (a, b)| acc.max((a - b).abs()));
    assert!(
        max_diff < 1e-8,
        "warm-started march diverged from cold by {max_diff:e}"
    );
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
fn immersed_solid_block_enforces_no_slip_and_no_penetration() {
    // A periodic box with a 2×2 immersed solid block, driven by a uniform x body force. B4's
    // staircase no-slip pins every edge incident to the solid block to zero; the flow goes
    // around it and stays divergence-free.
    let n = 8;
    let h = 1.0;
    let nu = 0.1;
    let lattice = LatticeComplex::<2, f64>::square_torus(n);

    let mut reg = CutCellRegistry::<2, f64>::new();
    for base in [[3, 3], [3, 4], [4, 3], [4, 4]] {
        let cell = LatticeCell::<2>::new(base, 0b11);
        let idx = lattice.cells(2).position(|c| c == cell).unwrap();
        reg.insert(idx, deep_causality_topology::CutCell::<2, f64>::solid(1.0));
    }
    let pinned = reg.solid_incident_edges(&lattice);
    assert!(!pinned.is_empty(), "the solid block must constrain edges");

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<2, f64>::uniform(h).with_cut_cells(reg);
    let m = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    let n1 = m.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[idx] = 0.2 * h;
        }
    }
    let force = BodyForceOneForm::new(CausalTensor::new(force, vec![n1]).unwrap(), &m).unwrap();
    let dt = 0.2;
    let solver = DecNsSolver::new(&m, nu, dt, Some(&force)).unwrap();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();

    for _ in 0..15 {
        let out = solver.step(&state).expect("immersed march must converge");
        assert!(
            out.divergence_residual() < 1e-8,
            "immersed projection lost divergence-freeness: {}",
            out.divergence_residual()
        );
        state = out.into_state();
    }

    // Every solid-incident edge is held at zero by the no-slip / no-penetration constraint.
    let field = state.as_one_form();
    for &e in &pinned {
        assert!(
            field.as_slice()[e].abs() < 1e-10,
            "edge {e} on the immersed body should be pinned to zero, got {}",
            field.as_slice()[e]
        );
    }
    // The flow is non-trivial somewhere away from the body (the force drives it).
    let max = field.as_slice().iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    assert!(
        max > 1e-3,
        "the body force should drive a non-trivial flow, max {max}"
    );
}

#[test]
fn aperture_resolved_disk_marches_divergence_free_with_body_no_slip() {
    use deep_causality_topology::Primitive;
    // A periodic box with an immersed disk built from a primitive (genuine Cut cells, so the
    // aperture-resolved path is active), driven by a uniform x body force. The weighted cut-face
    // rows are enforced on the STATE every step via the weighted re-entry projection — the property
    // the per-stage rate projection alone cannot guarantee (the re-entry's gradient correction would
    // otherwise reintroduce wall slip). The march stays divergence-free throughout.
    let n = 12;
    let h = 1.0;
    let nu = 0.1;
    let lattice = LatticeComplex::<2, f64>::square_torus(n);
    let metric_base = CubicalReggeGeometry::<2, f64>::uniform(h);
    let disk = Primitive::<2, f64>::ball([6.0, 6.0], 2.5);
    let reg = CutCellRegistry::from_primitive(&lattice, &metric_base, &disk).unwrap();
    assert!(
        !reg.cut_face_constraints(&lattice).is_empty(),
        "the disk primitive must produce aperture-resolved cut-face rows"
    );

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<2, f64>::uniform(h).with_cut_cells(reg);
    let m = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    let n1 = m.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[idx] = 0.2 * h;
        }
    }
    let force = BodyForceOneForm::new(CausalTensor::new(force, vec![n1]).unwrap(), &m).unwrap();
    let dt = 0.2;
    let solver = DecNsSolver::new(&m, nu, dt, Some(&force)).unwrap();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();

    // March well past the transient toward steady: the tangential-only constraint set stays
    // well-conditioned (no closed-body no-penetration rank-deficiency), so the projection converges
    // every step over a long run — the regime the thousands-of-steps cylinder validation needs.
    for _ in 0..40 {
        let out = solver
            .step(&state)
            .expect("aperture-resolved march must converge");
        assert!(
            out.divergence_residual() < 1e-8,
            "aperture-resolved projection lost divergence-freeness: {}",
            out.divergence_residual()
        );
        state = out.into_state();
    }

    // The aperture-resolved tangential no-slip holds on the STATE: every tangential cut-face row
    // residual Σ wₑ uₑ − target ≈ 0 to the projection tolerance. (No-penetration is carried in
    // aggregate by the solid-interior pins + divergence-freeness, not as an explicit row.)
    use deep_causality_topology::CutConstraintKind;
    let u = state.as_one_form().as_slice();
    let rows = m
        .metric()
        .unwrap()
        .cut_registry()
        .unwrap()
        .cut_face_constraints(m.complex());
    let mut checked = 0;
    for row in rows
        .iter()
        .filter(|r| r.kind() == CutConstraintKind::Tangential)
    {
        let mut s = 0.0;
        for &(e, w) in row.entries() {
            s += w * u[e];
        }
        assert!(
            (s - row.target()).abs() < 1e-6,
            "aperture-resolved tangential row not satisfied on the marched state: residual {}",
            s - row.target()
        );
        checked += 1;
    }
    assert!(checked > 0, "the disk must produce tangential no-slip rows");
    let max = u.iter().fold(0.0_f64, |mx, v| mx.max(v.abs()));
    assert!(
        max > 1e-3,
        "the body force should drive a non-trivial flow, max {max}"
    );
}

#[test]
fn aperture_resolved_warm_start_matches_cold_march() {
    use deep_causality_topology::Primitive;
    // The per-stage projection warm-starts BOTH the φ potential and the λ (cut-face multiplier)
    // block. On an aperture-resolved body that changes only the CG iteration count, not the marched
    // result: a warm-started march must track the cold march to within the projection tolerance.
    let n = 12;
    let h = 1.0;
    let nu = 0.1;
    let lattice = LatticeComplex::<2, f64>::square_torus(n);
    let metric_base = CubicalReggeGeometry::<2, f64>::uniform(h);
    let disk = Primitive::<2, f64>::ball([6.0, 6.0], 2.5);
    let reg = CutCellRegistry::from_primitive(&lattice, &metric_base, &disk).unwrap();
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<2, f64>::uniform(h).with_cut_cells(reg);
    let m = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    let n1 = m.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[idx] = 0.2 * h;
        }
    }
    let force = BodyForceOneForm::new(CausalTensor::new(force, vec![n1]).unwrap(), &m).unwrap();
    let dt = 0.2;
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();

    let march = |warm: bool| -> Vec<f64> {
        let solver = {
            let s = DecNsSolver::new(&m, nu, dt, Some(&force)).unwrap();
            if warm { s.with_warm_start() } else { s }
        };
        let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();
        for _ in 0..20 {
            state = solver.step(&state).unwrap().into_state();
        }
        state.as_one_form().as_slice().to_vec()
    };

    let cold = march(false);
    let warm = march(true);
    let gap = cold
        .iter()
        .zip(warm.iter())
        .fold(0.0_f64, |acc, (a, b)| acc.max((a - b).abs()));
    assert!(
        gap < 1e-8,
        "φ+λ warm-started aperture-resolved march diverged from cold by {gap:e}"
    );
}

#[test]
fn tiny_cut_cells_are_inherently_small_cell_stable() {
    // B1–B3 finding. Four deliberately tiny (0.1%-wetted) **free** cut cells meet at a shared
    // vertex — the classic small-cell hazard: cut↔cut edges stay free (the no-slip set pins
    // only solid-incident edges) and the shared vertex's dual mass s0 → 0. In a finite-volume
    // cut-cell solver this collapses the explicit time step and needs a Berger–Helzel /
    // Colella–Graves–Modiano stabilizer.
    //
    // It does NOT here: the cut Hodge star is a *consistent metric clip*, so the codifferential
    // δ = M⁻¹ ∂ M cancels it across grades — a sliver vertex (s0 ≈ ε) is fed by sliver edges
    // (s1 ≈ ε), so the δ entries are s1/s0 ≈ O(1) and the operator never goes stiff. A seeded
    // field therefore decays viscously and stably even at a normal time step, with no
    // stabilizer (the same "the structure-preserving discretisation dissolves the problem"
    // pattern as the graded-metric order study).
    //
    // The cell-merging stabilizer is implemented and available (`with_cell_merging`, B1/B2 —
    // flux-redistribution does not fit the projected-rate formulation), and this also checks it
    // preserves the stable, divergence-free decay rather than perturbing it.
    let n = 8;
    let h = 1.0;
    let nu = 0.1;
    let dt = 0.3; // a normal time step; a finite-volume sliver would force dt ~ ε·dt here.

    let run = |merge: Option<f64>, steps: usize| -> (bool, f64, f64) {
        let lattice = LatticeComplex::<2, f64>::square_torus(n);
        let top_idx = |base: [usize; 2]| {
            lattice
                .cells(2)
                .position(|c| *c.position() == base && c.cell_dim() == 2)
                .unwrap()
        };
        let mut reg = CutCellRegistry::<2, f64>::new();
        // Four 0.1%-wetted cut cells around vertex [4,4]; cut↔cut edges stay free.
        for b in [[3, 3], [3, 4], [4, 3], [4, 4]] {
            reg.insert(
                top_idx(b),
                deep_causality_topology::CutCell::<2, f64>::cut(
                    1.0,
                    1.0e-3,
                    [[1.0, 1.0], [1.0, 1.0]],
                    Vec::new(),
                ),
            );
        }
        if let Some(a) = merge {
            reg = reg.with_cell_merging(a);
        }

        let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
        let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
        let metric = CubicalReggeGeometry::<2, f64>::uniform(h).with_cut_cells(reg);
        let m = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

        // No body force: a seeded velocity field should simply decay viscously. On a clean grid
        // that is unconditionally stable at this dt; the sliver duals make the explicit viscous
        // operator stiff enough to diverge instead — which the cell-merging floor prevents.
        // (A fully-periodic body force would accelerate the mean flow and abort on the advective
        // CFL regardless of cut cells, masking the effect under test.)
        let solver = DecNsSolver::new(&m, nu, dt, None).unwrap();

        let n0 = m.complex().num_cells(0);
        let mut seed = vec![0.0; 2 * n0];
        for (v, cell) in m.complex().iter_cells(0).enumerate() {
            let p = cell.position();
            seed[2 * v] = ((p[0] + 2 * p[1]) as f64).sin();
            seed[2 * v + 1] = ((2 * p[0] + p[1]) as f64).cos();
        }
        let mut state = solver
            .seed_from_vertex_vectors(&CausalTensor::new(seed, vec![2 * n0]).unwrap())
            .unwrap();

        let speed = |s: &[f64]| s.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        let seed_speed = speed(state.as_one_form().as_slice());
        let mut worst_div = 0.0_f64;
        let mut peak_ratio = 1.0_f64;
        for _ in 0..steps {
            match solver.step(&state) {
                Ok(out) => {
                    worst_div = worst_div.max(out.divergence_residual());
                    state = out.into_state();
                }
                Err(_) => return (false, worst_div, peak_ratio), // CFL abort ⇒ not stable.
            }
            let s = state.as_one_form();
            if !s.as_slice().iter().all(|v| v.is_finite()) {
                return (false, worst_div, f64::INFINITY);
            }
            peak_ratio = peak_ratio.max(speed(s.as_slice()) / seed_speed);
        }
        (true, worst_div, peak_ratio)
    };

    // Unstabilized: the sliver does NOT destabilise the explicit march — it stays finite and
    // the speed never amplifies (the clip cancels in δ, so no small-cell CFL collapse). The
    // tiny masses do degrade the masked-CG projection's conditioning, so divergence-freeness is
    // only loosely held.
    let (finite, div, peak) = run(None, 300);
    eprintln!("# unstabilized: finite={finite}, worst_div={div:.3e}, peak_ratio={peak:.3}");
    assert!(
        finite,
        "the clip-based cut star must march a tiny cut cell stably with no stabilizer"
    );
    assert!(
        peak < 2.0,
        "a viscous decay must not amplify; peak speed ratio {peak}"
    );

    // Cell-merging restores the projection conditioning: finite, non-amplifying, AND tightly
    // divergence-free. This is where the stabilizer earns its place in this formulation — not
    // for explicit CFL (inherently fine) but for the masked-CG projection on sliver masses.
    let (m_finite, m_div, m_peak) = run(Some(0.5), 300);
    eprintln!("# cell-merging: finite={m_finite}, worst_div={m_div:.3e}, peak_ratio={m_peak:.3}");
    assert!(m_finite, "cell-merging must keep the march finite");
    assert!(
        m_peak < 2.0,
        "cell-merging must not amplify; peak speed ratio {m_peak}"
    );
    assert!(
        m_div < div,
        "cell-merging must improve the projection's divergence-freeness ({m_div:.3e} vs {div:.3e})"
    );
}

#[test]
fn axis_aligned_solid_layer_reproduces_the_wall_poiseuille() {
    // B6 — marched equivalence: an axis-aligned **solid layer reproduces the wall solver**. The
    // channel keeps its real bottom wall (vertex y = 0) and its top cell row is marked immersed
    // **solid**, so B4's staircase no-slip pins the x-edge at vertex row `ny-2` to zero — a
    // Dirichlet wall sitting exactly on a vertex row, just like a real wall. The fluid below must
    // therefore develop the **exact** Poiseuille parabola for the reduced height
    // `H' = (ny-2)·h` — the same analytic profile the vertex-collocated wall solver is validated
    // against (`poiseuille_tests`), to rounding. That the immersed solid layer and a real wall
    // yield the identical exact steady state *is* the marched equivalence.
    let ny = 9;
    let h = HEIGHT / (ny - 1) as f64;
    let h_fluid = (ny - 2) as f64 * h; // walls at vertex 0 (real) and vertex ny-2 (immersed solid).

    let lattice = LatticeComplex::<2, f64>::new([4, ny], [true, false]);
    let mut reg = CutCellRegistry::<2, f64>::new();
    for i in 0..4 {
        let cell = LatticeCell::<2>::new([i, ny - 2], 0b11);
        let idx = lattice.cells(2).position(|c| c == cell).unwrap();
        reg.insert(idx, deep_causality_topology::CutCell::<2, f64>::solid(1.0));
    }
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<2, f64>::uniform(h).with_cut_cells(reg);
    let m = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    let g = 8.0 * NU;
    let n1 = m.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[idx] = g * h;
        }
    }
    let force = BodyForceOneForm::new(CausalTensor::new(force, vec![n1]).unwrap(), &m).unwrap();
    let dt = 0.5 * h * h / (4.0 * NU);
    let solver = DecNsSolver::new(&m, NU, dt, Some(&force)).unwrap();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();

    // March to stationarity (early-exit on a settled field).
    let mut previous = state.as_one_form().as_slice().to_vec();
    for _ in 0..20_000 {
        state = solver.step(&state).unwrap().into_state();
        let now = state.as_one_form().as_slice();
        let delta = now
            .iter()
            .zip(previous.iter())
            .fold(0.0f64, |acc, (a, b)| acc.max((a - b).abs()));
        if delta < 1e-13 {
            break;
        }
        previous = now.to_vec();
    }

    // Every x-edge matches the exact parabola for the immersed-wall height; the immersed wall
    // (vertex ny-2) and the solid rows above are zero.
    let u = state.as_one_form().as_slice();
    let mut err = 0.0f64;
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize != 0 {
            continue;
        }
        let y = cell.position()[1] as f64 * h;
        let exact = if y <= h_fluid {
            (g / (2.0 * NU)) * y * (h_fluid - y) * h
        } else {
            0.0
        };
        err = err.max((u[idx] - exact).abs() / h);
    }
    assert!(
        err < 1e-8,
        "solid-layer Poiseuille profile error {err:e} — the immersed solid layer must act as a wall"
    );
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

#[test]
fn staircase_noslip_on_a_cut_cell_body_pins_the_solid_ring_and_marches() {
    use deep_causality_topology::Primitive;
    // `with_staircase_noslip` flips an aperture-resolved cut-cell body (genuine Cut cells) to the
    // staircase mechanism: the weighted cut-face rows are dropped and the full solid-incident edge
    // ring is pinned. The march must still converge and stay divergence-free, with the solid-interior
    // edges pinned to zero.
    let n = 12;
    let h = 1.0;
    let nu = 0.1;
    let lattice = LatticeComplex::<2, f64>::square_torus(n);
    let metric_base = CubicalReggeGeometry::<2, f64>::uniform(h);
    let disk = Primitive::<2, f64>::ball([6.0, 6.0], 2.5);
    let reg = CutCellRegistry::from_primitive(&lattice, &metric_base, &disk).unwrap();
    let solid = reg.solid_incident_edges(&lattice);
    assert!(!solid.is_empty(), "the disk must have a solid interior");

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<2, f64>::uniform(h).with_cut_cells(reg);
    let m = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    let n1 = m.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[idx] = 0.2 * h;
        }
    }
    let force = BodyForceOneForm::new(CausalTensor::new(force, vec![n1]).unwrap(), &m).unwrap();
    let dt = 0.2;
    let solver = DecNsSolver::new(&m, nu, dt, Some(&force))
        .unwrap()
        .with_staircase_noslip();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();

    for _ in 0..15 {
        let out = solver
            .step(&state)
            .expect("staircase cut-cell march must converge");
        assert!(
            out.divergence_residual() < 1e-8,
            "staircase projection lost divergence-freeness: {}",
            out.divergence_residual()
        );
        state = out.into_state();
    }

    // The staircase set pins every solid-incident edge to zero on the state.
    let u = state.as_one_form().as_slice();
    for &e in &solid {
        assert!(
            u[e].abs() < 1e-9,
            "staircase no-slip must pin solid edge {e} to zero, got {}",
            u[e]
        );
    }
    let max = u.iter().fold(0.0_f64, |mx, v| mx.max(v.abs()));
    assert!(max > 1e-3, "the body force should drive a non-trivial flow");
}
