# Note: the Causal Arrow — generalizing the causal monad

Status: **strategic / exploratory.** Not committed to any change. Captured from a
working session that started with "why won't BRCD fit the CDL" and ended at a
falsifiable generalization. The struggle to integrate BRCD was not a detour — it
located the boundary of the monadic fragment, which is the most publishable thing
in the arc.

## 0. The one-sentence claim (must stay falsifiable)

> **Causal-discovery operators form an Arrow (a strong profunctor / Freyd
> category). The causal monad is its Kleisli fragment. The Arrow strictly
> subsumes the monad — capturing K method families (a checklist), including at
> least two — information-decomposition (SURD) and intervention-localization
> (BRCD) — that lie *outside* the monadic fragment.**

This is the same epistemic shape as the causal-monad paper ("5 methods are one
monad"): a **subsumption with a checklist**, true/false per cell. That shape is
what both communities (category theory, computational causality) actually trust.
Everything below serves keeping this claim verifiable and tight.

## 1. Why the monad alone could not absorb BRCD/SURD

A monad/Kleisli arrow is `A -> M B`: its effect structure unfolds **dynamically**
from the data via `bind`. SURD and BRCD each carry an **irreducible static
structural input** that `bind` cannot absorb:

- **SURD**: the redundancy/synergy lattice over variable *subsets* — fixed by the
  variable set before any data flows.
- **BRCD**: the **graph** and the **two-cohort product** `Data ⊗ Data` — fixed
  (supplied or learned) before the per-family fits run.

Static effect structure that is fixed before data flows is the textbook case
**Arrows capture and monads do not** (Hughes' Arrow-vs-Monad distinction =
static vs dynamic). That is *why* forcing BRCD into a unary data-loader pipeline
(the CDL as designed) failed: it was forcing an Arrow-but-not-Kleisli operator
through `bind`.

## 2. Four fragments of one Arrow (the system reach)

The full DeepCausality pipeline is four arrows that share one interface:

| stage | operator type | categorical fragment | status |
|---|---|---|---|
| **discover** | `Data -> Model` (SURD, BRCD, PC/GES/BOSS) | static Arrow (not Kleisli) | **in progress** (this project) |
| **infer** | `Model ⊗ Evidence -> PropagatingEffect` (causal monad) | Kleisli / monad | **implemented** (the existing causal monad) |
| **govern** | `PropagatingEffect -> PropagatingEffect` (effect ethos / teloi) | endomorphism (Kleisli in an error monad) | **not yet an arrow/monad** — future work |
| **act** | `State ⊗ Effect -> State ⊗ Action` (state machine) | Mealy automaton = StateArrow | **not yet an arrow/monad** — future work |

**Implementation reality.** Today the causal monad
is confined to **infer**. **discover** is being brought under the same algebra *now*
(this project — SURD already lands, BRCD next). **govern** (the effect ethos /
teloi) and **act** (the causal state machine) exist as components but are **not yet
expressed as arrows or monads** — that is post-publication work. So §2 is the
*designed* reach of one interface, half realized: two fragments are conjecture with
a credible categorical home, not shipped code. The note's job is to keep that line
honest — see §7, where govern + act are scoped to *discussion only* for exactly this
reason.

Two of the four are the *canonical* examples category theorists reach for to show
Arrows generalize monads: **automata/stream transducers** (the state machine) and
**static-structure circuits** (the discovery operators). They are not bent to fit
a buzzword — they are the reason the unification is real. But "canonical example"
is an argument that they *should* instantiate, not evidence that they *do yet*.

**Composition mechanics (the joint to get right).** Discovery does **not** emit a
`PropagatingEffect` that gets `bind`-ed into the monad — that is the trap. The
fragments compose **side-by-side in the Arrow**, agreeing on *objects*:

```
Data --discover (static Arrow)--> Model
        Model ⊗ Evidence --infer (Kleisli = causal monad)--> PropagatingEffect
                          PropagatingEffect --govern (endo)--> PropagatingEffect
                                            PropagatingEffect ⊗ State --act (Mealy)--> Action ⊗ State
```

The monad's output object (`PropagatingEffect`) is what the next arrow consumes;
discovery's output object (`Model`) is inference's parameter. The **Arrow is the
shared interface**; the monad is one fragment. "Wire the CDL and the causal monad
together, granted they agree on the interface" is exactly this — and the Arrow
*is* that interface.

## 3. What is verifiable vs. what must be earned

**Verifiable now (the checklist).** For each method, exhibit it as an arrow of the
claimed signature and mark its fragment: PC, GES, the causal-monad's existing 5,
SURD, BRCD, (LiNGAM, NOTEARS?). A populated table is the contribution.

**The separation must be earned constructively (the upside, not a given).** Strong
monads *can* take products via tensorial strength, so the naive "monads can't do
multi-input" is **false**. The real, defensible separation is the *static-structure*
one: the lattice / the graph is a fixed parameter the effect structure depends on,
which is Arrow-but-not-Kleisli. You earn it by *attempting* the SURD/BRCD
instantiations and showing the structural parameter cannot thread through `bind`
without smuggling it into `M`'s type in a law-breaking way.
- If it holds → the **separation theorem** is the headline.
- If strong monads suffice for BRCD → downgrade gracefully to "the Arrow extends
  the monad to 2 more families" — *still* a verifiable subsumption, just without
  the dramatic separation. Either way the claim stays falsifiable.

## 4. Two load-bearing risks (where "profound" turns into "crank")

1. **"Agree on the interface" carries the whole load.** The four fragments today
   have different carriers (`Model`, `PropagatingEffect`, `State`, `Action`). For
   them to compose, the **objects of the category must line up** —
   discovery's `Model` = inference's parameter; inference's `Effect` = governance's
   input = the state machine's input. *Designing the objects so the fragments meet
   is the real work*, and where you find out whether the unification fully holds or
   only mostly holds. The Arrow gives the shape; you make the objects agree.
   **(Resolved in §10: the shared object is `PropagatingEffect`. This risk is now a
   design rule, not an open unknown.)**
2. **Resist the everything-bagel.** "Discovery + inference + action + governance are
   one thing" is one sentence from sounding like a theory-of-everything. The only
   defense is **precision + mechanization**: not "all is one," but "four named
   arrows, explicit typed signatures, the composite type-checks, the type system
   *enforces the signatures and wiring and propagates the algebraic-law claims via
   marker traits* (e.g. `Associative`), and *the laws themselves are property-tested*."
   (Rust — like Haskell — cannot type-encode the monad/arrow *equations*; that needs
   dependent types. Claiming the laws are "enforced by the type system" is false and a
   CT-literate reviewer will catch it; the marker-plus-property-test framing is honest
   and strictly stronger than a bare typeclass. See `arrow-assumption.md` §2/A4.)
   Every claim is a checkable signature, never a metaphor. That is the line, and the
   mechanized artifact keeps you on the right side of it.

## 5. Where the code already is (and the gap)

`deep_causality_haft` already has the scaffolding: `profunctor`, `promonad`
(composition on a profunctor ≈ a Freyd/Arrow category), `bifunctor` (the `⊗`),
`cybernetic_loop` (feedback ≈ `ArrowLoop`), plus `monad`/`comonad`. **Missing:**

1. **`Morphism<A,B>`** — the explicit typed-arrow base every operator instances.
2. **`Endomorphism<T>: Morphism<T,T>`** — the `T->T` operators (normalization, and
   the BRCD Meek-rule fixpoint hand-rolled today). Already scoped in
   `num-dual-endomorphism.md`; it is the cheapest, most self-justifying first step
   and lands on live code.
3. **Strength / the monoidal product on arrows** (`first` / `***`, the
   strong-profunctor piece) — the actual unlock for multi-input operators (BRCD).
   This is the conceptual heart and the paper's technical fulcrum.

## 6. Build order (de-risked, each its own change)

1. `Endomorphism` in `haft` + retrofit the Meek loop to `iterate_to_fixpoint`.
   Smallest, ships now, proves the primitive on real code.
2. `Morphism` base; re-express the CDL typestate stages as morphisms (validates the
   model on SURD's own pipeline — mechanical). Concurrently, migrate the CDL inner
   carrier from `CausalTensor` to `PropagatingEffect` (§10): load → `lift` →
   `Effect -> Effect` stages. SURD keeps working (its tensor rides inside the
   effect); the payoff is the monoid-of-endomorphisms interior and the single error
   channel. Hold the §10 invariant — structure stays a stage parameter.
3. Strength / monoidal product (lean on `bifunctor` + `profunctor`).
4. Re-cast SURD and BRCD as arrows — the witnessing cells. *This re-casting is the
   paper's demonstration section.*

## 7. Publication scoping (the 25-page reality)

**Reality check (do not let this note flatter the state).** *Nothing is published.*
The only artifacts that exist are the **monograph draft** (the everything-bucket)
and **this note**. There is **no causal-monad paper** — the monad exists as *code*,
not as a published result a later paper can cite. The whole ladder below is
unwritten. That changes the sequencing completely: the *first* paper is not the
grand one, it is the **least ambitious thing that stands alone**, and ambition only
increases as credibility banks.

- **Paper 1 — BRCD ↔ CDL (FIRST, near-term, the one to ship).** This is the work
  already scoped: a reproducible, type-safe port of a Bayesian root-cause-discovery
  method into a causal-reasoning framework, **verified to reproduce the reference
  exactly** (the verification examples), **plus a corrected ranking procedure** (the
  Python exp-underflow bug found and fixed — a concrete, checkable contribution that
  *adds value over the reference*, not just re-implements it). Frame it as an
  **empirical / methods + reproducibility** contribution. The job here is a form a
  **normal review board waves through without hard questions**, so:
  - **The category theory stays OUT.** No Arrow, no monad subsumption, no four
    fragments. At most *one* future-work sentence ("this integration is the first
    instance of a general discovery-operator program") — and even that is optional.
  - Lead with what is unimpeachable: it runs, it reproduces the reference bit-for-bit
    on real cases, it fixes a real bug, the implementation is open and typed. Claims
    stay **local** (this method, these datasets), never general.
  - This is the credibility-establishing paper. Everything categorical is a *later*
    paper standing on this one's reception.
- **Paper 2 — "The Causal Arrow."** The generalization: Arrow ⊋ causal monad; the
  K-method checklist; SURD/BRCD as the non-Kleisli witnesses; the separation (or its
  honest partial). **One home discipline: computational causality**; CT is the
  *instrument* (no CT-novelty claim). **Sequencing dependency:** this paper *cites*
  the monad's subsumption as the fragment it generalizes — but that result is **not
  yet published**. So Paper 2 must either (a) carry a compressed monad-subsumption of
  its own, or (b) be preceded by a short monad paper, or (c) cite the monograph.
  Decide this with the co-author; it is the gating question for Paper 2's scope.
  Govern + act stay in *discussion only* (per §2 status — not yet arrows, and
  experimental).
- **Paper 3 — end-to-end unification.** discover → generate-model → infer as one
  composable arrow (see §11). Needs the model-generation primitive built first.
- **Monograph (draft exists).** The everything-bucket: the four-fragment system
  story, the procedural mechanics, the parts no single paper can carry.

The discipline across all three: **each paper is the smallest claim that still
stands alone.** Paper 1 must not reach for the Arrow; Paper 2 must not reach for the
end-to-end system; the moment a paper carries more than its one falsifiable claim,
the review goes to committee.

## 8. Ergonomics — the builder *is* the syntax of the Arrow

Rust has no higher-kinded types (`* -> *` kinds), so the only way to express the
monad/Arrow traits is the **witness pattern**: a marker type carries the "shape"
and a GAT (`type Apply<T>`) recovers the applied type (defunctionalized HKT,
à la Yallop & White). Haskell's `do` is sugar over `>>=`; Rust has neither `do`
nor HKT — so the CDL's move is to make the **typestate builder *be* Rust's
`do`-notation**: each fluent method advances the witness in `Self`'s type and
applies the monadic op underneath. The witness is real but camouflaged; the user
sees a fluent chain.

The precise (and more defensible) framing for the paper: the builder is not
*hiding* the category theory — it **is the category theory's term syntax**. A
fluent chain that wires arrows together is the textual form of a **string diagram**
(wiring diagram), the canonical graphical language of monoidal categories. So
"clean API ⇔ well-typed wiring diagram" is a *semantic* claim, sound by
construction, not cosmetic camouflage. That is the version a reviewer respects.

**Where it bites — the same seam as everywhere else.** A linear fluent chain is
1-D: it expresses sequential `∘` beautifully. String diagrams are 2-D — the
*parallel* product `⊗` stacks vertically. BRCD's multi-input `Data ⊗ Data ⊗ Graph`
is a `⊗`, which a 1-D chain cannot express *fluently*: it needs explicit product
combinators (`.zip()`, `.and()`, `.with_graph()`) or a nested sub-builder. So the
ergonomics effort concentrates exactly on the monoidal/multi-input fragment —
BRCD again, the same fault line as Arrow-vs-Kleisli. Second, smaller tax: a
*mis*-typed chain leaks the witness types into the compiler error exactly when the
user most needs help; `#[diagnostic::on_unimplemented]` + sealed traits soften it,
never fully remove it.

This is the *right* version of the removed `ControlFlowBuilder`: that one bolted a
graph-builder *beside* the monad; this one is a builder *over* `PropagatingEffect`
that desugars *into* the monad's composition. Same surface ambition, correct
foundation. **Adoption value:** if seven-to-twelve fluent lines do discovery →
cause-selection → reasoning → action with the CT machinery shielded behind the
builder, that is a new category of API that does not exist today. In Paper 2 this
is the *artifact/mechanization* evidence, **not a third claim** — adoption is not
a scientific contribution; it is the motivation and the proof the algebra is real
code.

## 9. Axis 2 — carrier geometry (the dimensional polymorphism)

"Dimension" hides **three orthogonal axes**; conflating them is where this turns
to crank, so name them apart:

| axis | ranges over | witness at the high end |
|---|---|---|
| **syntactic arity** | `∘` (1-D chain) vs `⊗` (parallel wires) | BRCD's multi-input |
| **carrier geometry** | scalar · vector · multivector · manifold section | fluid dynamics (3-D) |
| **effect dynamics** | static-structure ↔ dynamic-`bind` (offline ↔ online) | SURD/BRCD vs the monad |

A 2-D *string diagram* (two wires) and a 2-D *velocity field* (carrier on a plane)
are unrelated; a reviewer who sees them merged stops reading. Once separated, the
unification is the **product of independent axes** — defensible precisely because
each axis is checkable alone and "they compose" is a type-checking claim.

**The move that makes 3-D work: don't extend the Arrow — make the *objects* carry
the geometry.** An Arrow `A ⇝ B` does not care whether `A` is `f64` or
`Multivector<ℝ³>`; the geometry is *internal structure of an object*, the arrow
only wires objects. So you never "lift the Arrow to 3-D" — you instantiate its
objects with geometric carriers and the *same* discover/infer/govern/act
combinators compose unchanged (the Arrow is *polymorphic over a monoidal category
of carriers* — not "enriched," which is stricter). This turns the fluid-dynamics
ticket from a separate mountain into a **carrier instance**: the geometric-algebra
/ Riemannian code already written becomes one `Morphism<Multivector<3>, …>` impl,
not a new analysis. The witness/HKT machinery is identical to the scalar case;
only the type parameter moves — so **the same fluent program does 1-D scalar or
3-D fluid discovery by swapping the carrier type.** Verifiable: same code, swap the
param, the laws still hold.

**"3 is enough, not 4" — a modeling choice, not a theorem.** Time is *not* a
carrier dimension; it is the `State` thread of the `act` fragment (the
Mealy/StateArrow). A 3-D spatial carrier marched by the state arrow gives 3+1
dynamics — the **method-of-lines / semidiscrete split** every non-relativistic PDE
solver uses (discretize space, step time). Valid *because* non-relativistic fluid
dynamics admits a clean 3+1 split (absolute time, no frame-mixing); it would break
only where time must be a geometric coordinate on equal footing with space
(relativistic fields), which fluids never need. State it as "valid for the intended
domain," not a law.

**Scoping (or the review dies in committee).** Paper 2 claims the *polymorphism*,
witnessed **small** — one 2-D vector-field discovery cell showing the identical
arrow runs over a geometric carrier. Full 3-D Navier-Stokes is a paper unto itself:
monograph / future work, never in the 25 pp, or the review goes to a PDE-numerics
committee and never returns. (None of this touches Paper 1, which carries no
categorical content at all.)

## 10. The carrier keystone — `PropagatingEffect` as the shared object

This resolves §4 Risk 1. The open question was "what object do the four fragments
agree on so they compose?" The answer: **the shared object is `PropagatingEffect`.**
Load raw data, lift it into a `PropagatingEffect` early, and pass *effects* (not
`CausalTensor`s) through the algebra. The tensor is *embedded in* the effect, so the
carrier is uniform while the payload varies.

**Why it composes — a monoid of endomorphisms.** Once every interior stage is
`Effect -> Effect`, source = target = one object, and a one-object category with
uniform composition is a **monoid** — the maximally composable shape. That is
literally build-order item #1: `Endomorphism<PropagatingEffect>`. Discover / infer
/ govern then compose by ordinary arrow composition *through a shared object* — no
impedance match at the discover→infer joint §2 worried about. "The last effect
exiting the CDL is already the monad's value" means the monad is not a stage you
enter; it is the sub-fragment where source = target = `Effect`. The CDL becomes
"more monadic" in the good way, and error handling collapses to **one short-circuit
channel** (an error/halt effect variant) instead of per-stage `Result` plumbing.

**The one invariant that protects the whole thesis.** Collapsing every carrier to
`PropagatingEffect` pushes the CDL *back toward* "everything is the monad," which
contradicts the §1 claim that discovery is **non-Kleisli**. The rule that keeps
both the ergonomic win and the publishable separation:

> **Data flows as `PropagatingEffect`. Static structure stays a parameter of the
> morphism — never payload inside the flowing effect.**

Tensor values, field values, regime cohorts ride *in* the effect. The graph, the
SURD subset-lattice, the manifold's metric/Clifford signature *parameterize the
stage* and never enter the value. The two properties are independent: **uniform
object ≠ uniform morphism-class.** Uniform carrier → objects agree → composes;
structural parameter → still an Arrow, not `bind` → stays non-Kleisli. Blur the
line — push the graph into the effect and `bind` it dynamically — and discover
collapses to Kleisli and "Arrow ⊋ monad" evaporates. This also refines §9's
fluid remark precisely: the *velocity field values* are payload (ride in the
effect, dimensional polymorphism for free); the *manifold structure* is a
parameter (same as BRCD's graph). Field → effect; geometry → parameter.

**Phase vs payload — the two-layer reconciliation with §8.** Making every stage
`Effect -> Effect` looks like it discards the typestate builder's compile-time
ordering. It does not, because the two live at different layers:

- **Witness / typestate = the phase**, compile-time, *outer*. The builder threads
  `WithRawData -> WithDiscovery -> WithInference` in the *type*; wiring stages out
  of order is a compile error.
- **`PropagatingEffect` = the payload**, runtime, *inner*. The flowing value;
  carries the tensor, the field, the error.

Switching the inner carrier from `CausalTensor` to `PropagatingEffect` changes
nothing about the outer guarantees — they were never in the payload. The builder
camouflages the witness (phase); the effect carries the data. You keep compile-time
stage ordering **and** gain the uniform value algebra.

**`lift: Effect -> Process` for cross-stage context.** When a stage must carry
state/context forward, lift the effect into a `PropagatingProcess` — `unit` into a
State/Writer enrichment. The next stage reads the carried context out of the
process. This is strictly cleaner than widening the builder's state tuple every
time a stage needs context, and it is standard monadic layering, so it reads as
principled rather than ad hoc.

**The honest tax.** The uniform error channel (the win) is the *same mechanism* as
the cost: a sprawling `PropagatingEffect` sum type where every consumer must
match-or-error on payloads. The outer witness layer is what buys back compile-time
ordering, so the runtime `match` is only for genuine payload variation, never stage
mis-wiring. Keep the error variant able to distinguish "stage failed" (recoverable
domain error) from "the propagation legitimately halted" (a real causal outcome),
or they fold together.

## 11. Three stages, one arrow (discover → generate-model → infer)

The discover→infer joint is not two stages but **three**, and naming the middle one
is what makes "discovery output becomes a runnable model" concrete instead of
hand-waved:

| stage | arrow | type | nature |
|---|---|---|---|
| 1. **discover** | static arrow | `Data -> Decomposition` | data-derived, then frozen (SURD's PID result) |
| 2. **generate-model** | pure functor | `Decomposition -> CausaloidModel` | **the one new primitive** — no evidence flows |
| 3. **infer** | Kleisli (existing monad) | `CausaloidModel ⊗ Evidence -> PropagatingEffect` | generic forward mechanism |

**The middle stage closes the mechanism gap without any estimator.** A SURD
decomposition maps to a recursive causaloid graph by the PID→aggregation
correspondence (unique → singleton; redundant → `Any`/OR-collection; synergistic →
`All`/AND-collection). Because the PID *roles already are the aggregation logic*,
generate-model produces a graph that already knows how to combine its inputs — so
stage 3 is just the existing monad propagating fresh evidence through it. **Evidence
in, effect out; no fitted regression, no two-cohort windows.**

**BRCD does *not* fit here — and the reason is structural, not incidental.** An
earlier idea (use BRCD's ridge-Gaussian estimator as the per-node mechanism-fitter)
is wrong: BRCD's estimator is **intrinsically contrastive** — it scores how a fit
*differs between two aligned regimes* (normal vs anomalous). The two aligned windows
are load-bearing, not plumbing; strip them and there is nothing to estimate, and a
counterfactual cannot synthesize the second *observed* window. So BRCD is not a
general `f`-fitter. Consequence to keep straight: **composite-RCA (SURD × BRCD) is a
distinct product** — two methods on the *same two-cohort data*, cross-validating
localization — *not* this single-stream forward pipeline. The three-stage pipeline
is SURD-native and broader precisely because it needs one stream, not two windows.

**Why the middle stage earns independent existence (three reasons):**
1. **It is where the static structural parameter is *born*.** discover + generate
   produce the fixed structure; infer consumes evidence *against* it. So the §10
   static|dynamic boundary — the invariant that keeps discover non-Kleisli — falls
   **exactly at the generate|infer seam.** This is the sharpest statement of the
   whole thesis.
2. **It is the reusable bridge.** Any discovery method emitting a role-decomposition
   feeds the *same* generator; heterogeneous discovery outputs normalize to one
   executable form here.
3. **It is pure** (no evidence) → easiest to test exhaustively, safest to mechanize
   first.

**The vision is buildable because only one primitive is new.** discover is Paper 2
work; infer is the *existing* monad. The end-to-end unification needs only the
**generate-model functor** + composition via the §10 carrier (the `CausaloidModel`
rides inside a `PropagatingEffect`). "Build one pure functor, wire two things that
already exist" — that is how the massive contribution slices into manageable tasks,
and it is the content of **Paper 3**.

**The one residual design risk (where the hard thinking goes):** generate-model must
define the **activation semantics** — how a discovered *role + strength* becomes a
propagating causal function (e.g. how "unique cause, strength 0.4" turns into "fires
when X crosses θ"). That is a generic, discovery-parameterized rule — *not* a fitted
statistical model and *not* cohort-dependent — but it **is a modeling choice** (same
status as the redundancy→OR reading or the 3+1 split): it must be stated and
validated, never assumed. That choice is the actual scientific content of Paper 3.

## 12. Open questions (for the co-author)

- Is the separation a clean theorem (static-parameter ⇒ non-Kleisli) or only a
  partial taxonomy? Earn it constructively before claiming it.
- Right categorical home: symmetric monoidal category + Arrows (Hughes), or a Freyd
  category / a PROP / wiring diagrams (Spivak)? The co-author's venue knowledge
  decides the framing and the one load-bearing theorem.
- Which method families actually instantiate the signature, and which need an
  enriched variant (gradient/NOTEARS may want a differentiable enrichment — see
  `num-dual-endomorphism.md` `Dual<T>`).
- Does the *values-in-effect / structure-in-parameter* invariant (§10) hold for
  every discovery operator, or is there a family whose structure genuinely must
  flow as data? That family would be the real test of the separation theorem.
- How far does `PropagatingEffect` widen before the god-enum tax (§10) outweighs
  the uniform-carrier win? Is the answer "sealed payload trait inside one variant"
  rather than a variant per payload type?
- **Paper 2 sequencing:** does it carry a compressed monad-subsumption itself, get
  preceded by a short monad paper, or cite the monograph? The monad result is not
  yet published (§7); this gates Paper 2's scope.
- **generate-model activation semantics (§11):** what is the right generic,
  discovery-parameterized rule mapping role + strength → causal function? This is
  Paper 3's core modeling commitment — settle it before it becomes code.
