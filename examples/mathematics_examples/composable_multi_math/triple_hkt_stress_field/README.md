# 3D Stress Analysis Blueprint: Strain to von Mises in One `extend`

This example demonstrates **modular composition** across (`topology`, `tensor`, `multivector`) inside a single comonadic step.
It runs a six-step linear-elastic stress pipeline on a  3D simplicial complex and produces per-vertex von Mises stress in Pascals.

## Introduction

Imagine you have a meshed part (a bracket, a beam, a turbine blade). At every vertex you know the local strain (how the material is deformed). You need to know the stress, project it onto a surface direction to get a traction, rotate it into the material's own frame (because steel cares about its grain direction), and reduce it to a single number an engineer can compare against a yield criterion.

That sequence shows up in structural engineering (bridges, buildings, machined parts), mechanical CAE (FEA solvers like Abaqus, Ansys, Nastran), materials science (anisotropic composites, single-crystal turbine blades), and any simulation that translates "what is the field at this point" into "what does that field do in a rotated local frame." The same pattern also lives inside graphics shaders that compute per-vertex lighting in a tangent-space basis.

## How to Run

```bash
cargo run -p mathematics_examples --example triple_hkt_stress_field_examples
```

## Sample Output

```
=== Triple HKT: 3D Stress Analysis Blueprint ===
Precision:   f64
Mesh:        2 tetrahedra sharing a face = 5 vertices, 9 edges, 7 triangles, 2 tetrahedra
Material:    steel  E = 2.00e11 Pa, nu = 0.3
Lame:        lambda = 1.154e11 Pa, mu = 7.692e10 Pa

Vertex  Position      von Mises (Pa)
------- ------------- ---------------
v0      (0,0,0)       0.000e0
v1      (1,0,0)       2.403e8
v2      (0,1,0)       0.000e0
v3      (0,0,1)       0.000e0
v4      (1,1,1)       2.403e8

One `extend` call. Three crates participated:
  topology    supplied the 3D mesh and the per-vertex walk
  tensor      ran the constitutive law and the Cauchy contraction
  multivector applied the material-frame rotation in Cl(3,0)
```

The 240 MPa peak is exactly what you would expect: the prescribed strain is proportional to the x-coordinate, so the two vertices at maximum `x` see the largest stress; the three vertices at `x = 0` see none.

---

## Key Pattern: Triple-Crate Composition

The entire stress analysis lives inside a single `ManifoldWitness::extend` call:

```rust
let result = ManifoldWitness::extend(&manifold, |w| {
    let i = w.cursor();
    if i >= N_VERTICES { return 0.0; }

    let strain = prescribed_strain(i);                                // STEP 1
    let stress = hooke_isotropic(&strain, lambda, mu);                // STEP 2
    let normal = vertex_normal(i);                                    // STEP 3
    let traction = cauchy_traction(&stress, &normal);                 // STEP 4
    let _t_local = rotate_into_frame(&traction, &rotor, &rotor_rev);  // STEP 5
    von_mises(&stress)                                                // STEP 6
});
```

Each step is a **standalone function** that can be:

- Tested independently
- Replaced without touching the others
- Reused across analyses with different material models
- Extended with state for plasticity, damage, or other history-dependent behaviour

---

## Physics Pipeline

```
strain field  ->  constitutive law  ->  normal  ->  Cauchy traction  ->  material frame  ->  scalar of interest
   eps(x)     ->  sigma = C : eps   ->   n(x)   ->     t = sigma . n  ->   t' = R t R~   ->   sigma_vm
```

| Step | Physics                                       | This Example                                  | Crate          |
|------|-----------------------------------------------|-----------------------------------------------|----------------|
| 1    | Strain field from displacement gradient       | Prescribed `eps(x)` (placeholder)             | `tensor`       |
| 2    | Constitutive law mapping strain to stress     | Isotropic linear elastic Hooke (placeholder)  | `tensor`       |
| 3    | Outward unit normal at each surface vertex    | Radial from mesh centroid (placeholder)       | -              |
| 4    | Cauchy traction `t = sigma . n`               | Real (tensor contraction via `EinSumOp`)      | `tensor`       |
| 5    | Rotate traction into material frame `R t R~`  | 10-degree Cl(3,0) rotor (placeholder)         | `multivector`  |
| 6    | Reduce to a single failure-relevant scalar    | Real (von Mises invariant)                    | -              |
| Mesh | Per-vertex walk over a 3D simplicial complex  | Real (two tets sharing a face)                | `topology`     |

---

## Simplifications in This Example

