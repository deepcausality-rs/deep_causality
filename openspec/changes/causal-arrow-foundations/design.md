## Context

This is stage 1 of the Causal Arrow program. The two source notes are authoritative:

- `openspec/notes/arrow/causal-arrow-generalization.md` — the strategic claim (discovery operators form an Arrow; the causal monad is its Kleisli fragment) and the de-risked build order (§6).
- `openspec/notes/arrow/num-dual-endomorphism.md` — the two primitives this change lands: `Endomorphism` (discrete dynamics: iterate / fixpoint) and `Dual<T>` (continuous dynamics: differentiate / tangent).

Current state, verified against the source tree:

- `deep_causality_haft` already has `Functor`, `Applicative`, `Monad`, `CoMonad`, `Bifunctor`, `Profunctor`, `Promonad`, `CyberneticLoop`, `Adjunction`, plus the witness/HKT machinery (`HKT`, `HKT2Unbound … HKT6Unbound`, `Satisfies`, `Placeholder`). It has **no** `Morphism` (typed-arrow base) and **no** iteration/fixpoint primitive. The crate convention: one trait per file under `src/traits/`, parameterized by a witness `P: HKTnUnbound`, methods as associated functions over `P::Type<…>`, registered in `traits/mod.rs`, re-exported from `lib.rs`.
- `deep_causality_num` has the full algebra tower (`Field`, `RealField`, `Ring`, `CommutativeRing`, `Module`, `Algebra`, `Zero`, `One`, monoids) and the hypercomplex types `Complex`, `Quaternion`, `Octonion`, each a folder module (`mod.rs` + per-trait files `arithmetic.rs`, `ops.rs`, `ops_shared.rs`, `algebra.rs`, `identity.rs`, `cast.rs`, `display.rs`). It has **no** dual numbers.

Constraints (from `AGENTS.md`): `unsafe_code = "forbid"`; static dispatch only, no `dyn`/trait objects; no external numeric crates; no macros in `src/`; one type per module; tests mirror the `src/` tree with a `_tests` suffix and register in the Bazel `BUILD.bazel`; 100% coverage on new code; the two writing guides bind all prose.

## Goals / Non-Goals

**Goals:**
- Add `Morphism` (typed-arrow / `Category` base) and `Endomorphism` (type-preserving fragment + iteration/fixpoint combinators) to `deep_causality_haft`, consistent with the existing witness-based traits.
- Add `Dual<T>` (ring of dual numbers, forward-mode AD primitive) to `deep_causality_num`, consistent with the `Complex` module layout and the algebra tower.
- Ship the iteration/fixpoint combinators in a form that is **immediately usable on real `T → T` transforms** under static dispatch, so stage 2 can retrofit the BRCD Meek loop with no further primitive work.
- Stay purely additive: no existing public API, behavior, or caller changes.

**Non-Goals (deferred to later stages — see Roadmap):**
- Retrofitting the BRCD Meek loop / context propagation / monad time step onto `iterate_to_fixpoint` (stage `causal-arrow-fixpoint`).
- Reverse-mode AD, and consuming `Dual<T>` in `deep_causality_topology` / `deep_causality_physics` (stage `causal-arrow-autodiff`).
- Strength / the monoidal product on arrows (`first`/`***`), recasting SURD/BRCD as arrows, the `PropagatingEffect` carrier, and the CDL unification (stages `causal-arrow-strength`, `causal-arrow-cdl-unification`).
- Any change to SURD, BRCD, or the causal monad numerics.

## Decisions

### D1 — `Morphism<P: HKT2Unbound>` as the typed-arrow base (identity + application)

`Morphism` is the witness-based interface a discovery operator instances: a family of arrows `P::Type<A, B>` with an identity arrow and the ability to apply an arrow to an input.

```rust
pub trait Morphism<P: HKT2Unbound> {
    fn identity<A>() -> P::Type<A, A>
        where A: Satisfies<P::Constraint>;
    fn apply<A, B>(arrow: &P::Type<A, B>, input: A) -> B
        where A: Satisfies<P::Constraint>, B: Satisfies<P::Constraint>;
}
```

