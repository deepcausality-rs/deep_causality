# Note: the Arrow Assumption — the premise the generalization never examined

Status: **strategic / adversarial.** A companion to `causal-arrow-generalization.md`,
written against it. Its job is not to defend the Arrow thesis but to find the load-bearing
premise the thesis takes for granted — the assumption every other argument stands on and
none inspects — to ground that hunt in the actual archived record (the June 1–4 changes), the
foundational source (`deep_causality_haft`, `deep_causality_num`, the effect core
`deep_causality_core`), and the EPP monograph (`papers/effect_propagation_process/epp.pdf`,
Ch 3–4 — the author vouches for the philosophy and the EPP definition; the later formalism and
implementation chapters predate the monad and are stale), and to keep a running register of
unexamined premises as they surface. "The hardest assumption is the one it never examined" —
and, this note's central correction, **the examined one that rides past its warrant** (§1).

Every claim below is checked against source. Where the source confirms or contradicts a
premise, the file:trait or paper §-reference is cited inline.

## 0. Method — what counts as "unexamined"

The generalization note interrogates a great deal: it earns the separation constructively
(§3), resists the everything-bagel (§4.2), scopes the 3-D claim to one cell (§9), splits phase
from payload (§10), and closes with seven open questions (§12). **None of those qualify here.**
An *open question* is examined-but-unresolved — the program knows it is open and has framed a
test. An *unexamined assumption* is a premise the program **reasons from**, never **about**:
invisible exactly because all the careful reasoning happens *inside* it. §12 is the list of
doors the program knows are unlocked. This note is about the floor it stands on.

Test for membership: take any sentence of the form "we will verify X." If the *verification
procedure itself* presupposes Y, then Y is unexamined however rigorously X is checked. The
register is ranked by depth — A0 is the floor; A1–A7 are joists resting on it.

## 1. A0 — the core assumption: **the inference→discovery transfer of categoricity**

This note's first draft named the floor "Categoricity" — that causal discovery's unity, if
any, is categorical, and that the Arrow is therefore the *right* abstraction rather than merely
an expressible one. **The EPP monograph (`papers/effect_propagation_process/epp.pdf`) refutes
that framing, and the refutation is the real finding.** Categoricity is not unexamined. For
causal *inference* it is the single most-examined commitment in the whole program — it is
*derived and proven*.

**What the EPP actually earns (Ch 3.6, 4.1 — the parts the author vouches for; the later
formalism predates the monad and is stale).** The classical definition of causality is
decomposed into three independent conditions — **process, determination, propagation**
(§4.1.2). A pure function `E₂ = f(E₁)` is shown to carry determination and propagation but
*not* process (it carries no state); the structure that carries all three is the monad; and the
result is stated as the single axiom **`m₂ = m₁ >>= f`** — "a causal relation is a monadic
dependency" (§4.1.3–4.1.4). It is then *proven* that classical causality is a parametric
specialization: Proposition 1 (the functional axiom is the monadic axiom under the unit
context, via left identity and `M ≅ I`), Proposition 2 (classical causality is the functional
form under bivalence + biconditional + temporal precedence), the Corollary combining them, and
Appendix A proving the EPP subsumes Pearl's SCM — with the three monad laws written out
explicitly (§4.1.5). Categoricity-for-inference is axiomatized, derived, and proven. It is the
*opposite* of unexamined.

**So the floor is not categoricity. It is the *transfer* of categoricity across a boundary the
EPP never crosses: from inference to discovery.** The axiom is about **effect propagation** —
the forward/inference direction, `m₁ >>= f → m₂`. Its objects are propagating effects; `f` is a
causal function the model already *has*; the causaloid graph (§4.5–4.6) is the static structure
inference runs *over*. Going the other way — from observational *data* to that structure — is
the inverse problem, and the EPP does not axiomatize it. Discovery appears as a **typestate
pipeline** (impl §10.10) and, tellingly, as **Future Work** ("Contextual Causal Discovery,"
§12.1). There is no axiom, no decomposition, no subsumption proof that discovery is monadic,
arrow-shaped, or categorical at all.

The Arrow generalization (`causal-arrow-generalization.md` §0) nonetheless asserts exactly
that: "causal-discovery operators form an Arrow; the causal monad is its Kleisli fragment; the
Arrow strictly subsumes the monad." This takes the *earned* inference monad and posits a
*larger* category that (a) contains discovery operators as arrows and (b) has the inference
monad as its Kleisli sub-part. **By the author's own account this unification is days old and
has no underlying formalism.** Nothing of the kind that earned the inference monad — a
decomposition argument, a specialization proof — has been done for the bridge. The claim rides
across the inference/discovery line on the credibility of the thing it sits next to.

