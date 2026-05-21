## Context

`deep_causality_physics` is built around two patterns that both bake `f64` into the public API:

1. **Validating newtype wrappers.** Each physical quantity (mass, speed, force, length, etc.) is a wrapper struct that stores a `f64` and validates the value at construction. The wrapper is the abstraction; the underlying `f64` is incidental. The pattern is uniform across the 47 wrapper types audited.
2. **Top-level `pub const` physical constants.** 76 declarations of the form `pub const NAME: f64 = ...;` for the universal, atomic, electromagnetic, thermodynamic, particle-physics, electroweak, and Earth-gravity constants the rest of the crate consumes.

Adjacent to these are the cross-crate consumers — `CausalMultiVector<f64>`, `CausalTensor<f64>`, `Complex<f64>` storage inside seven wrapper types (`PhysicalVector`, `SpacetimeVector`, `PhysicalField`, `AbcdMatrix`, `JonesVector`, `StokesVector`, `ComplexBeamParameter`) — plus the ODE / RK4 / Kalman-filter routines in `relativity/gravity.rs`, `chronometric/solve_gm.rs`, `theories/general_relativity/adm_state.rs`, `dynamics/estimation.rs`. These already attempt generalization (`T: Float + From<f64>` style bounds) but use the `From<f64>` round-trip crutch R0 removes from topology.

R0 (`generalize-topology-over-realfield`) establishes:

- `RealField::from_f64(value: f64) -> Self` is available on `deep_causality_num`.
- Topology types (`SimplicialManifold`, `CubicalReggeGeometry`, `Manifold`, etc.) are generic over `R: RealField`.
- The "hard rip-and-replace; no bridge code; propagation through libraries; concrete `R` only at end-consumer sites" policy is established.

This change set applies the same pattern to `deep_causality_physics`. Once it lands, the crate is precision-parametric end-to-end, the `::<f64>` temporary pins R0 introduces are removed, and `deep_causality_effects` is the last remaining migration target.

Stakeholders: high-precision relativity / cosmology workloads (`f128` post-Newtonian expansion); memory-bound particle-physics simulations (`f32` for millions of `FourMomentum` instances); anyone using the photonics / electromagnetism / mechanics surfaces; the cubical Regge calculus roadmap (R1–R6 consume `deep_causality_physics` constants like the gravitational constant).

## Goals / Non-Goals

**Goals:**

- After this change set ships, the public API of `deep_causality_physics` contains **zero** hardcoded `f64` or `f32` in any struct field, function/method signature, trait method, error variant, trait bound, or `pub const` declaration.
- Every wrapper struct is `WrapperName<R: RealField>(R)` (or the multi-field equivalent).
- Every physical constant is a function `pub fn name<R: RealField>() -> R { R::from_f64(literal) }` instead of a `pub const`.
- Every internal `<T as From<f64>>::from(literal)` round-trip is replaced by `R::from_f64(literal)` or a `RealField`-native expression.
- The temporary `::<f64>` pins R0 introduces in physics are removed; topology / tensor / multivector consumption propagates `R` naturally.
- Behavior at `R = f64` is bit-identical to the pre-change baseline. Every existing test passes after retyping call sites.
- `f32` property tests are added per domain to gate the parameterization is real.

**Non-Goals:**

- New physics. The algorithmic content of every solver (Cayley-Menger, RK4, Kalman filter, ADM evolution, Lund fragmentation) is preserved.
- Unit-tracking type machinery (e.g. dimensional analysis via the type system). Out of scope; would be a separate large change set.
- Replacing physical constants with measured-vs-defined CODATA distinctions. Out of scope.
- Generalizing over `ComplexField<R>` where complex values do not already appear. Mechanical retype of existing `Complex<f64>` to `Complex<R>` is in scope; introducing new complex-valued physics is not.
- Performance optimization beyond preserving the `R = f64` baseline.
- Type aliases like `Mass64`, defaults like `<R = f64>`, or any other bridge mechanism.

## Decisions

### Decision 1: Single `R: RealField` parameter per wrapper struct

Every wrapper struct carries one type parameter `R: RealField`. The 47 wrappers split into two shapes:

**Simple newtypes** (~28 of 47):
```rust
// Before: pub struct Mass(f64);
// After:  pub struct Mass<R: RealField>(R);
```