Concrete witness shipped for the contract and law tests: a function-pointer carrier `FnMorphism` with `Type<A, B> = fn(A) -> B`. `identity` is a real `fn(A) -> A`; `apply` calls the pointer. Fully static, zero-capture, no `dyn`.

**Why `apply` + `identity`, and not `compose` in the trait.** General arrow composition `P::Type<A,B> ⊕ P::Type<B,C> → P::Type<A,C>` over *capturing* closures has no single concrete carrier under the no-`dyn` rule (closures are unique unnameable types; `Box<dyn Fn>` is a forbidden trait object). Composition is therefore **not** a trait method in stage 1. It is available structurally where the carrier is a free/defunctionalized category, which is exactly the strength/monoidal-product work in a later stage (`causal-arrow-strength`). Putting a composition method here that only the fn-pointer witness can implement (and only for non-capturing `fn`s, which cannot be composed into a new `fn` either) would be a contract the design cannot honor. Identity + application is the honest, implementable base, and it is enough to host the iteration combinators (D2) and to type discovery operators in later stages.

*Alternative considered:* a free-category enum AST carrier (`Id`, `Comp`, `Lift(fn)`) that makes `compose` total. Rejected for stage 1 as premature: it is only needed once operators are recast as arrows, and it belongs with the strength work where its shape is driven by real multi-input operators (BRCD's `Data ⊗ Data ⊗ Graph`).

### D2 — `Endomorphism<P>: Morphism<P>` — marker + iteration/fixpoint combinators

`Endomorphism` is the type-preserving fragment: the diagonal `T → T`. As a marker it is trivial (blanket impl over any `Morphism` witness, per the note). Its value is the home it gives the iteration combinators, expressed by repeatedly applying the arrow via `Self::apply`:

```rust
pub trait Endomorphism<P: HKT2Unbound>: Morphism<P> {
    fn iterate_n<T>(arrow: &P::Type<T, T>, x: T, n: usize) -> T
        where T: Satisfies<P::Constraint>;

    /// Apply until a fixpoint (`apply(arrow, x) == x`) or `max_steps` is reached.
    /// Returns the final value and whether a fixpoint was actually reached.
    fn iterate_to_fixpoint<T>(arrow: &P::Type<T, T>, x: T, max_steps: usize) -> (T, bool)
        where T: Satisfies<P::Constraint> + Clone + PartialEq;

    /// Apply until `predicate(&x)` holds or `max_steps` is reached.
    fn iterate_until<T, Pred>(arrow: &P::Type<T, T>, x: T, predicate: Pred, max_steps: usize) -> (T, bool)
        where T: Satisfies<P::Constraint>, Pred: FnMut(&T) -> bool;
}
```

**Bounded, not unbounded.** Every combinator takes an explicit `max_steps` and reports whether convergence was actually reached (the `bool`), rather than looping forever. This is the safe shape for the Meek-rule fixpoint and effect propagation: a rule set that fails to converge returns "not converged" instead of hanging. The note's `iterate_to_fixpoint(f, x, eq)` is realized with the step bound made explicit.

**Fixpoint by successive-iterate equality.** `iterate_to_fixpoint` compares `x` against `apply(arrow, x.clone())`; reaching equality means a true fixpoint. This needs `T: Clone + PartialEq`, which `MixedGraph` (the stage-2 consumer) satisfies. `iterate_until` avoids both bounds when the caller supplies its own convergence predicate.

**The capturing-transform path.** The fn-pointer witness covers any rule expressible as a non-capturing `fn(T) -> T` — and the BRCD Meek pass qualifies, since it reads and writes only the graph. Rules that genuinely must capture configuration get a dedicated carrier when stage 2 needs one; that carrier choice is deferred so it is driven by the real call site, not guessed here. The algebra (`End(T)` is a monoid under composition) is documented on the trait but its `compose` is the strength-stage concern (D1).

### D3 — `Dual<T>` — ring of dual numbers / forward-mode AD in `deep_causality_num`

**Depends on `num-real-trait`** (the `Real` trait + `RealField: Real + Field` refactor).

```rust
pub struct Dual<T: Real> { re: T, du: T }   // a + b·ε,  ε² = 0
pub type Dual32 = Dual<f32>;
pub type Dual64 = Dual<f64>;
```

The bound sits **on the struct**, mirroring `Complex<T: RealField>`'s style (verified in `complex_number/mod.rs`); the parameter stays named `T` and is bounded by `Real` (`Real` is the bound, not the parameter name). The bound is **`Real`, not `RealField`** (see "Bound choice" below): a dual's component needs the analytic operations (`exp`/`ln`/`sin`/…) but never a field inverse, so `Real` is the honest, minimal requirement. `f32`/`f64` satisfy `Real` (via `RealField: Real + Field`).

Folder module `src/dual/dual_number/` mirroring `src/complex/complex_number/`: `mod.rs` (type + constructors + accessors), `arithmetic.rs`, `ops.rs`, `ops_shared.rs`, `algebra.rs`, `identity.rs`, `real.rs` (the `impl Real for Dual`), `cast.rs`, `display.rs`.

- **Constructors / accessors:** `Dual::new(re, du)`; `Dual::constant(re)` (`du = 0`); `Dual::variable(re)` (`du = 1`, the AD seed); `value()` → `re`; `deriv()` → `du`.
- **Arithmetic (the AD rules fall out of the ops):**
  - `(a+bε) + (c+dε) = (a+c) + (b+d)ε`
  - `(a+bε) − (c+dε) = (a−c) + (b−d)ε`
  - `(a+bε)(c+dε) = ac + (ad+bc)ε` (product/chain rule; `ε² = 0`)
  - `−(a+bε) = −a + (−b)ε`
  - `(a+bε)/(c+dε) = a/c + ((bc−ad)/c²)ε` for invertible `c` (quotient rule)
- **Algebra tower:** the three property markers `Associative`, `Commutative`, `Distributive` (from `algebra_properties.rs`) — `Dual` satisfies **all three**, since `T[ε]/(ε²)` is a quotient of the commutative ring `T[x]`, so its multiplication is both associative and commutative and distributivity is inherited (same marker row as `Complex`) — plus `Zero`, `One`, `AddMonoid`, `MulMonoid`, `Ring`, `AssociativeRing`, `CommutativeRing`, `Module<T>`. **Not `Field`:** unlike `Complex`, the three markers do *not* lift `Dual` to a field, because `ε` is a zero divisor (`ε·ε = 0`), so a general multiplicative inverse does not exist; this is documented, and `Div` is defined only for invertible real part.
- **`Dual<T>` implements `Real` (`impl Real for Dual<T> where T: Real`).** This is the payoff of the `num-real-trait` prerequisite: a dual *is* an analytic real scalar (commutative ring + every elementary function), but **not** a field — exactly what `Real` expresses and `RealField` cannot. Consequences: a dual is droppable into any `Real`-generic numeric (drop-in forward-mode AD, the autodiff stage's lever), and duals nest — `Dual<Dual<f64>>: Real` gives second derivatives. Because `impl Real for Dual` must satisfy the whole `Real` surface, `Dual` implements the **complete** `Real` analytic surface (constants, `sqrt`/`exp`/`ln`/`log*`/`powf`, `sin`/`cos`/`tan`/`asin`/`acos`/`atan`/`atan2`, `sinh`/`cosh`/`tanh`, `abs`/`floor`/`ceil`/`round`/`clamp`, NaN/finiteness), each carrying its closed-form derivative in the `ε` channel (`exp: (e^a, e^a·b)`, `sin: (sin a, cos a·b)`, `sqrt: (√a, b/(2√a))`, …). The AD-correctness property (D-tests) exercises a representative subset; the rest follow the same chain-rule pattern.

**Bound choice (`Real`, via the `num-real-trait` prerequisite).** The component bound is `T: Real`, the honest minimal requirement: a dual's component needs the analytic operations (`exp`/`ln`/`sqrt`/`sin`/`cos`/…) but never a field inverse. Before `num-real-trait`, the only carrier of those operations was `RealField`, which fused the analytic surface with field invertibility — so `Dual<T: RealField>` would have been *accidentally correct* (it admits `f32`/`f64`) but conceptually wrong (it demands a field the dual construction never uses), and `Dual` could not have honestly implemented its own scalar trait (it is not a field). `num-real-trait` splits `Real` (analytic) out of `RealField` (`= Real + Field`), so now `Dual<T: Real>` states exactly what it needs and `impl Real for Dual` is honest. `f32`/`f64` satisfy `Real` via `RealField: Real + Field`, so the admissible set is unchanged (real scalars), but for the right reason.

*Non-real bases (Complex/Quaternion/Octonion) remain excluded* — none implements `Real` (Complex is a `Field`/`ComplexField` with no elementary functions; Quaternion/Octonion are non-commutative). Holomorphic AD over `Dual<Complex>` would require both loosening the core to a commutative-ring bound and adding elementary functions to `Complex`; it has no consumer and stays out of scope. (Distinct aside: *complex-step differentiation* — `Im(f(x + i·h))/h` — is a finite-`h` numerical trick exploiting `i² = −1`, not this exact `ε² = 0` method; not conflated.)

**Why a new top-level `dual/` module rather than folding into `complex/`.** Dual numbers are an unrelated quotient ring (`T[ε]/(ε²)`), not a hypercomplex extension. Mirroring `complex/`'s layout while keeping it a sibling matches "one type, one module" and the note's framing of `Dual` as its own algebraic object.

*Alternative considered:* expose forward-mode AD as a free function `differentiate(f, x)` instead of a type. Rejected: the note is explicit that the win is *type-based* AD — differentiation falls out of the trait impls, composes through the same functorial machinery as the rest of `num`, and `Dual<T: Real>` **is** the algebraic tangent vector that the topology crate will consume directly (and, being a `Real`, drops straight into the existing `Real`-generic numerics). A free function would not give the topology crate a tangent type.

### D4 — Purely additive, no caller touched

Nothing in `haft` or `num` changes signature or behavior. The new traits and type are added and re-exported; existing modules are untouched except for the `mod`/`pub use` registration lines. This keeps the stage reviewable and risk-free, and defers every behavioral change to the stage that has a concrete consumer.

## Risks / Trade-offs

- **[Over-claiming a `Category`.]** Calling `Morphism` a "typed-arrow base" while omitting `compose` risks implying more than ships. → Mitigated by D1: the trait is scoped to identity + application, the composition gap is documented as the explicit entry point for `causal-arrow-strength`, and the prose does not claim a total category in stage 1.
- **[Thin abstraction.]** The note is candid that `Endomorphism` is a thin win (consolidation + a compile-time iterability fact + lawful folding). → Accepted: the value is realized in stage 2 when the BRCD Meek loop, context propagation, and the monad time step collapse onto one tested combinator. Stage 1 must not inflate the claim.
- **[fn-pointer witness can't capture.]** Some real transforms capture state. → Mitigated: the immediate stage-2 consumer (Meek) does not capture (graph-in, graph-out); a capturing carrier is added when a real call site needs it, not speculatively.
- **[`Dual` precision / forward-mode only.]** Forward-mode is efficient for few-inputs/many-outputs (sensitivities, tangent vectors), not scalar-loss-over-many-parameters learning, which wants reverse-mode. → Documented; reverse-mode is a separate, larger build in `causal-arrow-autodiff`. Stage 1 ships only the exact forward-mode primitive.
- **[`Div` partiality.]** `Dual` division is undefined at zero real part. → Encoded in the type's contract and tested; callers that need totality use the additive/multiplicative ring operations.

## Migration Plan

Additive only; no migration. New traits/type are introduced behind their own modules and re-exports. Rollback is deletion of the new files and their registration lines. The Bazel `BUILD.bazel` test targets for the new test files are added alongside.

**Prerequisite ordering.** The `dual-numbers` capability depends on the **`num-real-trait`** change (`Real` + `RealField: Real + Field`); it must land first so `Dual<T: Real>` and `impl Real for Dual` are expressible. The `morphism-algebra` capability (in `haft`) has no prerequisite and can land independently.

## Roadmap — the later stages of the Causal Arrow program

These are **not** part of this change. Each becomes its own proposal when its prerequisite lands, per the build order in `causal-arrow-generalization.md` §6. Recorded here so the foundations are understood in context.

| stage (change name) | scope | consumes | key risk / note |
|---|---|---|---|
| **`causal-arrow-fixpoint`** | Retrofit the BRCD Meek/orientation loop, hypergraph context/effect propagation, and the causal-monad `State` time step onto `iterate_to_fixpoint`/`iterate_until`. Behavior-preserving refactor. | `Endomorphism` (D2) | MODIFIED requirement against `brcd-algorithm`; must be golden-test-identical to the current hand-rolled loops. Pick the capturing-rule carrier here, driven by the real Meek call site. |
| **`causal-arrow-autodiff`** | **Re-point** selected `RealField`-generic numerics in `deep_causality_topology` (tangent vectors / directional derivatives / pushforwards) and `deep_causality_physics` (`F = −∇U` / Euler–Lagrange / geodesics) to the weaker `Real` bound, so `Dual<T>` (already a `Real`) flows through them unchanged — **drop-in forward-mode AD**. Add **reverse-mode AD** (a tape/graph) as the larger second build for many-parameter gradients. | `Dual<T>: Real` (D3), `Real` (`num-real-trait`) | The `Real` split makes the re-point a one-line bound change per function, not a rewrite. Forward-mode is few-inputs/many-outputs only; reverse-mode is a distinct, larger construction. Topology differential geometry is the cleanest first consumer. |
| **`causal-arrow-strength`** | Strength / the monoidal product on arrows (`first` / `***`), leaning on the existing `Bifunctor` + `Profunctor`. The free/defunctionalized category carrier that makes `Morphism::compose` total. | `Morphism` (D1), `Bifunctor`, `Profunctor` | The technical fulcrum for multi-input operators (BRCD's `Data ⊗ Data ⊗ Graph`). Shape is driven by real operators, so it follows the recast work. |
| **`causal-arrow-cdl-unification`** | `PropagatingEffect` as the shared object (note §10); `Endomorphism<PropagatingEffect>` as the interior monoid; migrate the CDL inner carrier from `CausalTensor` to `PropagatingEffect`; the discover → generate-model → infer three-stage arrow (note §11) with the new pure `generate-model` functor. Recast SURD/BRCD as arrows (the witnessing cells). | all of the above | The everything-bagel risk. The load-bearing invariant: **data flows as `PropagatingEffect`; static structure (graph, SURD lattice, manifold metric) stays a stage parameter, never payload** — this is what keeps discovery non-Kleisli and the subsumption claim falsifiable. |

## Open Questions

- **Capturing-rule carrier (stage 2).** Does the BRCD Meek pass stay expressible as a non-capturing `fn(MixedGraph) -> MixedGraph`, letting `FnMorphism` cover it, or does it need a dedicated carrier? Resolve at the stage-2 call site, not here.
- **`Dual` elementary-function surface.** `impl Real for Dual` requires the **complete** `Real` analytic surface, so `Dual` implements all of it (each method's `ε`-channel derivative is mechanical). The AD-correctness tests exercise a representative subset; no surface is deferred (the prior "ship 6 functions" plan is superseded by the `Real` decision).
- **Complex / holomorphic AD.** Out of scope and unchanged by the `Real` split: `Complex` does not implement `Real` (no elementary functions), and there is no consumer. Revisit only if a holomorphic-AD need appears, alongside `causal-arrow-autodiff`.
- **`compose` carrier shape.** Free-category enum vs. a profunctor-strength encoding — decided in `causal-arrow-strength` against real multi-input operators.
