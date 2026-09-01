<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Why

`deep_causality_quantum` ships 4432 lines that already answer the hard questions: the Markov
commutativity check, the C₃-exclusion search, four tolerance policies, the Choi round trip, the
orthomodular verdict carrier, the Haruna gate layer and the 𝔽₂ homology chain under it. Writing a
correct quantum causal model against that surface still costs four lines of turbofished closure
ceremony per stage, hand-packed tensors with manual shapes, and decisions taken as `min_by` on exact
reals. **The problem is not correctness. It is that correct code is unreasonably hard to write.**

Every prerequisite is now in place. All eighteen gaps in
[`qcl-gaps.md`](../../notes/quantum/qcl-gaps.md) are closed, both seams that produced confident wrong
answers are shut, and the design in [`qcl-design-note.md`](../../notes/quantum/qcl-design-note.md) has
been re-verified against the tree. What remains is six steps of construction against settled designs.

## What Changes

- **A decision form.** `Check<R>` and `CheckReport<R>` generalise `CommutatorCheck`'s
  `(measured, threshold, margin, verdict)` shape, and `Tolerance<R>` generalises the four shipped
  policies into a family with named members. No stage returns a bare `bool`, and every check reports
  how many items it examined, so a vacuous pass is visible as one.
- **Typed carriers** replacing hand-packed tensors: `QubitOperator`, `Channel` (CPTP checked once at
  construction), `QuantumPlant`, `Observable`. Each seals its validated interior and exposes
  operations, and each gets a `wrappers.rs`-style lift into the causal monad.
- **`ShotBudget` and `Evidence`**, turning read-out decisions from float comparisons into statistical
  ones, and deciding how a runtime shot budget relates to the compile-time modality split.
- **`Hypothesis` and `intervene`.** A structural candidate is a factorization
  `{ name, ProcessFactors<R>, FactorSupports }`; `intervene(do(node ← factor))` is a keyed
  replacement plus revalidation. `predict` marginalises only through the shipped boundary check.
- **`design` and `adjudicate`.** `design` returns a `DesignPlan` rather than one experiment, solved
  as minimum-cost set cover over the `C(n,2)` hypothesis pairs. `adjudicate` folds verdicts under the
  §4 rule: projection-valued folds check commutation, read-outs against a real-valued spec do not.
- **`Ledger<R, N>` and `fork`.** Counts are ℕ on `NaturalNumber`, not hardcoded widths; the draw-down
  is `checked_difference`. `fork` is built above core by cloning, because `Either` is the coproduct
  and a counterfactual fork is a product.
- **`QclBuilder`, the config and the stages.** One origin for configuration, naming two working types
  (`FloatType` for accuracy, `IntType` for headroom), branching on the subject, with `validate`
  terminating in a `Screened<R>` that `control` requires.
- **A decision on `check_class_invariance`.** The predicate must quantify over the **code space**,
  not the full Hilbert space. Measured: the full-space criterion decides Z̄ correctly on all 36
  (boundary, generator) pairs of the 3×3 torus and rejects S̄ and T̄ immediately, because Haruna's
  Eq. (3.21) closes on `S_Z(f)` acting as the identity *on the code space*. Deciding it needs the
  stabilizer generators, which `LogicalBasis` does not currently carry.

## Capabilities

### New Capabilities

- `qcl-decision-form`: `Check<R>`, `CheckReport<R>` and `Tolerance<R>`. Every decision carries a
  measured quantity, a threshold, a margin and a count of what was examined. The tolerance family
  derives from the scalar, and has no integer member.
- `qcl-carriers`: `QubitOperator`, `Channel`, `QuantumPlant` and `Observable`, each sealing a
  validated interior, each lifted into the causal monad by the crate's existing wrapper pattern.
- `qcl-evidence`: `Evidence` and `ShotBudget`. Shot noise as a first-class source of uncertainty,
  and where a named shot budget sits against the verifiable/emergent split.
- `qcl-hypothesis`: `Hypothesis` as a factorization with its `CausalStructure` derived, `intervene`
  as a keyed factor replacement, and `predict` gated on the boundary check.
- `qcl-experiment-design`: `design` returning a `DesignPlan` under `MinCostCover`, and `adjudicate`
  applying the verdict-fold rule that matches the kind of verdict a world carries.
- `qcl-pipeline`: `QclBuilder::config` with its two working types, the three subject constructors,
  the `validate` → `Screened<R>` → `control` hand-off, transactional failure, and `Ledger<R, N>`
  with its three invariants.
- `qcl-code-checks`: `derive_code`, `check_ldpc_weights`, and `check_class_invariance` decided over
  the code space, with the stabilizer generators the decision requires.

### Modified Capabilities

- `quantum-crate-scaffold`: the modality requirement's scenario states that in a default build "the
  `qpu` seam is absent". `GateOp` and `QuantumCircuit` left the `qpu` gate when the Haruna layer was
  retyped, because the always-on gate layer emits them; the `QpuSampler` seam and `SimQpu` remain
  gated. The requirement needs to distinguish the circuit data types from the sampler seam, and to
  say how a runtime shot budget selects a modality the compiler enforces.

## Impact

**Code.** New modules under `deep_causality_quantum/src/types/`, consuming the shipped
`qcm`, `qgates`, `qcode`, `verdict` and `qpu/circuit` layers without reimplementing them. The
`qcode` module gains the stabilizer generators `check_class_invariance` needs.

**APIs.** Additive for the existing surface. `LogicalBasis` gains a constructor carrying stabilizers;
`DiagonalPhase` and `is_diagonal_trivial` are already in the working tree and stay.

**Dependencies.** `deep_causality_num_rational` is already added, for exact phase arithmetic with no
tolerance. No new external dependency. The `qpu` feature keeps gating the sampler seam.

**Documentation.** `qcl-design-note.md` §9's sequencing is the plan of record; a check claiming a
theorem names its Rust witness through `lean/THEOREM_MAP.md`, and a check with no proof says so.

**Out of scope.** Decoding, fault-tolerance claims, device models, graph traversal, topology
ownership, and the benchmark suite of §10, whose framing changes now that the real-time loop is
understood to be FPGA rather than QCL.
