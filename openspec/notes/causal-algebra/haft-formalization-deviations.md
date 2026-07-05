<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Haft Formalization — Deviations from Accepted Category Theory

Companion to the Lean formalization of `deep_causality_haft`
(`lean/DeepCausalityFormal/Haft/`, bound to Rust witnesses in
`deep_causality_haft/tests/formalization_lean/` via `lean/THEOREM_MAP.md`).

Every categorical mechanism in the crate was checked against a reference text:
Mac Lane, *Categories for the Working Mathematician* 2nd ed. (functor §I.3, natural
transformation §I.4, monoid-as-category §I.1, bifunctor §II.3, coproduct §III.3, adjunction
§IV.1, monad §VI.1); McBride & Paterson, *Applicative programming with effects*, JFP 18(1)
2008; Hughes, *Generalising Monads to Arrows*, SCP 37 2000 (+ Paterson, ICFP 2001); Atkey,
*Parameterised notions of computation*, JFP 19 2009; Uustalu & Vene, *Comonadic Notions of
Computation*, ENTCS 203(5) 2008; Moggi, *Notions of computation and monads*, I&C 93(1) 1991;
Loregian, *(Co)end Calculus*, CUP 2021 (profunctors §5, promonads §5.2).

**Verdict in one line:** the mechanisms are sound — every checkable law holds on the crate's
canonical carriers (all proved in Lean, all witnessed in Rust) — but several *docstrings*
under- or mis-state their laws, one trait is misnamed, and the effect-system reference
implementation violates its own documented semantics.

## Verified correct as documented (no deviation)

| Structure | Reference | Status |
|---|---|---|
| `Functor` (fmap laws) | Mac Lane §I.3 | laws stated & hold |
| `Monad` (3 Kleisli laws; `join = bind id`) | Moggi 1991 | laws stated & hold |
| `CoMonad` (coKleisli laws) | Uustalu–Vene 2008 | laws stated & hold |
| `Bifunctor` (id + composition) | Mac Lane §II.3 | laws stated & hold |
| `ParametricMonad` (indexed laws) | Atkey 2009 | shape & laws correct |
| `Arrow` (category + arr + 5 strength laws + derived `second`/`***`/`&&&`) | Hughes 2000 | **fully conformant** — all 10 laws + 3 derived identities proved |
| `NaturalIso` (round-trip + naturality) | Mac Lane §I.4 | laws stated & hold |
| `Either` (binary coproduct) | Mac Lane §III.3 | universal property holds |
| `IoAction` (monad laws on `run`) | Moggi 1991 | laws stated & hold |
| `Adjunction` (triangles + adjunct bijection) | Mac Lane §IV.1 | laws stated & hold |

Rust-necessitated encodings, judged mathematically transparent (documented in
`Haft/Hkt.lean`): the HKT witness pattern (type-level defunctionalization — Reynolds 1972),
`Satisfies`/`NoConstraint` bounds, `Placeholder`, the `Context` parameter on `Adjunction`
(indexes a family of adjunctions; laws hold per fixed context), `CoMonad`'s
borrow-plus-`Clone` signatures, and `Morphism`'s deliberate lack of `compose` (no-`dyn`
policy; total composition lives in the value-level `Arrow`). No action needed.

## Deviations

### D1 — `Applicative`: the Composition law is missing from the docs
`src/applicative/mod.rs` lists three laws (Identity, Homomorphism, Interchange). The accepted
definition (McBride–Paterson 2008) has **four** — Composition
(`pure (∘) <*> u <*> v <*> w = u <*> (v <*> w)`) is absent, and the functor-compatibility law
(`fmap f x = pure f <*> x`) is unstated. Both hold for `OptionWitness` (proved in
`Haft/Applicative.lean`; witnessed in `applicative_tests.rs`).
**Recommendation:** complete the docstring's law list. No code change needed.

