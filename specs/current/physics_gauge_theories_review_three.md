# Physics vs Topology Kernel Review

This document provides a comprehensive analysis of `deep_causality_physics` kernels and their potential replacement with
`deep_causality_topology` Gauge Field operations.

---

## Executive Summary

| Theory             | Kernels Reviewed | Already Using Topology | Replaceable | Gaps Identified |
|--------------------|------------------|------------------------|-------------|-----------------|
| General Relativity | 6                | 3                      | 1           | 1               |
| Weak Force         | 2                | 1                      | 0           | 0               |
| Electromagnetism   | 2                | 0                      | 0           | 1               |
| Electroweak        | 3                | 0                      | 0           | 1               |

---

## 1. General Relativity (GR)

### 1.1 Einstein Tensor

| Attribute               | Value                                                       |
|-------------------------|-------------------------------------------------------------|
| **Kernel**              | `einstein_tensor_kernel`                                    |
| **Location**            | `deep_causality_physics/src/relativity/gravity.rs`          |
| **Topology Equivalent** | `CurvatureTensor::einstein_tensor`                          |
| **Topology Location**   | `deep_causality_topology/src/types/curvature_tensor/mod.rs` |
| **Status**              | **REPLACEABLE**                                             |

**Recommendation:** The manual kernel duplicates logic found in `CurvatureTensor`. The `GrOps` implementation should use
`CurvatureTensor::einstein_tensor` directly

---

### 1.2 Geodesic Deviation

| Attribute               | Value                                                                     |
|-------------------------|---------------------------------------------------------------------------|
| **Kernel**              | `geodesic_deviation_kernel`                                               |
| **Location**            | `deep_causality_physics/src/relativity/gravity.rs`                        |
| **Topology Equivalent** | `CurvatureTensorWitness::curvature` (implements `RiemannMap`)             |
| **Topology Location**   | `deep_causality_topology/src/extensions/hkt_gauge_field/hkt_curvature.rs` |
| **Status**              | **ALREADY USED**                                                          |

**Recommendation:** The code correctly utilizes the topology witness. No change needed.

---

### 1.3 Kretschmann Scalar

| Attribute               | Value                                                                   |
|-------------------------|-------------------------------------------------------------------------|
| **Kernel**              | Manual implementation inside `GrOps::kretschmann_scalar`                |
| **Location**            | `deep_causality_physics/src/theories/general_relativity/gr_ops_impl.rs` |
| **Topology Equivalent** | `CurvatureTensor::kretschmann_scalar`                                   |
| **Topology Location**   | `deep_causality_topology/src/types/curvature_tensor/mod.rs`             |
| **Status**              | **GAP IDENTIFIED**                                                      |

**Gap Analysis:**
The physics implementation performs full metric-aware index raising:

```
R^abcd = g^am g^bn g^cr g^ds R_mnrs
K = R_abcd × R^abcd
```

The current topology implementation simplifies this by using `R^d_abc` directly without lowering/raising indices via the
metric.

**Closing the Gap:**
Add a `kretschmann_scalar_with_metric(&inverse_metric)` method to `CurvatureTensor` that accepts an inverse metric
tensor and performs proper index raising before contraction.

---

### 1.4 Riemann from Christoffel

| Attribute               | Value                                                                         |
|-------------------------|-------------------------------------------------------------------------------|
| **Kernel**              | `GrOps::compute_riemann_from_christoffel`                                     |
| **Location**            | `deep_causality_physics/src/theories/general_relativity/gr_ops_impl.rs`       |
| **Topology Equivalent** | `GaugeFieldWitness::compute_field_strength_non_abelian`                       |
| **Topology Location**   | `deep_causality_topology/src/extensions/hkt_gauge_field/hkt_gauge_witness.rs` |
| **Status**              | **ALREADY USED**                                                              |

**Recommendation:** Implementation correctly uses the topology witness. No change needed.

---

### 1.5 Geodesic Integrator

| Attribute               | Value                                              |
|-------------------------|----------------------------------------------------|
| **Kernel**              | `geodesic_integrator_kernel` (RK4)                 |
| **Location**            | `deep_causality_physics/src/relativity/gravity.rs` |
| **Topology Equivalent** | None                                               |
| **Status**              | **NO EQUIVALENT**                                  |

**Note:** This is a numerical ODE solver, not a topological operation. It remains in physics as appropriate.

---

### 1.6 Parallel Transport / Proper Time

