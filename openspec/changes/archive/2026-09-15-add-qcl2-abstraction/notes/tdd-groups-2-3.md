<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# TDD record, groups 2 and 3: the dilation, the queries and the abstraction

## Provenance of the literals

| Test | Expected value | Source |
|---|---|---|
| Born rule on the two-rotation chain | `sin²(0.8)` for `R_y(0.7)` then `R_y(0.9)` on `|0⟩` | half-angle rotation, `R_y(θ)|0⟩ = cos(θ/2)|0⟩ + sin(θ/2)|1⟩`; the angles are asymmetric so no halving or sign error reproduces the number |
| root factor `|0⟩⟨0| ⊗ I` on `(in, out)` | entries `((0,0),(0,0)) = 1`, `((1,0),(1,0)) = 0`, trace 2 | the dilation's own definition of a fresh line, written out |
| declared input as `I/2 ⊗ I` | diagonal `1/2`, trace 2 | definition |
| opening the last node | identity on the fresh input | Lorenz & Tull §7.2: deleting `c_X` and making `X` an input |
| interchange on the first node | `sin²(0.8)` from the copy's `|0⟩`, `cos²(0.8)` from the copy's `|1⟩`, whatever the main input | §7.2's interchange channel: the main input to the set is discarded and the copy's output takes its place |
| `‖J(H) − J(I)‖_F = √8` | each unitary Choi has squared norm `d² = 4`, cross term `|Tr H|² = 0` | hand computation, corrected in the run: the first draft wrote 2 |
| entangling residual `2√2` | `8 + 8 − 2·4` with Kraus `K_b = I ⊗ ⟨b|` and `Tr(CNOT (I ⊗ |b⟩⟨b'|)) = 1` for every `(b, b')` | hand computation on the two-qubit input space, corrected in the run: the first draft used a one-qubit input |
| `R_z(θ)` pair | residual `2√2 sin(θ/2)`, diamond `2 sin(θ/2)`, upper bound loose by `2√2` | closed forms in `open-questions-resolved.md` §2 |
| opening equals mechanism replacement | `sin²(0.25)` on both sides for the input `R_y(0.5)|0⟩` | Kraus semantics on the rewired circuit against `Re Tr(σ τ)` with the factor at node 1 replaced; no shared code |

Two of the first-draft oracles were wrong and the code was right. The Frobenius norm of a unitary
Choi operator is `d`, not `√d`, and the entangling square's input is the two-qubit low-level space,
not the one-qubit high-level one. Both are recorded here because the protocol's point is that the
expected value be derived, and a derivation can be wrong in ways a run exposes.

## Corner-case rows

| Row | Case | Test |
|---|---|---|
| A empty | opening every node; an empty signature; an interchange on a model with no inputs | `queries_tests::test_opening_the_first_node…`, `abstraction_tests::test_empty_signature…`, `queries_tests::test_interchange_feeds…` |
| B single | a single-node model dilated; one alignment entry | `dilation_tests::test_declared_input…`, `type_alignment_tests` |
| C coinciding | the same node in two interchange sets | `query_tests::test_interchange_on_a_chain…` |
| D degenerate order | alignment entries given out of wire order; crossed high/low order | `type_alignment_tests::test_tau_is_assembled_in_ascending_wire_order` |
| E thresholds | the section check at `√8` against the state tolerance | `type_alignment_tests::test_a_section_that_does_not_invert…` |
| F zero | zero residual on a commuting square; zero-residual diamond bound | `abstraction_tests::test_a_commuting_square…`, `diamond_bound_tests` |
| G partial | a query renaming an aligned type in part; a type requested in part | `type_alignment_tests::test_extension_follows_a_renaming_as_a_whole`, `…ascending_wire_order` |
| H ill-typed | a low-level query whose output type is not `π` of the high-level one | `abstraction_tests::test_observe_and_a_failing_swapped_program` |
| I classical | outcome wires matched by count; encoders reading unwritten wires | `abstraction_tests`, `numeric_semantics_tests` |
| J cycles | a cyclic grouping refused by the dilation and at `build()` | `dilation_tests`, `circuit_subject_tests` |
| K precision | generic in `R`; exercised at three scalars by the group 6 consumers | deferred |

