# core-formalization Specification

## ADDED Requirements

### Requirement: Every core mechanism is proven, witnessed, and bridged
Every categorical or causal mechanism in `deep_causality_core` SHALL carry (1) a closed Lean 4 proof
under `lean/DeepCausalityFormal/Core/`, (2) an independent Rust witness under
`deep_causality_core/tests/formalization_lean/`, and (3) a row in `lean/THEOREM_MAP.md` binding them
by a shared `core.*` id. Each Lean file SHALL be self-contained (typechecking under bare `lean <file>`
with no Mathlib import), carry the SPDX header, use `namespace DeepCausalityFormal.Core.<X>`, cite its
literature reference and the Rust source, and cite the base haft theorem it extends rather than
re-proving it. Every Lean theorem SHALL be closed with **zero `sorry`**.

#### Scenario: Each Lean file typechecks standalone
- **WHEN** `lean lean/DeepCausalityFormal/Core/<File>.lean` is run for any Core file
- **THEN** it typechecks with no errors and no `sorry`

#### Scenario: The consistency gate enforces the bridge
- **WHEN** the CI job `.github/workflows/formalization.yml` runs its consistency step
- **THEN** every `core.*` id tagged in a Lean file has both a matching Rust witness and a
  `THEOREM_MAP.md` row, and the job fails if either side is missing

#### Scenario: The witness mirror parallels the Lean tree
- **WHEN** the directory `deep_causality_core/tests/formalization_lean/` is inspected
- **THEN** it contains one `<mechanism>_tests.rs` per Core Lean file with theorems (registered in its
  `mod.rs` and `deep_causality_core/tests/BUILD.bazel`), and each `core.*` id has one `#[test]`

### Requirement: The causal monad is verified as a lawful monad
The `CausalEffectPropagationProcess` carrier SHALL be proven a lawful monad: left identity, right
identity (unconditional, including errored carriers), associativity, and error left-zero, citing
`haft.monad.laws` as the base and proving only the state/context/log/error extension. Once the control
channel is separated (prerequisite `separate-control-channel`), the full `LawfulMonad`-with-effect
claim `core.causal_monad.lawful` SHALL be stated and closed — it SHALL NOT remain blocked on P1.

#### Scenario: The four base monad laws hold
- **WHEN** `Core/CausalMonad.lean` is checked
- **THEN** `bind_left_id`, `bind_right_id`, `bind_assoc`, and `bind_raise_left_zero` are closed and
  their ids (`core.causal_monad.{left_id,right_id,assoc,left_zero}`) are witnessed

#### Scenario: The lawful-monad claim is unblocked
- **WHEN** the control channel has been separated and `Core/CausalMonad.lean` is checked
- **THEN** `core.causal_monad.lawful` is a closed theorem (not a `— blocked on P1` entry) with a Rust
  witness

### Requirement: The effect log is verified as a free monoid
`EffectLog` SHALL be proven the free monoid on a message alphabet (the Writer output): left identity,
right identity, associativity, and append-only monotonicity, over the `List Λ` abstraction that
quotients timestamps. Its four ids SHALL be **bridged** (witnesses present, `THEOREM_MAP` rows added,
the "staged — bridged in the core-formalization phase" tag removed).

#### Scenario: The monoid laws hold and are bridged
- **WHEN** `Core/EffectLog.lean` and its witness are checked
- **THEN** `core.effect_log.{left_id,right_id,assoc,monotone}` are closed, each has a passing Rust
  witness, and each has a `THEOREM_MAP.md` row with no "staged" qualifier

### Requirement: The effect value is a lawful pointed functor
Post `separate-control-channel`, `EffectValue = {None, Value, ContextualLink}` SHALL be proven a lawful
pointed functor with **total** `fmap` (identity and composition holding on every constructor, no
fragment restriction), a congruent structural equality, and an honest `into_value`/`from` round-trip
that is the faithful `Maybe` projection (`Value → Some`, `None`/`ContextualLink → None`). No negative
lemma about `RelayTo` payload-drop or `Map` non-reflexivity SHALL be required, because those variants
no longer inhabit the type.

