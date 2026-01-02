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
|---------------------|------------------------------------------|--------------------------|---------
| Core traits         | `type Constraint` on HKT                 | ✅ Implemented            | PASS    |
| `Satisfies` trait   | Universal marker                         | ✅ Implemented            | PASS    |
| Unbounded types     | `NoConstraint`                           | ✅ `Vec`, `Option`, `Box` | PASS    |
| Physics types       | `TensorDataConstraint`                   | ❌ Uses `NoConstraint`    | **GAP** |
| Algebraic hierarchy | `RingConstraint`, `FieldConstraint` etc. | ❌ Not used               | **GAP** |

---

## Orphan Rule Clarification

> [!IMPORTANT]
> **Constraints MUST be defined in the crate where the HKT extension resides, NOT in `deep_causality_haft`.**

The spec originally suggested defining algebraic constraints centrally in `haft`. However, this violates Rust's **orphan
rule**:

```rust
// ❌ ILLEGAL in deep_causality_tensor:
// Both `Satisfies` and `FieldConstraint` are external to this crate
impl<T: Field> Satisfies<FieldConstraint> for T {}  // E0117: orphan rule violation
```

**Correct Pattern:**

```rust
// ✅ LEGAL in deep_causality_tensor:
// Define the constraint struct locally
pub struct TensorConstraint;

// Implement Satisfies for specific types (whitelist approach)
impl Satisfies<TensorConstraint> for f32 {}
impl Satisfies<TensorConstraint> for f64 {}
// ...
```

This is exactly what [
`ext_hkt_strict.rs`](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_tensor/src/extensions/ext_hkt_strict.rs)
does correctly.

---

## Deep Dive: Tensor Crate Strict HKT Analysis

### Current State

[
`ext_hkt_strict.rs`](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_tensor/src/extensions/ext_hkt_strict.rs)
defines:

- `TensorConstraint` struct with whitelisted types (f32, f64, Complex, integers)
- `StrictCausalTensorWitness` with `type Constraint = TensorConstraint`
- Only `Functor` and `Foldable` implemented

The file contains this comment:

```rust
/// **Note:** Because this constraint excludes closures (`Fn`), this witness CANNOT implement
/// `Applicative` or `Monad`. It is restricted to `Functor`, `Foldable`, and `CoMonad`.
```

### Trait Signature Analysis

Let's examine each trait's requirements:

| Trait         | Supertrait    | `Func` Bound                       | Can Implement? |
|---------------|---------------|------------------------------------|----------------|
| `Functor`     | —             | `Func: FnMut(A) -> B`              | ✅ Yes          |
| `Foldable`    | —             | `Func: FnMut(B, A) -> B`           | ✅ Yes          |
| `Applicative` | `Functor`     | `Func: Satisfies<C> + FnMut(A)->B` | ❌ No           |
| `Monad`       | `Applicative` | `Func: FnMut(A) -> F::Type<B>`     | ❌ No (blocked) |
| `CoMonad`     | `Functor`     | `Func: FnMut(&F::Type<A>) -> B`    | ✅ Yes          |

### Key Finding: The Inheritance Chain Problem

> [!CAUTION]
> **`Monad` is blocked by its supertrait, not its own signature.**

The trait hierarchy is:

```rust
pub trait Monad<F: HKT>: Applicative<F> { ... }  // Monad EXTENDS Applicative
pub trait CoMonad<F: HKT>: Functor<F> { ... }    // CoMonad only extends Functor
```

**Analysis:**

1. **Applicative::apply** requires `Func: Satisfies<F::Constraint>` because the function is wrapped: `F::Type<Func>`
2. Closures cannot implement custom constraints (no way to `impl Satisfies<TensorConstraint> for Fn...`)
3. **Monad::bind** itself does NOT require `Func: Satisfies<C>` — the closure is unwrapped
4. **However**, `Monad: Applicative` means you MUST implement `Applicative` first
5. Since `Applicative` is blocked, `Monad` is transitively blocked

**CoMonad is safe** because `CoMonad: Functor` — no `Applicative` in the chain.

### The Comment in ext_hkt_strict.rs is CORRECT

The original comment correctly states:

```rust
/// **Note:** Because this constraint excludes closures (`Fn`), this witness CANNOT implement
/// `Applicative` or `Monad`. It is restricted to `Functor`, `Foldable`, and `CoMonad`.
```

### Design Options to Unblock Monad

