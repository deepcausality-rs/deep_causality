<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# The Causal Full Stack: Pearl → Relativistic → Hardy/ICO → Quantum Causal Model

**What this is.** A roadmap note. It places four causal-modelling regimes — classical Pearl DAGs,
relativistic causal structure, Hardy-style indefinite causal order (ICO), and the Barrett–Lorenz–
Oreshkov quantum causal model (QCM) — on **one** substrate, and states precisely what DeepCausality
already provides, what each regime still needs, and the minimal set of invariants that must hold to
carry the whole stack. It is the integrating companion to
[`QCM-on-EPP.md`](QCM-on-EPP.md) (which does the level-4 work in depth) and
[`quantum-epp.md`](quantum-epp.md) (the substrate-agnostic-orchestration corollary).

Honesty convention (as elsewhere): **[holds]**, **[holds under precondition]**, **[open]**,
**[speculative]**.

---

## 1. The thesis: one engine, three orthogonal axes

The four regimes are not four frameworks. Each is a *coordinate* in the same three-axis space, so a
single engine spans all of them by fixing three parameters:

- **Axis 1 — Wiring.** The structural object. An **orderless hypergraph**; a DAG is a specialization,
  a cyclic graph a generalization. A hyperedge `{Pa(A_i)} → A_i` names the parent set directly.
- **Axis 2 — Order source.** Where "before/after" comes from. Nothing (topological only) · a
  **metric carried as monad-channel data** — in the **state** or **context** channel, applied
  *coordinate-free* inside the causal function, or supplied globally by an external context
  (Euclidean/Lorentzian/between) · left **indefinite**. Geometry is *data in a channel*, never a
  background the engine is embedded in; order is always *derived*, never baked into the wiring.
- **Axis 3 — Carrier.** The type of the relata — the effect value+state threaded by the causal
  monad. Classical **verdict** · **probabilistic** · **operator-valued** (Choi–Jamiołkowski
  operators).

The load-bearing design commitment that makes this work: **the hypergraph carries the causal
structure; the monad's `bind` carries only sequencing.** Quantum causal structure is inversion-
symmetric — a unitary `U` and its adjoint `U†` describe the same structure with every arrow reversed
(Lorenz 2022, §6) — while `bind` is irreversible by construction. So causal structure cannot live in
the bind order; it lives in the (order-neutral) hypergraph, and order re-enters only as a *derived*
refinement. This is why the same engine reaches from Pearl to QCM. **[holds]**

A second commitment sharpens Axis 2: **relativistic (and any) spacetime is not predicated on an
external Context.** The arity-5 monad's **state** channel carries the physical state, and the metric
is applied *coordinate-free* inside the causal function — the `event_horizon_probe` example computes
relativistic rapidity and time dilation with `Metric::Minkowski(4)` acting on the probe's own
4-vectors via geometric algebra, with the probe state threaded by the monad and the context holding
only the black-hole mass (a parameter, not a coordinate). The monad thereby **inverts the notion of
an external context into threaded internal state** — it does the same work as a supplied geometry
while *decoupling from any specific spacetime embedding*. State and context are interchangeable here
because both are monad channels; geometry is data in a channel, not a background. So there are two
equivalent routes to relativistic structure — **internal** (state-carried, coordinate-free, local:
the working `event_horizon_probe` route) and **external** (a context supplying a global coordinate
metric) — and the internal route needs no new substrate. **[holds — working example]**

## 2. What DeepCausality already provides (the shared substrate)

- **Orderless hypergraph**, DAG as a specialization; dual-state (dynamic for mutation, frozen for
  analytics) with a `freeze_dag` acyclicity gate. **[holds]**
- **External Context** carrying geometry/metric — the causal engine is spacetime-agnostic, the
  geometry is a parameter (Euclidean, Lorentzian, or interpolated). **[holds]**
