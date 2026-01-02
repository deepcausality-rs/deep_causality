# HKT GAT Implementation Review

**Date:** 2026-01-02  
**Spec:** [hkt_gat.md](file:///Users/marvin/RustroverProjects/dcl/deep_causality/specs/current/hkt_gat.md)  
**Status:** Gap Analysis

---

## Executive Summary

The unified GAT-based HKT system has been **partially implemented**. The core infrastructure (`Satisfies`,
`NoConstraint`, unified trait signatures) is complete in `deep_causality_haft`. However, downstream crates diverge from
the spec by using `NoConstraint` universally rather than domain-specific constraints.

| Aspect              | Spec Requirement                         | Current State            | Verdict |
|---------------------|------------------------------------------|--------------------------|---------|
| Core traits         | `type Constraint` on HKT                 | ✅ Implemented            | PASS    |
| `Satisfies` trait   | Universal marker                         | ✅ Implemented            | PASS    |
| Unbounded types     | `NoConstraint`                           | ✅ `Vec`, `Option`, `Box` | PASS    |
| Physics types       | `TensorDataConstraint`                   | ❌ Uses `NoConstraint`    | **GAP** |
| Algebraic hierarchy | `RingConstraint`, `FieldConstraint` etc. | ❌ Not used               | **GAP** |

---

## Per-Crate Analysis

### deep_causality_haft ✅

**Files:
** [hkt_vec_ext.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_haft/src/extensions/hkt_vec_ext.rs), [hkt_option_ext.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_haft/src/extensions/hkt_option_ext.rs),
etc.

| Witness                         | Constraint     | Spec Compliance |
|---------------------------------|----------------|-----------------|
| `VecWitness`                    | `NoConstraint` | ✅ Correct       |
| `OptionWitness`                 | `NoConstraint` | ✅ Correct       |
| `BoxWitness`                    | `NoConstraint` | ✅ Correct       |
| `ResultWitness`                 | `NoConstraint` | ✅ Correct       |
| `Tuple2Witness`/`Tuple3Witness` | `NoConstraint` | ✅ Correct       |

**Verdict:** Fully compliant. Generic containers correctly use `NoConstraint`.

---

### deep_causality_tensor ⚠️

**File:
** [ext_hkt.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_tensor/src/extensions/ext_hkt.rs)

```rust
impl HKT for CausalTensorWitness {
    type Constraint = NoConstraint;  // ❌ Should be TensorDataConstraint
    type Type<T> = CausalTensor<T>;
}
```

| Witness               | Current        | Spec Requires          |
|-----------------------|----------------|------------------------|
| `CausalTensorWitness` | `NoConstraint` | `TensorDataConstraint` |

**Gap:** Per spec §4.2, `CausalTensorWitness` should use `TensorDataConstraint` to enforce
`Field + Copy + Default + PartialOrd + Send + Sync`.

**Impact:**

- No compile-time enforcement that tensor elements are numeric
- Types like `String` can be wrapped in `CausalTensor` via HKT operations

---

### deep_causality_sparse ⚠️

**File:
** [ext_hkt.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_sparse/src/extensions/ext_hkt.rs)

```rust
impl HKT for CsrMatrixWitness {
    type Constraint = NoConstraint;  // ❌ Should be FieldConstraint
    type Type<T> = CsrMatrix<T>;
}
```

| Witness            | Current        | Spec Requires     |
|--------------------|----------------|-------------------|
| `CsrMatrixWitness` | `NoConstraint` | `FieldConstraint` |

**Gap:** Sparse matrices require field operations (division for inverses, etc.).

---

### deep_causality_multivector ⚠️

**Files:
** [hkt_multivector/mod.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_multivector/src/extensions/hkt_multivector/mod.rs), [hkt_multifield/mod.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_multivector/src/extensions/hkt_multifield/mod.rs)

```rust
impl HKT for CausalMultiVectorWitness {
    type Constraint = NoConstraint;  // ❌ Should be AssociativeRingConstraint
}

impl<B: LinearAlgebraBackend> HKT for CausalMultiFieldWitness<B> {
    type Constraint = NoConstraint;  // ❌ Should be TensorDataConstraint
}
```

| Witness                    | Current        | Spec Requires               |
|----------------------------|----------------|-----------------------------|
| `CausalMultiVectorWitness` | `NoConstraint` | `AssociativeRingConstraint` |
| `CausalMultiFieldWitness`  | `NoConstraint` | `TensorDataConstraint`      |

**Gap:** Multivectors require associative ring (for geometric product), multifields require tensor data.

---

### deep_causality_topology ⚠️ (Partial)

**File:
** [hkt_manifold.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_topology/src/extensions/hkt_manifold.rs)

```rust
// Unbounded witness (correct for pipelines)
impl HKT for ManifoldWitness {
    type Constraint = NoConstraint;
}

// Bounded witness ✅ UNIQUE: Actually defines a custom constraint!
pub struct ManifoldConstraint;
impl Satisfies<ManifoldConstraint> for f32 {}
impl Satisfies<ManifoldConstraint> for f64 {}
impl Satisfies<ManifoldConstraint> for Complex<f32> {}
// ...

impl HKT for StrictManifoldWitness {
    type Constraint = ManifoldConstraint;  // ✅ Custom constraint!
}
```

| Witness                 | Current              | Compliance                 |
|-------------------------|----------------------|----------------------------|
| `ManifoldWitness`       | `NoConstraint`       | ⚠️ Intentionally unbounded |
| `StrictManifoldWitness` | `ManifoldConstraint` | ✅ Custom constraint        |

**Verdict:** Topology is the **only crate** following the dual-witness pattern. However, its custom `ManifoldConstraint`
is not part of the spec's algebraic hierarchy.

---

## Missing Constraint Definitions

Per spec §3.2.3, these constraints should exist in `deep_causality_haft`:

| Constraint                  | Status      | Location                             |
|-----------------------------|-------------|--------------------------------------|
| `NoConstraint`              | ✅           | `haft/src/hkt/mod.rs`                |
| `CloneConstraint`           | ❌ Not found | —                                    |
| `ThreadSafeConstraint`      | ❌ Not found | —                                    |
| `AbelianGroupConstraint`    | ❌ Not found | —                                    |
| `RingConstraint`            | ❌ Not found | —                                    |
| `AssociativeRingConstraint` | ❌ Not found | —                                    |
| `CommutativeRingConstraint` | ❌ Not found | —                                    |
| `FieldConstraint`           | ❌ Not found | —                                    |
| `RealFieldConstraint`       | ❌ Not found | —                                    |
| `TensorDataConstraint`      | ❌ Not found | Should be in `deep_causality_tensor` |

---

## Recommendations

### Priority 1: Define Algebraic Constraint Hierarchy

Add to `deep_causality_haft/src/hkt/constraints.rs`:

```rust
pub struct FieldConstraint;
impl<T: deep_causality_num::Field + Copy> Satisfies<FieldConstraint> for T {}

pub struct AssociativeRingConstraint;
impl<T: deep_causality_num::AssociativeRing + Copy> Satisfies<AssociativeRingConstraint> for T {}

pub struct TensorDataConstraint;
impl<T: crate::TensorData> Satisfies<TensorDataConstraint> for T {}
```

### Priority 2: Update Physics Witnesses

| Crate                        | Change                                                             |
|------------------------------|--------------------------------------------------------------------|
| `deep_causality_tensor`      | `CausalTensorWitness::Constraint = TensorDataConstraint`           |
| `deep_causality_sparse`      | `CsrMatrixWitness::Constraint = FieldConstraint`                   |
| `deep_causality_multivector` | `CausalMultiVectorWitness::Constraint = AssociativeRingConstraint` |

### Priority 3: Add Dual Witnesses

Follow topology's pattern: provide both `FreeWitness` (NoConstraint) and `StrictWitness` (domain constraint) for physics
types.

---

## Summary

| Crate                        | Spec Compliance | Action Needed                   |
|------------------------------|-----------------|---------------------------------|
| `deep_causality_haft`        | ✅ 100%          | None                            |
| `deep_causality_tensor`      | ⚠️ 60%          | Add `TensorDataConstraint`      |
| `deep_causality_sparse`      | ⚠️ 60%          | Add `FieldConstraint`           |
| `deep_causality_multivector` | ⚠️ 60%          | Add `AssociativeRingConstraint` |
| `deep_causality_topology`    | ✅ 85%           | Minor: align constraint names   |

**Overall:** Core infrastructure is solid. Domain constraints are the gap.