**Why it is the hardest assumption — the Wittgensteinian turn.** The transfer is invisible
*precisely because* the inference half was examined so thoroughly. Once categoricity was
proven for inference it stopped being a question — and a premise that is no longer a question is
exactly what gets carried, unnoticed, across a boundary. The program examined categoricity so
well *for inference* that it ceased to see it as an assumption *at all*, then extended it to
discovery without re-opening it. "The hardest assumption is the one it never examined" resolves
here to its sharper form: **the hardest assumption is the examined one, riding past its
warrant.**

**The inference monad's own derivation is evidence *against* the transfer.** The monad was
earned by the propagation condition — consequence 5, "Inherently Asymmetric in Propagation"
(§3.6.2): `bind` is unidirectional, `m >>= f >>= g` admits no inverse. *That directional
propagation is the defining property.* Discovery has no propagation direction: it is
`{data} → structure`, an inverse inference, not an `m₁ >>= f → m₂` step. The very property that
earned the monad is the property discovery lacks — which is what the generalization note's
"non-Kleisli" instinct half-sees. But the note then assumes the non-Kleisli discovery *still
lives in one category with the monad*. That is the leap: not "is discovery a monad" (the note
correctly says no), but "is discovery a co-fragment of the *same category* whose Kleisli part
is the inference monad" — and that category has never been constructed.

**The test — do for the bridge what §4.1 did for classical→dynamic.** The EPP did not *assert*
that classical causality is a specialization of the monad; it *constructed the reduction and
proved it*. The bridge must clear the same bar:
1. **Construct the category C explicitly** — its objects and arrows — and *prove* the inference
   monad (`CausalMonad`) is its Kleisli fragment. Not assert; construct, as Proposition 1 does.
2. **Exhibit discovery operators (SURD, BRCD, PC/GES) as arrows of that same C**, composing
   with the monad through shared objects.
3. If C cannot be built so the inference monad is its Kleisli part *and* discovery inhabits it,
   then there are **two** categories and "unification" is an analogy, not a theorem — downgrade
   the claim to the composition-discipline / API contribution (the §7 discipline applied to the
   thesis).

A subsidiary check sharpens step 2 — the **discrimination test**: exhibit a deliberately
non-causal estimator (a polynomial fit, `Data -> Polynomial`) as an arrow of the discovery
signature. It satisfies every arrow law SURD does, because the laws
(`haft/tests/algebra/arrow_tests.rs`) constrain *composition*, not *content*. So name the
predicate **P** with {causal-discovery operators} = {arrows satisfying **P**}. If P is
non-categorical (it talks about identifiability or intervention semantics), the causal content
lives in P, not in the Arrow, and "discovery is an Arrow" is the trivial half. **The EPP shows
the program *can* clear this bar — it cleared it for inference.** A0 is the assumption that the
same bar has been cleared for the bridge, when no formalism for the bridge yet exists.