## Named-defect audit, group 3

Thirteen defects across the dilation's consumers, the alignment, the abstraction, the naturality
check and the bound; results appended below by `scratchpad/audit_g3.py`.

| # | Defect | Class | Caught by |
|---|---|---|---|
| G1 | diamond lower bound multiplies by `d_in` | coefficient | both `diamond_bound_tests`, the entangling-square bound |
| G2 | amplification without the square root | normalisation | both `diamond_bound_tests`, the entangling-square bound |
| G3 | naturality swaps measured and threshold | index | three `abstraction_tests` |
| G4 | covering accepts a partial type | boundary | `test_tau_is_assembled_in_ascending_wire_order` |
| G5 | assembly skips the leg permutation | ordering | `test_tau_is_assembled_in_ascending_wire_order` |
| G6 | section check never fires | branch | `test_a_section_that_does_not_invert…` |
| G7 | opening forgets the fresh inputs | skipped case | four tests across `queries_tests` and `abstraction_tests` |
| G8 | interchange inserts no swap | skipped case | `test_interchange_feeds_the_copy…` |
| G9 | `permute_legs` drops the input permutation | index | `…ascending_wire_order`, `…leg_permutation` |
| G10 | `tensor` reverses the Kronecker order | ordering | **survived**: the test compared two results that both went through `tensor`; `test_tensor_order_against_the_kronecker_product` added with the tensor crate's `kronecker` as oracle |
| G11 | `square` ignores the query's renaming | skipped case | `…zero_residual_on_io_and_open`, `…concrete_do…` |
| G12 | `concrete_do` plugs the high-level state into the right side | interpretation | `…concrete_do…` (the defect the run itself found and fixed) |
| G13 | classical identity emits one block | loop | `…observe…`, `…leg_permutation` |

Twelve of thirteen caught on the first pass; the thirteenth was a symmetric-comparison test of the
kind the workflow warns about, and it now has an independent oracle.

## Closure of group 3 (2026-09-09)

Three defects the numeric code path found in its own design, recorded here because the fix changed
group 3 types:

| Finding | Fix | Test |
|---|---|---|
| `Channel` for the eight-qubit encoder forms a `2^32`-entry Choi operator; each `IdealRecovery` and `CodeAbstraction` test ran over a minute | `τ`, `E` and the encoder are `QcMorphism` values; the encoder sits in a new `CircuitBox::Kraus` | `test_encoder_is_unitary_and_extends_the_isometry`, `test_kraus_box_agrees_with_apply_kraus_on_one_qubit`, `test_mis_dimensioned_kraus_box…` |
| Gram–Schmidt completion of the encoder is `O(d³)` | the coset-character basis of the `X`-stabilizer group, exact and `O(d · |S_X|)` | `test_encoder_is_unitary…` (W†W = I on `[[4,2,2]]`), `test_eight_two_two_encoder_columns` |
| The code square posed on the full physical input space: residual `30.5` for `S̄` on `[[8,2,2]]` | Example 58 shape with sided alignment entries (`AlignmentSide`) | `test_sided_entries_apply_to_their_side_only`, `code_abstraction_tests::test_numeric_path_agrees…` |

Oracles for the new tests: the written-out identity for `W† W`, the isometry's columns (themselves
checked against Pauli corrections), hand-computed amplitude-damping output, and error text naming
the box and wires. No test compares two results that share a code path.

## Mutation testing

`cargo mutants` over the group 1 to 3 kernel files was started after the audit, in the background,
with its output in the session scratchpad; a first partial run before a session restart caught 64
of 74 mutants and the ten misses were closed by the discriminating DAG tests and two removed
redundant guards. The complete run's table is appended here when it finishes.
