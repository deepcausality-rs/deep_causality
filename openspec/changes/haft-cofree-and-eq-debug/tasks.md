## 0. Context

- [ ] 0.1 Read `AGENTS.md`, `docs/writing_guides/AiStyleguide.md`, `deep_causality_haft/{README,LEAN_HAFT}.md`, `src/monad/{free_monad,comonad,mod}.rs`, `src/functor/functor_base.rs`, `src/hkt/mod.rs`, and `lean/DeepCausalityFormal/Haft/{FreeMonad,Comonad}.lean`. Confirm `Free`/`FreeWitness` are `alloc`-gated and exported, and that `FreeWitness` implements `HKT`+`Pure` but not `Functor`/`Monad`.

## 1. `EqFunctor` / `DebugFunctor` capability traits + built-in impls (H2)

- [ ] 1.1 Add `src/functor/eq_functor.rs` (`EqFunctor: HKT<Constraint = NoConstraint>`, `eq_type<T: PartialEq>`) and `src/functor/debug_functor.rs` (`DebugFunctor: HKT<Constraint = NoConstraint>`, `fmt_type<T: Debug>`); register in `src/functor/mod.rs`; export from `lib.rs`. No-std clean (`core::fmt`).
- [ ] 1.2 Implement `EqFunctor`/`DebugFunctor` for the built-in single-hole `Functor` witnesses that can carry `Free`/`Cofree`: `OptionWitness`, `VecWitness`, `BoxWitness`, and the other single-hole witnesses (`LinkedList`/`VecDeque`), in their existing witness/extension modules. Bodies are one-liners (`a == b`, `write!(f, "{:?}", fa)`).
- [ ] 1.3 Rust tests: `eq_type`/`fmt_type` agree with the underlying container `==`/`{:?}` for each built-in witness. Register in `tests/BUILD.bazel`.

## 2. `PartialEq`/`Eq`/`Debug` for `Free` (H2)

- [ ] 2.1 Add `impl<F: EqFunctor, A: PartialEq> PartialEq for Free<F, A>`, `impl<F: EqFunctor, A: Eq> Eq for Free<F, A>`, `impl<F: DebugFunctor, A: Debug> Debug for Free<F, A>` — recursion through `F::eq_type`/`F::fmt_type`, no `F::Type<..>: Trait` bound. Place in a sibling module of `free_monad.rs` (e.g. `src/monad/free_instances.rs`) so `free_monad.rs`'s existing body is untouched.
- [ ] 2.2 Update the `free_monad.rs` NOTE: keep the correct "`#[derive]` overflows on the generic type" statement (cite `E0275`), add a pointer to the opt-in `EqFunctor`/`DebugFunctor` route.
- [ ] 2.3 Rust property tests: `PartialEq` reflexive/symmetric/transitive on sample `Free` trees (over `OptionWitness` and a multi-hole `VecWitness`); `Debug` round-trip; a compile check that a witness *without* the capability yields no `PartialEq` (opt-in confirmed). Register in `tests/BUILD.bazel`.

## 3. `Cofree<F, A>` — the cofree comonad (H1)

- [ ] 3.1 Add `src/monad/cofree_comonad.rs` (`#[cfg(feature = "alloc")]`, registered in `src/monad/mod.rs` like `free_monad`): `struct Cofree<F, A> { head, tail }` (private fields) with `new`/`head`/`tail`/`into_parts`; `CofreeWitness<F>` implementing `HKT` (`Type<T> = Cofree<F, T>`, `Constraint = NoConstraint`). No `Functor`/`CoMonad` trait impl (D3/D4). Export `Cofree`/`CofreeWitness` from `lib.rs`.
- [ ] 3.2 Inherent comonad surface: `extract(&self) -> A where A: Clone`; `map<B, Fun: Fn(A)->B + Clone>`; `extend<B, K: Fn(&Cofree<F, A>)->B>(self, k: &K)`; `unfold<X, C: Fn(X)->(A, F::Type<X>)>(seed, coalg: &C)`. Document `unfold`'s finiteness precondition (functor must admit an empty shape). No `dyn`, no `unsafe`.
- [ ] 3.3 Rust law-tests: comonad left identity (`extend extract = id`), right identity (`extract ∘ extend f = f`), associativity, over a concrete functor (`VecWitness` and/or a small Env functor); `unfold` builds the expected finite tree and `extract`/`map` behave. Register in `tests/BUILD.bazel`.

## 4. `Eq`/`Debug` for `Cofree` (H1 × H2)

- [ ] 4.1 Add the `PartialEq`/`Eq`/`Debug` impls for `Cofree<F, A>` (structurally identical to task 2.1: `head == o.head && F::eq_type(tail, ..)`), in `cofree_comonad.rs` or a sibling.
- [ ] 4.2 Rust property tests mirroring 2.3 for `Cofree`. Register in `tests/BUILD.bazel`.

## 5. Lean — `Cofree` comonad laws (H1)

- [ ] 5.1 Add `lean/DeepCausalityFormal/Haft/Cofree.lean` proving `haft.cofree.comonad_laws` (left/right identity, associativity — reuse `Comonad.lean`'s statements) and `haft.cofree.unfold` (the anamorphism computation rule), over a representative functor per `FreeMonad.lean` (positivity: variable-functor `Cofree` is rejected). Self-contained, bare-`lean`; textbook citations (Uustalu–Vene 2008; Ghani–Uustalu–Vene tree/cofree comonads) + deviation notes.
- [ ] 5.2 Register `Cofree.lean` in `DeepCausalityFormal.lean`; add THEOREM_MAP rows `haft.cofree.comonad_laws` / `haft.cofree.unfold` with Rust witnesses in `tests/formalization_lean/cofree_tests.rs`. Confirm `haft` is already in the `formalization.yml` witness-crate allowlist (no allowlist change expected).

## 6. Verify & hand off

- [ ] 6.1 `bazel test //deep_causality_haft/...` green; feature matrix builds (`--no-default-features`, `+alloc`, `--all-features`); `make format && make fix` clean (fix clippy, do not suppress); bare-`lean` on `Haft/Cofree.lean` exit 0; `lake build` green; traceability passes. Confirm `unsafe_code = "forbid"` + no-`dyn` + macro-free `/src` intact in the new modules.
- [ ] 6.2 Confirm additivity: no existing type/trait/requirement changed; `Free`'s existing surface and `fold`-based comparison unaffected; new instances opt-in per witness. Prepare a commit message per task group (H2 traits+Free instances; Cofree; Lean) — do not commit (await user).