**A0 decomposes into three claims of decreasing certainty — keep them apart (owner's framing).**
The single "discover and infer are one Arrow" headline is really three claims, and conflating
them is what lets the certain one vouch for the uncertain one:
- **A0a — Kleisli ⇒ Arrow. Doable; a theorem to instantiate, not a conjecture.** Hughes (2000,
  *Generalising Monads to Arrows*) gives the explicit construction: the Kleisli arrows of *any*
  monad form an Arrow (`arr f = pure ∘ f`; `f >>> g = λa. f a >>= g`;
  `first f = λ(a,c). f a >>= λb. pure (b,c)`). Lifting the causal monad to an Arrow is this
  construction, contingent only on the monad laws holding (i.e. on **A1**). Low risk.
- **A0b — discovery as a co-fragment of that same Arrow. Open; the real risk.** Whether
  SURD/BRCD inhabit the *same* category whose Kleisli part is the inference monad. No formalism;
  may not hold (see the propagation-direction argument above). This is the conjecture A0 names.
- **A0c — API-convenience composition. A standalone *engineering* claim needing no formalism.**
  Even if A0b cannot be proven, "discovery slots into the pipeline and composes ergonomically"
  stands on its own. The discipline: keep the *engineering* claim (the builder composes; the
  step is useful) strictly apart from the *scientific* claim (discovery is categorically an
  arrow). Discovery's job is to discover something useful; the categorical status of that one
  step is secondary, and a working composition does not need a category proof to earn its keep.
  Ship A0a/A0b/A0c each at its own confidence — A0c always, A0a once A1 is settled, A0b only if
  the bridge is constructed.

## 2. A2 — the guard on the door: **consistency mistaken for fidelity**

A0 stays invisible because of a second, methodological premise that actively *protects* it:

> **A mechanized, type-checking encoding is evidence that the thesis is true.**

It is not. Compilation proves the encoding is **internally consistent**. It says nothing about
whether the encoding is **faithful** — whether it tracks something real about the method or
merely re-types it Procrustean-style. A *wrong* encoding type-checks as cleanly as a right one.
The generalization note's strongest defensive move — "every claim is a checkable signature,
never a metaphor" (§4.2) — is therefore the precise mechanism that hides A0: it offers
*consistency* as if it were *validation*, and the rigor of the artifact makes the substitution
feel like the opposite of hand-waving. Mechanization here is a hall of mirrors: it certifies
that the categorical re-description is self-consistent, never that causal discovery *has* the
structure.

**The source delivers this one on a plate.** The single most-quoted justification — "the laws
are enforced by the Rust type system" (generalization §4.2) — is **literally false in the
source.** `haft/src/monad/mod.rs` documents its laws under a heading that reads, verbatim,
`# Laws (Informal)`; the arrow laws live in `haft/tests/algebra/arrow_tests.rs` as runtime
`#[test]` functions (`test_composition_is_associative`, `test_identity_is_left_and_right_unit`).
**Zero laws are trait bounds or type-level constraints.** Rust enforces *structure* (the HKT
witnesses, the signatures); it does not and cannot enforce the *equations*. So the program's
flagship rigor claim is, at the foundation, a documentation comment plus a test suite — exactly
*consistency*, dressed as *type-level proof*. (This is A4 below.)

**Not a Rust wart and not fixable in code — a category fact about non-dependent type systems.**
The monad/arrow laws are equations *universally quantified over all programs* (associativity:
`∀ m f g. (m >>= f) >>= g ≡ m >>= (λx. f x >>= g)`). Enforcing them at the type level requires
the type system to express and check propositional equality between arbitrary closures — i.e.
**dependent types** (Agda/Coq/Lean/Idris2). Rust has none: no value-indexed types, no identity
type `a ≡ b`, no proof terms, no function extensionality; `const` generics reach only
`ConstParamTy` scalars, never closures or "∀ inputs these agree," and no nightly feature
(`generic_const_exprs`, `adt_const_params`) closes that. The witness/HKT pattern abstracts over
*kinds* (`F<_>`), giving `bind` a signature — it cannot reach *behavior*. **Haskell's `Monad`
is equally lawless** (GHC checks no monad law; they live in docs + QuickCheck), so property-
testing the laws is mainstream best practice, not a shortcut. **The real ceiling is the
assert-and-require marker pattern the repo already uses** — `pub trait Associative {}`
(`num/src/algebra/associative.rs:20`), empty, required downstream via `AssociativeRing: Ring +
Associative`. Implementing `Associative for T` *asserts* a law the compiler does not check and
*propagates the requirement* so a non-asserting type is refused where it is needed:
**assert-and-require, never prove.** The fix is therefore *prose, not code*: §4.2 (and EPP p.12,
"satisfy the corresponding mathematical laws as compile-time assertion") should say the type
system enforces the **signatures/wiring** and **propagates law-claims via markers**, while the
laws are **property-tested** — true, and a *stronger* story than Haskell's bare typeclass (which
cannot even require "this is the lawful instance"). Optionally add `trait LawfulCausalMonad:
CausalMonad {}`, implemented only for property-tested carriers; that is the most Rust can offer.

**Test for A2 itself.** Produce a *deliberately incorrect* arrow encoding of one method — wrong
fragment, wrong object semantics — that still compiles and still passes the law tests. If you
can (you can), type-checking is demonstrably not a fidelity criterion.

