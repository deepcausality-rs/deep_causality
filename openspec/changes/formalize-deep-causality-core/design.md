## Context

`deep_causality_core` sits one tier above `deep_causality_haft` (AGENTS.md dependency graph, Tier 1 →
Tier 0). Haft is fully formalized (21 Lean files, 28 witnessed ids, deviations audit); core is not.
The survey in `openspec/notes/causal-algebra/core-formalization-plan.md` enumerates the base→extension
layering (§0), the 8-file Lean plan (§1), 17 deviations D1–D17 (§2), the Option-B control-channel
decision (§2A), and a per-deviation resolution ledger (§2B). Two Core Lean files already exist:
`CausalMonad.lean` (proved, 5 ids) and `EffectLog.lean` (proved but staged/un-bridged, 4 ids).

The author's controlling decision (2026-07-06): **prove the clean, unconditional laws over corrected
code**, not fragments with negative lemmas. This makes the formalization the *capstone* of the causal-
algebra program — it targets code that two prior corrective changes have already fixed. The bridge
mechanism is fixed by the existing infrastructure: a shared `THEOREM_MAP` id per statement, a bare-
`lean`-checkable proof, an independent Rust witness, and a CI consistency gate
(`.github/workflows/formalization.yml`). This design does not re-invent that; it applies it to core.

## Goals / Non-Goals

**Goals:**
- Bring `deep_causality_core` to full formalization parity with haft: every mechanism proven,
  witnessed, bridged, documented; every deviation terminally dispositioned.
- Prove the *clean* laws (total `fmap`, congruent equality, unconditional right identity, state-
  threading category laws, unblocked `core.causal_monad.lawful`) — no fragment caveats, no negative
  lemma standing in for an unfixed defect.
- Mirror the haft artifacts exactly: `Core/*.lean` tree, `tests/formalization_lean/` witness mirror,
  `THEOREM_MAP` rows, `LEAN_CORE.md`, resolved-deviations audit.

**Non-Goals:**
- No runtime or public-API change in `deep_causality_core` (this change is additive verification).
- Not implementing the two prerequisite corrections (`separate-control-channel`,
  `causal-arrow-state-threading`) — they are separate changes, per the "depend on Option B first"
  decision. This change is *blocked on them* and encodes their post-state.
- Not the Pearl do-operator (deferred to the `deep_causality` hypergraph layer, D8) and not RFC-4180
  CSV quoting (D16 accepted as conditional).
- No Mathlib dependency; no `sorry`; no change to the CI gate's rules.

## Decisions

### D1. Depend on Option B first; prove clean laws (author decision)
The alternative — formalize current code with `{None,Value,ContextualLink}` fragment models and
machine-checked negative lemmas for `RelayTo`/`Map`/panic (plan §1 as literally written) — was
rejected. Rationale: the causal-algebra philosophy is "correct the implementation, don't just document
deviations." Fragment+negative-lemma formalization *documents* D5/D6/D14/D15 permanently; proving clean
laws over corrected code *retires* them. Cost: this change cannot start until both prerequisites land.
Accepted, because it yields a formalization that describes the faithful implementation and unblocks
`core.causal_monad.lawful`, the one claim blocked since the walking-skeleton era.

### D2. Two hard prerequisites, not one
The question posed named the control channel (Option B). But the clean-laws goal also requires the
arrow to thread state, which is a *separate* pending correction (D2-full,
`causal-arrow-state-threading-plan.md`). Both are prerequisites:
- `separate-control-channel` → `EffectValue.lean`, `CausalCommand.lean`, `Consistency.lean`,
  `CausalMonad.lean`'s `lawful`, `CausalFlow.lean`'s corrected `map` law.
- `causal-arrow-state-threading` → `CausalArrow.lean`'s state-threading category laws.
`CausalMonad.lean` (base 5 ids) and `EffectLog.lean` (4 ids) depend on *neither* — they can be
finalized independently and are the natural first slice.

### D3. Nine Lean files, base→dependent order
Execution order (each verified with bare `lean` before the next), extending plan §3 with the new
`CausalCommand.lean`: `EffectLog` → `EffectValue` → `CausalCommand` → (`CausalMonad` reframe + add
`lawful`) → `CausalArrow` → `Alternatable` → `CausalFlow` → `Consistency` → `Csv`. Each file is self-
contained (no imports), transcribes the Rust carrier channel-for-channel (as `CausalMonad.lean`
already does), cites the haft base id it extends, and proves only the delta.

