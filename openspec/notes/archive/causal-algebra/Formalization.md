<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# The Causal Algebra — Formalization & Verification Program

**What this is.** A scope document for formalizing and verifying the *core* of DeepCausality as a
**Causal Algebra**: an algebraic theory whose composition operator is Kleisli composition, whose
generators are a dedicated effect system, whose carrier is an arity‑5 effect monad, that is
**machine‑checked in Lean**, **implemented in Rust**, and **best‑effort Rust‑verified**.

**Thesis"** A causal algebra on paper may well exist
in some form (not yet searched). The contribution here is the *stack*: a single causal effect
**(monad)** ⟷ its algebraic **(theory)** presentation, **proved consistent in a proof assistant**,
**realized in a real systems language**, and **linked back to that code by verification tooling**.
Formalism + proof + implementation + code‑level verification, as one artifact, is the differentiator.

This note is the umbrella. Detail lives in:
[`../causaloid/Causal-Algebra.md`](../causaloid/Causal-Algebra.md) (the algebra construction),
[`../causaloid/Causaloid-Formalization.md`](Causaloid-Formalization.md) (singleton),
[`../causaloid/algebraic-causaloid-assumptions.md`](algebraic-causaloid-assumptions.md)
(open assumptions). Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**.

---

## 1. The object

The Causal Algebra presents the **Causal Monad** `M`, an arity‑5 effect monad:

```
M(T) ≅ (S × Option C) ⟶ ( Either E (Maybe T), S, Option C, L )
```

five channels — value `Maybe T`, Markovian state `S`, context `Option C`, error `E`, audit log `L` —
read as the transformer stack `StateCC ∘ ExceptT E ∘ Writer L` (order fixed by the error semantics:
log and state survive an error). `PE(T) = M_{1,1,CausalityError,EffectLog}(T)`.

**The algebra** = the Kleisli category `𝓒 = Kl(M)` with composition `>=>` (the *propagation operator*,
= the EPP axiom `m₂ = m₁ >>= f`), unit `η`, **extended** by a fixed effect signature:

| Generator | Faculty |
|---|---|
| `tell : L → 1` | provenance / audit log (monoid `L`) |
| `raise : E → 0` | hard failure (short‑circuit) |
| `none : 1 → 0` | absence of signal (`= raise` at value demand) |
| `get/put : 1→S / S→1` | Markovian state |
| context faculty | **decision pending**: Reader (`ask`) vs State (`getc/putc`) — `bind` currently threads context, so it is State‑shaped today |

The three causaloid forms are three *uses* of one monad: **singleton** = a Kleisli arrow; **collection**
= an Eilenberg–Moore algebra run + a commutative‑monoid fold over a verdict carrier; **graph** =
Kleisli composition extended with copy/join in a dataflow category.

---

## 2. Two non‑negotiable preconditions

The whole program is sound only on a cleaned base. Both are currently **unmet** and both are the
*same* fixes a reviewer and the Lean checker independently demand.

- **(P1) Control‑free.** Remove `RelayTo` (a computed jump = a control effect; not algebraic, needs
  handlers) and `Map` from the core. Relocate to a separate handler layer / future work.
- **(P2) Lawful carrier.** ✅ **Landed** (`openspec/changes/enforce-w-invariant`). The invariant
  **W: `error = Some ⇒ value = None`** is now enforced structurally: the carrier holds one private
  channel `outcome: Result<EffectValue<T>, Error>` (the `Either E (Maybe T)` encoding), with all
  fields private and construction through total constructors — the value‑AND‑error state is no
  longer representable. Under W the monad's **right‑identity law holds unconditionally** (it failed
  before, on errored carriers); right identity, associativity, and the error left‑zero are proved in
  `lean/DeepCausalityFormal/Core/CausalMonad.lean`, machine‑checked by the Kani harnesses in
  `deep_causality_core/tests/kani_proofs.rs`, and witnessed in `causal_monad_tests.rs`.

> P2 was the single load‑bearing fix; with it landed, the singleton Causal Monad is a genuine monad
> and the downstream algebra / Lean proof / parity work is sound. (P1 — removing `RelayTo`/`Map` —
> remains gating for the full `LawfulMonad` instance.)

