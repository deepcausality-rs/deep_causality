## MODIFIED Requirements

### Requirement: Every core mechanism is proven, witnessed, and bridged
Every categorical or causal mechanism in `deep_causality_core` SHALL carry (1) a closed Lean 4 proof
under `lean/DeepCausalityFormal/Core/`, (2) an independent Rust witness under
`deep_causality_core/tests/formalization_lean/`, and (3) a row in `lean/THEOREM_MAP.md` binding them
by a shared `core.*` id. Each Lean file SHALL be self-contained (typechecking under bare `lean <file>`
with no Mathlib import), carry the SPDX header, use `namespace DeepCausalityFormal.Core.<X>`, cite its
literature reference and the Rust source, and cite the base haft theorem it extends rather than
re-proving it. Every Lean theorem SHALL be closed with **zero `sorry`**.

**Causaloid-layer extension.** Core Lean files whose Rust realization lives in the main
`deep_causality` crate (the causaloid fixpoint, Verdict closure, graph algebra, and catamorphism
layers) SHALL place their witnesses under `deep_causality/tests/formalization_lean/` — a witness
mirror in the main crate following the same conventions (one `<mechanism>_tests.rs` per Lean file,
registered in its `mod.rs` and `BUILD.bazel`, one `#[test]` per id). The CI consistency gate
(`.github/workflows/formalization.yml`) SHALL include `deep_causality` in its Rust-witness search
scope so these ids are enforced identically.

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

#### Scenario: The main-crate witness mirror is enforced by the gate
- **WHEN** a causaloid-layer `core.*` id is tagged in a Core Lean file and its witness exists only
  under `deep_causality/tests/formalization_lean/`
- **THEN** the consistency gate finds the witness (its search scope includes `deep_causality`) and
  passes; removing the witness makes the gate fail
