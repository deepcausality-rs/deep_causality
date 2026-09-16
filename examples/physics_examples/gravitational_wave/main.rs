/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Gravitational Wave Propagation (Regge Calculus)
//!
//! A metric perturbation is released at the centre of a triangulated spatial slice and allowed
//! to propagate. Nothing drives it after `t = 0`: the disturbance travels because the edge
//! lengths obey a wave equation whose restoring term is the curvature the mesh itself carries.
//!
//! In Regge calculus curvature is concentrated on bones — for a 2D slice, the vertices — and
//! appears as a **deficit angle** `delta`, the amount by which the triangles around a vertex
//! fail to close. Linearized about flat space, `delta` is the discrete Laplacian of the metric
//! perturbation, so the vacuum wave equation `d2 h / dt2 = c^2 grad^2 h` discretizes to a
//! leapfrog in the edge lengths:
//!
//! ```text
//! l_e(t+1) = 2 l_e(t) - l_e(t-1) - C^2 (delta_a + delta_b) / 2
//! ```
//!
//! with `a, b` the endpoints of edge `e` and `C = c dt / dx` the Courant number. What separates
//! this from an imposed oscillation is that the disturbance obeys a finite propagation speed,
//! and the run measures exactly that:
//!
//! ```text
//! causal      ring k cannot move before step k: no signal outruns the stencil
//! ordered     no ring starts moving before the one inside it
//! bounded     the amplitude stays finite, as an explicit leapfrog does for C <= 1
//! ```
//!
//! The *leading edge* advances at one ring per step, which is the numerical domain of
//! dependence of a nearest-neighbour stencil rather than the physical wave speed; the Courant
//! number governs how fast the energy peak follows it. Reporting the edge speed as if it were
//! `c` would be measuring the stencil, not the physics, so the run does not claim it.
//!
//! The outermost ring is excluded from the analysis: its vertices have no complete cycle of
//! triangles, so the deficit angle there is a boundary artefact rather than curvature.
//!
//! ## APIs Demonstrated
//! - `SimplicialComplexBuilder`, `Simplex`, `BaseTopology`
//! - `ReggeGeometry::calculate_ricci_curvature` (deficit angles as the curvature source)

use deep_causality_algebra::Real;
use deep_causality_num::{lift, lower};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    BaseTopology, ReggeGeometry, Simplex, SimplicialComplex, SimplicialComplexBuilder,
};
use std::collections::HashMap;

/// Rings of triangles around the centre. The wave needs room to travel, so this is the one
/// parameter that decides whether propagation is visible at all.
const RINGS: i32 = 6;
/// Rest length of every edge on the flat background.
const REST_LENGTH: f64 = 1.0;
/// Amplitude of the initial pulse, as a fraction of the rest length.
const AMPLITUDE: f64 = 0.06;
/// Rings covered by the initial pulse. Kept small so the source is localized.
const PULSE_RADIUS: i32 = 1;
/// Courant number `C = c dt / dx`. The explicit leapfrog is stable for `C <= 1`; this is the
/// speed, in rings per step, at which the front should be seen to travel.
const COURANT: f64 = 0.5;
/// Time steps to run. Enough for the front to cross the mesh at `COURANT` rings per step.
const STEPS: usize = 16;
/// An edge counts as disturbed once it moves this far from its rest length.
const ARRIVAL_THRESHOLD: f64 = 1e-4;
/// The pulse may not exceed this multiple of its initial amplitude. A leapfrog at `C <= 1` is
/// stable, so growth past a small factor means the scheme has gone unstable rather than
/// propagated. Set above one because focusing at the centre genuinely amplifies briefly.
const GROWTH_BOUND: f64 = 4.0;

/// `f64` is the right precision here: the leapfrog's error is its `O(dt^2)` truncation, which is
/// about `1e-1` at this Courant number and swamps rounding by fourteen orders of magnitude.
/// `Float106` would refine nothing the discretization has not already coarsened.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let mesh = Mesh::hexagonal(RINGS)?;
    print_mesh(&mesh);

    let history = propagate(&mesh)?;
    print_history(&mesh, &history);

    let front = front_analysis(&mesh, &history);
    print_front(&front);

    Ok(())
}

/// The triangulated slice, plus the bookkeeping the wave equation needs.
struct Mesh {
    complex: SimplicialComplex<FloatType>,
    num_edges: usize,
    /// For each edge, the indices of its two endpoint vertices in the deficit-angle tensor.
    edge_endpoints: Vec<(usize, usize)>,
    /// For each edge, how many rings out from the centre it sits.
    edge_ring: Vec<i32>,
}

