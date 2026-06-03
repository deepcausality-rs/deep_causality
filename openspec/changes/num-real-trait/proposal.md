## Why

`deep_causality_num::RealField` conflates two independent algebraic axes:

1. **Analytic structure** — the elementary functions (`sqrt`, `exp`, `ln`, `sin`, `cos`, `powf`, …), constants (`pi`, `e`, `epsilon`), ordering/rounding, and the commutative-ring arithmetic (`+ − ×`).
2. **Field structure** — total multiplicative inverse / division (`Field: CommutativeRing + InvMonoid + Div`).

A real *scalar* needs axis 1; only a *field* needs axis 2. Today there is no trait for "real analytic scalar that is **not** a field," so any type that is analytic but not invertible cannot be expressed honestly. The trigger is forward-mode automatic differentiation: a dual number `a + b·ε` (`ε² = 0`) has every elementary function (the AD chain rule), so it *is* an analytic real scalar — but it is **not a field**, because `ε` is a zero divisor. To flow a dual number through existing `T: RealField`-generic code (the basis of drop-in forward-mode AD), it would have to `impl RealField`, which forces a dishonest `Field` claim with a panicking inverse. `RealField`'s own doc comment already anticipates this ("could be implemented for other types like dual numbers (for automatic differentiation)"), and the algebra tower already separates structural axes elsewhere (`Ring` requires the `Distributive` marker but deliberately **not** `Associative`, so non-associative rings like `Octonion` fit). The missing `Real`/`Field` split is the same kind of separation, accidentally omitted.

This change closes that gap by introducing a `Real` trait (the analytic real-scalar axis) and refactoring `RealField` to extend it (`RealField: Real + Field`). It is a **prerequisite** for the Causal Arrow foundations (the `Dual<T>` type binds on `Real` and implements `Real`, not `RealField`) and for the later automatic-differentiation stage (which re-points selected `RealField`-generic numerics to `Real` so duals flow through them).

## What Changes

- Add a `Real` trait to `deep_causality_num` capturing the **analytic real-scalar** surface decoupled from field invertibility: supertraits `CommutativeRing + PartialOrd + Neg + Copy + Clone + AddAssign + SubAssign + MulAssign` (no `Div`/`DivAssign`), plus the elementary functions, constants, ordering/rounding helpers, and NaN/finiteness predicates currently declared on `RealField`.
- Refactor `RealField` to `RealField: Real + Field`. The analytic method **declarations move from `RealField` into `Real`**; `RealField` retains only the field-specific surface (division-based operations such as `inverse`). This is **behavior-preserving for every existing consumer**: a `T: RealField` bound still resolves all the same methods (now inherited via the `Real` supertrait), so no `T: RealField`-generic code changes.
- Move the `Real`-surface method **implementations** for the three concrete `RealField` impls — `f32`, `f64`, `Float106` — from their `impl RealField` blocks into new `impl Real` blocks; the `impl RealField` blocks reduce to the field-specific remainder.
- **No public API is removed and no `RealField` bound elsewhere is touched.** The change is additive (a new trait) plus an internal relocation of method declarations/impls behind the supertrait. No external or numeric dependency is added.

## Capabilities

### New Capabilities
- `real-scalar`: a `Real` trait for analytic real scalars (commutative-ring arithmetic + elementary functions/constants/ordering, **without** field invertibility), with `RealField` refactored to `RealField: Real + Field`. Lets analytic-but-non-field types (dual numbers for AD) be expressed and bounded honestly, and lets numeric code that needs only analytic operations bound on `Real` so such types flow through it.

### Modified Capabilities
<!-- None. The refactor is behavior-preserving: all existing specs that bound on `T: RealField` (generic-precision-algorithms, generic-precision-discovery, brcd-algorithm, linalg-numeric-primitives, fluid-dynamics-*, …) keep the identical RealField contract and method surface via the new supertrait, so no existing requirement changes. -->

## Impact

- **New code:** `Real` trait in `deep_causality_num/src/algebra/` (one trait, one module), re-exported from `src/lib.rs`; mirrored tests under `tests/algebra/`.
- **Refactored, behavior-preserving:** `field_real.rs` (`RealField` body slims to `Real + Field` plus field-only methods); the `Real`-surface impls for `f32`/`f64` move into `impl Real` blocks; `float_106/traits_algebra.rs` likewise for `Float106`.
- **Unaffected:** every `T: RealField` bound across the workspace (10 specs and their code: SURD/MRMR, CDL, BRCD, linalg, fluid-dynamics, …) — they keep the same resolved method surface; no call sites change.
- **APIs:** one new public trait (`Real`); `RealField` gains `Real` as a supertrait (a strict widening of what `RealField` implies, never a narrowing). No signature changes, no removals.
- **Dependencies:** none added.
- **Downstream consumers (later changes):** `causal-arrow-foundations` binds `Dual<T: Real>` and provides `impl Real for Dual<T>` (honest: analytic, not a field); `causal-arrow-autodiff` re-points selected `RealField`-generic physics/topology numerics to `Real` for drop-in forward-mode AD.
