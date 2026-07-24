/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Group Z — the boundary-zone abstraction (`add-boundary-zone-abstraction`).
//!
//! The static zone composition is the canonical surface for the explicit boundary actuators; the
//! numerical-equivalence gate pins that a zone-built solver marches **bit-identically** to the
//! legacy construction it replaces:
//! - a `BodyForceZone` reproduces the `DecNsSolver::new(.., Some(force))` Poiseuille march;
//! - a `MovingWall` zone reproduces the `.with_moving_wall(..)` lid-driven cavity march;
//! - the two compose statically as `(BodyForceZone, MovingWall)`.

use deep_causality_cfd::{
    BodyForceOneForm, BodyForceZone, DecNsSolver, MovingWall, SolenoidalField,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const NU: f64 = 0.1;

fn channel(ny: usize, periodic: [bool; 2]) -> (Manifold<LatticeComplex<2, f64>, f64>, f64) {
    let h = 1.0 / (ny - 1) as f64;
    let lattice = LatticeComplex::<2, f64>::new([4, ny], periodic);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(h);
    (
        Manifold::from_cubical_with_metric(lattice, data, metric, 0),
        h,
    )
}

/// The Poiseuille streamwise body force as a grade-1 edge cochain (`G·h` on the x-edges).
fn poiseuille_force(m: &Manifold<LatticeComplex<2, f64>, f64>, h: f64) -> CausalTensor<f64> {
    let g = 8.0 * NU;
    let n1 = m.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[idx] = g * h;
        }
    }
    CausalTensor::new(force, vec![n1]).unwrap()
}

fn march(solver: &DecNsSolver<'_, 2, f64>, seed: &SolenoidalField<f64>, steps: usize) -> Vec<f64> {
    let mut state = seed.clone();
    for _ in 0..steps {
        state = solver.step(&state).unwrap().into_state();
    }
    state.as_one_form().as_slice().to_vec()
}

#[test]
#[cfg_attr(miri, ignore)]
fn body_force_zone_marches_bit_identically_to_the_legacy_solver() {
    let (m, h) = channel(9, [true, false]);
    let dt = 0.5 * h * h / (4.0 * NU);
    let force_tensor = poiseuille_force(&m, h);

    let force = BodyForceOneForm::new(force_tensor.clone(), &m).unwrap();
    let legacy = DecNsSolver::new(&m, NU, dt, Some(&force)).unwrap();
    let zoned = DecNsSolver::with_zones(&m, NU, dt, BodyForceZone::new(force_tensor)).unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let seed = legacy.seed_from_vertex_vectors(&rest).unwrap();

    let a = march(&legacy, &seed, 50);
    let b = march(&zoned, &seed, 50);
    assert_eq!(
        a, b,
        "body-force zone must reproduce the legacy march bit-for-bit"
    );
}

#[test]
fn moving_wall_zone_marches_bit_identically_to_with_moving_wall() {
    let (m, h) = channel(7, [false, false]);
    let dt = 0.5 * h * h / (4.0 * NU); // diffusive-safe at this ν.
    let lid = 1.0;

    let legacy = DecNsSolver::new(&m, NU, dt, None)
        .unwrap()
        .with_moving_wall(1, true, [lid, 0.0])
        .unwrap();
    let zoned =
        DecNsSolver::with_zones(&m, NU, dt, MovingWall::new(1, true, [lid, 0.0]).unwrap()).unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    // Seed through the legacy solver (its lift is applied at seed time); the zoned solver carries
    // the identical lift, so seeding it gives the same field — march from the legacy seed for both.
    let seed = legacy.seed_from_vertex_vectors(&rest).unwrap();

    let a = march(&legacy, &seed, 40);
    let b = march(&zoned, &seed, 40);
    assert_eq!(
        a, b,
        "moving-wall zone must reproduce with_moving_wall bit-for-bit"
    );
}

#[test]
fn body_force_and_moving_wall_compose_statically() {
    let (m, h) = channel(7, [true, false]);
    let dt = 0.2 * h * h / (4.0 * NU);
    let force_tensor = poiseuille_force(&m, h);
    let lid = 0.5;

    let force = BodyForceOneForm::new(force_tensor.clone(), &m).unwrap();
    let legacy = DecNsSolver::new(&m, NU, dt, Some(&force))
        .unwrap()
        .with_moving_wall(1, true, [lid, 0.0])
        .unwrap();
    let zoned = DecNsSolver::with_zones(
        &m,
        NU,
        dt,
        (
            BodyForceZone::new(force_tensor),
            MovingWall::new(1, true, [lid, 0.0]).unwrap(),
        ),
    )
    .unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let seed = legacy.seed_from_vertex_vectors(&rest).unwrap();

    let a = march(&legacy, &seed, 30);
    let b = march(&zoned, &seed, 30);
    assert_eq!(
        a, b,
        "the composed zone set must reproduce the legacy march bit-for-bit"
    );
}

