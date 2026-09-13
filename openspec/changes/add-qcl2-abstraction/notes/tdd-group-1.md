<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# TDD record, group 1: the circuit model and its two semantics

The unified-math protocol (`openspec/changes/archive/2026-09-08-unified-math-next/tdd/`) applied to
the five new kernels of group 1: `gate_unitary`, the Kraus-level evaluation in `numeric_semantics`,
the `QcMorphism` carrier with its two caps, `GaugeFieldGate`, and the exact propagation in
`exact_semantics`.

## Provenance of the literals

| Test | Expected value | Source |
|---|---|---|
| gate matrices | `H`, `X`, `Y`, `Z`, `S`, `T`, `CNOT`, `CZ` entries | Nielsen & Chuang §4.2, the published matrices |
| `CZ` Choi entries `((3,3),(0,0)) = −1`, `((0,1),(0,1)) = 0` | closed form `J[(i,i),(k,k)] = u_i ū_k` for a diagonal unitary | the crate's Choi convention written out for a diagonal `U`; not a reading of `choi_from_kraus` |
| amplitude damping | Kraus `[[1,0],[0,√0.7]]`, `[[0,√0.3],[0,0]]` | Nielsen & Chuang §8.3.5 |
| `‖J(id) − J(Z)‖_F = 2√2` | four off-diagonal entries of modulus 2 | hand count on the `4 × 4` Choi operators |
| `Z̄`, `S̄`, `T̄`, `CZ̄` phase tables | `1/2`, `1/4`, `1/8`, `1/2` on the odd pattern | Haruna Table 1 column 3, Eqs. 3.14, 3.37, 3.56, 3.63 |
| `T̄` remainder `1/8`, `7/8` and coefficients `1/√2`, `i/√2` | `exp(iπ/4 · (1 − 2p))` | derived in `open-questions-resolved.md` §3 and checked in numpy before the Rust existed |
| `S̄² = Z̄`, `T̄² = S̄` | Clifford hierarchy identities | textbook |
| `[[4,2,2]]` parameters | `n = 4`, `k = 2`, checks `ZZZZ`, `XXXX` | Gottesman, the smallest CSS code; `∂₁∂₂ = 0` by hand |
| `2^40` entries refused | `2^(2·18 + 2·2)` | arithmetic |

The strongest test is `exact_semantics_tests::test_t_bar_remainder_matches_the_numeric_conjugation_at_three_weights`:
the exact carrier's prediction `X_q · exp(±iπ/4 Z̄(γ))` is built into a matrix from its phase table and
compared entry for entry with `U X_q U†` where `U` is the physical `T̄` program's unitary from the
Kraus-level kernel. The two sides come from different algorithms (a `2^m`-entry phase table against a
`2^w × 2^w` unitary product), at `w = 3, 4, 5`.

## Corner-case rows

| Row | Case | Test |
|---|---|---|
| A empty | circuit with no boxes; empty graph; empty Kraus family; `Inc` with no sets | `numeric_semantics_tests::test_no_boxes…`, `induced_dag_tests::test_empty_graph…`, `qc_morphism_tests::test_push_validates…` |
| B single | one box; one vertex; one block | `circuit_model_tests::test_no_boxes_and_one_box`, `induced_dag_tests::test_single_vertex` |
| C coinciding | start equals target in `reaches_avoiding`; the same block on both sides of a product | `induced_dag_tests::test_single_vertex`, `gauge_field_gate_tests::test_hierarchy_identities…` |
| D degenerate index | a wire listed out of ascending order in a box; `Cnot` with control above target | `numeric_semantics_tests::test_gate_on_a_subset…`, `gate_unitary_tests::test_cnot_respects…` |
| E thresholds | entry cap at 16 and 15 for a 16-entry Choi | `qc_morphism_tests::test_entry_cap_is_exact_at_the_boundary` |
| F zero | even-overlap fault leaves a zero remainder; Z-type fault | `exact_semantics_tests::test_even_overlap…` |
| G negative | `−1/8` turn reduced to `7/8`; a negative denominator | `gauge_field_gate_tests::test_construction_errors_and_reduction` |
| H exact boundary | a classical value equal to its outcome count | `qc_morphism_tests::test_push_validates…` |
| I non-finite | n/a: every quantity is a count, a rational or an exact matrix entry | — |
| J overflow reach | `2^40` entries saturating past the cap; a dimension product that would overflow | `numeric_semantics_tests::test_eighteen_qubits…` |
| K every precision | the kernels are generic in `R`; the consumers of group 6 run them at `f32`, `f64`, `Float106` | deferred to group 6 |

## Named-defect audit

Twelve defects, one at a time, each file restored byte for byte after its run
(`scratchpad/audit_g1.sh`). Every defect was caught by at least one test whose subject is the
defective behaviour; the one that was not on the first pass gained its test.

| # | Defect | Class | Caught by |
|---|---|---|---|
| D1 | `Cnot` control and target bits swapped | index | `test_cnot_respects_ascending_order…`, `test_gate_on_a_subset…` |
| D2 | `apply_small` reads the transposed gate entry | index | `test_channel_box_agrees_with_apply_kraus…`, `test_encoder_and_measurement…` |
| D3 | measurement Kraus `|y⟩⟨0|` instead of `|0⟩⟨y|` | index | `test_encoder_and_measurement…` |
| D4 | `O_k` phase `1/2^{k−1}` instead of `1/2^k` | coefficient | 8 tests, first `test_t_bar_remainder…` |
| D5 | remainder `φ(idx) − φ(idx ⊕ flips)` | sign | `test_conjugation_flips_parities`, `test_t_bar_remainder…` |
| D6 | entry cap `>` to `>=` | boundary | `test_entry_cap_is_exact_at_the_boundary` |
| D7 | `reduce_turns` with `%` instead of `rem_euclid` | sign | `test_construction_errors_and_reduction` |
| D8 | `choi_entries` without the block multiplier | normalisation | **survived**; `test_entry_count_multiplies_by_the_block_count` added |
| D9 | `is_constant` always true | plausible neighbour | `test_conjugation_flips_parities`, `test_program_outside_the_normal_form…` |
| D10 | traced-leg offset dropped in the Kraus assembly | skipped case | `test_no_boxes_is_the_identity_and_traced_legs_are_summed` |
| D11 | diagonal phase on the all-but-one state | index | `test_diagonal_family_phases_the_all_ones_state_only`, `test_cz_choi…` |
| D12 | parity flips read from the Z part | branch | four `exact_semantics_tests` |

## Mutation testing

`cargo mutants` over the six kernel files, 497 mutants, run after the audit; results in the
section below once the run completes.
