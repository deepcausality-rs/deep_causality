## 0. Prerequisite

- [x] 0.1 `num-generic-monoid-tower` has landed (the generic `Monoid` that `fold_map` folds into). Confirm before H1. — `deep_causality_algebra::Monoid` (`empty`/`combine`) is present and a path dependency of `deep_causality_haft`.

## 1. Foldable::fold_map (H1)

- [x] 1.1 Add `fn fold_map<A, M: Monoid>(fa, f) -> M` to `Foldable` (default via `fold` + `Monoid::combine`); export. — provided default on the exported `Foldable` trait.
- [x] 1.2 Rust law-tests: `fold_map(pure a, f) = f a`; monoid-homomorphism coherence (respects `empty`/`combine`); order-independence when `M: CommutativeMonoid`. — `tests/formalization_lean/foldable_tests.rs` (`test_fold_map_pure`, `test_fold_map_monoid_coherence` incl. the `Count` commutative order-independence check); Bazel dep on `//deep_causality_algebra` added.
- [x] 1.3 Lean: `DeepCausalityFormal/Haft/Foldable.lean` (extend) proving `haft.foldable.fold_map_pure`, `haft.foldable.fold_map_monoid_coherence`; THEOREM_MAP rows; witnesses; bare-`lean`. — extended with `foldMap` over the free monoid `List`, textbook citation + deviation notes in the header; `lake build` green; THEOREM_MAP rows added; traceability passes.

## 2. Category + Kleisli (H2)

