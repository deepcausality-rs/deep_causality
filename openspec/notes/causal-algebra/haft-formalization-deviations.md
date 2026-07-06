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
canonical carriers (all proved in Lean, all witnessed in Rust). All deviations found by the
audit are now **resolved**: documentation gaps fixed, the unlawful reference implementation
made lawful (D7), and the three structural proposals executed (P-1 rename + `fuse` removal,
P-2 `Default`-bound removal, P-3 curvature laws at the concrete implementations). The
proposal section at the end is retained as the record of what was decided and why.

## Verified correct as documented (no deviation)

| Structure | Reference | Status |
|---|---|---|
| `Functor` (fmap laws) | Mac Lane §I.3 | laws stated & hold |
| `Monad` (3 Kleisli laws; `join = bind id`) | Moggi 1991 | laws stated & hold |
| `CoMonad` (coKleisli laws) | Uustalu–Vene 2008 | laws stated & hold |
| `Bifunctor` (id + composition) | Mac Lane §II.3 | laws stated & hold |
| `Profunctor` (dimap laws) | Loregian §5 | laws now stated (see D-P) & hold |
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

## Deviations and their resolution status

### D1 — `Applicative`: Composition law missing from the docs — **RESOLVED (docs)**
The accepted definition (McBride–Paterson 2008) has four laws; the docstring listed three.
The Composition law and functor-compatibility (`fmap f x = pure f <*> x`) are now in
`src/applicative/mod.rs`, both proved in `Haft/Applicative.lean` and witnessed in
`applicative_tests.rs`.

### D2 — `Monad: Functor + Pure` instead of `Monad: Applicative` — **RESOLVED (docs)**
Deliberate hierarchy (strict constrained witnesses cannot satisfy `Applicative::apply`'s
closure constraint); mathematically harmless but it makes coherence an obligation. The
coherence law `apply f_ab f_a = bind f_ab (fun f => fmap f f_a)` is now law 4 in
`src/monad/mod.rs`, proved in `Haft/Monad.lean`, witnessed in `monad_tests.rs`. The hierarchy
itself stays — it is a justified Rust deviation.

### D3 — `Promonad` is not a promonad — **RESOLVED (P-1 executed)**
A categorical promonad is a monad in `Prof` (Loregian §5.2; Jacobs–Heunen–Hasuo, JFP 2009).
The trait's `merge` restricted to the diagonal is `liftA2` — a lax monoidal functor's
structure map. **Executed:** the trait is renamed `MonoidalMerge`
(`src/monad/monoidal_merge.rs`, git-mv'd), all workspace users updated (haft extensions,
examples, tests; topology `GaugeFieldWitness`; physics docs), and `fuse` is **removed** —
every workspace implementation was degenerate (two `panic!`s, one discard-inputs-and-return-
empty). Lean file renamed to `Haft/MonoidalMerge.lean`
(id `haft.monoidal_merge.merge_naturality`); the trait docstring carries the naming history.

### D4 — `Pure`: naturality claimed, never stated — **RESOLVED (docs)**
The naturality square `fmap f ∘ pure = pure ∘ f` is now a stated law in `src/pure/mod.rs`,
proved in `Haft/Pure.lean`, witnessed in `pure_tests.rs`.

### D5 — `Traversable`: vacuous Identity law — **RESOLVED (docs)**
The docstring's `t.sequence == t.map(id).sequence` was vacuously true. Replaced in
`src/traversable/mod.rs` by the accepted Identity-applicative form (Jaskelioff–Rypacek 2012),
proved in `Haft/Traversable.lean`, witnessed with a real Identity applicative. The
Composition law remains deferred (needs lawful-applicative hypotheses; tracked in
`THEOREM_MAP.md`).