#### Scenario: Total functor laws
- **WHEN** `Core/EffectValue.lean` is checked
- **THEN** `core.effect_value.{fmap_id,fmap_comp}` are closed as total laws over all three
  constructors, and are witnessed

#### Scenario: Faithful Maybe projection
- **WHEN** `into_value`/`from` and the `≅ Option` section are checked
- **THEN** `core.effect_value.{into_from_roundtrip,maybe_section}` are closed and witnessed, with
  `Value(v) ↦ Some v` and `None`/`ContextualLink ↦ None`

### Requirement: The control channel is a free monad over CausalCommand
Post `separate-control-channel`, the control operations `CausalCommand` (`RelayTo`, `Dispatch`) SHALL
be proven a functor, and the adaptive-reasoning program SHALL be the **free monad** on that functor,
citing `haft.free_monad.*` for the monad laws. Program equality SHALL be by `fold`-canonicalization (as
in the haft free-monad witnesses); a lawful congruent equality (structural over `RelayTo` payload and
the `Dispatch` map) SHALL replace the former partial-equivalence relation.

#### Scenario: CausalCommand is a lawful free-monad operation functor
- **WHEN** `Core/CausalCommand.lean` is checked
- **THEN** the `CausalCommand` functor laws are closed, and the free-monad laws over it hold (citing
  `haft.free_monad.left_id`/`right_id`/`assoc`), with ids `core.causal_command.*` witnessed by
  `fold`-canonicalization

### Requirement: The causal arrow is a lawful Kleisli category with state threading
Post `causal-arrow-state-threading`, the causal arrow SHALL be proven the Kleisli category of the
causal monad with **full state/context threading**: left identity, right identity, and associativity
of `>>>`, plus the error `left_zero`, threading `(value, state, context)` exactly as the monad's
`bind`. The proofs SHALL NOT carry an "the model erases `S,C`" caveat, and right identity SHALL hold
unconditionally (the `and_then` `None`-collapse is corrected).

#### Scenario: Category laws thread state and context
- **WHEN** `Core/CausalArrow.lean` is checked
- **THEN** `core.causal_arrow.{category_laws,left_zero}` are closed with state/context threaded on both
  sides of each equation, and are witnessed

#### Scenario: Right identity is unconditional
- **WHEN** a stage that can emit `None` is composed with the identity arrow
- **THEN** `f >>> arr id = f` holds (no `None → Err` collapse), machine-checked and witnessed

### Requirement: The alternatable lens family satisfies the lens laws up-to-log
The value/state/context setter family SHALL be proven to satisfy the lens laws under the audit-log-
erasing projection `proj` — set-get, set-set idempotence, channel independence, and error no-op — for
`alternate_value`, `alternate_state`, `alternate_context`, and `clear_context`. The deliberate
success-path log entry (deviation D9, an accepted Writer property) SHALL be documented with a machine-
checked lemma that the full carrier grows the log (the laws hold only up-to-log). The `intervene` alias
SHALL NOT appear (removed from core); `clear_context` SHALL be proven the `None`-setting counterpart
`alternate_context` lacked.

#### Scenario: Lens laws hold on the value projection
- **WHEN** `Core/Alternatable.lean` is checked
- **THEN** `core.alternatable.{set_get,set_set_proj,channel_independence,error_noop}` are closed under
  `proj` and witnessed

#### Scenario: The up-to-log caveat is honest
- **WHEN** the full-carrier (non-projected) set-set is checked
- **THEN** a negative lemma shows the log differs (the laws are up-to-log, not on-the-nose), and
  `clear_context` sets the context to `None` with a `!!ContextCleared!!` entry, no-op on error