- **Causal monad** (`PropagatingProcess`/`PropagatingEffect`, arity-5: value · error · log · state ·
  context), whose Kleisli category is a **lawful, unconditional** monad/category — machine-checked
  (`core.causal_monad.{lawful,…}`, `core.causal_arrow.category_laws`). Non-commutative = it carries
  order. **[holds]**
- **Commutative-monoid fan-in** `∇` and the symmetric-monoidal PROP generators (copy `Δ`/`ε`, merge
  `∇`/`η`, symmetry `σ`) — `deep_causality_haft::SymMonoidal`, laws in
  `Haft/SymmetricMonoidal.lean`. The classical order-free fusion of co-present siblings. **[holds]**
- **Reified free Arrow** (`ArrowTerm`/`ArrowCore`) + one-way **interpreter into `Kleisli<M>`** +
  `NaturalTransformation` — the syntax→semantics machinery (`haft.arrow_term.*`,
  `haft.interpreter.*`). **[holds]**
- **Carrier headroom**: the value/state channel can hold `Complex`/`Quaternion` division algebras,
  `HilbertState`, Clifford algebras, gauge fields; `Float106` double-double precision; `Uncertain<T>`
  for nondeterminism. **[holds for representation]**

The categorical substrate is deliberately the **classical corner** — the Kleisli category of
`Writer ∘ Exception ∘ State/Reader`, a copy-ful Markov category, not the dagger-compact / no-cloning
CQM corner. This bounds what "native" quantum means here (§6). **[holds]**

## 3. The four levels — HAVE / GAP

