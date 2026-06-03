## Context

`deep_causality_num` has an algebra tower in which traits separate structural axes:

```
RealField : Field + PartialOrd + Neg + Copy + AddAssign + SubAssign + MulAssign + DivAssign
            + { nan, is_nan, is_infinite, is_finite, clamp, sqrt, abs, floor, ceil, round,
                exp, ln, log, log2, log10, powf, sin, asin, cos, acos, tan, sinh, cosh, tanh,
                atan, atan2, pi, e, epsilon, conjugate, norm_sqr, inverse }
Field           : CommutativeRing + InvMonoid + Div + DivAssign
CommutativeRing : Ring + Commutative
Ring            : AbelianGroup + MulMonoid + Distributive      // Associative is NOT required here
```

The tower already decouples markers à la carte — `Ring` requires `Distributive` but **not** `Associative`, precisely so a non-associative ring (`Octonion`) fits. The one axis it fails to decouple is **analytic-vs-field**: every elementary function lives on `RealField`, which sits above `Field`, so a type cannot offer `sin`/`exp`/… without also claiming a total inverse. That is the gap this change closes.

Verified facts that bound the work:
- `RealField` is implemented by exactly **three** concrete types: `f32`, `f64` (`field_real.rs`), and `Float106` (`float_106/traits_algebra.rs`). That is the entire impl-split surface.
- **No openspec spec defines `RealField`'s method surface;** all 10 that mention it only *bound* on `T: RealField`. So the refactor changes no existing requirement.
- `Float` (the IEEE-float trait) is `Num`-based and outside the algebra tower, so it is not the right home for an algebra-integrated analytic scalar.

## Goals / Non-Goals

**Goals:**
- Introduce `Real`: the analytic real-scalar axis (commutative-ring arithmetic + elementary functions/constants/ordering/rounding/finiteness), **without** field invertibility.
- Refactor `RealField` to `RealField: Real + Field`, moving the analytic declarations into `Real`, with **zero behavior change** for any existing `T: RealField` consumer.
- Make it possible to bound on `Real` (for analytic-only numerics) and to implement `Real` for analytic-but-non-field types (dual numbers), honestly.

**Non-Goals:**
- Defining the `Dual<T>` type or any AD machinery (that is `causal-arrow-foundations`).
- Re-pointing existing `RealField`-generic numerics (physics/topology) to `Real` (that is `causal-arrow-autodiff`, done where drop-in AD is actually consumed).
- Adding a `Rational` type (`Float` already subsumes the representable rationals) or any new numeric type.
- Changing `Field`, `Complex`, `Quaternion`, `Octonion`, or the marker traits.

## Decisions

### D1 — `Real` is `CommutativeRing` + analytic surface, with no division

```rust
pub trait Real:
    CommutativeRing + PartialOrd + Neg<Output = Self> + Copy + Clone
    + AddAssign + SubAssign + MulAssign      // NO DivAssign — division is the field axis
{
    // constants
    fn pi() -> Self; fn e() -> Self; fn epsilon() -> Self;
    // analytic / elementary
    fn sqrt(self) -> Self; fn exp(self) -> Self; fn ln(self) -> Self;
    fn log(self, base: Self) -> Self; fn log2(self) -> Self; fn log10(self) -> Self;
    fn powf(self, n: Self) -> Self;
    fn sin(self) -> Self; fn cos(self) -> Self; fn tan(self) -> Self;
    fn asin(self) -> Self; fn acos(self) -> Self; fn atan(self) -> Self; fn atan2(self, other: Self) -> Self;
    fn sinh(self) -> Self; fn cosh(self) -> Self; fn tanh(self) -> Self;
    // sign / rounding / shape
    fn abs(self) -> Self; fn floor(self) -> Self; fn ceil(self) -> Self; fn round(self) -> Self;
    fn clamp(self, min: Self, max: Self) -> Self;
    // exceptional values
    fn nan() -> Self; fn is_nan(self) -> bool; fn is_infinite(self) -> bool; fn is_finite(self) -> bool;
}
```

The exact final method list mirrors what `RealField` declares today, minus the field-only members. **`CommutativeRing` (not `Ring`)** is the base because real scalars commute, and both `Dual` and `f64` are commutative; this keeps `Real` honest without admitting non-commutative carriers.