### D6 — `MonadEffect3/4/5`: `U: Default` bound foreign to the monad — **RESOLVED (P-2 executed)**
The mathematical bind has no `Default` constraint; the bound existed to let product-encoded
carriers manufacture a `U` in the error branch. After the D7 fix nothing used it.
**Executed:** the bound is removed from all three trait declarations
(`src/effect_system/monad_effect.rs`) and the one impl that repeated it, restoring symmetry
with the unbound variants (which never had it). Semver: breaking for external implementors →
ships with the P-1 breaking release.

### D7 — Reference implementation ran the continuation under error — **RESOLVED (code)**
`src/utils_tests.rs`'s carriers held `value: T` in product position, so the error branches of
all six `apply`/`bind` impls (arity 3/4/5) had to fabricate a value by running the
continuation — keeping its value, dropping its error and warnings. Fixed by making the value
slot `Option<T>`: error branches now return `value: None` with the continuation **not run**
(raise is a left zero — `Haft/EffectSystem.lean :: bind3_raise_left_zero`), `fmap` maps under
`Option`, and absence (`None` value, no error) propagates. Dependent tests and the two
examples (`deep_causality_haft/examples/effect_system.rs`,
`examples/mathematics_examples/tensor/effect_system_causal_tensor.rs`) were updated; the
haft example now *demonstrates* short-circuit (its old comment apologizing for the missing
short-circuit is gone — the failed pipeline shows the tax step never runs). Left-zero is
pinned by continuation-must-not-run assertions in `tests/utils_tests.rs`,
`tests/effect_system/monad_effect_tests.rs`, and `tests/formalization_lean/effect_system_tests.rs`.

### D8 — `Foldable`: law referencing nonexistent operations — **RESOLVED (docs)**
The `foldr`/`flip`/`reverse` law (none of which exist in the crate) is dropped from
`src/foldable/mod.rs`; the real fold–pure law remains, proved and witnessed.

### D9 — `FnMut` admits stateful closures — **RESOLVED (docs)**
Every law-bearing trait docstring (`Functor`, `Applicative`, `Monad`, `CoMonad`, `Bifunctor`,
`Profunctor`, `Foldable`, `Traversable`, `Promonad`) now states that laws are guaranteed only
for pure closures; purity cannot be enforced by Rust's type system.