| Attribute               | Value                                              |
|-------------------------|----------------------------------------------------|
| **Kernel**              | `parallel_transport_kernel`, `proper_time_kernel`  |
| **Location**            | `deep_causality_physics/src/relativity/gravity.rs` |
| **Topology Equivalent** | None currently                                     |
| **Status**              | **NO EQUIVALENT**                                  |

**Note:** These are metric-based computations that could potentially be added to topology in the future, but are
appropriately placed in physics for now.

---

## 2. Weak Force (SU(2))

### 2.1 Field Strength Calculation

| Attribute               | Value                                                                         |
|-------------------------|-------------------------------------------------------------------------------|
| **Kernel**              | `WeakFieldOps::weak_field_strength`                                           |
| **Location**            | `deep_causality_physics/src/theories/weak_force/weak_field_ops_impl.rs`       |
| **Topology Equivalent** | `GaugeFieldWitness::compute_field_strength_non_abelian`                       |
| **Topology Location**   | `deep_causality_topology/src/extensions/hkt_gauge_field/hkt_gauge_witness.rs` |
| **Status**              | **ALREADY USED**                                                              |

**Recommendation:** The code correctly utilizes the topology witness. No change needed.

---

### 2.2 Propagators and Decay Widths

| Attribute               | Value                                                                                |
|-------------------------|--------------------------------------------------------------------------------------|
| **Kernel**              | `charged_current_propagator`, `neutral_current_propagator`, `weak_decay_width`, etc. |
| **Location**            | `deep_causality_physics/src/theories/weak_force/weak_field_ops_impl.rs`              |
| **Topology Equivalent** | None                                                                                 |
| **Status**              | **NO EQUIVALENT**                                                                    |

**Note:** These are quantum field theory calculations (perturbative physics), not topological operations. They remain in
physics as appropriate.

---

## 3. Electromagnetism (U(1))

### 3.1 Field Strength from E/B Vectors

| Attribute               | Value                                                                         |
|-------------------------|-------------------------------------------------------------------------------|
| **Kernel**              | Manual $F_{\mu\nu}$ population from $\vec{E}, \vec{B}$ in `from_fields`       |
| **Location**            | `deep_causality_physics/src/theories/electromagnetism/gauge_em_ops_impl.rs`   |
| **Topology Equivalent** | `GaugeFieldWitness::compute_field_strength_abelian`                           |
| **Topology Location**   | `deep_causality_topology/src/extensions/hkt_gauge_field/hkt_gauge_witness.rs` |
| **Status**              | **GAP IDENTIFIED**                                                            |

**Gap Analysis:**
The topology witness requires a 4-potential $A_\mu$ to compute:

```
F_μν = ∂_μ A_ν - ∂_ν A_μ
```

However, the physics implementation constructs $F_{\mu\nu}$ directly from $\vec{E}$ and $\vec{B}$ field vectors, where
the potential is not the primary input.

**Closing the Gap:**

1. **Option A (Preferred):** Add `GaugeFieldWitness::field_strength_from_fields(e: &[T], b: &[T])` to topology that
   constructs $F_{\mu\nu}$ from $E_i, B_i$ components directly.
2. **Option B:** Refactor EM to use potential-first approach, then use existing witness.

---

### 3.2 Computed Field Strength via Witness

| Attribute               | Value                                                                       |
|-------------------------|-----------------------------------------------------------------------------|
| **Kernel**              | `GaugeEmOps::computed_field_strength`                                       |
| **Location**            | `deep_causality_physics/src/theories/electromagnetism/gauge_em_ops_impl.rs` |
| **Topology Equivalent** | Could use `GaugeFieldWitness::compute_field_strength_abelian`               |
| **Status**              | **PARTIALLY ALIGNED**                                                       |

**Note:** The trait method signature suggests using the witness, but the implementation currently returns the stored
field strength directly. Consider adding a variant that computes from the connection.

---

## 4. Electroweak (SU(2) × U(1))

### 4.1 Field Creation

| Attribute               | Value                                                                 |
|-------------------------|-----------------------------------------------------------------------|
| **Kernel**              | `ElectroweakOps::new_field`                                           |
| **Location**            | `deep_causality_physics/src/theories/electroweak/electroweak_impl.rs` |
| **Topology Equivalent** | `GaugeField::new` with `Electroweak` gauge group                      |
| **Status**              | **ALREADY USED**                                                      |

**Recommendation:** No change needed.