| Level | Wiring | Order source | Carrier | HAVE | GAP |
|---|---|---|---|---|---|
| **1. Pearl DAG** | DAG ⊂ hypergraph | none (topological) | classical verdict | hypergraph names parent sets directly; classical Kleisli corner; sequencing + fan-in | probabilistic **verdict carrier** (bounded-lattice/MV, assumption #5); Markov factorization + **do-intervention**/counterfactual as first-class checked ops |
| **2. Relativistic** | hypergraph | Minkowski metric as **channel data** (state or context) | classical verdict + frame data | geometry is monad-channel data, not an external embedding: the metric (`Metric::Minkowski`) is applied **coordinate-free** (geometric algebra) inside `f` on the state's own 4-vectors — **`event_horizon_probe` is a working example**; **timelike → `bind`, spacelike → commutative `∇`** (relativity of simultaneity = the commutative fan-in) | **invariant → causal-order derivation** (Minkowski interval from state 4-positions → timelike/spacelike → `bind` vs `∇`); **frame-covariance** + no-signalling as freeze checks |
| **3. Hardy / ICO** | orderless hypergraph | indefinite / dynamical (contextual) | classical mixture, or oracle | order-neutral base = "no presupposed order"; contextual metric = dynamical causal structure; `∇` = classical order-indifference; inner `f` can **encapsulate** a process matrix | *classical* ICO (convex mixture over orders, order-as-random-variable) via `Uncertain`/`continue_with`; *quantum* ICO (superposition-of-orders + interference) needs the linear carrier — shared with 4b |
| **4a. QCM / Lorenz (hosted)** | hypergraph, incl. **cyclic** | operator-level structure | **operator-valued** CJ state (Float106 complex matrices) | `QCM-on-EPP.md`: CJ operators as arity-5 **monad-state**; encapsulation = flat = **monad law-3** (proven, inherited); immutable context for `ρ_A`; cyclic-QCM aligns with native non-DAG | **Layer D — operator commutativity `[ρ_j, ρ_k] = 0`** (Lorenz Def 3.3, load-bearing at ≥3 factors); the **partial-trace-preserves-commutativity** theorem; freeze-time commutator walk; depth-aware `Float106` tolerance; the **coproduct/direct-sum generator `⊕`** for causally faithful reification (Lorenz–Barrett 2021 §3: sequential+tensor is insufficient; roadmap Stage 2b) |
| **4b. Native CQM** | — | — | linear / dagger-compact | (not a goal) | drop the **copy comonoid** (no-cloning), compact-closed carrier, coherence *across* nodes — **bounded out** by the Kleisli / measurement cut |

### 3.1 Reference definitions (level 4, Lorenz 2022)

- **Process operator** `σ_{A₁…Aₙ}`: a positive operator on `⊗ᵢ (H_{Aᵢ} ⊗ H*_{Aᵢ})`, the neutral
  object that, given any interventions at the nodes, predicts the joint outcome distribution — the
  quantum analogue of `P(X₁,…,Xₙ)`, on the same footing regardless of the nodes' spatio-temporal
  relations.
- **CJ operator** `ρ_{Aᵢ|Pa(Aᵢ)}`: a CPTP map represented as a positive operator (channel–state
  duality).
- **Quantum Markov condition** (Def 3.3): `σ` is Markov for a graph `G` iff it factorizes into
  pairwise-**commuting** CJ operators, `σ = ∏ᵢ ρ_{Aᵢ|Pa(Aᵢ)}`. The commutativity clause is free at
  two factors (hermiticity) and **independent and substantial at three or more** — it bites where
  parent sets overlap on a shared Hilbert support. This is the genuinely-quantum content.
- **Causal relation** (Allen et al. 2017): defined via **influence through unitary evolution** — `C`
  influences `E` iff the marginal output `ρ_E` depends on the input `ρ_C` for some assignment to the
  other inputs ("no influence without disturbance").
- Classical causal models are **formally contained** as the special case where `σ` is diagonal in a
  product basis (via classical split-node models). Cyclic QCM (Barrett–Lorenz–Oreshkov 2021) lifts
  the DAG restriction — aligning with the native non-DAG hypergraph.

## 4. What must be true to support the full stack (seven invariants)

The first five are largely in hand; the last two are the frontier.

1. **One order-neutral structural core.** The hypergraph is primitive and order-agnostic; every
   notion of causal order is *derived* (metric, schedule, or indefinite), never baked into the
   structure. This linchpin makes levels 1–4 instances of one engine. — **[holds]**
2. **Geometry as monad-channel data, not a required external embedding.** The metric lives in the
   **state** or **context** channel and is applied *coordinate-free* inside the causal function
   (geometric algebra on the state's own 4-vectors — `event_horizon_probe`), or is supplied globally
   by an external context when a shared coordinate geometry is wanted. The monad **inverts the
   external-context notion into threaded internal state**, achieving the relativistic content while
   decoupling from any specific spacetime. A derivation `invariant → causal partial order` (Minkowski
   interval → timelike/spacelike → `bind` vs `∇`) reads the state's own invariants. — the internal
   (state-carried, coordinate-free) route **[holds — working example]**; the invariant→order
   derivation + frame-covariance freeze checks **[open]** (level 2, reused by 3).
3. **A carrier tower with an explicit boundary.** `O` climbs `Verdict` (bounded lattice / MV-algebra,
   #5) → probabilistic (`Uncertain` / distribution) → **operator-valued** (complex matrices / CJ
   operators, `Float106`). The classical↔quantum **Kleisli boundary** — collapse to classical at
   every node exit, the measurement cut — must be a stated, type-enforced seam. — classical +
   operator-representable **[holds]**; #5 closure + operator-state property-tests **[open]**.
4. **Two distinct `Commutative` notions, never conflated** (the `QCM-on-EPP.md` layer table):
   - **value/order commutativity** = the commutative monoid `∇` at fan-in (classical order-
     indifference) — **[holds]**;
   - **operator commutativity** = `[ρ_j, ρ_k] = 0` on shared support (the quantum Markov condition)
     — **[open]**. This is the *only* genuinely-quantum check in the whole stack.
5. **Global properties enforced at the freeze boundary**, not during forward evaluation — because
   Markovianity, no-signalling, acyclicity, and verdict-closure are structural facts about the whole
   graph, and `σ` is inversion-symmetric so it cannot live in the directional `bind`. — freeze
   mechanism **[holds]** (`freeze_dag`); the relativistic + operator-commutativity walks **[open]**.
6. **The one hard theorem (QCM):** partial trace preserves operator commutativity under
   encapsulation — proven under identifiable conditions (candidate: single-node interface, or shared
   supports lying on the encapsulation boundary) or refuted by counterexample. Everything else
   structural is *inherited* from monad law-3 associativity; this is the single proof obligation that
   is actually about quantum mechanics. — **[open]** (see `QCM-on-EPP.md` §"What is left to do", item 2).
7. **An honest, stated carrier boundary.** Coherence is confined to one causaloid (the Kleisli
   boundary = the measurement cut). QCM is **hosted** — operator data *verified* classically at
   freeze, achievable on the classical substrate because it *checks* a quantum condition rather than
   *propagating* amplitudes. **Native** CQM and compositional quantum-ICO (levels 3-quantum / 4b)
   require dropping the cartesian copy comonoid for a dagger-compact / linear monoidal layer — a
   different substrate. — the boundary must be **stated, not crossed** ([speculative] beyond it).

## 5. The three "commutativity/order" regimes (do not conflate)

The recurring trap across the stack is treating one `Commutative` as if it settled all of them. They
are three different things at three layers:

| Regime | Meaning | Structure | Where in the stack |
|---|---|---|---|
| **Definite order** | fixed `A` then `B` | non-commutative monad (`bind`) | sequencing everywhere; timelike edges (L2) |
| **No / irrelevant order** | both orders give the same result (classical invariance) | commutative **monoid** `∇` | fan-in fusion; spacelike edges (L2); classical ICO (L3) |
| **Indefinite order (quantum)** | coherent **superposition** of orders, observable interference | *neither* — linear / higher-order supermap | quantum ICO / 4b — **[open], needs linear carrier** |

Commutativity is the *classical* shadow of indefinite order (invariance, no fact-of-the-matter); the
quantum article is superposition-with-interference, which erasure-by-commutativity cannot express.
The commutative `∇` correctly carries the classical/relativistic case; the quantum case is the one
carrier-gated frontier.

## 6. The honest boundary

The categorical substrate is classical, so **once a value crosses the Kleisli boundary it has
collapsed to classical data** (`quantum-epp.md` §5). Consequences that hold across the whole stack:

- **Quantum coherence is confined to a single causaloid `f`.** Inter-node hyperedges carry classical
  bits — or, for level-4a, CJ operators as *data to be checked*, not live amplitudes. **[holds]**
- Levels 1–3-classical and **4a (hosting QCM)** are reachable on the classical substrate: **L2 is
  already demonstrated** (`event_horizon_probe` runs Minkowski relativity coordinate-free from
  monad-state, no external spacetime); L1/L3 by carrier + a derived-order scheduler; 4a by
  operator-valued monad-state + the freeze-time commutativity check, with structural soundness
  inherited from monad law-3. **[holds / holds under precondition]**
- **3-quantum (quantum ICO)** and **4b (native distributed CQM)** are the genuine frontier and are
  *carrier-gated*: they need a linear / dagger-compact monoidal layer without the copy comonoid.
  These are `[speculative]` and explicitly outside the current stack — a stated boundary, not an
  accidental gap.

## 7. Crate topology — where each level is codified

The **formalized `deep_causality` core** is the substrate: the order-neutral hypergraph, the causal
monad with its Lean-checked monad/category/arrow laws, the external Context, `SymMonoidal`, and the
`ArrowTerm` → `Kleisli<M>` machinery. Every higher stack crate builds **atop the formalized core and
inherits its guarantees**, so a level-specific crate adds only its *own* new obligations rather than
re-establishing the substrate. Each crate depends **downward only** (the Pearl and quantum crates both
depend on the formalized core, not on each other), so both stacks compose in one model — a causaloid
graph can carry a classical SCM subgraph and a quantum subgraph joined by the *same* hyperedges —
without either crate knowing about the other. **[holds — substrate; planned — the crates below]**

- **`deep_causality_do_calculus`** — codifies the **Pearl stack (level 1)** atop the formalized core:
  causal Bayesian networks / SCMs over the hypergraph, the causal Markov factorization, Pearl's
  **do-operator** (intervention = graph surgery), counterfactuals (SEM / twin-network), d-separation,
  and identification. Its carrier requirement is the probabilistic **verdict** tier (assumption #5).
  Classical and fully verifiable. **[planned]**

- **`deep_causality_quantum`** — holds the **quantum + QCM implementation (levels 3-quantum / 4)**
  atop the formalized core: operator-valued **CJ-operator state**, quantum nodes and process
  operators, the **quantum Markov condition** with the Layer-D operator-commutativity check at freeze
  (`Float106`, depth-aware tolerance), cyclic-QCM support, and ICO / quantum-switch representations.
  Its wiring language requires the **coproduct/direct-sum generator `⊕`** (Lorenz–Barrett 2021:
  causally faithful decomposition needs direct sums beyond sequential+tensor; formalized at the haft
  layer per roadmap Stage 2b, instantiated here as the Hilbert direct sum). Its carrier requirement
  is the operator-valued (complex-matrix) tier; its one open theorem is partial-trace preservation
  (§4, invariant 6). **[planned]**

- **Relativistic (level 2)** needs no dedicated substrate: the metric is **channel data** applied
  coordinate-free inside the causal function (geometric algebra on the state's 4-vectors — the
  `event_horizon_probe` example already does this with `Metric::Minkowski`, no external spacetime).
  What a crate would add is optional convenience — an **invariant→order scheduler** (Minkowski
  interval → timelike/spacelike → `bind`-vs-`∇`) and frame-covariance / no-signalling freeze checks —
  thin enough to live as a module over the formalized core rather than a new stack crate.
  **[partially holds — working example; scheduler planned]**

This gives the "single graph across the classical/quantum divide" of `QCM-on-EPP.md` §"Why it
matters" a concrete crate boundary: the semantics of each level live in a dedicated crate, the
substrate and its proofs live once in the formalized core.

### 7.1 The logical/physical bridge — a real QPU is just an effect

Orthogonal to the four **logical** levels is a **substrate** axis: classical silicon versus
**physical quantum hardware**. A causaloid's inner function `f` may call a **cloud QPU** (e.g. AWS
Braket, IBM Qiskit Runtime, IonQ, Quantinuum) to perform **genuine quantum-native sampling** —
prepare → evolve → measure — and return the shots as classical data at the Kleisli boundary (the
measurement cut). To `bind`, that step is indistinguishable from CPU arithmetic (`quantum-epp.md`
§2–3): a QPU is simply the most exotic effect the monad will ever compose. This makes **physical
quantum randomness** — genuine Born-rule samples and entanglement correlations, not a PRNG —
available to *any* level's nondeterministic modality.

Home and modality: the device adapters live in **`deep_causality_quantum`** (or a thin hardware
sub-layer beneath it). The crate therefore spans two modalities that must be kept distinct:

- **Simulated QCM** — deterministic CJ operators evaluated in-process; stays in the **verifiable**
  region (the Layer-D commutativity check is meaningful, results reproducible). **[planned,
  verifiable]**
- **Physical QPU sampling** — a call into an open, queued, noisy, calibration-dependent device;
  couples to the world and sits in the **emergent / unverifiable** modality (`quantum-epp.md`
  §4.9.4; `QCM-on-EPP.md` §"Why it matters", caveat 3). The arity-5 monad supplies exactly the
  scaffolding this needs — state (variational params), error (job failure / decoherence / timeout),
  log (backend, calibration snapshot, shot count — tamper-evident provenance), context (device
  topology / calibration as queryable fact). **[planned, emergent]**

The two are one crate because they share the operator/Hilbert carrier and the causal wiring; they
are kept apart by *modality*, so a model states plainly whether a quantum verdict rested on a
**checked simulation** or on **physical evidence** before the safety layer clears it.

## 8. Bottom line

DeepCausality already owns the **entire classical spine**: Pearl (needs only the #5 verdict carrier +
a do/counterfactual operator), relativistic (needs only the metric→order scheduler + frame-covariance
checks), and Hardy's *classical* indefinite structure (orderless hypergraph + contextual metric +
`∇`). The Lorenz **QCM is hostable** by carrying CJ operators as operator-valued monad-state, with
structural soundness inherited from monad law-3 — collapsing the quantum frontier to **exactly two
open items**: the operator-commutativity check (Layer D) and the partial-trace-preservation theorem.
Genuine coherent/compositional quantum order is the one thing outside the stack, and it is outside
because the carrier is cartesian/classical — a boundary to state, not a gap to paper over.

Codification path: the substrate and its proofs live once in the **formalized `deep_causality`
core**; the **Pearl stack** is a dedicated `deep_causality_do_calculus` crate atop it, the **quantum
+ QCM stack** a dedicated `deep_causality_quantum` crate atop it, and the **physical bridge** — a
causaloid calling a cloud QPU for real quantum sampling — enters as an effect inside a causaloid,
its device adapters homed in the quantum crate under the emergent (unverifiable) modality (§7).

## 9. Relation to the assumption tracker

This note operationalizes assumption **#11a** ("`CausaloidType`/`F` is closed at three forms"): the
**carrier tower** of §4.3 *is* the extensible `F` that #11a concludes must exist. The stack's new
generators are not new wiring arms — they are **carrier upgrades** (verdict → probabilistic →
operator-valued) plus **derived-order** and **freeze-time-check** machinery over the *same*
hypergraph. It also depends on #5 (the verdict-carrier closure is level 1's gap), and it inherits the
#2 fan-in ruling (commutative `∇` per input type) as the classical order-free layer.

## 10. Related notes

- [`QCM-on-EPP.md`](QCM-on-EPP.md) — the level-4a reconstruction in depth (CJ operators as monad-
  state; the Layer A–D table; the partial-trace proof obligation).
- [`quantum-epp.md`](quantum-epp.md) — the substrate-agnostic quantum–classical orchestration
  corollary and the Kleisli-boundary / measurement-cut discussion.
- `Quantum causal models-lorenz2022.pdf` — Lorenz 2022 (process operator; quantum Markov condition
  Def 3.3; causal relation via unitary evolution).
- `Cyclic Quantum Causal Models-2002.12157v3.pdf` — Barrett–Lorenz–Oreshkov 2021 (non-DAG QCM;
  the frontier the hypergraph aligns with).
- `Causal and compositional structure of unitary transformations-2001.07774v2.pdf` — the reversible/
  compositional (unitary-circuit) structure that lives *inside* the inner function, not in `bind`.
- `deep_causality/papers/causaloid.pdf` — Hardy 2005 (arXiv:gr-qc/0509120), the level-3 primary
  source: the symmetric causaloid product ⊗^Λ (Eq. 2, p. 4), Λ matrices as the symmetry-breaking
  connection data (p. 4), and the ⊗-vs-sequential-product unification (§2, p. 3).
- [`../causal-algebra/Causaloid-structure.md`](../causal-algebra/Causaloid-structure.md) — the
  Hardy-inversion thesis (symmetric element, asymmetric composition) and the three-shape
  isomorphic-recursive structure.
- [`../causal-algebra/causaloid-formalization-roadmap.md`](../causal-algebra/causaloid-formalization-roadmap.md)
  — the staged formalization program that closes this note's gaps and establishes the foundation for
  `deep_causality_do_calculus` and `deep_causality_quantum`.
- [`../causal-algebra/algebraic-causaloid-assumptions.md`](../causal-algebra/algebraic-causaloid-assumptions.md)
  — assumptions #2 (fan-in `∇`), #5 (verdict closure), #11a (extensible `F`).
- `examples/physics_examples/event_horizon_probe/` — the working level-2 demonstration: Newtonian↔
  relativistic regime switching through the causal monad, with `Metric::Minkowski(4)` rapidity/time-
  dilation computed coordinate-free from monad-state — geometry carried in a channel, no external
  spacetime embedding.