This is a **pedagogical blueprint**, not a production FEA solver. The point of the example is to expose the wiring 
so an engineer can plug in the material-specific physics.

| Aspect                  | This Example                              | Production Reality                          |
|-------------------------|-------------------------------------------|---------------------------------------------|
| **Mesh**                | 2 tetrahedra sharing a face               | Millions of elements, often hex-dominant    |
| **Mesh source**         | Hard-coded                                | Gmsh, Cubit, Salome, or CAD-derived         |
| **Strain field**        | Prescribed analytic `eps(x)`              | Symmetric gradient of unknown `u(x)`        |
| **Constitutive law**    | Isotropic Hooke (5 lines)                 | J2 plasticity, Mooney-Rivlin, anisotropic C |
| **Normal field**        | Radial from centroid                      | Per-face normals, area-weighted at vertices |
| **Material frame**      | Fixed 10-degree rotor                     | Per-vertex from grain/fiber orientation     |
| **Failure criterion**   | von Mises only                            | Tresca, Mohr-Coulomb, Tsai-Hill, Hashin     |
| **Equilibrium**         | None (stress is input)                    | Outer Newton-Raphson loop on residual       |
| **Boundary conditions** | None                                      | Dirichlet, Neumann, Robin, contact          |
| **Time evolution**      | Single-shot                               | Quasi-static load steps or full dynamics    |
| **Solver**              | Per-vertex closure                        | Sparse assembled global `K`, direct or PCG  |

---

## Path to Production Code

To evolve this example into a credible FEA inner loop, replace the placeholder functions one at a time. Each replacement is local: the `extend` walk, the cross-crate composition, and the surrounding pipeline stay where they are.

### Step 1: Real Strain from a Displacement Field

```diff
- fn prescribed_strain(vertex_idx: usize) -> Sym3 { /* analytic toy */ }
+ fn strain_from_displacement(
+     vertex_idx: usize,
+     displacement: &CausalTensor<FloatType>,  // shape [N_VERTICES, 3]
+     d1: &CsrMatrix<i8>,                       // for the discrete gradient
+ ) -> Sym3 {
+     // eps = (grad u + grad u^T) / 2, computed locally from neighbor u values
+ }
```