### Requirement: The causal-flow facade laws hold and extensions are documented
The `CausalFlow` facade SHALL be proven to lower faithfully: the `≅ Process` iso (`rfl`), functor
identity/composition, and `map f = and_then(pure ∘ f)` holding on the **full** effect value (D14
corrected). Operations that exceed the base monad — `recover` (`MonadError.catch`), `iterate_until` /
`iterate_to_fixpoint` (bounded search injecting `MaxStepsExceeded`), and `finish` (value-observation
terminal that drops state/context/log) — SHALL be formalized as **documented extensions** with their
own stated laws, not as monad sugar.

#### Scenario: Facade lowering and corrected map law
- **WHEN** `Core/CausalFlow.lean` is checked
- **THEN** `core.causal_flow.{flow_iso,map_id,map_comp,map_eq_andThen}` are closed and witnessed, with
  `map_eq_andThen` holding on `None`/`ContextualLink` as well as `Value`

#### Scenario: Extensions carry their own contracts
- **WHEN** `recover`, the iterate combinators, and `finish` are formalized
- **THEN** each has its own id and stated law (catch law; bounded-search termination/`MaxStepsExceeded`
  contract; terminal projection with an explicit note that the log is dropped), each witnessed

### Requirement: The CSV codec round-trips under its precondition
The IO CSV codec SHALL be proven to satisfy `parse (render header rows) = header :: rows` under the
explicit hypothesis that no field contains `','` or `'\n'` (no quoting/escaping), citing the base
`haft.io.laws` for the underlying IO monad. The precondition SHALL be stated as a theorem hypothesis,
not assumed away.

#### Scenario: Conditional round-trip
- **WHEN** `Core/Csv.lean` is checked with comma/newline-free fields
- **THEN** `core.io.csv_roundtrip` is closed under that hypothesis and witnessed

### Requirement: Witness functors agree with the inherent functor
The HKT witness `fmap` implementations SHALL be proven to agree with the inherent `fmap` that
`CausalFlow::map` uses, over the clean value functor (post `separate-control-channel`), for both the
effect and process witnesses. The former disagreement (deviation D15 — four `fmap`s diverging, one panicking via `.expect`)
SHALL be gone: there SHALL be no panic path and no fragment on which the witnesses diverge.

#### Scenario: The functors coincide
- **WHEN** `Core/Consistency.lean` is checked
- **THEN** `core.witness.agree` is closed showing witness `fmap` = inherent `fmap` on every carrier
  (no `Ok(Value _)`-only restriction), and is witnessed with no reachable panic

### Requirement: Every surveyed deviation has a resolved disposition
The deviation ledger SHALL be finalized: `core-formalization-plan.md` becomes the resolved audit
(`core-formalization-deviations.md`, mirroring `haft-formalization-deviations.md`), and every deviation
D1–D17 SHALL carry a terminal disposition — **Fixed**, **Documented extension**, **Accepted property**,
or **Deferred** — with no item left as an open **Fix-planned** once its prerequisite change has landed.

#### Scenario: No unresolved deviation remains
- **WHEN** the finalized audit is reviewed after both prerequisite changes have landed
- **THEN** each of D1–D17 has a terminal disposition, and the two prior soft-flagged items (D10, D16)
  are settled

### Requirement: A crate-local status document mirrors LEAN_HAFT
`deep_causality_core/LEAN_CORE.md` SHALL exist, mirroring `deep_causality_haft/LEAN_HAFT.md`: a summary
of the proof/witness/bridge counts, the `how to check` commands (`lake build`, bare `lean`, the witness
test invocation), a per-mechanism status table with references, and a pointer to the resolved-deviations
audit.

#### Scenario: Status document is complete and accurate
- **WHEN** `deep_causality_core/LEAN_CORE.md` is reviewed
- **THEN** it lists every Core mechanism with its reference and `laws stated & hold` status, the check
  commands run green, and the counts match `THEOREM_MAP.md`