**The important qualification — the program *has* a fidelity criterion; it just hasn't applied
it to discovery.** The EPP is the proof: §4.1 does not mechanize the inference monad and call
it done — it *derives* the monad from a decomposition of causality and *proves* classical
causality is a specialization (Propositions 1–2, Appendix A). That derivation-plus-subsumption
*is* a fidelity criterion, and a strong one. So A2 is not "the program cannot tell consistency
from fidelity"; it is sharper and more damning: **the program knows exactly how to establish
fidelity — it did so for inference — and the discovery extension has simply not been held to
that standard.** The gap is one of unfinished work, not of missing capability, which is why it
is fixable (see §1's test) and why leaving it unstated would be the error.

## 3. The surfacing history — what June 1–4 examined, and the pattern in what it didn't

Eleven archived changes (2026-06-01 → 2026-06-04) built the substrate. Reading their `design.md`
decisions as an assumption-trajectory shows a **strong and consistent examination habit — and
its exact boundary.**

**Assumptions the program examined well (with a real test, not a hand-wave):**

| change | assumption surfaced | how it was *examined* |
|---|---|---|
| `real-field-discovery` (06-01) | precision is a free type parameter; `RealField` covers every op | golden tests pin `f64` results bit-identical after generification |
| `mixed-graph` (06-02) | the CPDAG endpoint invariant holds | enforced **structurally** at the `BTreeMap` canonical-pair key, not by scattered checks |
| `brcd-estimator` → `brcd-bootstrap` (06-03) | "the reference Python is ground truth" | **falsified**: the BIC score sign is inverted (learns the empty graph); replaced with the causal-learn form. The model case of examining a premise and finding it wrong. |
| `num-real-trait` (06-04) | `Dual` can be an honest analytic scalar | forced the `Real`/`Field` split so `Real` is **division-free** (`num/src/algebra/real.rs` has no `Div`; `Field` adds it, `field.rs:38`) — because `ε²=0` makes `Dual` a non-field; coherence checked by an `rustc` orphan-rule probe |
| `causal-arrow-foundations` (06-04) | forward-mode AD is exact | property test `f(a+ε) = f(a) + f'(a)·ε` to machine precision |
| `causal-arrow-strength` (06-04) | composition can be a trait method | **falsified by prototype** (no single concrete carrier under no-`dyn`); only value-level combinator structs type-check |
| `causal-arrow-calculus` (06-04) | "precision is a parameter" via `From<f64>` | **falsified**: `f32 ∉ From<f64>`; replaced with the `FromPrimitive` blanket for `Dual` (`num/src/dual/from_primitive.rs:16`). A second precision premise caught and corrected. |
| `causal-arrow-calculus` (06-04) | a concrete `Arrow<f64,f64>` can be differentiated | **falsified by prototype** (the "encoding wall", E0308); only a scalar-generic `DifferentiableArrow::run<S>` lifts over `Dual` |

That is an unusually honest record: four premises *falsified by their own tests* (the reference
sign bug, the `From<f64>` precision bug, the trait-method composition, the concrete-arrow
lift). The program's examination muscle is real.

**Predating the window, the effect core shows the same habit at the monad itself.** The authors
examined "is the textbook `Monad` the right abstraction for a causal effect?" and **found it
insufficient**: the generic `bind`'s continuation `FnMut(A) -> M<B>` cannot thread the Markovian
`State` channel, so it could only freeze state (`core/src/types/causal_effect_propagation_process/hkt.rs:114`).
They deviated to a bespoke `CausalMonad` (`core/src/traits/causal_monad/mod.rs:34`) whose `bind`
continuation takes `(EffectValue<Value>, State, Option<Context>)` and threads state, context, and
log forward. The monad was *confronted*, not assumed — which is exactly why its one remaining gap
(its **laws**, A1) is worth stating precisely rather than waving through.

**The boundary — and it is the whole point.** Every premise the program examined is a
**consistency or numerical-fidelity** question — something the type-checker or a numeric test
can settle: does `f64` stay identical, does the orphan rule hold, does `f(a+ε)` match, does
this signature compile, does the sign give the right graph. **Not one examined premise is about
whether the categorical *reading* is faithful.** And those readings were asserted at exactly
the same moments:

- `causal-arrow-calculus` asserts differentiation **is** the tangent functor and Leibniz **is**
  its naturality. The source verifies the *numerics* — `T(∫f) = ∫(Tf)` holds as a numerical
  law over `Dual`. It does **not** verify that this *is* a functor's naturality rather than a
  true equation that any correct AD-over-quadrature would also satisfy. The categorical word is
  laid over a verified number; only the number was checked.
- `causal-arrow-strength` asserts the fluent chain **is** a string diagram (§8). The combinator
  structs exist and compile; that they *are* the term syntax of a monoidal category is a
  semantic reading, untested.
- `causal-arrow-foundations` asserts `Endomorphism` **is** the monoid of `T->T` maps. The trait
  exists with bounded iteration and convergence flags (`haft/src/endomorphism/mod.rs:28`); the
  monoid laws are, again, `#[test]`, not types.

So the surfacing history *is* A2 playing out across eleven changes: **the program examines
relentlessly wherever the type-checker or a numeric oracle can adjudicate, and goes silent at
precisely the seam where the categorical claim lives** — because nothing in the toolchain can
adjudicate there, and the absence of an adjudicator was read as the absence of a question.

**What has not yet been surfaced at all** (no change raises it, examined or deferred): A0 (the
inference→discovery transfer), A2 (fidelity≠consistency), A5 (is there a "they"), A6 (does
anyone run the composite). These appear nowhere in the eleven `design.md` files — not as
decisions, not as non-goals, not as open questions. The asymmetry is the tell: the EPP proves
categoricity *for inference* at book length, and the discovery side inherits the conclusion
without a line of the proof. That silence is the evidence A0 is floor, not furniture.

## 4. The register — joists resting on the floor

Each entry: the unexamined premise / why invisible / the test that would examine it / source
status where checkable.

- **A1 — the baseline is sound.** "Arrow ⊋ monad" presupposes the causal-monad characterization
  of inference is *correct*, not merely *expressible*. **Source (corrected — the base exists and
  was confronted):** the causal monad *is* realized in `deep_causality_core` — `PropagatingEffect<T>`
  is a type alias over `CausalEffectPropagationProcess`, carried by a dedicated `CausalMonad` trait
  (`pure` + state-threading `bind`, `core/src/traits/causal_monad/mod.rs:34`), built precisely
  because the textbook `Monad` could not thread state (see §3). So the *presence* half of A1 is
  settled, and the abstraction was examined, not assumed. **The unexamined half is lawfulness.**
  `core/tests/types/causal_monad/causal_monad_tests.rs` tests *behavior* — `test_pure`,
  `test_bind`, `test_bind_threads_and_updates_state`, `test_bind_error`, `test_fmap_*` — and
  contains **no left-identity, right-identity, or associativity test**. Because the bind is
  deliberately *non-standard* (its continuation takes value, state, and context, not just a
  value), the laws that must hold are not the textbook three but the **monad-transformer-stack
  laws** of the composition the bind actually is: State (threaded `state`) + Writer (appended
  `EffectLog`) + Except (short-circuit on `error`). Two concrete targets a law test would pin and
  the behavior tests miss: `pure` sets `state: State::default()`, so **left identity**
  (`pure(a).bind(f) ≡ f(a)`) holds only if that default is genuinely the unit of the state thread;
  and **associativity** must survive the log-append order and the error-precedence the hand-rolled
  bind imposes. **Test:** state the laws *for this bind* and add the three law tests beside the
  behavior tests. "Lawful by construction" is an argument the suite does not yet make — and "⊋
  monad" is only as strong as the monad it generalizes being *lawful*, not merely *present*.

- **A3 — "static vs dynamic" is a clean binary.** The separation rests on: structure fixed
  before data flows (Arrow) vs structure unfolding via `bind` (Kleisli). But a *learned* CPDAG
  — BOSS (`brcd-bootstrap`, 06-03), GES — fixes its structure **from the data, then freezes
  it**. The note asserts this is static ("the parameter is *born* at generate-model," §11.1)
  yet never examines that derive-then-freeze may be **a `bind` that happened to halt** — Kleisli
  in a static mask. If the freeze is a manual cut the modeler imposes rather than a natural
  operation, the separation leaks at exactly the structure-learners the program most wants to
  claim. **Test:** express BOSS order-search as an arrow; ask whether the freeze is a natural
  transformation or an ad-hoc truncation.

- **A4 — Rust's types enforce the laws.** **Necessarily false, not merely unfulfilled** (see
  §2 for the full argument). It is not that the program *hasn't* encoded the laws in types; it
  *cannot* — no non-dependently-typed language can (Haskell included), because the laws are
  universally-quantified equations over closures and Rust has no dependent types, identity type,
  or closure-carrying const generics. Source confirms the status quo: laws are `#[test]`
  (`haft/tests/algebra/arrow_tests.rs`) + informal doc comments (`haft/src/monad/mod.rs`
  `# Laws (Informal)`); the ceiling is assert-and-require markers (`pub trait Associative {}`).
  The spine of the not-a-metaphor defense (generalization §4.2) is therefore false as written,
  and so is EPP p.12 ("…compile-time assertion"). **Consequence:** a *prose* fix — claim
  type-enforced **signatures/wiring** + **propagated law-claims** + **property-tested laws** —
  which is honest, defensible to a CT reviewer, and stronger than Haskell. Cheapest item on the
  list; do it before either document reaches a categorically-literate reader.