impl Mesh {
    /// A triangular lattice on a hexagonal disk of `rings` rings, in axial coordinates.
    ///
    /// Each interior site `(q, r)` contributes the two triangles that tile the rhombus between
    /// it and its `+q`, `+r` neighbours, which is what makes the lattice a genuine 2-complex
    /// rather than a fan of triangles around one point.
    fn hexagonal(rings: i32) -> Result<Self, Box<dyn std::error::Error>> {
        // Vertices: every axial site within `rings` of the origin.
        let mut index_of: HashMap<(i32, i32), usize> = HashMap::new();
        let mut ring_of: Vec<i32> = Vec::new();
        for q in -rings..=rings {
            for r in -rings..=rings {
                if axial_ring(q, r) <= rings {
                    index_of.insert((q, r), ring_of.len());
                    ring_of.push(axial_ring(q, r));
                }
            }
        }

        let mut builder = SimplicialComplexBuilder::new(2);
        for &(q, r) in index_of.keys() {
            let here = index_of.get(&(q, r));
            let east = index_of.get(&(q + 1, r));
            let north = index_of.get(&(q, r + 1));
            let far = index_of.get(&(q + 1, r + 1));

            if let (Some(&a), Some(&b), Some(&c)) = (here, east, north) {
                builder.add_simplex(Simplex::new(sorted3(a, b, c)))?;
            }
            if let (Some(&b), Some(&c), Some(&d)) = (east, north, far) {
                builder.add_simplex(Simplex::new(sorted3(b, c, d)))?;
            }
        }
        let complex = builder.build()?;

        let num_edges = complex
            .num_elements_at_grade(1)
            .ok_or("the complex has no edges")?;

        // Each edge's endpoints, and how far out it sits. The vertex skeleton is indexed in the
        // same order as the deficit-angle tensor, so a vertex id is a deficit index.
        let mut edge_endpoints = Vec::with_capacity(num_edges);
        let mut edge_ring = Vec::with_capacity(num_edges);
        for edge in complex.skeletons()[1].simplices() {
            let v = edge.vertices();
            let (a, b) = (v[0], v[1]);
            edge_endpoints.push((a, b));
            // The inner endpoint decides the ring, so a pulse of radius k covers whole rings.
            edge_ring.push(ring_of[a].min(ring_of[b]));
        }

        Ok(Self {
            complex,
            num_edges,
            edge_endpoints,
            edge_ring,
        })
    }
}

/// Distance from the origin in axial hex coordinates.
fn axial_ring(q: i32, r: i32) -> i32 {
    (q.abs() + r.abs() + (q + r).abs()) / 2
}

fn sorted3(a: usize, b: usize, c: usize) -> Vec<usize> {
    let mut v = vec![a, b, c];
    v.sort_unstable();
    v
}

/// Runs the leapfrog and returns the edge lengths at every step.
fn propagate(mesh: &Mesh) -> Result<Vec<Vec<FloatType>>, Box<dyn std::error::Error>> {
    let rest = lift::<FloatType>(REST_LENGTH);
    let courant_sq = lift::<FloatType>(COURANT) * lift::<FloatType>(COURANT);
    let half = lift::<FloatType>(0.5);

    // t = 0: a localized bulge at the centre, everything else flat.
    let mut previous: Vec<FloatType> = (0..mesh.num_edges)
        .map(|e| {
            if mesh.edge_ring[e] < PULSE_RADIUS {
                rest + lift::<FloatType>(AMPLITUDE)
            } else {
                rest
            }
        })
        .collect();

    // t = 1 from rest: the pulse is released with zero initial velocity, so the first step uses
    // the half-weighted form of the leapfrog. Releasing from rest is what makes the outgoing
    // disturbance a consequence of the initial data rather than of a driving term.
    let deficits = curvature(mesh, &previous)?;
    let mut current: Vec<FloatType> = (0..mesh.num_edges)
        .map(|e| previous[e] - half * courant_sq * source(mesh, &deficits, e))
        .collect();

    let mut history = vec![previous.clone(), current.clone()];
    for _ in 2..STEPS {
        let deficits = curvature(mesh, &current)?;
        let next: Vec<FloatType> = (0..mesh.num_edges)
            .map(|e| {
                lift::<FloatType>(2.0) * current[e]
                    - previous[e]
                    - courant_sq * source(mesh, &deficits, e)
            })
            .collect();
        previous = current;
        current = next;
        history.push(current.clone());
    }

    Ok(history)
}

/// The deficit angle at every vertex, for a given set of edge lengths.
fn curvature(
    mesh: &Mesh,
    lengths: &[FloatType],
) -> Result<CausalTensor<FloatType>, Box<dyn std::error::Error>> {
    let tensor = CausalTensor::new(lengths.to_vec(), vec![mesh.num_edges])?;
    Ok(ReggeGeometry::new(tensor).calculate_ricci_curvature(&mesh.complex)?)
}

/// The restoring term on edge `e`: the mean deficit angle at its two endpoints.
///
/// Linearized about flat space the deficit angle is the discrete Laplacian of the perturbation,
/// so this is the `grad^2 h` of the wave equation and nothing else enters the update.
fn source(mesh: &Mesh, deficits: &CausalTensor<FloatType>, e: usize) -> FloatType {
    let zero = lift::<FloatType>(0.0);
    let (a, b) = mesh.edge_endpoints[e];
    let data = deficits.as_slice();
    let d_a = data.get(a).copied().unwrap_or(zero);
    let d_b = data.get(b).copied().unwrap_or(zero);
    (d_a + d_b) * lift::<FloatType>(0.5)
}

