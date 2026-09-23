# Gravitational Wave: Regge Calculus

This example releases a metric perturbation at the centre of a triangulated spatial slice and lets it propagate under a discrete wave equation whose restoring term is the curvature the mesh carries. It then measures whether the perturbation propagated.

## How to Run

```bash
cargo run -p physics_examples --example gravitational_wave
```

The run exits with status 0 when every propagation check passes and with an error otherwise.

---

## Engineering Value

Regge Calculus serves:
- **Numerical Relativity**: black hole mergers, gravitational waves
- **Quantum Gravity**: discrete approaches to spacetime quantization
- **Mesh-Based Physics**: games and simulations with dynamic geometry

`calculate_ricci_curvature` computes the curvature on a simplicial mesh, and a leapfrog driven by that curvature carries a disturbance outward.

---

## Physics Background

### Regge Calculus

Regge Calculus replaces continuous curvature with:
- **Simplicial Complex**: Spacetime as triangles/tetrahedra
- **Edge Lengths**: Metric encoded in edge lengths
- **Deficit Angles**: Curvature concentrated at "bones" (n-2 simplices)

### Deficit Angle

For a 2D surface, curvature at a vertex is:
```
δ = 2π - Σ(angles at vertex)
```

Angles that sum to less than 2π mean positive curvature (like a sphere).

### The wave equation on the mesh

Linearized about flat space, the deficit angle is the discrete Laplacian of the metric perturbation, so the vacuum wave equation discretizes to a leapfrog in the edge lengths:

```
l_e(t+1) = 2 l_e(t) - l_e(t-1) - C² (δ_a + δ_b) / 2
```

with `a, b` the endpoints of edge `e` and `C = c dt / dx` the Courant number. The pulse is released from rest, so nothing drives it after `t = 0`.

---

## Geometry Setup

A hexagonal disk of six rings of triangles, in axial coordinates: 127 vertices, 336 edges, 210 triangles. The outermost ring is boundary, where the deficit angle is an artefact of the missing triangles rather than curvature, and the analysis excludes it.

---

## What the run measures

The table shows the largest displacement from rest in each ring at each step; a dot marks a ring that has not moved yet.

```
--- Displacement from rest, by ring (x1e3) ---
  step     r=0     r=1     r=2     r=3     r=4     r=5     r=6
     0   60.00       .       .       .       .       .       .
     1   39.76    8.10       .       .       .       .       .
     2    4.09   21.45    1.75       .       .       .       .
     3   33.53   19.02    7.08    0.50       .       .       .
     4   25.76    5.50   12.57    2.70    0.12       .       .
```

Four checks follow, each stated as a measurement:

```
  first motion at ring: r1@t1  r2@t2  r3@t3  r4@t4  r5@t6
  every ring reached     = yes: rings 1..5 all moved within 16 steps
  ordered outward front  = yes: no ring moves before the one inside it
  finite signal speed    = yes: nothing reached ring k before step k
  peak within 16 steps   = yes: below the bound (peak 0.0600, bound 0.24, started at 0.06)
```

- **every ring reached**: a ring that never moved would make the next two claims vacuous, so the run requires an arrival on every ring before it makes them.
- **ordered outward front** and **finite signal speed**: together they define "an outward front". A nearest-neighbour stencil carries a signal at most one ring per step, so ring `k` moving before step `k` would mean the scheme is not a wave.
- **peak within 16 steps**: a threshold on the sampled peak over the run. An explicit leapfrog at `C ≤ 1` should stay below it. A finite window cannot prove stability, and the report does not claim it.

The leading edge advances one ring per step, which is the stencil's domain of dependence; the Courant number governs how fast the energy peak follows it.

---

## Precision

`main.rs` names the working scalar once, `pub type FloatType = Float106;`. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16` and the mesh, the deficit angles, the leapfrog and the checks recompute at that precision. `f32`, `f64` and `Float106` pass. `BFloat16` fails the checks and exits with an error: eight significand bits resolve a unit edge to about `8e-3`, so the rounding noise in the deficit angles crosses the `1e-4` arrival threshold on every ring at once.

---

## Adapting This Example

1. **3D mesh**: use tetrahedra for a full 3D+1 simulation
2. **Different topologies**: try torus, sphere or hyperbolic meshes
3. **Source terms**: add matter/energy sources
4. **Wave detection**: add "LIGO-like" detector nodes

---

## Key APIs Used

- `SimplicialComplexBuilder`: constructs the discrete spacetime
- `ReggeGeometry::calculate_ricci_curvature()`: deficit angles
- `CausalTensor`: edge length storage
- `BaseTopology` trait: mesh navigation