1. **Break inheritance**: Change `Monad: Functor` instead of `Monad: Applicative`
   - Requires duplicating `pure` in both traits
   - Breaks Haskell convention but enables strict Monad

2. **Create StrictMonad**: New trait without Applicative dependency
   ```rust
   pub trait StrictMonad<F: HKT>: Functor<F> {
       fn pure<T>(value: T) -> F::Type<T> where T: Satisfies<F::Constraint>;
       fn bind<A, B, Func>(...) -> F::Type<B>;
   }
   ```

3. **Accept limitation**: Use unbounded witness for full monadic stack

### Trait Coverage Summary (CORRECTED)

| Trait         | NoConstraint Witness | Strict Witness (Current) | Strict Witness (Max Possible) |
|---------------|----------------------|--------------------------|-------------------------------|
| `HKT`         | ✅                    | ✅                        | ✅                             |
| `Functor`     | ✅                    | ✅                        | ✅                             |
| `Foldable`    | ✅                    | ✅                        | ✅                             |
| `Applicative` | ✅                    | ❌                        | ❌ Blocked (closure problem)   |
| `Monad`       | ✅                    | ❌                        | ❌ Blocked (supertrait)        |
| `CoMonad`     | ✅                    | ❌                        | ✅ Implementable               |
| `Adjunction`  | TBD                  | ❌                        | ⚠️ Needs analysis             |

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

**Files:**

- [ext_hkt.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_tensor/src/extensions/ext_hkt.rs) —
  Unbounded
- [ext_hkt_strict.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_tensor/src/extensions/ext_hkt_strict.rs) —
  Strict (partial)

| Witness                     | Constraint         | Traits Implemented             |
|-----------------------------|--------------------|--------------------------------|
| `CausalTensorWitness`       | `NoConstraint`     | All (Functor, Monad, CoMonad…) |
| `StrictCausalTensorWitness` | `TensorConstraint` | Functor, Foldable only         |

**Gap:** `Monad` and `CoMonad` can be added to strict witness.

---

### deep_causality_sparse ⚠️

**File:
** [ext_hkt.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_sparse/src/extensions/ext_hkt.rs)

Uses `NoConstraint`. Should follow tensor's dual-witness pattern with a `SparseConstraint`.

---

### deep_causality_multivector ⚠️

**Files:
** [hkt_multivector/mod.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_multivector/src/extensions/hkt_multivector/mod.rs), [hkt_multifield/mod.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_multivector/src/extensions/hkt_multifield/mod.rs)

Uses `NoConstraint`. Should follow tensor's dual-witness pattern.

---

### deep_causality_topology ✅ (Best Example)

**File:
** [hkt_manifold.rs](file:///Users/marvin/RustroverProjects/dcl/deep_causality/deep_causality_topology/src/extensions/hkt_manifold.rs)

Correctly implements dual-witness pattern with `ManifoldWitness` (unbounded) and `StrictManifoldWitness` (bounded).

---

## Action Items

### Tensor Crate

1. Add `CoMonad<StrictCausalTensorWitness>` implementation
2. ~~Add Monad~~ — Blocked by Applicative supertrait (see Design Options above)

### Other Crates

Follow tensor/topology pattern: define local constraint struct + dual witnesses.

### Architectural Decision Needed

> [!IMPORTANT]
> **Should `Monad` be decoupled from `Applicative`?**

The current Haskell-style hierarchy (`Functor → Applicative → Monad`) blocks strict Monad. Options:

1. Keep current design — Accept that strict witnesses only get `Functor`, `Foldable`, `CoMonad`
2. Create `StrictMonad` trait — Parallel hierarchy for constrained types
3. Refactor `Monad: Functor` — Breaking change to enable strict Monad

---

## Summary

| Crate                        | Spec Compliance | Action Needed                  |
|------------------------------|-----------------|--------------------------------|
| `deep_causality_haft`        | ✅ 100%          | None (or consider Monad decoupling) |
| `deep_causality_tensor`      | ⚠️ 70%          | Add CoMonad to strict witness  |
| `deep_causality_sparse`      | ⚠️ 60%          | Add dual-witness pattern       |
| `deep_causality_multivector` | ⚠️ 60%          | Add dual-witness pattern       |
| `deep_causality_topology`    | ✅ 85%           | Already has dual-witness       |

**Overall:** Core infrastructure is solid. Strict witnesses are limited to `Functor`, `Foldable`, `CoMonad` due to `Monad: Applicative` inheritance.