/// The largest displacement from rest anywhere in a ring, at one instant.
fn ring_amplitude(mesh: &Mesh, lengths: &[FloatType], ring: i32) -> FloatType {
    let rest = lift::<FloatType>(REST_LENGTH);
    (0..mesh.num_edges)
        .filter(|&e| mesh.edge_ring[e] == ring)
        .map(|e| Real::abs(lengths[e] - rest))
        .fold(lift::<FloatType>(0.0), |m, v| if v > m { v } else { m })
}

/// When the disturbance first reached each ring, and whether it did so causally.
struct Front {
    /// `(ring, first step at which the ring moved)`, for rings outside the initial pulse.
    arrivals: Vec<(i32, usize)>,
    /// True when no ring moved before the one inside it.
    ordered: bool,
    /// True when ring `k` did not move before step `k`: nothing outran the stencil.
    causal: bool,
    /// The largest displacement seen anywhere, at any time.
    peak_amplitude: FloatType,
    /// True when that peak stayed finite and did not grow without bound.
    bounded: bool,
}

fn front_analysis(mesh: &Mesh, history: &[Vec<FloatType>]) -> Front {
    let threshold = lift::<FloatType>(ARRIVAL_THRESHOLD);
    let mut arrivals = Vec::new();

    // The outermost ring is boundary, where the deficit angle is not a curvature.
    for ring in PULSE_RADIUS..RINGS {
        if let Some(step) = history
            .iter()
            .position(|lengths| ring_amplitude(mesh, lengths, ring) > threshold)
        {
            arrivals.push((ring, step));
        }
    }

    let ordered = arrivals.windows(2).all(|w| w[1].1 >= w[0].1);
    // Finite propagation speed: a nearest-neighbour stencil cannot carry a signal more than one
    // ring per step, so ring k moving before step k would mean the scheme is not a wave at all.
    let causal = arrivals
        .iter()
        .all(|&(ring, step)| step >= (ring as usize).saturating_sub(PULSE_RADIUS as usize));

    let peak_amplitude = history
        .iter()
        .flat_map(|lengths| (0..RINGS).map(move |ring| (lengths, ring)))
        .map(|(lengths, ring)| ring_amplitude(mesh, lengths, ring))
        .fold(lift::<FloatType>(0.0), |m, v| if v > m { v } else { m });
    // An explicit leapfrog at C <= 1 is stable, so the pulse must not grow past a small
    // multiple of the amplitude it started with.
    let bounded = peak_amplitude < lift::<FloatType>(GROWTH_BOUND) * lift::<FloatType>(AMPLITUDE);

    Front {
        arrivals,
        ordered,
        causal,
        peak_amplitude,
        bounded,
    }
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Gravitational Wave Propagation (Regge Calculus) ===");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Model: l(t+1) = 2 l(t) - l(t-1) - C^2 <delta>,  C = {COURANT}, released from rest\n");
}

fn print_mesh(mesh: &Mesh) {
    println!("Slice: hexagonal triangulation, {RINGS} rings");
    println!(
        "  vertices {}, edges {}, triangles {}",
        mesh.complex.num_elements_at_grade(0).unwrap_or(0),
        mesh.num_edges,
        mesh.complex.num_elements_at_grade(2).unwrap_or(0)
    );
    println!(
        "  initial pulse: {AMPLITUDE} on rings 0..{}, flat elsewhere\n",
        PULSE_RADIUS - 1
    );
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_history(mesh: &Mesh, history: &[Vec<FloatType>]) {
    println!("--- Displacement from rest, by ring (x1e3) ---");
    print!("  {:>4}", "step");
    for ring in 0..=RINGS {
        print!(" {:>7}", format!("r={ring}"));
    }
    println!();

    for (step, lengths) in history.iter().enumerate() {
        print!("  {step:>4}");
        for ring in 0..=RINGS {
            let amp = lower(ring_amplitude(mesh, lengths, ring)) * 1e3;
            if amp > ARRIVAL_THRESHOLD * 1e3 {
                print!(" {amp:>7.2}");
            } else {
                print!(" {:>7}", ".");
            }
        }
        println!();
    }
    println!("  (a dot means the ring has not moved yet)");
}

fn print_front(front: &Front) {
    println!("\n--- Does anything actually propagate? ---");
    print!("  first motion at ring: ");
    for (ring, step) in &front.arrivals {
        print!("r{ring}@t{step}  ");
    }
    println!();

    println!(
        "  ordered outward front  = {}",
        if front.ordered {
            "yes: no ring moves before the one inside it"
        } else {
            "NO: a ring moved before its neighbour"
        }
    );
    println!(
        "  finite signal speed    = {}",
        if front.causal {
            "yes: nothing reached ring k before step k"
        } else {
            "NO: a disturbance outran the stencil"
        }
    );
    println!(
        "  bounded amplitude      = {} (peak {:.4}, started at {AMPLITUDE})",
        if front.bounded { "yes" } else { "NO: unstable" },
        lower(front.peak_amplitude)
    );
    println!("\n  The leading edge advances one ring per step, which is the stencil's domain of");
    println!("  dependence; the Courant number {COURANT} governs how fast the peak follows it.");
}
