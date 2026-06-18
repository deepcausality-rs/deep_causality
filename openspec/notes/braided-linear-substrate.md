# Braided-Linear Substrate — Updated Feasibility Assessment

**Date:** 2026-06-18
**Status:** Analysis note. Nothing built. Feeds the Causal-Arrow generalization
(`openspec/changes/archive/arrow-notes/causal-arrow-generalization.md`).
**One-line thesis:** Three "new-math" capabilities that first looked independent and
mostly blocked turn out to share a single missing substrate — *linear maps over a
commutative ring with a chosen R-matrix* — and the repo already supplies most of it.
Closing the gap is moderate engineering; the unreduced risk is the mathematics and,
above all, the **semantic** justification that any of it means something for causality.

---

## 1. The capabilities (corrected during review)

The original pitch was a Hecke-algebra "crossroads" unifying topology, algebra, and
combinatorics, mapped onto DeepCausality's monadic stack. Stripped of romance, three
candidate capabilities survived scrutiny:

1. **Causal-history trace invariant** — a Markov-trace / Jones-polynomial analogue that
   collapses a whole causal history into an invariant, so two histories that are "the
   same up to deformation" get the same fingerprint. *Highest value:* it gives
   counterfactual and RCA work a question they currently cannot pose
   ("is this the same causal process, up to deformation?").

2. **Non-commuting composition + `q`-dial** — order-sensitive effect composition governed
   by a braiding, with one parameter `q` interpolating between commuting (order-indifferent)
   and non-commuting (order-sensitive) regimes.

3. **`SO(3)` harmonic (deviatoric/trace-free) tensor decomposition** — *downgraded.*
   Originally pitched as Schur–Weyl; corrected: CFD wants the **orthogonal-group**
   decomposition, not the symmetric-group one. Schur–Weyl gives `GL × Sₙ` (permutation
   symmetry); physics wants `SO(3)` irreps (harmonic/deviatoric). They coincide only at
   rank 2 by low-rank accident. In CFD, rank-2 is ~95% of tensors you actually decompose,
   and the physically central deviatoric split reduces to `CausalTensor::trace` + arithmetic.
   So #3 is feasible *because it is nearly trivial* — and barely "new."

## 2. Key finding — one substrate, not three problems

A first-pass feasibility read concluded #1 and #2 were blocked by deliberate design choices:

