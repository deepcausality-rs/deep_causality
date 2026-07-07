## Why

`deep_causality_haft` is fully formalized: every categorical mechanism has a Lean 4 proof, a
Rust witness test, a `THEOREM_MAP` row, and a resolved-deviations audit
(`deep_causality_haft/LEAN_HAFT.md`, `openspec/notes/causal-algebra/haft-formalization-deviations.md`).
`deep_causality_core` — the crate built directly on haft — is only *partially* formalized: three Lean
files exist (`Core/CausalMonad.lean` proved and congruence-noted, `Core/CausalArrow.lean` proved,
`Core/EffectLog.lean` staged/un-bridged), covering the monad/arrow surface but not the ~30 laws the
crate actually relies on. The rest of the causal extension (the success-channel functor, the command
functor, the lens/alternatable family, the flow facade, the IO codecs) is unverified, and the
deviation survey in `core-formalization-deviations.md` found 17 gaps between the code and its accepted
mathematical definition. This change closes that gap: it brings core to full parity with haft —
every mechanism proven, witnessed, bridged, and its deviation resolved.

Per the author's decision (2026-07-06), this change **proves the clean, unconditional laws over the
corrected code** rather than modeling fragments with negative lemmas. The two corrective prerequisites
have **already landed** (`causal-arrow-state-threading` and `separate-control-channel`; see Impact), so
the proofs describe the faithful implementation, not the deviations — and the long-blocked
`core.causal_monad.lawful` claim is now unblocked (P1 resolved: control is separated into
`CausalCommand`/`CausalEffect` and the carrier is the transformer stack `Except ∘ Free ∘ Maybe`).

## What Changes

- Add **7 new Lean files** (and finish 2 existing ones) under `lean/DeepCausalityFormal/Core/`, one
  per core mechanism, mirroring the haft tree — each self-contained (bare-`lean`-checkable, no
  Mathlib), SPDX header, `namespace DeepCausalityFormal.Core.<X>`, `THEOREM_MAP` tags, Rust witnesses
  cited:
  - `CausalMonad.lean` *(exists, congruence-noted)* — cites `haft.monad.laws` as the base; **add
    `core.causal_monad.lawful`** (now unblocked — P1 resolved).
  - `EffectLog.lean` *(exists, staged)* — **bridge** its 4 ids (witnesses + `THEOREM_MAP` rows; drop
    the "staged" tag).
  - `CausalArrow.lean` *(exists, landed in `causal-arrow-state-threading`)* — the Kleisli category of
    the causal monad with **full state/context threading** (category laws + `left_zero`), no
    `S,C`-erasure caveat. Verify registration + `THEOREM_MAP` rows; no re-authoring.
  - `CausalEffect.lean` *(new)* — the success channel `CausalEffect<V> = Free<CausalCommandWitness,
    Option<V>>`: the value content is `Option` (cite `haft.functor.laws`, no bespoke functor), plus
    the honest `into_value` = `Maybe` projection. Replaces the deleted `EffectValue.lean`; no seam caveat.
  - `CausalCommand.lean` *(new)* — the single-hole control functor `CausalCommand { RelayTo }` and its
    **free monad** (`Free` over `CausalCommand`), citing `haft.free_monad.*`; structural `RelayTo`-tree
    equality.
  - `Alternatable.lean` *(new)* — the lens-setter family (value/state/context + `clear_context`):
    `set_get`, `set_set` up-to-log (with the `proj` eraser), channel independence, error no-op.
  - `CausalFlow.lean` *(new)* — the facade lowering: `flow_iso`, `map` = `bind(pure∘f)` on the value
    fragment (D14 fixed), iterate/branch/recover/finish as **documented extensions**.
  - `Csv.lean` *(new)* — the conditional codec round-trip (precondition as an explicit hypothesis).
  - `Consistency.lean` *(new)* — witness `pure`/`fmap` **agreement** (`CausalEffect::map` is total and
    uniform; the panic and non-reflexive `Map` eq are gone with `EffectValue`).
- Add a **`deep_causality_core/tests/formalization_lean/` witness mirror** (one `*_tests.rs` per Lean
  file, one `#[test]` per id), matching the haft convention; keep the existing Kani harnesses.
- Add **`THEOREM_MAP.md` rows** for every new `core.*` id and wire each Lean file into
  `DeepCausalityFormal.lean` (`lake build`) — the existing CI consistency gate then enforces the
  Lean↔Rust bridge automatically.
- Add **`deep_causality_core/LEAN_CORE.md`** (status table mirroring `LEAN_HAFT.md`) and finalize
  `core-formalization-plan.md` into the resolved-deviations audit
  (`core-formalization-deviations.md`).
- No runtime/public-API change in this crate: this change adds verification artifacts and
  documentation only. All behavior corrections live in the two prerequisite changes (already landed).

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

- **Hard prerequisites (LANDED — clean-laws goal depends on corrected code):**
  1. `separate-control-channel` — **deleted `EffectValue`**, made the success channel `CausalEffect<V>
     = Free<CausalCommandWitness, Option<V>>`, moved `RelayTo` into a single-hole `CausalCommand`
     operation functor (and deleted the unused `Map`/`Dispatch`); resolved D5/D6/D14/D15, fixed the two
     must-fix bugs (the arity-5 `fmap` **panic**, the non-reflexive `Map` equality), and unblocked
     `core.causal_monad.lawful`. Archived `2026-07-06-separate-control-channel`.
  2. `causal-arrow-state-threading` — widened the arrow stage to the state-receiving Kleisli arrow so
     composition threads state/context exactly as the monad's `bind`; resolved D2 fully (removed the
     `S,C`-erasure caveat) and shipped `Core/CausalArrow.lean`. Archived.
  Both were **separate changes** that have merged; this change targets the code *after* they landed and
  neither implements nor folds them in.
- **New files:** `lean/DeepCausalityFormal/Core/{CausalEffect,CausalCommand,Alternatable,
  CausalFlow,Csv,Consistency}.lean`; `deep_causality_core/tests/formalization_lean/*`;
  `deep_causality_core/LEAN_CORE.md`. (`CausalArrow.lean` already exists from the prerequisite.)
- **Edited files:** `lean/DeepCausalityFormal/Core/{CausalMonad,EffectLog}.lean`,
  `lean/DeepCausalityFormal.lean`, `lean/THEOREM_MAP.md`,
  `deep_causality_core/tests/BUILD.bazel` (register the witness mirror),
  `openspec/notes/causal-algebra/core-formalization-plan.md` (→ resolved audit).
- **CI:** `.github/workflows/formalization.yml` — no rule change; the added ids are covered by the
  existing `lake build` + witness-test + consistency-gate steps.
- **Toolchain:** Lean via `~/.elan/bin/lean` (bare typecheck per file) and `lake build` (project);
  Rust witnesses run under `bazel test //...` and `cargo test -p deep_causality_core`. Workspace MSRV
  stays pinned at Kani's nightly (1.93.0).
- **Risk:** low for the artifacts themselves (additive, no runtime code). The ordering risk is
  retired — both prerequisites have landed and the workspace is green, so the proofs target the
  corrected code rather than the deviations.