- [x] 2.1 `src/category/`: `Category` trait (`id`, `compose`); `Kleisli<M: Monad>` newtype (`compose = bind`) implementing it; make `Arrow` satisfy `Category`. Export. — witness-based encoding (design choice, user-approved): `trait Category { type Hom<B>; fn id; fn compose }` (RPITIT closures); `Kleisli<M>` (`Hom=M::Type`, `id=pure`, `compose=bind`, scoped to `Constraint = NoConstraint` monads) + `Fun` (the function category the value-level `Arrow` runs in — `Arrow`'s `run` IS the `Fun` morphism). Exported from `lib.rs`.
- [x] 2.2 Rust law-tests: category left/right identity + associativity for `Kleisli<M>` (and `Arrow`). — `formalization_lean/{category_tests,kleisli_tests}.rs` (`test_fun_category_laws`, `test_kleisli_category_laws` incl. `compose = bind` / `id = pure`).
- [x] 2.3 Lean: `Haft/Category.lean` + `Haft/Kleisli.lean` proving `haft.category.laws`, `haft.kleisli.category_laws` (citing `core.causal_arrow.category_laws`'s shape / `haft.arrow.category_laws`); THEOREM_MAP rows; witnesses; bare-`lean`. Retire the informal Kleisli language in `io/mod.rs`. — both Lean files typecheck (textbook citations: Mac Lane §I.1 / §VI.5, Moggi 1991; deviation notes); THEOREM_MAP rows added; the three `io/mod.rs` "Kleisli" mentions now link the named `Kleisli` type.

## 3. Reified free Arrow — ArrowTerm (H3)

- [x] 3.1 `src/arrow/arrow_term.rs`: a typed builder API (well-typed `In`/`Out` by construction) lowering into an erased core enum reifying `id/lift/compose/split/fanout/first/second` (no `dyn`, no `unsafe`). Export. — `ArrowTerm<In, Out, G>` (phantom `fn(In)->Out` façade) lowers to the erased `ArrowCore<G>` (heap recursion via `alloc::boxed::Box`, gated on `feature = "alloc"` like `Free`); `ArrowVal<V>` is the uniform interpretation universe. Exported from `lib.rs`.
- [x] 3.2 Rust tests: the builder rejects mistyped wiring at compile time (trybuild or type-level tests); the erased core round-trips a representative term; interpretation (`run`) agrees with the eager combinators. — `compile_fail` doctest on `ArrowTerm` (no `trybuild` dependency); `formalization_lean/arrow_term_tests.rs` checks structural round-trip (`into_core` = expected `ArrowCore`) and that `ArrowCore::interpret` matches the eager `Lift`/`Compose`/`Split`/… pipeline of identical shape.
- [x] 3.3 Lean: `Haft/ArrowTerm.lean` proving `haft.arrow_term.interpret_sound` (interpreting a term equals composing its combinators) and the free-object statement (interpretation determined by the generators); THEOREM_MAP rows; witnesses; bare-`lean`. — `interpret_sound` (homomorphism equations, definitional) + `free` (structural induction); textbook citations (Hughes 2000; Awodey §5; Goguen et al. 1977 initial-algebra semantics; Reynolds 1972) + 3 deviation notes; `lake build` green; THEOREM_MAP rows added; traceability passes.
- [x] 3.4 Reword the "type system rejects every nonsensical graph" claim to "well-typed by construction at build time, executed from an erased core" (assumption #3 Q3). — reworded in `openspec/notes/causal-algebra/algebraic-causaloid.md` Part 3.8.

## 4. One-way interpreter (H4)

- [x] 4.1 `src/natural_transformation/` and/or `src/arrow/interpreter.rs`: `NaturalTransformation<F, G>` (naturality) and/or `ArrowInterpreter<A, M: Monad>` mapping `ArrowTerm` → `Kleisli<M>`. Export. — both: `ArrowCore::interpret_kleisli<M, V, Phi>` (`src/arrow/interpreter.rs`) is the ArrowTerm→Kleisli<M> functor (`Id ↦ pure`, `Compose ↦ bind`, strength threads `M` over pairs); `NaturalTransformation<F, G>` trait + `OptionToVec` component (`src/natural_transformation/mod.rs`). Both exported (alloc-gated where they touch the erased core / `Vec`).
- [x] 4.2 Rust law-tests: functoriality — `preserves id`, `preserves compose`; naturality (commutes with `fmap`). — `formalization_lean/interpreter_tests.rs`: `interpret_kleisli(Id) = pure = Kleisli::id`; `interpret_kleisli(Compose f g) = Kleisli::compose(interpret f, interpret g)` (over a partial `Option` generator, so `bind` short-circuit is on-path); `OptionToVec::transform ∘ fmap = fmap ∘ transform`.
- [x] 4.3 Lean: `Haft/Interpreter.lean` proving `haft.interpreter.{preserves_id, preserves_compose, naturality}` (citing `haft.arrow.*`); THEOREM_MAP rows; witnesses; bare-`lean`. — interpreter modelled as the unique fold into an `ArrowAlg` target (Kleisli = `aid:=pure`, `acompose:=bind`); `preserves_id`/`preserves_compose` definitional, `naturality` by cases on `optionToList`; textbook citations (Hughes 2000; Awodey §1.4/§5; Mac Lane §VI.5/§I.4; Moggi 1991) + deviations; `lake build` green; THEOREM_MAP rows added; traceability passes.

## 5. Symmetric-monoidal PROP with Δ / ∇ (H5)

- [ ] 5.1 `src/monoidal/`: a symmetric-monoidal structure over the effect monad with copy comonoid (`Δ`, discard `ε`), merge (`∇`), and symmetry (swap) generators. Built on `CoMonad`/`MonoidalMerge` where they help; no `dyn`/`unsafe`. Export.
- [ ] 5.2 Rust law-tests: comonoid laws (coassociativity, counit), merge monoid laws (associativity, unit), symmetry/naturality, and copy–merge coherence (bialgebra/Frobenius as scoped).
- [ ] 5.3 Lean: `Haft/SymmetricMonoidal.lean` proving `haft.monoidal.{comonoid_laws, merge_monoid_laws, symmetry}` (transcribed self-contained, no heavy Mathlib); THEOREM_MAP rows; witnesses; bare-`lean`.
- [ ] 5.4 Record that this is the substrate the deferred reconvergence-merge (∇) extension consumes (assumption #2 Q2); the graph wiring is NOT in scope here.

## 6. Verify & hand off

- [ ] 6.1 `bazel test //deep_causality_haft/...` green; `make format && make fix` clean (fix clippy, do not suppress); bare-`lean` on every new `Haft/*.lean` exit 0; `unsafe_code = "forbid"` + no-`dyn` + macro-free `/src` intact.
- [ ] 6.2 Note the unblock: `formalize-main-crate` (causaloid = free-Arrow → Kleisli; `evaluate` as catamorphism) and the deferred ∇-merge extension can now proceed. Prepare a commit message per task group; do not commit (await user).