- The `Arrow` (`deep_causality_haft/src/arrow/mod.rs:102`) is **cartesian-symmetric by
  design**: morphisms are Rust closures (`run(&self, In) -> Out`), the product is the
  tuple `(A,B)`, and `fanout` requires `In: Clone` (a diagonal). A non-trivial braiding
  on a cartesian category is either redundant with the symmetry or breaks its laws, and
  the closure carrier has nowhere to *store* an R-matrix. This is intentional — the
  `Arrow` is the **static** fragment (Hughes' arrow-vs-monad = static structure vs dynamic).

- `PropagatingEffect` (`deep_causality_core/.../propagating_effect/mod.rs:17`) **collapses**
  the bind-chain: `bind` keeps the final `EffectValue<T>` and a flattened writer log
  (`EffectLog` + `LogAppend`), but not the composition *term*. A topological invariant must
  see the braid; the braid is not reified. (The `EffectValue` enum *does* reify branching —
  `RelayTo`/`Map` — but not sequence.)

**The correction:** both blockers dissolve on the *same* parallel layer. Represent
morphisms as **matrices** (data) instead of **closures** (opaque):

- #2's braiding becomes a fixed invertible **R-matrix** — a `CausalTensor` you can store.
- #1's history becomes an explicit **braid word** = a product of R-matrices, and the
  Markov trace is `partial_trace` of that product. Reified by construction; no surgery
  on the effect monad.

So #1 and #2 are not two problems — they are two readings of one linear, non-cartesian,
braided-monoidal carrier. #3 rides the same tensor machinery but does not even need the
braiding.

## 3. What the substrate is, mapped to existing crates

Target: a braided monoidal category `(𝒞, ⊗, I, R)` — objects = finite-dim modules
(a dimension), morphisms = linear maps (matrices over a ring `R`), `⊗` = Kronecker product,
braiding = a fixed R-matrix satisfying Yang–Baxter.

| Ingredient | Existing piece | Status |
|---|---|---|
| Coefficient ring `R` | `deep_causality_num::CommutativeRing` / `Field` (`algebra/ring_commutative.rs:30`, `algebra/field.rs:38`) | present, abstract (not float-locked) |
| Objects (modules) | `Module<R: Ring>` (`algebra/module.rs:33`); dimension as metadata | present |
| Morphisms = linear maps | rank-2 `CausalTensor<R>` (dense) / `CsrMatrix<R>` (sparse) | present |
| Composition `∘` | matmul via `ein_sum("ij,jk->ik")` / `contract` | present |
| Monoidal product `⊗` (Kronecker) | `ein_sum` + reshape | present; thin `kron` wrapper |
| Trace / partial trace | `CausalTensor::trace` (`tensor_ein_sum/ein_sum_impl.rs:331`) | present |
| Algebra of morphisms (braid group / **Hecke `Hₙ(q)`**) | `Algebra<R: Ring>` (`algebra/algebra_base.rs:48`) | present **trait**; Hecke is an *instance* |
| Categorical interface (joins the HKT stack) | `Morphism<P: HKT2Unbound>` (`morphism/mod.rs:26`), `Profunctor`, `Bifunctor` | present scaffolding |
| Braiding **R-matrix as data** | — (a fixed `CausalTensor<R>`) | net-new datum, but it is just a matrix |
| `q`-ring / invariant codomain `ℤ[q,q⁻¹]` | — | **net-new** (the one real algebraic gap) |
| `BraidedMonoidal` trait (⊗, unit, braid + YBE) | — | net-new (the "substrate" trait) |

**Precise relation to the `Arrow`:** the new carrier is *the `Arrow` minus `fanout`, plus a
braid*. The missing diagonal is the **defining** non-cartesian property (you cannot clone a
generic module element linearly), not a regression. Bonus: composing matrices returns a
concrete owned `CausalTensor` — so the linear carrier composes *more* cleanly under
`unsafe_code = "forbid"` / static dispatch than the closure `Arrow`, which produced
unnameable composed-closure types.

Alignment worth noting: **`Hₙ(q)` is literally an `Algebra<LaurentPolynomial<ℤ>>`** —
generators `Tᵢ`, relation `(Tᵢ − q)(Tᵢ + 1) = 0`. The abstract `Algebra<R>` trait already
exists; you instantiate it, you do not invent it.

## 4. Net-new surface — bounded

In dependency order:

1. **`LaurentPolynomial<R>` scalar** under `deep_causality_num`, implementing
   `CommutativeRing` (the tower is abstract enough to absorb it). The `q`-ring and the
   codomain of any knot/trace invariant. For the Kauffman bracket / Jones you need only the
   **ring**, not the fraction field — do not over-build it to `Field`.
2. **`BraidedMonoidal` trait** (a monoidal-category supertrait + `braid` + a Yang–Baxter law
   in `tests`), expressed over the `Morphism<P: HKT2Unbound>` witness so it composes with
   `Functor`/`Monad`.
3. **`LinearMap<R>` carrier** — newtype over rank-2 `CausalTensor<R>` implementing `Morphism`
   and `BraidedMonoidal`; deliberately **not** implementing the cartesian diagonal.
4. **Thin ops** — `kron` and `partial_trace` / Markov closure (both `ein_sum` one-liners).

## 5. Updated risk-reward — two axes, kept separate

The investigation moved **feasibility** sharply up. It did **not** move **validated value**.
These are different axes and conflating them is the trap.

| Capability | Feasibility (post-finding) | Validated value | Net |
|---|---|---|---|
| #1 trace invariant | now plausible on the substrate | **unproven** — no demonstration it distinguishes causal histories usefully | high-risk, high-upside |
| #2 braided + `q` | now plausible on the substrate | **unproven** — the causal reading of `q` is a thesis, not a result | high-risk, high-upside |
| #3 `SO(3)` decomposition | high (nearly trivial) | modest, concrete (CFD/physics) | low-risk, low-upside |

What changed: #1 and #2 are no longer *blocked by architecture*. What did **not** change:
nobody has shown they produce a useful answer to a real causal question. Accessibility is
not payoff.

## 6. The unreduced risk

"Close the gap and the math is sorted out" hides the hard part. The engineering (Section 4)
is moderate and well-scoped. The risk lives elsewhere:

- **Theory:** choosing R-matrices, proving the chosen representation yields a faithful and
  computable invariant. Standard but non-trivial.
- **Semantics (the real risk):** justifying that `q` *is* an order-commutativity dial for
  causation, and that the trace invariant tracks something a practitioner cares about. This
  is a modelling claim about causality, not a theorem about braids. It can be wrong even if
  every line of Rust is correct. This is where the project should demand a falsification
  test before committing real effort.

## 7. Recommended next step — a falsifiable spike, not a build

Before any feature commitment, run one bounded spike whose **purpose is to fail fast**:

1. Implement `LaurentPolynomial<ℤ>` as `CommutativeRing` (item 4.1 only).
2. Implement a minimal `LinearMap` + `kron` + a YBE property test (4.3, 4.4 partial).
3. **Falsification target:** reproduce a *known* invariant (Kauffman bracket of the trefoil,
   landing in `ℤ[q,q⁻¹]`) from an explicit braid word. If the substrate cannot reproduce a
   textbook knot invariant, the causal ambitions are moot.
4. **Only if (3) passes:** write the semantic-justification note for #1/#2 — the argument
   that the invariant means something causal — and treat *that* note, not the code, as the
   real go/no-go gate.

The spike is days, not weeks, and it converts the open question from "is it buildable"
(answer: largely yes) to "does it mean anything" (answer: unknown, and that is what to test).

---

### Related
- `openspec/changes/archive/arrow-notes/causal-arrow-generalization.md` — the static `Arrow`
  fragment; this substrate is the dynamic/linear counterpart.
- `openspec/notes/composite_rca.md` — the consumer of #1 if it validates.
- `docs/UNIFORM_MATH.md` — the "many objects, one interface" framing this extends with a
  "one object, many faces" core.