**Multi-field structs** (5 of 47 — `FourMomentum`, `WeakIsospin`, `LightconeEndpoint`, `LundParameters`, `ParticleData`):
```rust
// Before: pub struct FourMomentum { e: f64, px: f64, py: f64, pz: f64 }
// After:  pub struct FourMomentum<R: RealField> { e: R, px: R, py: R, pz: R }
```

**Wrappers around generalized cross-crate types** (7 of 47):
```rust
// Before: pub struct PhysicalVector(pub CausalMultiVector<f64>);
// After:  pub struct PhysicalVector<R: RealField>(pub CausalMultiVector<R>);
```

**`JonesVector` carries `Complex<R>` end-to-end** (1 of 47):
```rust
// Before: pub struct JonesVector(CausalTensor<Complex<f64>>);
// After:  pub struct JonesVector<R: RealField>(CausalTensor<Complex<R>>);
```

**Why single `R`, not separate `R_position`, `R_momentum`, etc.:** every multi-field struct's components are dimensionally consistent (a `FourMomentum`'s four components live in the same field; mixed precision would corrupt the Lorentz invariant). Single `R` enforces this by construction.

### Decision 2: Physical constants stay as `pub const X: f64` (carve-out)

The 76 physical-constant declarations under `deep_causality_physics/src/constants/{universal,atomic,electromagnetic,thermodynamics,particle,electro_weak,earth}.rs` and the PDG quark-mass constants in `nuclear/pdg.rs` SHALL remain unchanged as `pub const X: f64 = literal`. They are explicitly carved out of the "zero hardcoded `f64`" invariant in the spec.

**Why no parameterization:** physical constants are *values*, not computations. Generalizing them adds zero precision benefit:

- **Exact-defined CODATA constants** (post-2019 SI: `c`, `h`, `k_B`, `e`, `N_A`, `Δν_Cs`) are integers or short decimals that fit in `f64` exactly. `f128::from_f64(SPEED_OF_LIGHT)` is bit-identical to the true `f128` value because `299_792_458.0` has no precision loss in `f64` to begin with.
- **Measured constants** (`G`, `α`, electron mass, etc.) have measurement uncertainty far worse than `f64` precision. `G` is known to ~5 sig figs; `f64` over-represents it by ~10 orders of magnitude. `f128` adds no real precision because the *measurement* is the precision floor.
- **The only place precision parametricity matters** is the *calculation* that consumes the constant. Calculations are at `R`; reading `R::from_f64(SPEED_OF_LIGHT)` lifts the constant into `R` losslessly for the exact-defined case and with insignificant rounding for the measured case.