### D2 — `Monad: Functor + Pure` instead of `Monad: Applicative`
Deliberate and documented in the source (strict constrained witnesses cannot satisfy
`Applicative::apply`'s closure constraint). Mathematically harmless — a monad induces its
applicative — but that makes coherence a proof obligation: any witness implementing both must
satisfy `apply f_ab f_a = bind f_ab (fun f => fmap f f_a)`. Proved for Option
(`Haft/Monad.lean :: opt_monad_applicative_coherence`).
**Recommendation:** state the coherence law in the `Monad`/`Applicative` docs as a
requirement on witnesses implementing both.

### D3 — `Promonad` is not a promonad
In the literature a *promonad* is a monad in the bicategory of profunctors, equivalently an
identity-on-objects functor (Loregian §5.2; Jacobs et al., *Categorical semantics for
arrows*, JFP 2009 — where arrows ARE promonads). The trait in `src/monad/promonad.rs` is a
different (perfectly useful) thing: restricted to the diagonal `D A = P⟨A,A,A⟩`, `merge` is
`liftA2` — the structure map of a **lax monoidal functor** (the docstring's own reference to
Day convolution points there). What is lawful about it — binaturality of `merge` — is proved
(`Haft/Promonad.lean`). Separately, `fuse : A → B → P⟨A,B,C⟩` leaves `C` fully free: an
implementor must produce a `P⟨A,B,C⟩` for *every* `C` from an `A` and a `B`, which only
phantom-like carriers can do — the crate's own test carrier `panic!`s on it.
**Recommendation:** rename (e.g. `MonoidalMerge`) or re-document as a lax-monoidal merge, and
either constrain `fuse`'s `C` or remove it.

### D4 — `Pure`: "natural transformation" claimed, naturality never stated
`src/pure/mod.rs` calls `pure` "the natural transformation η: Id → F" but does not state the
square `fmap f ∘ pure = pure ∘ f` as a law. Without it the claim is unearned. Proved for
Option (`Haft/Pure.lean`).
**Recommendation:** add the naturality law to the docstring.

### D5 — `Traversable`: the documented Identity law is vacuous
`src/traversable/mod.rs` states `t.sequence == t.map(id).sequence` — since `map id = id`
(Functor law), both sides are identical and the law constrains nothing. The accepted identity
law runs `sequence` at the **Identity applicative** (`sequence ∘ fmap Id = Id`;
Jaskelioff–Rypacek 2012). The correct law is proved (`Haft/Traversable.lean :: seq_identity`)
and witnessed with a real Identity applicative. Naturality is proved for arbitrary
applicative morphisms; the Composition law (composite applicative `M ∘ N`) is deferred —
tracked in `THEOREM_MAP.md`'s deferred section.
**Recommendation:** replace the docstring's Identity law with the Identity-applicative form.

### D6 — `MonadEffect3/4/5`: the `U: Default` bound is foreign to the monad
The mathematical bind has no `Default` constraint on the result type. The bound exists
because implementing carriers keep `value` and `error` in **product** position, so the error
branch must manufacture a `U` from nothing. The sum-encoded model (`Haft/EffectSystem.lean`,
`Except E T × List Λ`) satisfies all three monad laws with no default anywhere — demonstrated
in Rust by `effect_system_tests.rs`, whose `MonadEffect3` impl compiles against the trait
without ever using `U::default()`. This is the same value/error-product defect as the core
carrier's W-invariant (Formalization.md, precondition P2).
**Recommendation:** when P2 lands in `deep_causality_core`, drop the `Default` bound and
require sum-encoded carriers.

### D7 — The reference `MonadEffect3` implementation violates error short-circuit
`src/utils_tests.rs :: MyCustomEffectType::bind` (and the arity-4 twin), in the error branch,
**executes the continuation anyway** — `value: f(m_a.value).value` — keeping `f`'s value,
discarding `f`'s error and warnings, and contradicting both the trait docs ("If effect
contains an error, the error is propagated") and the algebraic requirement that raise be a
left zero (`bind (raise e) f = raise e`, `f` not invoked). The lawful semantics is proved in
Lean (`bind3_raise_left_zero`) and pinned in Rust by a continuation-must-not-run witness.
**Recommendation:** fix the reference implementation (return the error with `U`'s slot
untouched — which is exactly what forces D6's `Default` in a product carrier, or switch the
carrier to a sum).

### D8 — `Foldable`: docstring law references operations that do not exist
Law 1 in `src/foldable/mod.rs` (`foldr f z t = foldl (flip f) z (reverse t)`) mentions
`foldr` and `reverse`, neither of which the trait or crate defines — a Haskell law quoted out
of context. Law 2 (fold–pure compatibility) is real and proved.
**Recommendation:** drop law 1 or define the missing operations.

### D9 — `FnMut` in law-bearing signatures admits stateful closures
`fmap`/`bind`/`apply`/… accept `FnMut`, i.e. observably stateful functions, for which the
equational laws are not even well-posed (two calls to the "same" `f` may differ). All laws —
in Lean and in the literature — are about pure functions.
**Recommendation:** document the purity precondition on each law-bearing trait ("laws are
guaranteed only for pure closures"); Rust's type system cannot enforce it.

### D10 — `RiemannMap` / `CyberneticLoop`: signatures, not structures
Neither trait carries an equational theory. `RiemannMap`'s docstring invokes "a Multilinear
Map in a Tensor Category", but multilinearity is an equation system over types this trait
does not require to carry any algebra, and the actual curvature symmetries (antisymmetry,
first Bianchi identity — do Carmo, *Riemannian Geometry*, Ch. 4) are unstatable at this
signature. What is provable is proved: `control_step` factors as a Kleisli composite in the
error monad (`Haft/Signatures.lean`).
**Recommendation:** state the tensor laws where the concrete implementations live
(`deep_causality_topology` / `deep_causality_physics`, whose types do carry algebra), or
soften the two docstrings to "typed interface".
