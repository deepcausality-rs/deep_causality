<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# TDD record, group 4: the structural precheck, the code as an abstraction, the pipeline stages

## Provenance of the literals

| Literal | Source | Where used |
|---|---|---|
| Example 54: `α(X) = {X, Z}`, `α(Y) = {Y, Z}`, `α(W) = {W}`; simple, not extra-simple | Lorenz & Tull §7.3, Example 54, with the reading that a path passes through its start | `alignment_structure_tests` |
| Example 54, partition `{X}, {Y}, {W, Z}`: extra-simple and full | Example 54, second partition | `alignment_structure_tests` |
| Example 55: `X ∈ α(Y)`, not simple, witness `(Y, X)` | Example 55 (the high-level `X` is a discarded input) | `alignment_structure_tests` |
| Table 1 verdicts on `[[18,2,3]]` and `[[32,2,4]]` | v1 `check_class_invariance`, `check_clifford_action` | `code_abstraction_tests` (the generation regression) |
| `S̄` without its `CZ` pairs: phase `1/2` at overlap two | Haruna Eq. 3.14 against a parity function | `code_abstraction_tests` |
| `[[8,2,2]]` recovery: 64 syndromes, 16 weight-one corrections, max weight the table's largest and at least 2 | on the `2 × 2` torus parallel edges share their vertex pair and their face pair: `4 + 4 + 8` distinct single-edge syndromes | `ideal_recovery_tests` |
| Opened `[[8,2,2]]` square: `n = 10`, `k = 8`, `2^26` entries | `2^(8 + 8) · 2^(2 + 8)` working storage | `code_abstraction_tests` |
| Opened `[[4,2,2]]` square: `d_in = 64`, `d_out = 4` | `2^(2 + 4)` and `2^2` | `code_abstraction_tests` |
| Pipeline: two ordered pairs, one query, folded examined 3 | Definition 49 over two vertices; one `Io` query | `circuit_subject_tests` |

## Corner-case rows

| Row | Case | Test |
|---|---|---|
| A empty | empty partition on wire-only models; `Vacuous` naturality | `abstraction_tests::test_scope…`, `…empty_signature…` |
| B one | one high-level vertex, no pairs | `abstraction_tests::test_scope…` (circuit case) |
| C boundary | the cap at exactly the working storage of the opened code | `code_abstraction_tests::test_opening_the_eight_qubit_code…` |
| D symmetry-breaking | the swapped partition `[[1], [0]]` against `[[0], [1]]` | `circuit_subject_tests::test_a_rejecting_precheck…` |
| E error text | `α(0) meets π(1)`, `Necessary`, `not the screened circuit`, `not in the signature` | `circuit_subject_tests`, `code_abstraction_tests` |
| F zero | zero residual on `Io` and `Open` for `Z̄`, `X̄`, `CZ̄` on `[[4,2,2]]` | `code_abstraction_tests::test_open_square…` |
| G partial | an input-side entry never copied to fresh wires | `type_alignment_tests::test_extension_copies_output_entries…` |
| H ill-typed | a foreign low-level model at either stage | `circuit_subject_tests::test_an_abstraction_over_another_circuit…` |
| I classical | `Equivalent` only when both sides are classical | `abstraction_tests::test_scope…` |
| J order | stages recorded in order, sticky failure with a zero cap as the sentinel | `circuit_subject_tests` |
| K precision | generic in `R`; exercised at `f64` | deferred to the group 6 consumers |

## Findings the tests forced

| Finding | Fix |
|---|---|
| `permute_legs` multiplied every Kraus operator by two dense permutation matrices: `64 · 16 · 1024²` operations for the opened code's `τ_in`, a minute per gate | an index gather, `O(d_in · d_out)` per operator |
| The opened `[[8,2,2]]` square is a ten-qubit register above the default cap | `check_naturality_on` decides a subset of the signature; the opened square is decided on `[[4,2,2]]` |
| Fresh inputs of an opened mechanism carry its output type | `TypeAlignment::extended` copies output-side entries as two-sided and never input-side ones (D16) |
| `Observe(Ō)` to the logical measurement needs a classical coarse-graining the alignment does not carry | deferred, D16 |

## Mutation testing

The first `cargo mutants` run over the group 1 to 3 kernels (826 mutants) was stopped after eleven
results to fold in group 4's tests; its misses were `abstraction.rs:130` (`&&` to `||` in the scope
decision, now caught by `test_scope_is_equivalent_only_when_both_sides_are_classical`),
`ideal_recovery.rs:72` (`max_correction_weight`, now asserted against the table on `[[8,2,2]]`; a first draft of that assertion claimed `24` weight-one corrections and was wrong, the torus has `16`), and seven
equivalent mutants on `[[4,2,2]]` and `[[8,2,2]]`: sign flips by `±1` written as `*` or `/`, the
imaginary part of real amplitudes, `|` against `^` on distinct bits, and the sign convention of
`Z` on generators of even weight. The `>` against `>=` at the ten-qubit limit has no fixture of
exactly ten qubits and stays open. The rerun over the same files plus `code_abstraction.rs` and
`alignment_structure.rs` runs in the background; its table is appended here when it completes.
