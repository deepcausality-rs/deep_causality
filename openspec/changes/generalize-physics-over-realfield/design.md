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

### Decision 2: Physical constants become free functions, not `pub const`s

`pub const X: f64 = literal;` cannot be parameterized over `R` at the language level (`const` items don't accept type generics in their bodies in stable Rust). The unambiguous fix is to convert every constant declaration:

```rust
// Before:
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

// After:
pub fn speed_of_light<R: RealField>() -> R {
    R::from_f64(299_792_458.0)
}
```

**Migration impact:** every consumer that referenced `SPEED_OF_LIGHT` as a value updates to `speed_of_light()` as a call. At `R = f64`, the LLVM optimizer inlines the `from_f64` call to a `f64` literal load — performance is identical to the `pub const` version.

**Naming convention:** the `pub const SCREAMING_SNAKE` becomes `pub fn snake_case`. The change is visible at every call site (no aliasing, no preserving the old name). This is the hard rip-and-replace policy.

**Why not a `Constants<R>` zero-sized struct with associated functions?** Premature decomposition. A flat namespace of free functions is the minimum sufficient API and matches the existing `pub const` flat namespace.

**Why not a `Constants<R>` trait with associated `const` items?** `const` trait items also can't be generic over `R` (the `const_trait_impl` feature is unstable and orthogonal). The free-function path is the only stable option today.

**Alternatives considered:**
- Keep `pub const X: f64` and add `pub fn x<R: RealField>() -> R { R::from_f64(X) }` alongside. Rejected: bridge code, violates rip-and-replace policy.
- Pre-bake one constant per precision: `SPEED_OF_LIGHT_F64`, `SPEED_OF_LIGHT_F32`. Rejected: combinatorial explosion as precisions are added; doesn't compose with generic call sites.

### Decision 3: Method bodies use `R::from_f64` for ad-hoc literals, `RealField` methods for derivable values

Inside the rewritten wrapper methods, RK4 solvers, and ODE step routines:

- Algebraic constants like `0.5`, `2.0`, `6.0` (RK4 coefficients) — recommended pattern is `R::from_f64(0.5)` for clarity. `R::one() / (R::one() + R::one())` for `0.5` is more `RealField`-native but reads worse.
- Mathematical constants — `R::pi()`, `R::e()`, `R::epsilon()` from the trait.
- Physical constants — call the new function form: `speed_of_light::<R>()`, `planck_constant::<R>()`, etc.
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
- No "use the constant or call the function, your choice" `pub const` survives — rejected.

Downstream library `deep_causality_effects` temporarily pins its physics consumption to `::<f64>` until its own change set lands, tagged `// TEMP: removed by generalize-effects-over-realfield`. This is the single permitted exception, mirroring R0's permitted exception for physics.

## Risks / Trade-offs

- **[Risk] `pub const` → `pub fn` migration is a wide breaking-change ripple.** Every consumer of `SPEED_OF_LIGHT`, `PLANCK_CONSTANT`, `BOLTZMANN_CONSTANT`, etc. (in this crate, in physics examples, in any downstream binary or test) needs the `_CONST` → `_fn_call()` update. The audit finds ~76 constants; downstream call sites are unknown without a workspace-wide grep.
  → **Mitigation:** the change set's task list includes a workspace-wide grep for every constant name post-rename. Each hit is patched mechanically. Compile errors from `cargo build --workspace` enumerate any missed site.

- **[Risk] LLVM optimizer doesn't inline `pub fn x<R: RealField>() -> R { R::from_f64(literal) }` to a constant load.** If the function call survives to the final binary, every physical-constant access is a function call instead of a constant load — measurable performance regression.
  → **Mitigation:** add `#[inline]` to every constant function (the bodies are one line). Verify by `cargo asm` or benchmarking that at `R = f64`, the call inlines to the same instruction sequence as the `pub const` baseline.

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

1. **Precondition:** R0 (`generalize-topology-over-realfield`) has shipped. Verify by running the four invariant greps from R0's task 11 in `deep_causality_topology/src/`. Verify `RealField::from_f64` is on the `deep_causality_num` trait.
2. **Phase 1 (Infrastructure, ~0.5 h):** verify R0's preconditions; create a small test fixture for the `f32` duplicate test pattern.
3. **Phase 2 (Mechanics & Materials, ~2 h):** retype 12 wrappers in `dynamics/quantities.rs` and `materials/quantities.rs`; retype the Kalman filter in `dynamics/estimation.rs`; remove R0 pins.
4. **Phase 3 (Electromagnetism, ~2 h):** retype 3 wrappers in `em/quantities.rs`; retype the 11 EM constants in `constants/electromagnetic.rs`; retype the Maxwell solver.
5. **Phase 4 (Thermodynamics, ~1 h):** retype 2 wrappers in `thermodynamics/thermodynamics_quantities.rs`; retype the 8 thermodynamics constants.
6. **Phase 5 (Relativity & Chronometry, ~3 h):** retype 2 wrappers in `relativity/quantities.rs`; retype the RK4 solver in `relativity/gravity.rs`; retype the ODE step in `chronometric/solve_gm.rs`; retype the ADM state in `theories/general_relativity/adm_state.rs`; retype the 10 universal constants in `constants/universal.rs`.
7. **Phase 6 (Nuclear & Particle Physics, ~2 h):** retype 7 wrappers in `nuclear/quantities.rs`; retype `FourMomentum` and its methods; retype Lund fragmentation in `nuclear/lund/`; retype the PDG database; retype the 5 PDG constants + 7 particle / 9 electroweak constants.
8. **Phase 7 (Photonics, ~2 h):** retype 11 wrappers in `photonics/quantities.rs`; retype `JonesVector`, `StokesVector`, `AbcdMatrix`, `ComplexBeamParameter` with `Complex<R>` storage; retype the 9 atomic constants.
9. **Phase 8 (Constants cleanup, ~0.5 h):** sweep for any remaining `pub const X: f64` in `constants/` not yet caught (recall: 76 total); rename and rewrite.
10. **Phase 9 (Effects temporary pin, ~0.5 h):** `cargo build --workspace` reveals any `deep_causality_effects` consumption sites that need `::<f64>` pins; tag each `// TEMP: removed by generalize-effects-over-realfield`.
11. **Phase 10 (Verification, ~0.5 h):** `cargo test -p deep_causality_physics`; benchmark vs. baseline; run the four invariant greps from R0 task 11 inside `deep_causality_physics/src/`.
12. **Rollback:** revert. Behavior at `R = f64` is bit-identical pre and post; rollback restores the old surface.

## Open Questions

1. **Should `ParticleData<R>` be a `Vec`-constructed-on-demand database or a `LazyLock<HashMap<i32, ParticleData<R>>>` per-precision cache?** Recommendation: `Vec` constructed on demand. The PDG lookup pattern is typically once per simulation initialization; the allocation cost is negligible. A `LazyLock` would require keying by precision (separate cache per `R`), which is awkward.
2. **Does the audit miss any internal `pub const` outside `constants/`?** Grep `pub const \w+: f64` across the entire `deep_causality_physics/src/` at implementation time to confirm. The PDG quark masses in `nuclear/pdg.rs` are an example; expect a handful of similar in-domain constants.
3. **Does `RealField` need `from_f32` alongside `from_f64`?** Recommendation: no. The few `f32`-precision literal sites can use `R::from_f64(value as f64)` with a documented intentional precision-narrowing. Adding `from_f32` doubles the surface for marginal benefit.
4. **Should the constants migration include a centralized `pub trait PhysicsConstants<R: RealField>` with all 76 as methods?** Recommendation: no in this change set. Free functions are simpler and the audit doesn't show a use case for the trait shape (no code needs to be generic over "any source of physics constants"). If demand appears, add the trait later in a small follow-up.
5. **Should `Mass::new(value)` validation (`value >= 0`) use `R::epsilon()` or strict equality with `R::zero()`?** Recommendation: strict equality. `Mass::new(R::zero())` is valid; `Mass::new(-R::epsilon())` is not. Match the pre-change `f64` baseline behavior.