- **A5 — there is a "they."** That PC, GES, SURD, BRCD, LiNGAM, NOTEARS form a natural kind
  whose unification is *discovery* rather than *curation*. The membership list is the program's;
  the "?" on NOTEARS admits the boundary is chosen. **Test:** exhibit a method uncontroversially
  called causal discovery that *cannot* take the Arrow signature even in principle. If none can,
  suspect the class is defined *by* Arrow-membership — circular. (A0 from the membership side.)

- **A6 — the composite is the payoff.** The value proposition is that the four fragments
  *compose* — discover→infer→govern→act as one arrow. But discovery is offline/batch; inference
  is online. If no one ever *runs* the composite, the composition is **conceptual decoration**.
  **Test:** name one real deployment that runs `discover ∘ infer` as a single executed arrow,
  not two stages bridged by a human and a file.

- **A7 — the carrier choice is free.** §10 makes `PropagatingEffect` the universal object so the
  fragments share a carrier. It exists (`core`, the stateless `CausalEffectPropagationProcess`),
  so this is not about absence — it is about the premise that *which* object you unify on is a
  matter of composability, not of truth. The choice *shapes what the algebra can say*: a
  State+Writer+Except carrier bakes error-handling, logging, and a state thread into *every*
  fragment's type. That may be why discover "must" stay non-Kleisli (A3) — an artifact of a
  carrier that always carries a `bind`-able state, not a fact about discovery. **Test:** redo one
  fragment with a different universal object; if the separation theorem depends on the carrier
  choice, it is a theorem about the encoding, not about causal discovery.