*Rationale for excluding division:* a dual number has total `+ − ×` (a commutative ring) and every elementary function, but only a **partial** inverse (defined when the real part is invertible). Putting `Div`/`DivAssign`/`inverse` on `Real` would re-introduce exactly the field assumption that blocks honest AD. They stay on `RealField`.

*Alternative considered:* base `Real` on `Ring` rather than `CommutativeRing`, for maximal generality. Rejected: it buys nothing (no non-commutative real scalar exists) and weakens the contract.

### D2 — `RealField: Real + Field`; analytic declarations move down

```rust
pub trait RealField: Real + Field {
    // retains only the field-specific surface (e.g. `inverse`, and any division-based provided methods)
}
```

The analytic method **declarations** move from `RealField`'s body into `Real`. Because `Real` is a supertrait of `RealField`, every existing `T: RealField` bound resolves the identical method set — `x.sin()`, `x.exp()`, constants — with no source change. `Field` continues to supply `Div`/`DivAssign`; `inverse` (which is `1/self`, a field operation) stays on `RealField`. The diamond `Real → CommutativeRing` and `Field → CommutativeRing` is permitted (shared supertrait, not a conflict).

### D3 — impl relocation for the three concrete types (behavior-preserving)

For `f32`, `f64`, `Float106`: the bodies of the analytic methods move from `impl RealField for X` into a new `impl Real for X`; the `impl RealField for X` block keeps only the field-specific remainder (and becomes a thin marker plus `inverse`). The method bodies are unchanged — they are relocated, not rewritten — so numeric results are bit-identical. Tests for the analytic surface move from the `RealField` test files into `Real` test files; field-specific tests stay.

### D4 — why this is a prerequisite, not folded into foundations

`Dual<T>` must bind on `Real` (its component needs analytic ops, not a field) and `impl Real for Dual` (so duals are first-class real scalars: nestable as `Dual<Dual<f64>>` for second derivatives, and droppable into `Real`-generic code). Both require `Real` to exist first. Keeping the trait split as its own change makes it reviewable as the `num` algebra-tower refactor it is, independent of the AD type that motivates it.

## Risks / Trade-offs

- **[Behavior drift during impl relocation.]** Moving method bodies could subtly change results. → Mitigation: relocate verbatim (no rewrites); keep the existing `RealField` analytic tests but run them through the `Real` supertrait; assert bit-identical results for `f32`/`f64`/`Float106`.
- **[Trait proliferation.]** One more trait in the tower. → Justified: it removes a real conflation (analytic vs field) and mirrors the tower's existing à-la-carte marker style; it is not speculative — two concrete consumers (the `Dual` type, the AD stage) depend on it.
- **[Diamond supertrait confusion.]** `RealField: Real + Field`, both `: CommutativeRing`. → Standard and supported in Rust; documented on the trait.
- **[Scope creep into `Field`/Complex.]** Tempting to also re-home `conjugate`/`norm_sqr`. → Keep the cut minimal: move only the clearly analytic, field-independent members; leave anything division-based on `RealField`. The precise placement of `conjugate`/`norm_sqr` is settled in implementation against their existing call sites, defaulting to `Real` only if field-independent.

## Migration Plan

Additive + internal relocation; no downstream migration. Steps: (1) add `Real`; (2) move analytic declarations into it and set `RealField: Real + Field`; (3) relocate the `f32`/`f64`/`Float106` analytic impls into `impl Real`; (4) move the analytic tests; (5) build/test `deep_causality_num`, then a workspace build to confirm every `RealField` consumer still compiles unchanged. Rollback is re-inlining `Real` into `RealField` and deleting the trait.

## Open Questions

- **`conjugate` / `norm_sqr` placement.** Field-independent enough for `Real`, or leave on `RealField`? Decide against actual call sites during implementation; default to `Real` if no division is involved.
- **Provided-method defaults.** Some `Real` methods could carry default bodies (e.g. `tan = sin/cos`) — but `tan` uses division, so a default would impose `Div`. Keep such members abstract on `Real` (each impl supplies them) to avoid leaking a field assumption into a default.