Read `u` at the focused vertex and its neighbors (the manifold's `w.data()` exposes the full field; neighbors come from the boundary operator `d1`).

### Step 2: Real Constitutive Law

```diff
- fn hooke_isotropic(strain: &Sym3, lambda: FloatType, mu: FloatType) -> Sym3
+ // Pick the one your material requires:
+ fn j2_plasticity_return_mapping(strain: &Sym3, state: &mut PlasticState) -> Sym3
+ fn neo_hookean(F: &Mat3) -> Sym3                       // hyperelastic (rubber)
+ fn anisotropic_hooke(strain: &Sym3, C: &CausalTensor<FloatType>) -> Sym3  // rank-4 C
+ fn drucker_prager(strain: &Sym3, friction: FloatType) -> Sym3
```

The `CausalTensor` machinery handles rank-4 stiffness tensors directly via `EinSumOp::contraction` with `lhs_axes = vec![2, 3]` against the strain.

### Step 3: Real Surface Normals

```diff
- fn vertex_normal(vertex_idx: usize) -> [FloatType; 3] { /* radial */ }
+ fn vertex_normal(
+     vertex_idx: usize,
+     boundary_faces: &[TriangleIdx],   // pre-computed list of boundary triangles
+     vertices: &[[FloatType; 3]],
+ ) -> Option<[FloatType; 3]> {
+     // None for interior vertices.
+     // Sum face normals of incident boundary triangles, weighted by area.
+     // Normalize.
+ }
```

Detect boundary triangles by looking at d3: any triangle with only one non-zero column entry in d3 is on the boundary.

### Step 4: Per-Vertex Material Rotor

```diff
- fn material_rotor() -> (CausalMultiVector, CausalMultiVector) { /* fixed */ }
+ fn material_rotor_at(
+     vertex_idx: usize,
+     orientation_field: &CausalTensor<FloatType>,  // 3-axis per vertex
+ ) -> (CausalMultiVector<FloatType>, CausalMultiVector<FloatType>) {
+     // Build rotor from local material axes: Euler angles, quaternion-derived,
+     // or pulled from polar decomposition F = R U in finite-strain plasticity.
+ }
```

### Step 5: Failure Criterion of Choice

```diff
- fn von_mises(sigma: &Sym3) -> FloatType
+ fn tresca(sigma: &Sym3) -> FloatType                       // metals
+ fn mohr_coulomb(sigma: &Sym3, phi: FloatType, c: FloatType) -> FloatType  // soils
+ fn tsai_hill(sigma_local: &Sym3, strengths: &OrthotropicStrengths) -> FloatType  // composites
+ fn hashin(sigma_local: &Sym3, mode: FailureMode) -> FloatType  // composite damage
```

Note: anisotropic criteria need `sigma` in the **material frame**, so they consume `t_local` rather than `traction`. The rotor step (currently dropped with an underscore) becomes load-bearing.

### Step 6: Bigger Mesh

```diff
- const N_VERTICES: usize = 5;
- const TETS: [[usize; 4]; 2] = [ ... ];
+ // Load from a mesh file:
+ fn build_manifold_from_gmsh(path: &Path) -> Manifold<f64, FloatType> { ... }
+ // Or generate a structured grid:
+ fn build_structured_grid(nx: usize, ny: usize, nz: usize) -> Manifold<f64, FloatType> { ... }
```

A practical add: a `mesh_io` helper module that parses a Gmsh `.msh` or VTK `.vtu` file into the same `Skeleton + boundary matrix` structure the example builds by hand. Orientation consistency (the topology crate's manifold-validation check) is the part to be careful about; for unstructured tet meshes produced by Gmsh, the writer outputs consistently oriented tets and the check passes.

### Step 7: From "Apply Stress" to "Solve for Equilibrium"

```diff
- // Single extend pass: stress in, scalar out.
+ // Outer Newton-Raphson loop wrapped in the causal monad:
+ let result = PropagatingEffect::pure(initial_displacement)
+     .bind(stage_assemble_residual)        // r(u) = K(u) u - f
+     .bind(stage_assemble_tangent)         // K_T(u)
+     .bind(stage_solve_linear_system)      // delta_u = -K_T^-1 r
+     .bind(stage_check_convergence);       // ||r|| < tolerance ?
+ // Iterate until convergence.
```

The per-vertex closure in this example becomes the **kernel** invoked during residual assembly. The outer loop runs in a `bind` chain that handles convergence, line search, and load stepping. Same architectural pattern as the multi-physics pipeline.

---

## Architecture for Production

```rust
// Production FEA inner loop, using the same composition pattern:
let stress_field = ManifoldWitness::extend(&manifold, |w| {
    let i = w.cursor();
    if !is_solid_vertex(i, &manifold) { return SymTensor::zero(); }

    let eps    = strain_from_displacement(i, &u, &grad_op);
    let sigma  = constitutive::j2_plasticity(&eps, &mut state.at_vertex(i));
    let normal = boundary_normal_at(i, &boundary_faces, &VERTICES)?;
    let t      = cauchy_traction(&sigma, &normal);
    let rotor  = material_rotor_at(i, &orientation_field);
    let t_loc  = rotate_into_frame(&t, &rotor.fwd, &rotor.rev);
    failure::tsai_hill(&sigma_local_from(&sigma, &rotor), &composite_strengths)
});
```

The **cross-crate composition pattern remains the same**. Only the function bodies change.

---

## Key APIs Used

| API                                | Purpose                                              |
|------------------------------------|------------------------------------------------------|
| `Manifold::new`                    | Wraps the simplicial complex with vertex data       |
| `ManifoldWitness::extend`          | Per-vertex walk (CoMonad)                            |
| `SimplicialComplex::new`           | 3D mesh with vertex/edge/triangle/tetrahedra skeletons |
| `CsrMatrix::from_triplets`         | Boundary operators `d1`, `d2`, `d3`                  |
| `CausalTensor::ein_sum`            | Cauchy contraction `t_i = sigma_ij n_j`              |
| `CausalMultiVector::geometric_product` | Material-frame rotor sandwich `R t R~`           |
| `Metric::Euclidean(3)`             | `Cl(3,0)` signature for the rotor                    |

---

## Engineering Value

This pattern applies to any per-vertex field-computation pipeline that crosses topological, linear-algebraic, and orientational layers:

- **Structural FEA**: stress, strain, displacement, failure criteria
- **Computational fluid dynamics**: per-cell velocity, pressure, stress-tensor manipulations
- **Computer graphics**: per-vertex shading in tangent-space bases
- **Geophysics**: strain-rate tensors on subduction-zone meshes
- **Biomechanics**: tissue stress on patient-specific anatomical meshes

The key insight: **mesh walk, tensor algebra, and frame rotation are orthogonal concerns; composing them through HKT witnesses keeps each replaceable.**