## 5. Triage — which failures end the program vs scope it

- **Inference is not at risk.** The EPP earns causality-as-monadic-composition for the
  inference direction by derivation and proof (§4.1, Appendix A). Nothing on this list touches
  it. The whole register is about the *discovery side* and the *bridge* to it.
- **Guts the unification thesis (shrinks it to the API/mechanization claim):** A0 — the
  inference→discovery transfer — and its guard A2. If the bridge category cannot be constructed
  (§1's test), "discover and infer are one Arrow" is an analogy, and the honest residue is the
  composition-discipline / builder contribution. Survivable, but only if caught *before* the
  headline ships.
- **Cracks the separation theorem specifically:** A1, A3, A7. Each can turn "Arrow ⊋ monad" into
  "Arrow ≈ monad in disguise" for some family. Gating for Paper 2, not Paper 1.
- **Deflates the motivation, not the math:** A5, A6. The unification could be *true* and
  *unmotivated* — correct algebra for a use no one runs.
- **Must-fix regardless:** A4 — a factual misstatement about the artifact, cheap to correct,
  expensive if a reviewer finds it first.

None of these touch Paper 1 (BRCD↔CDL), which carries no categorical content (§7), or the EPP's
inference monad, which is proven. Note also that the foundations are *sound as code* — every
machine the audit checked exists and compiles (`Arrow`, `Morphism`, `Endomorphism`,
`Dual`, the `Real`/`Field` split, the `FromPrimitive` blanket). The assumptions on this list
are not about whether the code works, nor about whether *inference* is monadic (it is, and
proven); they are about whether the *categorical reading is transferable across the
inference→discovery line* — a finding or a decoration.

## 6. How to use this note

Add an entry the moment you catch yourself reasoning **from** something instead of **about** it
— when a step says "since X is an arrow, …" rather than "X is an arrow because …", or when a
change *asserts* a categorical identity ("differentiation **is** the tangent functor") right
after *verifying* a number — or carries a result proven on one side of a boundary (inference)
to the other (discovery) without re-proving it. The register's value is not the entries already
here; it is the discipline of noticing the floor. The strongest version of the Arrow thesis is
not the one with no assumptions — it is the one that has **named its hardest assumption and run
the test that could break it.** A0's bridge-construction test (§1) — build the category C, prove
the inference monad is its Kleisli fragment, exhibit discovery inside it — has never been run.
It is the first to run, and the EPP's §4.1 is the template for how.

Cross-reference: `causal-arrow-generalization.md` (the thesis this note audits); its §12 is the
*examined-but-open* list, disjoint by construction from this *unexamined* one.