---

## 3. The four‑layer verification architecture

The program is layered by *what each tool can actually prove*. No layer claims another's guarantee.

```
  ┌───────────────────────────────────────────────────────────────┐
  │ L1  MATH MODEL — Lean 4 + Mathlib                              │  deductive, unbounded, higher‑order
  │     LawfulMonad(M); effect equations; collection order‑indep.  │  → "the rules are consistent"
  ├───────────────────────────────────────────────────────────────┤
  │ L2  RUST ↔ MODEL (empirical) — proptest                        │  random f, real bind
  │     monad laws on the actual Rust bind                         │  → "code obeys rules on samples"
  ├───────────────────────────────────────────────────────────────┤
  │ L3  RUST BEHAVIOR (concrete) — Kani (bounded model checking)   │  first‑order, bounded‑exhaustive
  │     evaluate: no‑panic, W‑invariant, error short‑circuit, …    │  → "implementation behaves"
  └───────────────────────────────────────────────────────────────┘
```

### L1 — Lean (the mathematical truth) — **tractable, high value**
Verify, against a Lean model of the cleaned `M`:
- `LawfulMonad M` (left/right identity, associativity). If `M` is the transformer stack, the laws are
  near‑inherited (`StateT`/`ExceptT` are lawful in core/Batteries; `Writer` is a few lines). **[holds
  under P1,P2]**
- W is preserved by every generator (or made true‑by‑construction via the `Either E (Maybe T)`
  encoding — nothing to prove). **[holds]**
- The effect equations (state laws, writer monoid laws, `raise` left‑zero). **[holds]**
- Kleisli category laws for `>=>`; the singleton as a generated morphism. (`CategoryTheory.Monad`,
  `Kleisli` exist in Mathlib.) **[holds under P1,P2]**
- Collection: value is permutation‑invariant — a commutative‑monoid fold over a `Multiset`
  (Mathlib `CommMonoid`/`Multiset.fold`). The scoped #1 theorem. **[holds; carrier `V` to be named, #5]**
- **Parity** (target theorem): the free monad of the effect theory ≅ `M`. Routine assembly of
  standard algebraic‑effect results for the control‑free core; **must be written out**. **[open —
  proof not yet written]**