// --- The zone-supplied constrained-edge hook (item 15) --------------------------------------------
//
// `collect_constrained_edges` was the one documented hook the solver never read: declared on the
// trait, composed by the cons-tuple, implemented by nothing and folded nowhere. These pin the three
// properties that make it a real seam rather than a promise — it has an effect, overlapping with the
// structural set is idempotent, and it outranks a free-slip un-pin.

/// A test zone that pins an explicit edge set — the shape `aperture-resolved-noslip` will take, which
/// is why the hook exists. No shipped zone implements it, so this is also the only way to observe it.
struct PinEdges(Vec<usize>);

impl deep_causality_cfd::BoundaryZone<2, f64> for PinEdges {
    fn collect_constrained_edges(
        &self,
        _m: &Manifold<LatticeComplex<2, f64>, f64>,
        out: &mut Vec<usize>,
    ) {
        out.extend_from_slice(&self.0);
    }
}

/// The streamwise (x-oriented) edges, in cell order.
fn x_edges(m: &Manifold<LatticeComplex<2, f64>, f64>) -> Vec<usize> {
    m.complex()
        .iter_cells(1)
        .enumerate()
        .filter(|(_, c)| c.orientation().trailing_zeros() == 0)
        .map(|(i, _)| i)
        .collect()
}

fn poiseuille_setup() -> (
    Manifold<LatticeComplex<2, f64>, f64>,
    f64,
    CausalTensor<f64>,
) {
    let (m, h) = channel(9, [true, false]);
    let dt = 0.5 * h * h / (4.0 * NU);
    let force = poiseuille_force(&m, h);
    (m, dt, force)
}

