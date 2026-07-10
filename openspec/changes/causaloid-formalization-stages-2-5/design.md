## Context

Stage 1 of `openspec/notes/causal-algebra/causaloid-formalization-roadmap.md` landed 2026-07-09:
the carrier stack `Except E (Free CausalCommand (Maybe V))` is proven lawful
(`core.causal_effect.transformer_stack`), `fold` is the unique handler
(`core.causal_effect.fold_universal`), relay termination is fuel-bounded
(`core.causal_effect.relay_termination`, `MAX_RELAY_ROUNDS` in both engine loops), and the trait
surface is sealed at the three `CausaloidType` forms (#11a DECIDED, `traits/causable/sealed.rs`).
This change executes Stages 2–5 against that base. Standing constraints: `unsafe_code = "forbid"`,
no `dyn`, no crate-defined macros, clippy `-D warnings`, bare-`lean` self-contained proofs with
textbook citations and deviation notes, THEOREM_MAP traceability (lowercase `[a-z0-9_.]+` ids), the
Bazel workspace gate, and the established per-stage loop: Lean → Rust witness → THEOREM_MAP row →
traceability → `bazel test //...`.

## Goals / Non-Goals

**Goals:**
- Prove the organizing theorem's premises and conclusion: `Causaloid ≅ μX.F(X)` (Stage 2), the
  carrier closure (Stage 3), the schedule-invariant graph algebra (Stage 4), and `evaluate` as the
  unique catamorphism into `Kleisli` (Stage 5).
- Add the `⊕` choice generator across the haft term language and interpreters (Stage 2b) — the
  prerequisite for causally faithful quantum reification and classical case-splitting.
- Land the two Rust refactors the theorems require: per-edge Λ slots and the `O: Verdict`
  collection bound; rewrite the graph fold behind the #10 characterization corpus.
- Extend the witness mirror and CI gate to the main `deep_causality` crate.

**Non-Goals:**
- `core.causaloid.hardy_correspondence` (the ⊗^Λ reconstruction) — open publication target, blocks
  neither downstream crate.
- Stages 6–7: the extensibility contract, `deep_causality_do_calculus`, `deep_causality_quantum` —
  a dedicated follow-up change set.
- Assumption #4 (generation/atom registry) and full rig-category coherence for `⊗`/`⊕` (only the
  equations used are proved).
- `quantum.partial_trace_preservation` and everything else in roadmap §8's open list.

## Decisions

