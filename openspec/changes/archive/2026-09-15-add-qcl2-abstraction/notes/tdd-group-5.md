<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# TDD record, group 5: fault sets and the fault-tolerance predicate

## Provenance of the literals

| Literal | Source | Where used |
|---|---|---|
| `C(8, 1) · 3 = 24`, `C(4, 2) · 9 = 54`, `C(10, 3) · 27 = 3240`, `C(18, 1) · 3 = 54` | binomial by hand | `fault_set_tests`, `fault_tolerance_tests` |
| `C(200, 100) · 3^100` overflows `u64` | `3^100 > 2^64` | `fault_set_tests` |
| `T̄` remainder table `[1/8, 7/8]`, two Pauli terms of modulus `1/√2`, at `w = 3` and `w = 4` | Haruna Eq. (3.63): `exp(iπ/4) · O_2(γ)†` under `⟨a, γ⟩ = 1`; `notes/open-questions-resolved.md` §3 | `fault_tolerance_tests` |
| `S̄` remainder table `[1/4, 3/4]`, one Pauli term | `exp(iπ/2) · O_1(γ)† = i · Z̄(γ)` | `fault_tolerance_tests` |
| `CZ̄` remainder `[0, 0, 1/2, 1/2]` under a fault on `γ₁`, `[0, 1/2, 0, 1/2]` on `γ₂` | `exp(iπ p₁ p₂)` conjugated: `p₁ ↦ 1 − p₁` leaves `exp(iπ p₂)` up to phase | `fault_tolerance_tests` |
| `Z̄`, `X̄` hold; `S̄`, `T̄`, `H̄`, `CZ̄` fail under weight one on both tori | the derivation above; `H̄` by tableau spreading | `fault_tolerance_tests::test_haruna_filter…` |
| Representatives of weight 3 on the `3 × 3` torus and 4 on the `4 × 4` | a straight cycle of `l` edges | `fault_tolerance_tests` |
| A Z fault on a traced wire has residual zero; an X on the kept wire above one | `τ` traces wire 1; `‖J(X R_y) − J(R_y)‖_F = 2√2 sin(θ'/2)` with `θ'` the angle between the two rotations | `fault_tolerance_tests::test_numeric_path…` |

## Corner-case rows

| Row | Case | Test |
|---|---|---|
| A empty | `declared(&[])` reads `Vacuous`, examined zero | `test_an_empty_fault_set_is_vacuous` |
| B one | weight-one faults, one error per fault | throughout |
| C boundary | count exactly at the cap admitted; above refused by count | `test_above_the_cap…` |
| D symmetry-breaking | `X0 Z2` against `Z0 X2` through the X/Z parts; X against Y against Z digits | `test_a_fault_reads…`, `test_weight_one…` |
| E error text | count and cap; `wire 3` twice; `node 7`; `not a quantum wire` | `fault_set_tests`, `test_the_faulted_model…` |
| F zero | zero residual on tolerated faults on both paths | both paths' tests |
| G partial | faults on a subset of the register; unsorted, non-contiguous locations | `test_counts_follow…` |
| H ill-typed | a fault over the wrong register; on a classical wire | `test_an_empty…`, `test_the_faulted_model…` |
| I classical | `from_dem` mechanisms with empty entries skipped | `test_declared_and_dem…` |
| J order | faults sorted by wire; sets keep declaration order; the inserted node's box index | `fault_set_tests`, `test_the_faulted_model…` |
| K precision | `Turns` exact on the exact path; `f64` on the numeric path | as above |

## Named-defect audit

Twelve defects across the fault set, the inserted fault node and both decision paths; applied by
`scratchpad/audit_g5/audit.py` with the originals kept in the scratchpad, the two fault test
binaries run under Bazel per defect.

| # | Defect | Class | Result |
|---|---|---|---|
| D1 | binomial step divides by `i + 2` | coefficient | caught |
| D2 | cap refuses at equality | boundary | caught |
| D3 | Pauli pattern digit rotated | ordering | caught |
| D4 | X and Z parts swapped in the Pauli | interpretation | caught |
| D5 | fault inserted before the node's last box | index | caught |
| D6 | weight growth ignored in the decision | branch | caught |
| D7 | weight bound strict | boundary | caught |
| D8 | exact residual inverted | sign | caught |
| D9 | numeric square ignores the fault | skipped case | caught |
| D10 | parity flips read from the Z part | interpretation | caught |
| D11 | DEM duplicates kept | loop | caught |
| D12 | filter inverts `holds` | branch | caught |

Twelve of twelve caught on the first pass.

## Oracle correction

The first draft of the `S̄` test expected the remainder table `[0, 1/2]`, the table of `Z̄(γ)`.
The remainder is `exp(iπ/2^{k−1}) · O_{k−1}(γ)†`, so it carries the global phase `1/4` and reads
`[1/4, 3/4]`; the code was right and the test was corrected, with the derivation now in the test's
header and in the spec.

## Mutation testing

The `cargo mutants` run over the group 1 to 4 kernels continues in the background; `fault_set.rs`
and `fault_tolerance.rs` join the file list on its next run, and the tables are appended to this
note and to `tdd-group-4.md` when the runs complete.
