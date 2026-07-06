## Why

`CausalEffectPropagationProcess` carries its value and error channels as two independent
`pub` fields, so the invalid state "has a value AND has an error" is representable — and
`bind` destroys data on it (right identity `bind(m, pure) = m` fails: the value is silently
replaced by `None`). This is precondition **P2** of the Causal Algebra program
(`openspec/notes/causal-algebra/Formalization.md` §2, "the single load-bearing fix"): until
it lands, the Causal Monad is only a lax monad, the Lean `LawfulMonad` theorems
(`core.causal_monad.right_id`, `assoc`, `lawful` in `lean/THEOREM_MAP.md`) stay blocked, and
the monograph claim is unsound. The haft formalization independently rediscovered the same
defect three times in miniature (deviations D6, D7; both resolved by sum encodings), which
confirmed the fix and the urgency.

## What Changes

- **BREAKING** `CausalEffectPropagationProcess` replaces the two fields
  `value: EffectValue<Value>` + `error: Option<Error>` with ONE channel
  `outcome: Result<EffectValue<Value>, Error>` — the `Either E (Maybe T)` encoding prescribed
  by Formalization.md. The W-invariant (`error ⇒ no value`) becomes true by construction:
  invalid states are unrepresentable.
- **BREAKING** All five carrier fields become private (crate field-visibility convention);
  construction goes through constructors (`pure`, `none`, `from_error`, `from_effect_value*`,
  plus a new total `new(outcome, state, context, logs)` — total because with one channel
  every combination is valid) and access through the existing getter surface (adjusted
  shapes: `value() -> Option<&EffectValue<Value>>`, `error() -> Option<&Error>`, new
  `outcome() -> &Result<EffectValue<Value>, Error>`).
- `bind` / `fmap` / `bind_or_error` internals rewritten over the single channel; the
  `CausalMonad` trait contract (continuation shape) is unchanged. Error short-circuit
  semantics (state/context/logs survive an error) are preserved and now lawful.
- Repo-wide mop-up: every construction literal and field access adapts — `deep_causality_core`
  (carrier modules, HKT witnesses, `causal_flow`, ~78 literal sites incl. tests),
  `deep_causality` (causaloid evaluate/stateful/utils, graph reasoning, CSM eval, collection
  reasoning + ~38 test files), spot-checks in `deep_causality_physics` / `deep_causality_cfd`
  (no literal construction in either; field reads only), 4 example files.
- Formalization unblocked and delivered in the same change: Lean right-identity and
  associativity theorems for the cleaned carrier model (unblocking
  `core.causal_monad.right_id` / `assoc` in `THEOREM_MAP.md`), Kani harnesses updated
  (W-well-formedness becomes true-by-construction; short-circuit and log-monotonicity
  harnesses strengthened), Rust witnesses added.

Out of scope: **P1** (removal of the `RelayTo` / `Map` control variants from `EffectValue`)
is a separate follow-up change; this design must not preclude it and does not touch the
`EffectValue` enum itself.

## Capabilities

### New Capabilities
- `lawful-effect-channel`: the value-XOR-error channel of the causal-effect carrier — its
  encoding, constructor/accessor surface, monad-law guarantees (left identity, right
  identity, associativity, error short-circuit as a left zero), and their machine-checked
  verification (Lean theorems + Kani harnesses + Rust witnesses).

### Modified Capabilities

<!-- none — no existing spec covers the carrier; io-monad is unaffected -->

## Impact

- **Crates:** `deep_causality_core` (heavy — carrier + HKT witnesses + causal_flow + tests),
  `deep_causality` (moderate — reasoning paths + tests), `deep_causality_physics` /
  `deep_causality_cfd` (light — read-site adaptation), `examples/*` (4 files).
  `deep_causality_haft` is untouched (the trait layer has no carrier).
- **Public API:** semver-major for `deep_causality_core` and `deep_causality` (field access
  removed, getter shapes changed). Workspace releases in lockstep.
- **Formalization assets:** `lean/DeepCausalityFormal/Core/CausalMonad.lean` (model extended
  with the error channel + two new theorems), `deep_causality_core/tests/kani_proofs.rs`,
  `lean/THEOREM_MAP.md` (two ids move from "blocked" to proved), Formalization.md work-plan
  item 1 closes.
- **Docs:** carrier/trait docstrings, `deep_causality_core/README.md` and `Notes.md` if they
  show field construction.
