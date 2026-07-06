## Why

`deep_causality_haft` is fully formalized: every categorical mechanism has a Lean 4 proof, a
Rust witness test, a `THEOREM_MAP` row, and a resolved-deviations audit
(`deep_causality_haft/LEAN_HAFT.md`, `openspec/notes/causal-algebra/haft-formalization-deviations.md`).
`deep_causality_core` — the crate built directly on haft — is only *partially* formalized: two Lean
files exist (`Core/CausalMonad.lean` proved, `Core/EffectLog.lean` staged/un-bridged), covering 5 of
the ~30 laws the crate actually relies on. The causal extension (the arrow, the effect-value functor,
the control channel, the lens/alternatable family, the flow facade, the IO codecs) is unverified, and
the deviation survey in `core-formalization-plan.md` found 17 gaps between the code and its accepted
mathematical definition. This change closes that gap: it brings core to full parity with haft —
every mechanism proven, witnessed, bridged, and its deviation resolved.

Per the author's decision (2026-07-06), this change **proves the clean, unconditional laws over the
corrected code** rather than modeling fragments with negative lemmas. That makes two already-designed
corrective changes hard prerequisites (see Impact) so the proofs describe the faithful implementation,
not the deviations — and it unblocks the long-blocked `core.causal_monad.lawful` claim.

## What Changes

- Add **9 Lean files** under `lean/DeepCausalityFormal/Core/`, one per core mechanism, mirroring the
  haft tree — each self-contained (bare-`lean`-checkable, no Mathlib), SPDX header,
  `namespace DeepCausalityFormal.Core.<X>`, `THEOREM_MAP` tags, Rust witnesses cited:
  - `CausalMonad.lean` *(exists)* — reframe to cite `haft.monad.laws` as the base; **add
    `core.causal_monad.lawful`** (unblocked once the control channel is separated).
  - `EffectLog.lean` *(exists, staged)* — **bridge** its 4 ids (witnesses + `THEOREM_MAP` rows; drop
    the "staged" tag).
  - `EffectValue.lean` *(new)* — the now-clean pointed functor (**total** `fmap_id`/`fmap_comp`,
    congruent `PartialEq`, honest `into/from` round-trip, `≅ Option` section). No seam caveat.
  - `CausalCommand.lean` *(new)* — the separated control functor and its **free monad** (`Free` over
    `CausalCommand`), citing `haft.free_monad.*`; program equality by `fold`-canonicalization.
  - `CausalArrow.lean` *(new)* — the Kleisli category of the causal monad with **full state/context
    threading** (category laws + `left_zero`), no `S,C`-erasure caveat.
  - `Alternatable.lean` *(new)* — the lens-setter family (value/state/context + `clear_context`):
    `set_get`, `set_set` up-to-log (with the `proj` eraser), channel independence, error no-op.
  - `CausalFlow.lean` *(new)* — the facade lowering: `flow_iso`, `map` = `bind(pure∘f)` on the full
    effect-value (D14 fixed), iterate/branch/recover/finish as **documented extensions**.
  - `Csv.lean` *(new)* — the conditional codec round-trip (precondition as an explicit hypothesis).
  - `Consistency.lean` *(new)* — witness `pure`/`fmap` **agreement** (post-B the four `fmap`s unify;
    the panic and non-reflexive eq are gone).
- Add a **`deep_causality_core/tests/formalization_lean/` witness mirror** (one `*_tests.rs` per Lean
  file, one `#[test]` per id), matching the haft convention; keep the existing Kani harnesses.
- Add **`THEOREM_MAP.md` rows** for every new `core.*` id and wire each Lean file into
  `DeepCausalityFormal.lean` (`lake build`) — the existing CI consistency gate then enforces the
  Lean↔Rust bridge automatically.
- Add **`deep_causality_core/LEAN_CORE.md`** (status table mirroring `LEAN_HAFT.md`) and finalize
  `core-formalization-plan.md` into the resolved-deviations audit
  (`core-formalization-deviations.md`).
- No runtime/public-API change in this crate: this change adds verification artifacts and
  documentation only. All behavior corrections live in the two prerequisite changes.

## Capabilities

### New Capabilities
- `core-formalization`: The machine-checked verification of `deep_causality_core` — the requirement
  that every categorical/causal mechanism in the crate carries a closed Lean proof, an independent
  Rust witness bound by a shared `THEOREM_MAP` id, a CI consistency gate enforcing that bridge, and a
  resolved disposition for every surveyed deviation. Defines *what must be proven and witnessed*, not
  the runtime behavior (which the prerequisite changes and existing specs already fix).

### Modified Capabilities
<!-- None. This change verifies existing behavior; it does not alter any spec-level requirement.
     `core.causal_monad.lawful` becoming proved is a formalization-status change, not a new runtime
     requirement on `lawful-effect-channel`. -->

## Impact

- **Hard prerequisites (must land first — clean-laws goal depends on corrected code):**
  1. `separate-control-channel` (control Option B) — moves `RelayTo`/`Map` out of `EffectValue` into
     a `CausalCommand` operation functor; resolves D5/D6/D14/D15, fixes the two must-fix bugs (the
     arity-5 `fmap` **panic**, the non-reflexive `Map` equality), and unblocks
     `core.causal_monad.lawful`. Designed in `core-formalization-plan.md §2A`; not yet an OpenSpec
     change.
  2. `causal-arrow-state-threading` (arrow Option B) — widens the arrow stage to the state-receiving
     Kleisli arrow so composition threads state/context exactly as the monad's `bind`; resolves D2
     fully (removes the `S,C`-erasure caveat). Designed in `causal-arrow-state-threading-plan.md`; not
     yet an OpenSpec change.
  These are **separate changes**, per the author's "depend on Option B first" decision — this change
  neither implements nor folds them in; it targets the code *after* they land.
- **New files:** `lean/DeepCausalityFormal/Core/{EffectValue,CausalCommand,CausalArrow,Alternatable,
  CausalFlow,Csv,Consistency}.lean`; `deep_causality_core/tests/formalization_lean/*`;
  `deep_causality_core/LEAN_CORE.md`.
- **Edited files:** `lean/DeepCausalityFormal/Core/{CausalMonad,EffectLog}.lean`,
  `lean/DeepCausalityFormal.lean`, `lean/THEOREM_MAP.md`,
  `deep_causality_core/tests/BUILD.bazel` (register the witness mirror),
  `openspec/notes/causal-algebra/core-formalization-plan.md` (→ resolved audit).
- **CI:** `.github/workflows/formalization.yml` — no rule change; the added ids are covered by the
  existing `lake build` + witness-test + consistency-gate steps.
- **Toolchain:** Lean via `~/.elan/bin/lean` (bare typecheck per file) and `lake build` (project);
  Rust witnesses run under `bazel test //...` and `cargo test -p deep_causality_core`. Workspace MSRV
  stays pinned at Kani's nightly (1.93.0).
- **Risk:** low for the artifacts themselves (additive, no runtime code). The real risk is ordering —
  if the formalization is written before the two prerequisites land, the proofs would encode the
  deviations. This change is explicitly gated on them.