**Consumer pattern:** at `R` precision, read a constant via `R::from_f64(SPEED_OF_LIGHT)` (using R0's `RealField::from_f64`). This is a one-token cost at the call site; the resulting `R`-typed value flows into the calculation normally.

**Why not convert to `pub fn` form anyway for ergonomic consistency:** the breaking-change cost (76 constants × every downstream call site in workspace binaries, examples, tests) is real; the precision benefit is zero. The cost-benefit math is negative. If the ergonomic improvement is wanted later, it ships as a separate trivial change set (`add-physics-constants-fns`) without changing any of the precision math.

**Alternatives considered:**
- Convert every `pub const X: f64` to `pub fn x<R: RealField>() -> R { R::from_f64(X) }`. Rejected: wide breaking-change ripple for zero precision gain.
- Pre-bake one constant per precision: `SPEED_OF_LIGHT_F64`, `SPEED_OF_LIGHT_F32`. Rejected: combinatorial explosion as precisions are added; doesn't compose with generic call sites.
- Add `pub fn x<R: RealField>() -> R` alongside the existing `pub const X: f64`. Rejected: bridge code, violates rip-and-replace policy.

The four invariant greps from R0 task 11 (`f64`, `From<f64>`, `Into<f64>`, `Mul<f64`) are run inside `deep_causality_physics/src/` post-change with the carve-out in mind: hits inside `constants/` and the PDG masses in `nuclear/pdg.rs` are expected and permitted; hits elsewhere are not.

### Decision 3: Method bodies use `R::from_f64` for ad-hoc literals, `RealField` methods for derivable values

Inside the rewritten wrapper methods, RK4 solvers, and ODE step routines:

- Algebraic constants like `0.5`, `2.0`, `6.0` (RK4 coefficients) — recommended pattern is `R::from_f64(0.5)` for clarity. `R::one() / (R::one() + R::one())` for `0.5` is more `RealField`-native but reads worse.
- Mathematical constants — `R::pi()`, `R::e()`, `R::epsilon()` from the trait.
- Physical constants — read the `pub const X: f64` declaration and convert at the call site: `R::from_f64(SPEED_OF_LIGHT)`, `R::from_f64(PLANCK_CONSTANT)`, etc. The constants themselves are untouched (Decision 2 carve-out).
- Comparisons against zero — `R::epsilon()` for the tolerance, not `R::from_f64(0.0)` for the value (use `R::zero()` instead, from the `Zero` supertrait on `RealField`).

**Why not require maximally `RealField`-native expressions everywhere?** Readability. `R::one() / (R::one() + R::one())` for `0.5` is a four-token expression replacing a one-token literal. The `R::from_f64(0.5)` form is one token shorter and clearer, with identical inlined performance.

### Decision 4: Generic ODE / RK4 / Kalman-filter solvers use `R: RealField` directly, no `+ From<f64>`

The existing pre-generalization attempts in `relativity/gravity.rs` and `chronometric/solve_gm.rs` bound `T: Field + Float + From<f64>` and `R: RealField + From<f64>`. Both halves of the `From<f64>` part are removed: `RealField::from_f64` from R0 supersedes the trait bound.

```rust
// Before:
fn rk4_step<T>(...) where T: Field + Float + From<f64> { ... }

// After:
fn rk4_step<R>(...) where R: RealField { ... }
```

The body's `<T as From<f64>>::from(0.5)` sites become `R::from_f64(0.5)`. The semantic is identical; the trait surface is tighter.

### Decision 5: Lund fragmentation random-sampling parameters are `R: RealField`

`select_quark_flavor<Rng>(rng, strange_suppression: f64)` etc. take their random-sampling control parameters as `R`. The RNG itself remains `f64`-producing (same boundary exception as topology's Metropolis step); the result is converted via `R::from_f64(rng.random::<f64>())`.

**Tagged exception:** the one `let rnd: f64 = rng.random();` line per Lund routine is marked `// PERMITTED-F64: RNG boundary; see design.md Decision 5`, matching the topology pattern.

### Decision 6: `Complex<f64>` storage in photonics retypes to `Complex<R>`

Three photonics types use `Complex<f64>` directly or indirectly: `JonesVector(CausalTensor<Complex<f64>>)`, `ComplexBeamParameter(Complex<f64>)`, and several method signatures in beam transport. All retype to `Complex<R>` against `deep_causality_num::Complex` (which is already generic over its base ring).

**No `ComplexField<R>` trait bound is needed** — `Complex<R>` already implements `ComplexField<R>` for any `R: RealField` per `deep_causality_num`. The retype is a mechanical parameter swap.

### Decision 7: Cross-crate consumption flows through; `::<f64>` pins from R0 removed

The seven wrapper types storing topology / tensor / multivector types (`PhysicalVector`, `SpacetimeVector`, `PhysicalField`, `AbcdMatrix`, `JonesVector`, `StokesVector`, `ComplexBeamParameter`) propagate `R` through the storage. Every `CausalMultiVector::<f64>`, `CausalTensor::<f64>`, `SimplicialManifold::<f64>` call site tagged `// TEMP: removed by generalize-physics-over-realfield` is now genuinely propagated, and the temporary pin is removed.

**This is the deciding piece of evidence that R0 must ship first.** If R0 has not landed, `CausalMultiVector<R>` doesn't compile and these wrappers are stuck on `Complex<f64>`-style hardcoding. R0 is a strict precondition.

### Decision 8: Test layout mirrors R0 — duplicate algorithmic tests at `R = f32`

Same policy as R0:
- Existing `f64` tests retype with explicit `::<f64>`. No behavior change.
- Algorithmically-meaningful tests (mass / energy / momentum conservation; RK4 convergence; Lund fragmentation distribution moments; Maxwell evolution invariants) duplicate at `R = f32` with widened tolerances.
- `_f32` suffix on duplicate test names. Same file as the `f64` original. Macro-generated where logic is identical.

### Decision 9: No bridge code, no parallel APIs, no deprecation paths

Identical policy to R0:
- No type aliases (`Mass64`, `FourMomentum64`, etc.) — rejected.
- No default type parameters (`<R = f64>`) — rejected.
- No `#[deprecated]` `f64`-returning methods alongside generic replacements — rejected.
- No `From<f64>` impls on generalized wrapper types — rejected.

Downstream library `deep_causality_effects` temporarily pins its physics consumption to `::<f64>` until its own change set lands, tagged `// TEMP: removed by generalize-effects-over-realfield`. This is the single permitted exception, mirroring R0's permitted exception for physics.

## Risks / Trade-offs

- **[Note] Physical constants stay as `pub const X: f64`.** No breaking change on the constants surface; consumers at `R` precision add a one-token `R::from_f64(SPEED_OF_LIGHT)` at the call site. See Decision 2 for the rationale.

- **[Risk] Multi-field structs need `Clone + Copy` on `R` to support method-chaining patterns like `FourMomentum::new(...).boost_z(beta)`.** `RealField` requires `Copy`, so this is satisfied; but custom precisions (e.g. arbitrary-precision rationals) may not implement `Copy`. The audit needs to confirm no struct method requires `Copy` beyond what `RealField` already requires.
  → **Mitigation:** audit during implementation. If a method body relies on `R: Copy` beyond the trait's existing `Copy` superbound, refactor to use `&R` or `.clone()` explicitly.

- **[Risk] The `Default` impl on wrapper structs.** Several wrappers currently derive `Default` (returning the `f64` default `0.0`). With `R` generic, `Default` would require `R: Default`, which is implied by `RealField`'s `Zero` superbound (`R::zero()` is the natural default).
  → **Mitigation:** retype `#[derive(Default)]` to manual `impl<R: RealField> Default for Mass<R> { fn default() -> Self { Self(R::zero()) } }` where the wrapper validates non-negativity (zero passes), or remove `Default` for wrappers where zero is not a valid value (e.g. `HalfLife` validates positive — `zero()` would fail). Per-wrapper decision; audit during implementation.

- **[Risk] PDG database `ParticleData` is currently a static array of values.** If `ParticleData<R> { mass: R, ... }` becomes generic, the static array can't be `static` anymore — it has to be a function or a per-precision initializer.
  → **Mitigation:** the standard fix is `pub fn pdg_database<R: RealField>() -> Vec<ParticleData<R>> { ... }` constructed on demand, or a lazy-init pattern via `std::sync::LazyLock`. Per the AGENTS.md "no external crates if avoidable" rule, the on-demand function is the recommended choice. Performance impact: one-time `Vec` allocation per call; negligible for the typical PDG-lookup pattern.

- **[Risk] Cross-crate dependencies in `deep_causality_effects`.** Effects consumes physics types via `EffectData::Custom(Arc<dyn Any>)` (the audit found no first-class `EffectData` variants for physics types). The `Arc<dyn Any>` boundary erases generics, so consumers downcast to a concrete type — `Arc<dyn Any>` downcast to `Mass<f64>` works only if every producer constructs `Mass<f64>`. Generalizing physics breaks this if effects has any `Mass`-shaped consumer.
  → **Mitigation:** audit `deep_causality_effects` for any consumer that downcasts to a physics wrapper. The expected count is zero (the effects audit found physics types appear only behind `Arc<dyn Any>`, and no downcast sites named physics types). If a downcast exists, it gets a temporary `::<f64>` pin tagged for cleanup in the effects change set.

- **[Trade-off] This change set is large.** ~147 `f64` locations, ~14 hours of focused work, seven domain phases. Reviewer fatigue is real.
  → **Justification:** the change is uniform across all 47 wrappers (same audit-and-rewrite pattern). Splitting into per-domain change sets would require seven separate proposals / designs / specs / tasks for what is fundamentally one consistent refactor. The seven-phase task structure inside this change set gives reviewers natural stopping points.

- **[Trade-off] `pub fn` constants are a syntactic break from the standard physical-constants idiom.** Most Rust crates expose physical constants as `pub const`. Our crate diverges.
  → **Justification:** the entire point is precision parametricity. The alternative (hardcoded `f64`) is exactly the problem we're solving. The cost is one syntactic deviation in exchange for the precision parameter — worth it.

## Migration Plan

1. **Precondition:** R0 (`generalize-topology-over-realfield`) has shipped. Verify by running the four invariant greps from R0's task 11 in `deep_causality_topology/src/`. Verify `RealField::from_f64`, `from_f32`, `from_i64`, `from_i32` are all on the `deep_causality_num` trait.
2. **Phase 1 (Infrastructure, ~0.5 h):** verify R0's preconditions; create a small test fixture for the `f32` duplicate test pattern.
3. **Phase 2 (Mechanics & Materials, ~2 h):** retype 12 wrappers in `dynamics/quantities.rs` and `materials/quantities.rs`; retype the Kalman filter in `dynamics/estimation.rs`; remove R0 pins.
4. **Phase 3 (Electromagnetism, ~1.5 h):** retype 3 wrappers in `em/quantities.rs`; retype the Maxwell solver. Constants in `constants/electromagnetic.rs` stay as `pub const X: f64` (Decision 2).
5. **Phase 4 (Thermodynamics, ~1 h):** retype 2 wrappers in `thermodynamics/thermodynamics_quantities.rs`. Constants stay.
6. **Phase 5 (Relativity & Chronometry, ~2.5 h):** retype 2 wrappers in `relativity/quantities.rs`; retype the RK4 solver in `relativity/gravity.rs`; retype the ODE step in `chronometric/solve_gm.rs`; retype the ADM state in `theories/general_relativity/adm_state.rs`. Constants in `constants/universal.rs` stay.
7. **Phase 6 (Nuclear & Particle Physics, ~2 h):** retype 7 wrappers in `nuclear/quantities.rs`; retype `FourMomentum` and its methods; retype Lund fragmentation in `nuclear/lund/`; retype the PDG database. PDG quark-mass constants in `nuclear/pdg.rs` and constants in `constants/particle.rs`, `constants/electro_weak.rs` stay.
8. **Phase 7 (Photonics, ~2 h):** retype 11 wrappers in `photonics/quantities.rs`; retype `JonesVector`, `StokesVector`, `AbcdMatrix`, `ComplexBeamParameter` with `Complex<R>` storage. Constants in `constants/atomic.rs` stay.
9. **Phase 8 (Effects temporary pin, ~0.5 h):** `cargo build --workspace` reveals any `deep_causality_effects` consumption sites that need `::<f64>` pins; tag each `// TEMP: removed by generalize-effects-over-realfield`.
10. **Phase 9 (Verification, ~0.5 h):** `cargo test -p deep_causality_physics`; benchmark vs. baseline; run the four invariant greps from R0 task 11 inside `deep_causality_physics/src/` (with the `constants/` and `nuclear/pdg.rs` carve-out applied). `Phase 8 (Constants cleanup)` from the prior plan is removed entirely — constants are left alone.
11. **Rollback:** revert. Behavior at `R = f64` is bit-identical pre and post; rollback restores the old surface.

## Open Questions

1. **Should `ParticleData<R>` be a `Vec`-constructed-on-demand database or a `LazyLock<HashMap<i32, ParticleData<R>>>` per-precision cache?** Recommendation: `Vec` constructed on demand. The PDG lookup pattern is typically once per simulation initialization; the allocation cost is negligible. A `LazyLock` would require keying by precision (separate cache per `R`), which is awkward.
2. **Are there any `pub const X: f64` declarations *outside* `constants/` and `nuclear/pdg.rs`?** Grep `pub const \w+: f64` across `deep_causality_physics/src/` at implementation time. Any hits outside the carved-out locations should either be moved into `constants/` (if they're genuinely physical constants) or generalized appropriately (if they're tolerance values or other algorithm-internal constants). Audit during implementation.
3. **Should `Mass::new(value)` validation (`value >= 0`) use `R::epsilon()` or strict equality with `R::zero()`?** Recommendation: strict equality. `Mass::new(R::zero())` is valid; `Mass::new(-R::epsilon())` is not. Match the pre-change `f64` baseline behavior.