**D1 — The fixpoint is a Lean inductive; well-foundedness comes from inductive semantics.**
`Core/Causaloid.lean` models `Causaloid` as an inductive type with three constructors
(`atom`/`coll`/`graph`) whose recursive positions hold sub-causaloids. In Lean, an inductive type
*is* the initial algebra μX.F(X), so well-foundedness (no infinite tree) is definitional — this is
the honest model of the Rust side, where `Arc`-nesting is finite by construction (values are built
bottom-up; the sealed surface admits no other builder). The deviation note records the Rust/Lean
gap: Rust enforces finiteness by construction order, Lean by the inductive datatype. Alternative
considered: an explicit fuel-indexed fixpoint (mirroring Stage 1's `run`) — rejected as noise; the
inductive already carries the μ (not ν) content, and Stage 1 established the fuel pattern where it
matters (relays, which are runtime jumps, not structure).

**D2 — `Bag` is modeled as `List` plus permutation-invariance lemmas.** The `Coll` summand's
order-free multiset is a Lean `List` with the aggregation proven invariant under `Perm` — the same
device the collection order-invariance proofs already use (assumption #1). No quotient types in a
bare-`lean` file.

**D3 — The inversion theorem is a factorization, not a new engine.** `core.causaloid.inversion`
states that the model's `evaluate` equals `wiring ∘ element` where `element` is defined without any
order/position parameter (its arguments are identity-keyed values) and `wiring` never inspects
payloads. This formalizes the Hardy inversion as a property of the existing design; no Rust change
is needed for it beyond the Λ slots.

**D4 — Λ decorations are fn-pointer slots keyed by edge identity, default = identity.** The
hypergraph's edges gain an optional decoration (a plain `fn`-typed transform, consistent with the
crate's existing `CausalFn`-family style — no `dyn`, no closures in stored positions), looked up by
the edge's intrinsic id at join time. `None` means identity, so undecorated graphs are bit-for-bit
unchanged and the corpus (D7) can prove it. Alternative considered: a generic type parameter for
the decoration — rejected: it would infect `Causaloid<I, O, PS, C>`'s already-long signature for a
per-edge concern.

**D5 — `⊕` extends the existing enums; no parallel term language.** `ArrowCore<G>` gains
`Left`/`Right`/`Choice`/`Fanin` variants and `ArrowVal<V>` gains `InL`/`InR` — additive enum
extension, exactly how H3 was built. The typed `ArrowTerm` façade gains `left::<C>`, `right::<C>`,
`choice`, `fanin` typed over `Either<_, _>`, keeping typed-by-construction/erased-core intact, with
a `compile_fail` doctest for mistyped branch wiring. Both interpreters gain the dispatch arms.
Stage 2b is independent of Stages 2–3 (haft-only) and can land in parallel after Stage 2 starts.

**D6 — The `O: Verdict` bound lands where aggregation is defined, not on the struct.**
The bound tightens the collection-aggregation code path (the `AggregateLogic` evaluation and the
collection constructors/eval methods), not `Causaloid`'s struct definition — singletons and graphs
over non-`Verdict` outputs remain untouched. `Some(k)` thresholds go through the existing `Count`
commutative monoid with the comparison at the aggregation boundary. This is the **BREAKING**
surface: any downstream collection over a non-`Verdict` `O` stops compiling; the migration note
says "implement `Verdict` for your carrier or aggregate to `bool`/`Prob`".

**D7 — The #10 characterization corpus is a hard gate, committed before the fold rewrite.**
The corpus snapshots current outputs (all five channels) for chains, trees, fan-outs, and the
loud-fail diamond. The Stage-4 rewrite must keep chains/trees bit-identical; the diamond's
expectation is updated in the same commit that changes it, with the rationale inline. The relay
mechanism is untouched: `RelayTo` is an effect the engine handles (Stage 1); the join algebra is
orthogonal to it, and `MAX_RELAY_ROUNDS` stays.

**D8 — Single-writer state at joins is checked at freeze, never merged.** Merging state at a
reconvergent join is undefined by ruling (the per-channel policy). The freeze step statically
verifies at most one incoming branch writes state; violation is a freeze error naming the join.
This keeps `core.causaloid.graph_fold_order_invariant`'s state clause trivial (only one writer
exists) instead of requiring a state-merge algebra nobody has semantics for.

**D9 — Uniqueness follows Stage 1's `fold_unique` pattern, lifted to the causaloid.**
`core.causaloid.catamorphism_unique` takes a hypothesis interpreter `h` plus the three
case equations (`h ∘ atom = …`, `h ∘ coll = …`, `h ∘ graph = …`) and proves `h = evaluate` by
induction on the inductive from D1 — the same shape as `fold_unique`, at the next level up. The
carrier (semantic algebra) is an explicit fixed parameter; the header states that cross-carrier
uniqueness is not claimed (assumption #6's correct scoping).

**D10 — The bridge extension is a grep-list edit plus a new witness mirror.** The CI gate's
crate list in `.github/workflows/formalization.yml` gains `deep_causality`;
`deep_causality/tests/formalization_lean/` mirrors the core convention (one `<mechanism>_tests.rs`
per new Core Lean file, `mod.rs` + `BUILD.bazel` registration, one `#[test]` per id). Alternative
considered: putting causaloid witnesses in `deep_causality_core` — rejected: the witnesses must
exercise the real `Causaloid`, which lives in the main crate.

## Risks / Trade-offs

- **[Stage 4 rewrites the reasoning engine — the highest-blast-radius change in the program]** →
  the #10 corpus is committed and green *before* any engine edit (D7); chains/trees bit-identical
  is a hard acceptance criterion; the diamond change is the single documented behavior change;
  `bazel test //...` green at every stage boundary.
- **[The `O: Verdict` bound breaks downstream collections]** → survey all in-repo usages
  (examples, tests, benches) in the same stage; migration note in the changelog; the two shipped
  carriers (`bool`, `Prob`) cover every in-repo case found so far.
- **[Lean model drift: the inductive model may diverge from the Rust engine's actual fold]** →
  every theorem gets a Rust witness that runs the *real* engine (not a test-local model), the
  Stage-1 discipline; the arrow-fragment theorem (`evaluate = interpret_kleisli`) pins the two
  ends together.
- **[Per-edge Λ lookup adds a hash lookup per join input]** → joins are already the expensive
  path; `None`-fast-path for undecorated edges; corpus (D7) doubles as the perf-regression canary.
- **[`ArrowVal` gains `InL`/`InR` — existing interpreters must be total over the enlarged value
  type]** → the compiler enforces exhaustive matches; every existing `ArrowVal` match is revisited
  in Stage 2b with explicit arms, no wildcard fallthroughs.

## Migration Plan

Stages land sequentially on the dependency spine (2 → 2b ∥ 3 → 4 → 5), each closing with the
established loop (Lean bare-`lean` + `lake build`, witnesses, THEOREM_MAP, traceability, clippy
`-D warnings`, `bazel test //...`) and a prepared commit message at the stage boundary (no commits
without the user). Stage 2b can proceed in parallel with Stage 3 once Stage 2 lands. Breaking
changes (the `Verdict` bound, the diamond merge) each carry a changelog entry with migration text.
Rollback per stage = revert the stage's commit; no stage leaves the workspace red.

## Open Questions

- Whether the Λ decoration should also be permitted on `Collection` members (per-member Λ before
  the fold) or stay graph-only in this change. Default: graph-only; the collection fold is already
  order-free, and per-member transforms are expressible inside the member causaloids.