### D-P — `Profunctor` had no stated laws — **RESOLVED (docs)**
(Fell out of the initial report's list; recorded here for completeness.) The two profunctor
laws, with the contravariant twist spelled out, are now in `src/functor/profunctor.rs`.

### D10 — `RiemannMap` / `CyberneticLoop`: signatures, not structures — **RESOLVED (P-3 executed)**
`src/riemann_map/mod.rs` no longer claims to *be* a multilinear map — it is documented as a
typed interface whose laws belong to the concrete implementations.
`src/cybernetic_loop/mod.rs` records its one provable fact (`control_step` = Kleisli
composite, proved in `Haft/Signatures.lean`). **Executed:** the curvature laws are now stated
where the types carry algebra — `lean/DeepCausalityFormal/Topology/RiemannCurvature.lean`
proves antisymmetry, the first Bianchi identity, and linearity for the canonical
constant-curvature operator `R(u,v)w = g(v,w)·u − g(u,w)·v` (do Carmo Ch. 4), with each
hypothesis (symmetry/bilinearity of `g`) carrying its law; the Rust witness
(`deep_causality_topology/tests/types/curvature_tensor/curvature_tensor_law_tests.rs`)
instantiates the same form on the concrete `CurvatureTensor` and checks the same statements
through `contract` and `check_bianchi_identity` (including that the detector detects
violations). This opens the Topology layer of `THEOREM_MAP.md`
(ids `topology.curvature.*`). Physics-side (`gr_ops`) Bianchi checks on computed Schwarzschild
curvature remain natural follow-ups within that layer.

---

## Proposal for the remaining structural deviations

Ordered by recommendation. P-1 and P-2 are one breaking release of `deep_causality_haft`
(the workspace releases in lockstep via release_plz, so bundling them is natural).

### P-1 — Rename `Promonad` → `MonoidalMerge`; remove `fuse`
**What:** `git mv src/monad/promonad.rs src/monad/monoidal_merge.rs`; rename the trait to
`MonoidalMerge` (name says what it is: the lax-monoidal merge / `liftA2`); **remove `fuse`**.
**Why remove `fuse` rather than constrain it:** its free `C` requires manufacturing a
`P⟨A,B,C⟩` for every `C` from an `A` and a `B` — only phantom carriers can comply. The
crate's own **shipped** impl (`Tuple3Witness::fuse`, `src/extensions/hkt_tuple_ext.rs`)
`panic!`s with "C cannot be derived from A and B alone", and the only call site in the
workspace is the `#[should_panic]` test pinning that panic (verified by grep). Dead,
unlawful API.
**Blast radius (all in-workspace, verified by grep):** `deep_causality_haft` — `lib.rs`
export, `src/extensions/hkt_tuple_ext.rs` impl, `examples/unbound_haft.rs`, 3 test files;
`deep_causality_physics/src/theories/mod.rs` (mention/use); `deep_causality_topology` tests.
No external crates.io consumer is known, but the rename is semver-breaking → next breaking
release. Optionally keep `#[deprecated] pub use ... as Promonad;` for one release.
**Effort:** ~1 hour, mechanical. **Lean side:** `Haft/Promonad.lean` renames to
`MonoidalMerge.lean`; the binaturality theorem and its witness carry over unchanged.

### P-2 — Drop `U: Default` from `MonadEffect3/4/5::bind`
**What:** remove the bound from the three trait declarations in
`src/effect_system/monad_effect.rs` and from the (workspace-only) impls that repeat it.
**Why now:** after D7, no implementation uses `U::default()` — the bound is pure friction and
misrepresents the algebra (a monadic bind constrains nothing about `U`). The unbound variants
(`monad_effect_unbound.rs`) never had the bound; this restores symmetry.
**Blast radius:** haft-internal only (verified: no `MonadEffect` users outside the crate in
the workspace). Semver: breaking for external *implementors* (impl signatures must match the
trait), non-breaking for callers → bundle with P-1's release.
**Effort:** ~15 minutes.

### P-3 — State the tensor laws at the concrete `RiemannMap` implementations — **EXECUTED** (see D10)
**What:** where the types actually carry algebra — `deep_causality_topology`
(`CurvatureTensor`, `hkt_curvature.rs`) and `deep_causality_physics` (`gr_ops*.rs`) — state
and test antisymmetry `R(u,v)w = -R(v,u)w`, linearity in each slot, and the first Bianchi
identity `R(u,v)w + R(v,w)u + R(w,u)v = 0` (do Carmo Ch. 4).
**Outcome:** done. The laws are proved in `lean/DeepCausalityFormal/Topology/RiemannCurvature.lean`
for the canonical constant-curvature operator `R(u,v)w = g(v,w)·u − g(u,w)·v` (antisymmetry, first
Bianchi identity, linearity in `w`), each with its `g`-symmetry/bilinearity hypothesis carrying its
law; the Rust witness `deep_causality_topology/tests/types/curvature_tensor/curvature_tensor_law_tests.rs`
checks the same statements on the concrete `CurvatureTensor` via `contract` and
`check_bianchi_identity`. This opened the Topology layer of `THEOREM_MAP.md` (ids
`topology.curvature.*`). Physics-side (`gr_ops`) Bianchi checks on computed Schwarzschild curvature
remain a natural follow-up within that layer.

### Explicitly not proposed
- **D2's hierarchy** (`Monad: Functor + Pure`) stays — justified by the constraint system,
  now with its coherence obligation documented, proved, and witnessed.
- **The witness pattern, `Satisfies`, `Context`-parameterized `Adjunction`** — transparent
  Rust encodings; the Lean bridge (`Haft/Hkt.lean`) is their permanent documentation.