**Not in Lean now (open math, not a Lean limitation):** the graph dataflow PROP with copy/join (needs
the join *decided* first, #2) and `RelayTo` handlers (thin Mathlib support). Lean is the forcing
function: a claim that cannot be stated in Lean is not ready for the monograph.

### L2 — proptest (the practical bridge) — **exists today**
`CausalMonadProptest` already checks the laws on the real `bind` with randomly generated `f`. This is
the only layer that handles *arbitrary* `f` against the real code (random, not exhaustive). Keep it;
extend to the effect equations and the singleton pipeline. **[holds empirically]**

### L3 — Kani (bounded model checking) — **first‑order only**
Kani **cannot** check the monad laws — they quantify over functions, and bounded model checkers
cannot quantify over `f`. But the **concrete `Causaloid::evaluate` uses a fixed, finite set of
continuations** (`log_input`, `execute_causal_logic`, `log_output`), so its properties are
first‑order and Kani‑able on bounded inputs:
- `evaluate` never panics;
- output is W‑well‑formed;
- an errored input yields an errored output with preserved logs (short‑circuit);
- log is monotone (output log ⊇ input log).
Bound the log length and fix concrete `T`. **[holds, bounded]**

### L4 — deductive Rust↔Lean extraction — **non‑goal**
Extracting the actual Rust into a proof assistant and proving the laws about the *extracted* code
would be the strongest possible claim, but it is **out of scope for this program**: the ladder ends at
L1+L2+L3. The honest claim is that the laws are machine‑checked in Lean and the implementation is
pinned to them by property testing and bounded model checking — not that the shipped Rust is proved
correct.

> No tool converts a Lean *proof* into a Kani test (or any Rust proof): proofs are not portable across
> logics/objects. What is shared is the **property statement**, transcribed once; each layer is an
> independent witness.

---

## 4. CI integration

- **Lean** lives in a **separate `lake` project** (e.g. `lean/CausalAlgebra/`), *not* inside the Rust
  crates (different toolchain). CI: `lake exe cache get` + `lake build`; a broken law fails the build.
  This lifts the project's "compile‑time law enforcement" ethos up to the proofs.
- **proptest** runs in the existing Rust test suite (already CI‑gated).
- **Kani** runs as a separate CI job (`cargo kani`) over the `evaluate` harnesses; bounded, minutes.

---

## 5. Work plan (dependency‑ordered)

| # | Item | Layer | Status |
|---|---|---|---|
| 1 | **Enforce W (P2)** — `Either E (Maybe T)` encoding / smart constructor; remove the representable‑invalid state | Rust | ✅ **done** (`openspec/changes/enforce-w-invariant`: carrier is `outcome: Result<EffectValue<T>, Error>`, all fields private; right‑identity/associativity/left‑zero now proved in `Core/CausalMonad.lean` + Kani + witnesses) |
| 2 | **Remove `RelayTo`/`Map` (P1)** from the core; isolate as handler/future work | Rust | gating |
| 3 | Decide **context = Reader or State**; enforce it | Rust + math | open |
| 4 | Lean model of cleaned `M`; `LawfulMonad` instance | L1 | not started |
| 5 | Lean: effect equations + W + Kleisli + singleton | L1 | not started |
| 6 | Lean: collection commutative‑monoid order‑independence; name verdict carrier `V` (#5) | L1 | not started |
| 7 | Write out the **parity proof** (free monad of theory ≅ `M`) | L1 | open |
| 8 | proptest: extend to effect equations + singleton pipeline | L2 | partial (monad laws exist) |
| 9 | Kani harnesses for `evaluate` (no‑panic, W, short‑circuit, log‑monotone) | L3 | not started |
| 10 | **Graph join `∇_G`** decision (#2) → then graph algebra + its Lean proof | math → L1 | open (blocked) |
| 11 | `RelayTo` handler semantics (control layer) | math | open (future work) |

Items 1–9 deliver a **sound, Lean‑checked, Rust‑linked Causal Algebra for singleton + collection**.
Items 10–11 are the frontier (graph join + control), gated on decided math, and should be presented
in the monograph as precisely‑posed open problems, not as solved.

---

## 6. Scope of the claim (for the monograph)

**Defensible once 1–8 land:** "DeepCausality's core is a Causal Algebra — an algebraic theory over an
arity‑5 effect monad, with Kleisli composition as the propagation operator and a dedicated effect
system as generators — whose laws are machine‑checked in Lean (CI‑enforced) and whose Rust
implementation is checked against them by property testing and bounded model checking."

**Never claim:** that Kani proves the monad laws (it cannot — they are higher‑order); that the graph
form has an algebra before the join is defined; that the implementation is *proved* correct — it is
*property‑tested and bounded‑model‑checked*, which is a weaker, honest, still‑strong statement.

---

## 7. Related work & positioning

A literature pass has been done. Findings, and where they leave the contribution.

### 7.1 The two names

- **"Causal Monad"** does **not** appear in the peer‑reviewed literature; the only occurrences trace
  back to DeepCausality's own materials. The name is free — but absence of the name is **not** evidence
  the idea is novel; do not lean on it.
- **"Causal Algebra" / an algebraic theory of causation** *does* exist, in spirit, under other names.
  DeepCausality is **not** the first to propose that causation has an algebraic/categorical theory. The
  serious prior art must be distinguished, not ignored.

### 7.2 The dominant frame — Markov categories — and the reviewer question

The field models causality in **Markov categories** (Fritz 2020): symmetric monoidal categories with
a **copy/discard comonoid** structure; interventions are **functorial string‑diagram surgery**
(Jacobs–Kissinger–Zanasi 2019/2021); conditioning via disintegration (Cho–Jacobs 2019); do‑calculus
over **free** Markov categories (Yin–Zhang 2022). These are deliberately **monad‑agnostic and
structural**. The "algebra of causation" itself is established: **Fong 2013** (a causal theory = the
free symmetric monoidal / Markov category on a DAG), **Patterson 2020** (functorial semantics for
statistical models — the foundational source), **Mahadevan 2025** (causal models as cPROPs, described
*explicitly* as "a Lawvere algebraic theory").

**The mature, rigorous versions of the free‑Markov‑category‑from‑a‑DAG → functorial‑semantics →
do‑calculus construction** (the live comparators to position against — *not* the BSc thesis below):

- **Yin & Zhang 2022**, *"Markov categories, causal theories, and the do‑calculus"* (arXiv:2204.04821) —
  the syntactic do‑calculus over a free Markov category generated by a DAG.
- **Lorenz & Tull 2023** (Quantinuum), *"Causal models in string diagrams"* (arXiv:2304.07638, ~105pp) —
  the full diagrammatic treatment of interventions / conditioning / counterfactuals / identifiability;
  the **mainstream framework**, continued into 2026 (causal abstraction, XAI). Belongs in must‑cite.
- **Jacobs 2025**, first‑party causal (arXiv:2512.00209) + the living book *Structured Probabilistic
  Reasoning*; **Mahadevan** (prolific, idiosyncratic topos/K‑theory framing); **Fritz** (foundations).

**Gao 2022** (BSc thesis, *Functorial Causal Models*) is an early, **near‑uncited** independent sketch
of that same construction (one genuine citation: Fritz–Klingler, *JMLR* 2023); its one original step
ends on an unproven conjecture, and its lineage (the Brown→Topos chain — Lynch, Patterson) **abandoned
the causal thread**. Treat it as a *historical sketch and a foil*, not a live competitor. Its §7.2 — the
admission (following Lewis: non‑transitivity, late pre‑emption) that SCM/FCM causation **cannot ground
moral attribution** — is the one citable item (see §7.4). Cite Patterson/Fong for foundations.

**Consequence.** An ACT‑literate reviewer's first question is **"why a bespoke effect monad instead of
a Markov category?"** A prepared answer plus an explicit mapping is the single highest‑value defensive
move in the monograph (see §7.4).

### 7.3 Three things the literature *gives* us

1. **The graph join is the comonoid copy/discard — assumption #2's abstraction already has a name.**
   The graph form's "copy + reconverge" is exactly the copy/discard comonoid that *defines* Markov
   categories. Do not invent a structure for the graph: adopt the established one (free Markov category
   / cPROP); the join `∇_G` is then constrained by the comonoid laws rather than ad hoc. This is the
   most useful single import.
2. **`RelayTo` / interventions / counterfactuals = effect handlers — P1's decision is validated.**
   There is a line — "Modular Probabilistic Models via Algebraic Effects," "Effect Handlers for
   Programmable Inference," and **ChiRho** (causal PP where counterfactuals *are* handlers over Pyro) —
   that puts interventions in a handler layer. This both supports excising `RelayTo` from the algebraic
   core (P1) and supplies the citation for the handler layer (item 12).
3. **Lean precedent exists for the monad, not for causality — the verified‑causality angle is a real
   gap.** The **Giry (probability) monad is in Mathlib**; **CertRL** formalizes it in Coq with Kleisli
   composition; Hölzl did Giry in Isabelle. But no machine‑checked formalization of *categorical
   causality* (causal Markov categories, the categorical do‑calculus, JKZ surgery) was found. So a
   Lean‑checked causal *effect* monad is plausibly first, and Mathlib/CertRL are the methodological
   templates for L1.

### 7.4 Positioning (the answer to "why not a Markov category?")

DeepCausality is **not** competing with Markov categories on probabilistic semantics — it provides the
**effectful computational substrate they do not**. Markov categories model copy/discard and conditional
independence; they say nothing about state‑threading, error short‑circuit, audit logs, sequencing, or
handlers — the arity‑5 channels that make a *dynamic, executable* causal engine. The reconciliation,
and the bridge worth stating as a theorem:

> Instantiate the value channel with a probability monad (DeepCausality already has `Uncertain<T>`).
> Then the Kleisli category of the causal effect monad, restricted to that channel, **maps to a Markov
> category / Stoch**.

So position DeepCausality as **extending, not contradicting** the Markov‑category line: the effect
monad is the computational wrapper; the probabilistic core is (or maps to) a Markov category. A
comparison/translation to Stoch, JKZ surgery, and Yin–Zhang do‑calculus converts the reviewer's
objection into a contribution. Gao §7.2 supplies a second, independent hook: that lineage *concedes*
(following Lewis: non‑transitivity, late pre‑emption) that its notion of causation **cannot ground
moral attribution** — precisely the gap the monograph's epistemology/teleology → programmable‑ethics
thread claims to address. A clean, citable foil — state it without overclaiming.

**Methodological precedent & the executable‑ACT comparator (Patterson / AlgebraicJulia).** The
"algebra of causality" claim has an established lineage that the monograph should *cite, not
rediscover*: **Lawvere 1963** (functorial semantics of algebraic theories) → **Patterson 2020** (*The
Algebra and Machine Representation of Statistical Models* — a statistical model is a *functor* of a
statistical *theory*, the exact move you make for a causal effect monad). Frame the contribution as
*instantiating that method for an effect monad*; this inoculates against the obvious ACT‑reviewer
objection. Two further Patterson findings: (a) **double‑functorial semantics** (Lambert–Patterson,
*Cartesian Double Theories*, Adv. Math. 2024, arXiv:2310.05384; "Representing Knowledge and Querying
Data using Double‑Functorial Semantics," arXiv:2403.19884) is the current state of the art for *models
and morphisms between them* — the right citation if/when the monograph treats inter‑model relations
(Gao's "requirement #3"), not a dependency. (b) The **AlgebraicJulia** ecosystem (GATlab,
arXiv:2404.04837; Catlab; ACSets; Decapodes, arXiv:2401.17432) is the closest *executable* applied
category theory and the sharpest comparator for the "industrial Rust" claim: it is Julia
*specification* tooling with laws by convention and interpreted execution; DeepCausality is compiled
Rust with **laws as compile‑time assertions** (`deep_causality_num`) and a production engine that
*runs*. The contrast strengthens, not threatens, the differentiator. (Decapodes — PDEs as executable
string diagrams — is also the nearest prior art to the physics/CFD layer; be ready to contrast.)

### 7.5 Honest novelty

- **Not novel:** causality as an algebraic theory in a monoidal/Lawvere sense (Fong, Patterson,
  Mahadevan); interventions/counterfactuals categorically (JKZ, ChiRho); the probability monad + its
  Kleisli category (Stoch); formalizing probability monads in a proof assistant (Mathlib, CertRL).
- **Plausibly novel (defensible):** (a) the **bespoke arity‑5 *effect* monad** (value/state/context/
  error/log) with Kleisli `>=>` as the single propagation axiom — the field uses monad‑agnostic Markov
  categories or the generic Giry monad, not a multi‑channel *causal effect* monad foregrounding
  computation/handlers; (b) a **machine‑checked (Lean) causal‑effect monad** with interventions — the
  gap in §7.3; (c) the **production Rust implementation** unifying singleton/collection/graph under one
  monad.

The novelty is the **engineered, verified, effectful stack** — *not* "the first algebra of causality."
Tighten the monograph's claim to that.

### 7.6 Must‑cite

Verified venues/IDs below (cross‑checked against arXiv/DBLP/journal sites). **Must‑cite (foundational
anchors):**
- **Fritz 2020**, *A synthetic approach to Markov kernels…* — Adv. Math. 370:107239 (arXiv:1908.07021).
  The Markov‑category foundation.
- **Jacobs–Kissinger–Zanasi**, *Causal inference by string diagram surgery* — FoSSaCS 2019
  (DOI 10.1007/978‑3‑030‑17127‑8_18); journal MSCS **31(5):553–574, 2021** (DOI
  10.1017/S096012952100027X); arXiv:1811.08338. **Canonical** "intervention = endofunctor cutting
  wires."
- **Lorenz–Tull 2023** (Quantinuum), *Causal models in string diagrams* — arXiv:2304.07638, ~105pp,
  **preprint** (extended abstract ACT 2023). The **mainstream** diagrammatic causal calculus
  (interventions/conditioning/counterfactuals/identifiability); the SOTA the effect‑monad departs from.
- **Yin–Zhang**, *Markov categories, causal theories, and the do‑calculus* — **Studies in Logic
  14(6):1–24, 2021** (arXiv:2204.04821, 2022). Canonical "DAG ⟹ free Markov category + syntactic
  do‑calculus." Cite as an **adequacy / "as strong as full do‑calculus"** result — *not* a
  soundness+completeness metatheorem (no paper in this corpus states one).
- **Cho–Jacobs**, *Disintegration and Bayesian inversion via string diagrams* — MSCS **29:938–971,
  2019** (arXiv:1709.00322). The categorical‑probability foundation beneath the line. (Title is *not*
  "The Mathematics of Changing One's Mind" — a different work; do not conflate.)
- **Lawvere 1963** (functorial semantics of algebraic theories) → **Patterson 2020**, *The Algebra and
  Machine Representation of Statistical Models* (arXiv:2006.08945). The methodological lineage for
  "rendering X as an algebra"; cite as the precedent the Causal Algebra instantiates.

**Could‑cite (machinery / comparators / framing):**
- **Yin 2022**, *A graphical construction of free Markov categories* (arXiv:2204.04920) — explicit
  functorial free construction; **Fritz–Liang 2022**, *Free gs‑monoidal and free Markov categories*
  (arXiv:2204.02284) — the algebraic comparator.
- **Fong 2013**, *Causal theories* (arXiv:1301.6201) — the original free‑SMC‑from‑a‑DAG.
- **Mahadevan 2025**, cPROPs / Lawvere framing (arXiv:2508.08295) — distinguish from.
- **Gao 2022**, *Functorial Causal Models* (Brown BSc) — historical sketch + §7.2 moral‑inadequacy foil
  (near‑uncited; one citation, Fritz–Klingler *JMLR* 2023, arXiv:2207.05740).
- **Jacobs–Széles–Stein 2025**, *Compositional inference for Bayesian networks and causality*
  (arXiv:2512.00209, MFPS XLI/ENTICS 5) + Jacobs, *Structured Probabilistic Reasoning* (online **draft**,
  2025 — not a published book; cite with access date).
- **Lorenz–Tull 2026**, *Causal and compositional abstraction* (arXiv:2602.16612) — abstraction/
  multi‑level angle; **its §8 names the open bridge to programming‑language/type semantics = the
  effect‑monad niche** (headline positioning hook, see §7.7). **Tull et al. 2024**, *Towards
  compositional interpretability for XAI* (arXiv:2406.17583) — peripheral.
- **Robin Lorenz — quantum‑causal cluster** (cite as the categorical counterpart to the quantum‑native
  claim; the convergence is external validation of the causaloid, see §7.7): Barrett–Lorenz–Oreshkov,
  *Quantum Causal Models* (arXiv:1906.10726, 2019); *Cyclic Quantum Causal Models* (Nat. Commun. 2021,
  arXiv:2002.12157); Lorenz–Barrett, *Causal and compositional structure of unitary transformations*
  (Quantum 2021, arXiv:2001.07774); Lorenz, PhD *Quantum Causal Structure* (Oxford 2020).
- **Effect handlers for interventions:** ChiRho + arXiv:2203.04608, arXiv:2303.01328.
- **Verified‑monad precedent:** Mathlib Giry monad / Markov kernels (Degenne, arXiv:2510.04070 — notes
  Markov *categories* are not yet in Mathlib); CertRL (arXiv:2009.11403).
- **Executable‑ACT comparator:** Patterson et al. AlgebraicJulia — GATlab (arXiv:2404.04837), Decapodes
  (arXiv:2401.17432); double‑functorial semantics (arXiv:2310.05384, arXiv:2403.19884).

**Two confirmations from the deep dive (both differentiators hold):** (i) **no work in any of these
lines is tied to a proof assistant** (Lean/Coq/Agda) — the verified‑categorical‑causality niche is
empty; (ii) **no work uses a non‑probabilistic effect monad** (state/error/writer/log) as the causal
carrier — every carrier is a probability structure (D / Giry) inside a Markov/cd category. The
effect‑monad framing is a genuine gap, not a re‑skin.

**Caveats (do not overread):** the phrase "causal monad" not appearing ≠ the idea is new; the Markov
category / Giry‑monad work is not *causality* machine‑checked, but verify before claiming first; an
unpublished Lean formalization of categorical causality cannot be ruled out. The novelty claim should
remain the *engineered, verified stack*, not merely "an algebra of causality."

### 7.7 The Robin Lorenz axis — quantum causality independently confirms the foundation

The closest comparator resolves to **one person: Robin Lorenz** (Quantinuum; PhD *Quantum Causal
Structure*, Oxford 2020, under Jonathan Barrett). His record is the exact bridge DeepCausality straddles
— traversed from the *physics* side:

- **Quantum causal foundations:** Barrett–Lorenz–Oreshkov, *Quantum Causal Models* (arXiv:1906.10726,
  2019); *Cyclic Quantum Causal Models* (Nat. Commun. 2021, arXiv:2002.12157); Lorenz–Barrett, *Causal
  and compositional structure of unitary transformations* (Quantum 2021, arXiv:2001.07774).
- **Categorical causality:** Lorenz–Tull, *Causal models in string diagrams* (arXiv:2304.07638, 2023).
- **Causal abstraction:** Lorenz–Tull, *Causal and compositional abstraction* (arXiv:2602.16612, 2026).

**Why this matters, two ways.**

1. **It confirms the foundation — and the confirmation runs the right direction.** Lorenz earned a
   doctorate establishing the categorical/quantum structure of causality. DeepCausality arrives in the
   same territory **not by targeting quantum, but as a byproduct of adopting Hardy's causaloid**: folding
   cause and effect into one entity removes temporal order; removing temporal order makes the framework
   spacetime‑agnostic; spacetime‑agnosticism makes it general‑relativistic‑ and quantum‑native *as a
   derived consequence of the single axiom*, never a design goal. That an independent quantum‑causal‑
   structure line (Lorenz, Barrett, Oreshkov) converges on the same categorical shape is **external
   validation that the causaloid was the right primitive.** Cite the Barrett–Lorenz–Oreshkov cluster as
   the rigorous categorical counterpart to the intro's quantum‑native claim — and state the convergence
   honestly: the EPP did not set out to formalize quantum causality; it *inherited* it from Hardy.

2. **Lorenz–Tull 2026, §8, names DeepCausality's exact niche as open future work.** Verbatim from the
   paper's discussion: it would be interesting "to explore connections between (causal) abstractions and
   **categorical notions of abstraction from computer science (such as those between programming languages
   or types)** … often also formalised in terms of functors and natural transformations." That is the SOTA
   authors flagging, as *unfinished*, the bridge from categorical causal models to **programming‑language /
   effect semantics**. The causal monad *is* that bridge (Moggi–Wadler effect monad ⟷ Kleisli causal
   composition), instantiated and executable. Citing §8 converts the reviewer's "why an effect monad?"
   into "the field's leaders named this gap; the EPP fills it — and runs it."

**Re‑confirmed at the primary‑source level** (full 2026 paper, not a summary): the semantics category is
still **FStoch** (Def. 2 Markov category; Ex. 3) or **QC** (Ex. 57 quantum channels) — a probability/
quantum structure, **never** a state/error/writer/log effect monad; the work is pen‑and‑paper (no proof
assistant); a model is a functor `S → C` (denotational — nothing runs). All four differentiators
(effect‑monad carrier · machine‑checkable · executable/compiled · quantum‑native‑by‑derivation) hold
against the most advanced paper in the field.

**Resonances to exploit** (rigorous prior art for items flagged open): their *open causal model* +
*opening* operation (delete a mechanism = do‑intervention, §3.2.3) is the diagrammatic do‑operator —
the reference for the `RelayTo`/intervention story; their *abstraction as natural transformation* is
Gao's "requirement #3" (inter‑model morphisms) done properly; their *component‑/mechanism‑level
abstraction* (§6.1, "unbox a high‑level component as a diagram of low‑level ones") is conceptually the
**recursive causaloid** ("a graph of causaloids is again a causaloid") — note the parallel.