### D4. Model fidelity via representative concrete carriers (haft house style)
Lean cannot take an arbitrary Rust generic, so each proof uses the crate's own canonical instance with
`fmap`/`bind`/`run` transcribed literally from source — exactly as haft does (`OptionWitness` for the
functor hierarchy, the `run` denotation for IO). `CausalCommand`'s free monad is proved over a
representative functor (as `Haft/FreeMonad.lean` proves over `f a = E × a`), because Lean's positivity
checker rejects nested inductives over a variable functor; the proof uses only the functor laws, so it
generalizes. Program equality for the free monad is `fold`-to-canonical-value (the haft free-monad
witness technique), sidestepping the recursive-GAT `PartialEq` trait-solver overflow already documented
in `free_monad.rs`.

### D5. Witness style: `formalization_lean` mirror + keep Kani
Plan §1 mandates a new `deep_causality_core/tests/formalization_lean/` mirror "matching the haft
convention" — one `*_tests.rs` per Lean file, one `#[test]` per id, checking the law empirically on the
real Rust implementation at representative inputs. The existing Kani harnesses (`tests/kani_proofs.rs`,
first-order bounded checks for `left_id`/`left_zero`) are kept and their `THEOREM_MAP` `Kani` column
entries preserved. New ids default to a `formalization_lean` witness; Kani is added only where a
bounded model check adds value over a point-witness (as the map already reflects: some core ids carry
both `Test ✓` and `Kani ✓`). BUILD.bazel registers the mirror (the free-monad-tests precedent: list
`crate_features` explicitly since Bazel does not resolve Cargo feature transitivity).

### D6. `THEOREM_MAP` and CI are extended, not modified
Every new `core.*` id gets a row; `EffectLog`'s four staged rows lose the "staged" qualifier and gain
witnesses; `CausalMonad`'s `lawful` row flips from "blocked on P1" to `proved`. Each Lean file is added
to `lean/DeepCausalityFormal.lean` so `lake build` covers it. The consistency gate needs no rule change
— it already fails on any id missing a side, so correctness is enforced mechanically once rows + tags +
witnesses are in place.

### D7. Deviation ledger finalized into an audit
`core-formalization-plan.md` graduates to `core-formalization-deviations.md` (mirroring
`haft-formalization-deviations.md`): every D1–D17 carries a terminal disposition. Post-prerequisite,
the **Fix-planned** entries (D2, D5, D6, D14, D15) become **Fixed**; D9/D13/D16/D17 stay **Accepted
property**; D11/D12 stay **Documented extension**; D8 stays **Deferred**. No open item remains.

## Risks / Trade-offs

- **[Ordering: proofs written before prerequisites land would encode the deviations]** → This change is
  explicitly gated. `tasks.md` puts the prerequisite-agnostic slice first (`EffectLog`, base
  `CausalMonad`) and blocks the B-dependent files on a checked precondition (control channel separated;
  arrow threads state). Apply MUST NOT proceed on a B-dependent file until its prerequisite is merged.
- **[The two prerequisite changes are not yet OpenSpec changes (only notes)]** → Flagged in the
  proposal Impact. They must be authored and landed first; this change references them by working name.
  If the author wants them created now, that is a separate `propose` invocation.
- **[Lean positivity / trait-solver limits on the free monad]** → Mitigated by the proven haft
  technique (representative functor + `fold`-canonicalization); no new Lean capability required.
- **[Witness drift: a Rust refactor silently breaks a law the point-witness doesn't catch]** → The
  witnesses check representative inputs, not ∀; this is the accepted haft trade-off (Lean proves ∀,
  Kani bounds it, the witness pins the code). Kani is added on the load-bearing monad/arrow laws to
  widen coverage beyond point inputs.
- **[Scope creep into `deep_causality`]** → The do-operator (D8) is explicitly out; the Lean
  `Alternatable` docstring points forward to the hypergraph layer without formalizing it here.

## Migration Plan

Not a runtime change — no deploy/rollback. Sequencing:
1. Land `separate-control-channel` and `causal-arrow-state-threading` (prerequisites, separate changes).
2. Finalize the prerequisite-independent slice: `EffectLog.lean` (bridge), `CausalMonad.lean` (reframe;
   `lawful` deferred to step 4). Witnesses + `THEOREM_MAP` rows.
3. Author the B-dependent Lean files in the D3 order, each bare-`lean`-checked, each witnessed.
4. Add `core.causal_monad.lawful`; flip its `THEOREM_MAP` row to `proved`.
5. `LEAN_CORE.md`, finalize the deviations audit, wire `lake build` + BUILD.bazel; run the full gate
   (`lake build`, `cargo test -p deep_causality_core`, `bazel test //...`).

## Open Questions

- Should the two prerequisite changes be authored as OpenSpec changes now (so this change has concrete
  dependency ids to reference), or left as design notes until scheduled? (Recommendation: author them
  next, since this change is inert until they land.)
- Kani breadth: which additional core ids (beyond the monad) warrant a bounded Kani harness vs. a
  point-witness — the arrow category laws are the strongest candidate. Resolve during apply.
