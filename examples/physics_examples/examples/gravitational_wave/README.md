# Gravitational Wave: Regge Calculus

This example simulates gravitational wave propagation on a discrete spacetime mesh using Regge Calculus.

## How to Run

```bash
cargo run -p physics_examples --example gravitational_wave
```

---

## Engineering Value

Regge Calculus is useful for:
- **Numerical Relativity**: Simulating black hole mergers, gravitational waves
- **Quantum Gravity**: Discrete approaches to spacetime quantization
- **Mesh-Based Physics**: Games, simulations with dynamic geometry

This example shows how `calculate_ricci_curvature` computes spacetime curvature on a simplicial mesh.

---

## Physics Background

### Regge Calculus

Instead of continuous curvature, Regge Calculus uses:
- **Simplicial Complex**: Spacetime as triangles/tetrahedra
- **Edge Lengths**: Metric encoded in edge lengths
- **Deficit Angles**: Curvature concentrated at "bones" (n-2 simplices)

### Deficit Angle

For a 2D surface, curvature at a vertex is:
```
δ = 2π - Σ(angles at vertex)
```

If angles sum to less than 2π → positive curvature (like a sphere).

### Gravitational Waves

Metric perturbations propagate as waves:
1. Curvature creates stress
2. Stress changes edge lengths
3. Changed lengths create new curvature
4. Wave propagates

---

## Geometry Setup

```text
        1 ─── 2
       / \ / \
      6 ─ 0 ─ 3
       \ / \ /
        5 ─── 4

Hexagonal mesh with internal vertex 0
```

---

## Output Interpretation

```
[t=0] Center Curvature: +0.0000  ← Flat spacetime
[t=3] Center Curvature: +0.4311  ← Wave peaks
[t=6] Center Curvature: -0.3545  ← Wave troughs
```

Oscillating curvature represents the gravitational wave.

---

## Adapting This Example

1. **3D mesh**: Use tetrahedra for full 3D+1 simulation
2. **Different topologies**: Try torus, sphere, or hyperbolic meshes
3. **Source terms**: Add matter/energy sources
4. **Wave detection**: Implement "LIGO-like" detector nodes

---

## Key APIs Used

- `SimplicialComplexBuilder` - Construct discrete spacetime
- `ReggeGeometry::calculate_ricci_curvature()` - Deficit angles
- `CausalTensor` - Edge length storage
- `BaseTopology` trait - Mesh navigation