---

### 4.2 Weinberg Angle Mixing (Photon/Z Extraction)

| Attribute               | Value                                                                 |
|-------------------------|-----------------------------------------------------------------------|
| **Kernel**              | `extract_photon`, `extract_z`                                         |
| **Location**            | `deep_causality_physics/src/theories/electroweak/electroweak_impl.rs` |
| **Topology Equivalent** | None                                                                  |
| **Status**              | **GAP IDENTIFIED**                                                    |

**Gap Analysis:**
The Weinberg mixing rotates the gauge fields:

```
A_μ = B_μ cos(θ_W) + W³_μ sin(θ_W)   (Photon)
Z_μ = -B_μ sin(θ_W) + W³_μ cos(θ_W)  (Z boson)
```

This is a gauge-algebraic operation that should be in topology.

**Closing the Gap:**
Add `GaugeFieldWitness::gauge_rotation<G1, G2>(field: &GaugeField<G1>, angle: T) -> GaugeField<G2>` to topology. This
would generalize Weinberg mixing as a rotation in the product gauge bundle space.

---

### 4.3 Mass and Mixing Parameters

| Attribute               | Value                                                                 |
|-------------------------|-----------------------------------------------------------------------|
| **Kernel**              | `sin2_theta_w`, `w_mass`, `z_mass`                                    |
| **Location**            | `deep_causality_physics/src/theories/electroweak/electroweak_impl.rs` |
| **Topology Equivalent** | None                                                                  |
| **Status**              | **NO EQUIVALENT**                                                     |

**Note:** These are physical constants, not topological operations. They remain in physics as appropriate.

---

## Summary of Gaps and Recommended Actions

### Gap 1: Kretschmann Scalar with Full Index Raising

| Attribute          | Value                                                         |
|--------------------|---------------------------------------------------------------|
| **File to Modify** | `deep_causality_topology/src/types/curvature_tensor/mod.rs`   |
| **Change**         | Add `kretschmann_scalar_with_metric(&CausalTensor<T>)` method |
| **Complexity**     | Medium                                                        |
| **Priority**       | High (GR correctness)                                         |

### Gap 2: Field Strength from E/B Vectors

| Attribute          | Value                                                                         |
|--------------------|-------------------------------------------------------------------------------|
| **File to Modify** | `deep_causality_topology/src/extensions/hkt_gauge_field/hkt_gauge_witness.rs` |
| **Change**         | Add `field_strength_from_eb_vectors(e: &[T], b: &[T])` method                 |
| **Complexity**     | Low                                                                           |
| **Priority**       | Medium (EM convenience)                                                       |

### Gap 3: Gauge Rotation for Weinberg Mixing

| Attribute          | Value                                                                         |
|--------------------|-------------------------------------------------------------------------------|
| **File to Modify** | `deep_causality_topology/src/extensions/hkt_gauge_field/hkt_gauge_witness.rs` |
| **Change**         | Add `gauge_rotation<G1, G2>(field, angle)` method                             |
| **Complexity**     | High                                                                          |
| **Priority**       | Medium (EW generalization)                                                    |

---

## HKT Operations Summary

The following table summarizes the available HKT operations in the topology layer:

| Operation                            | Trait               | Location                   | Used By               |
|--------------------------------------|---------------------|----------------------------|-----------------------|
| `compute_field_strength_abelian`     | `GaugeFieldWitness` | `hkt_gauge_witness.rs`     | EM (potential)        |
| `compute_field_strength_non_abelian` | `GaugeFieldWitness` | `hkt_gauge_witness.rs`     | Weak, GR              |
| `merge_fields`                       | `GaugeFieldWitness` | `hkt_gauge_witness.rs`     | Field coupling        |
| `gauge_transform`                    | `GaugeFieldWitness` | `hkt_gauge_witness.rs`     | Gauge transformations |
| `curvature` (R(u,v)w)                | `RiemannMap`        | `hkt_curvature.rs`         | GR geodesic deviation |
| `scatter`                            | `RiemannMap`        | `hkt_curvature.rs`         | Scattering            |
| `exterior_derivative`                | `StokesAdjunction`  | `hkt_adjunction_stokes.rs` | Differential forms    |
| `boundary`                           | `StokesAdjunction`  | `hkt_adjunction_stokes.rs` | Chains                |
| `integrate`                          | `StokesAdjunction`  | `hkt_adjunction_stokes.rs` | Form-chain pairing    |