#[test]
#[cfg_attr(miri, ignore)]
fn a_zone_supplied_constraint_pins_its_edges() {
    // The property whose absence made the hook vestigial: a zone that pins edges must actually
    // change the march. Poiseuille drives every x-edge away from zero, so a pinned middle band is
    // held at zero only if the hook reaches the constrained projection.
    let (m, dt, force) = poiseuille_setup();
    let xs = x_edges(&m);
    let pinned: Vec<usize> = xs[xs.len() / 3..2 * xs.len() / 3].to_vec();
    assert!(!pinned.is_empty(), "the test needs a non-empty pinned band");

    let control = DecNsSolver::with_zones(&m, NU, dt, BodyForceZone::new(force.clone())).unwrap();
    let zoned = DecNsSolver::with_zones(
        &m,
        NU,
        dt,
        (BodyForceZone::new(force), PinEdges(pinned.clone())),
    )
    .unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let seed = control.seed_from_vertex_vectors(&rest).unwrap();

    let a = march(&control, &seed, 20);
    let b = march(&zoned, &seed, 20);

    for &e in &pinned {
        assert!(
            b[e].abs() < 1e-12,
            "edge {e} was pinned by the zone but marched to {}",
            b[e]
        );
    }
    // The control must move those same edges, or the assertion above would hold vacuously.
    assert!(
        pinned.iter().any(|&e| a[e].abs() > 1e-6),
        "the unpinned march must drive the band away from zero, else the test proves nothing"
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn a_zone_constraint_overlapping_the_structural_set_is_idempotent() {
    // The composition rule is union, so a zone re-declaring edges the no-slip set already pins must
    // march bit-identically — pinning an edge twice is pinning it once.
    let (m, dt, force) = poiseuille_setup();

    let plain = DecNsSolver::with_zones(&m, NU, dt, BodyForceZone::new(force.clone())).unwrap();

    // The wall-tangential x-edges on the y=0 and y=max rows are already structurally no-slip.
    let xs = x_edges(&m);
    let walls: Vec<usize> = xs
        .iter()
        .copied()
        .take(4)
        .chain(xs.iter().rev().copied().take(4))
        .collect();
    let overlapped =
        DecNsSolver::with_zones(&m, NU, dt, (BodyForceZone::new(force), PinEdges(walls))).unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let seed = plain.seed_from_vertex_vectors(&rest).unwrap();

    let a = march(&plain, &seed, 20);
    let b = march(&overlapped, &seed, 20);
    for (i, (p, q)) in a.iter().zip(&b).enumerate() {
        assert_eq!(
            p, q,
            "edge {i}: union with the structural set must be a no-op"
        );
    }
}

/// A zone whose every hook returns whatever it was handed, so a test can enable one at a time and
/// attribute the resulting change in the march to exactly one fold site in `with_zones`.
#[derive(Default, Clone)]
struct Probe {
    lift_value: f64,
    rate: Vec<usize>,
    constrained: Vec<usize>,
    lift: Vec<usize>,
    slip: Vec<usize>,
    prescribed: Vec<usize>,
    reference: Vec<usize>,
}

impl deep_causality_cfd::BoundaryZone<2, f64> for Probe {
    fn collect_rate_source(&self, _m: &Manifold<LatticeComplex<2, f64>, f64>, acc: &mut [f64]) {
        for &e in &self.rate {
            acc[e] += 0.25;
        }
    }
    fn collect_constrained_edges(
        &self,
        _m: &Manifold<LatticeComplex<2, f64>, f64>,
        out: &mut Vec<usize>,
    ) {
        out.extend_from_slice(&self.constrained);
    }
    fn collect_lift(
        &self,
        _m: &Manifold<LatticeComplex<2, f64>, f64>,
        _step: usize,
        out: &mut Vec<(usize, f64)>,
    ) {
        out.extend(self.lift.iter().map(|&e| (e, self.lift_value)));
    }
    fn collect_prescribed_edges(
        &self,
        _m: &Manifold<LatticeComplex<2, f64>, f64>,
        out: &mut Vec<usize>,
    ) {
        out.extend_from_slice(&self.prescribed);
    }
    fn collect_reference_vertices(
        &self,
        _m: &Manifold<LatticeComplex<2, f64>, f64>,
        out: &mut Vec<usize>,
    ) {
        out.extend_from_slice(&self.reference);
    }
    fn collect_slip_edges(&self, _m: &Manifold<LatticeComplex<2, f64>, f64>, out: &mut Vec<usize>) {
        out.extend_from_slice(&self.slip);
    }
}

/// The streamwise (x-oriented) edge indices, in cell order.
fn stream_edges(m: &Manifold<LatticeComplex<2, f64>, f64>) -> Vec<usize> {
    m.complex()
        .iter_cells(1)
        .enumerate()
        .filter(|(_, c)| c.orientation().trailing_zeros() == 0)
        .map(|(i, _)| i)
        .collect()
}

/// Poiseuille through 12 steps with `probe` added alongside the driving body force.
fn march_with_probe(
    m: &Manifold<LatticeComplex<2, f64>, f64>,
    dt: f64,
    force: &CausalTensor<f64>,
    probe: Probe,
) -> Vec<f64> {
    let solver =
        DecNsSolver::with_zones(m, NU, dt, (BodyForceZone::new(force.clone()), probe)).unwrap();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let seed = solver.seed_from_vertex_vectors(&rest).unwrap();
    march(&solver, &seed, 12)
}

fn max_abs_delta(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(p, q)| (p - q).abs())
        .fold(0.0f64, f64::max)
}

#[test]
#[cfg_attr(miri, ignore)]
fn each_independent_zone_hook_reaches_the_solver() {
    // Item 15 was a hook the trait documented and `with_zones` never read — declared, composed by
    // the cons-tuple, folded nowhere. Nothing in the type system prevents that, and nothing stops
    // the next hook added from repeating it.
    //
    // The behavioural check: a zone implementing one hook is added to a Poiseuille march, and its
    // result must differ from the same march without it. A hook the solver never folds cannot
    // change anything, so it fails here. Testing the *effect* rather than the presence of a call
    // site also catches a fold that is wired but ignored.
    //
    // `collect_prescribed_edges` and `collect_reference_vertices` are not here: an inflow with no
    // outflow reference is rejected outright (the net flux cannot balance), so neither is
    // independently exercisable. They are covered by the open-boundary test below.
    let (m, h) = channel(9, [true, false]);
    let dt = 0.5 * h * h / (4.0 * NU);
    let force = poiseuille_force(&m, h);
    let xs = stream_edges(&m);
    // A middle band: interior, so it is neither already no-slip pinned nor on the periodic seam.
    let band: Vec<usize> = xs[xs.len() / 3..2 * xs.len() / 3].to_vec();

    let baseline = march_with_probe(&m, dt, &force, Probe::default());

    let cases: [(&str, Probe); 4] = [
        (
            "collect_rate_source",
            Probe {
                rate: band.clone(),
                ..Default::default()
            },
        ),
        (
            "collect_constrained_edges",
            Probe {
                constrained: band.clone(),
                ..Default::default()
            },
        ),
        (
            "collect_lift",
            Probe {
                lift: band.clone(),
                ..Default::default()
            },
        ),
        (
            // Slip un-pins edges from the structural no-slip set, so it must act on wall edges.
            "collect_slip_edges",
            Probe {
                slip: xs.iter().copied().take(4).collect(),
                ..Default::default()
            },
        ),
    ];

    for (name, probe) in cases {
        let moved = march_with_probe(&m, dt, &force, probe);
        let delta = max_abs_delta(&baseline, &moved);
        assert!(
            delta > 1e-12,
            "{name} changed nothing (max |Δ| = {delta:.3e}); it is declared on BoundaryZone but \
             DecNsSolver::with_zones does not fold it into the march"
        );
    }
}

#[test]
#[cfg_attr(miri, ignore)]
fn both_open_boundary_hooks_reach_the_solver() {
    // `collect_prescribed_edges` and `collect_reference_vertices` cannot be tested in isolation:
    // the solver folds them together through `set_open_boundary`, and a prescribed inflow with no
    // outflow reference is refused because the net flux cannot balance. So each is isolated by
    // *varying* it while the other is held fixed — if the solver ignored either, changing it alone
    // could not move the march.
    //
    // The index sets come from the shipped `Inflow` / `Outflow` zones rather than being invented,
    // so they are the sets the solver actually expects.
    use deep_causality_cfd::{BoundaryZone, Inflow, Outflow};

    let (m, h) = channel(9, [false, false]);
    let dt = 0.5 * h * h / (4.0 * NU);
    let force = poiseuille_force(&m, h);

    let mut prescribed = Vec::new();
    Inflow::<2, f64>::new(0, false, 1.0)
        .unwrap()
        .collect_prescribed_edges(&m, &mut prescribed);
    let mut reference = Vec::new();
    Outflow::<2>::new(0, true)
        .unwrap()
        .collect_reference_vertices(&m, &mut reference);
    assert!(
        prescribed.len() > 1 && reference.len() > 1,
        "the variation below needs at least two of each (got {} prescribed, {} reference)",
        prescribed.len(),
        reference.len()
    );

    let base = Probe {
        lift_value: 0.05,
        lift: prescribed.clone(),
        prescribed: prescribed.clone(),
        reference: reference.clone(),
        ..Default::default()
    };
    let baseline = march_with_probe(&m, dt, &force, base.clone());

    // Vary the prescribed set, reference held fixed. Dropping *one* edge is not enough — the last
    // inflow edge is a corner already pinned by the structural no-slip set, so removing it leaves
    // the union unchanged and the march bit-identical. Halving the set is the variation that bites.
    let fewer_prescribed = Probe {
        prescribed: prescribed[..prescribed.len() / 2].to_vec(),
        ..base.clone()
    };
    let d_prescribed = max_abs_delta(
        &baseline,
        &march_with_probe(&m, dt, &force, fewer_prescribed),
    );
    assert!(
        d_prescribed > 1e-9,
        "collect_prescribed_edges changed nothing (max |Δ| = {d_prescribed:.3e}); the solver is \
         not folding the zone's inflow edges"
    );

    // Vary the reference set, prescribed held fixed. Same caveat: dropping one is a no-op, and
    // dropping half destabilises the solve outright (the outflow can no longer carry the inflow's
    // flux, which is itself evidence the hook is read). Two is finite and unambiguous.
    let fewer_reference = Probe {
        reference: reference[..reference.len() - 2].to_vec(),
        ..base
    };
    let d_reference = max_abs_delta(
        &baseline,
        &march_with_probe(&m, dt, &force, fewer_reference),
    );
    assert!(
        d_reference > 1e-9,
        "collect_reference_vertices changed nothing (max |Δ| = {d_reference:.3e}); the solver is \
         not folding the zone's outflow pressure reference"
    );
}
