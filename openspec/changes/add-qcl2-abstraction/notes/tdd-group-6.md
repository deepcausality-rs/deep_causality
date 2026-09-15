<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# TDD record, group 6: composition and the three chains

## Provenance of the literals

| Literal | Source | Where used |
|---|---|---|
| `‖U‖_{F→F} = 1` for a unitary, `1` for a state preparation, `1` for the completely depolarising channel (attained on `ρ ∝ I`), `√d` for a partial trace over a `d`-dimensional factor | `Tr_B(X ⊗ I_B) = d · X` against `‖X ⊗ I_B‖_F = √d ‖X‖_F` | `composition_tests::test_frobenius_induced_norm…` |
| a scaled unitary `cU` has norm `|c|²` | the natural representation is `K ⊗ conj(K)` | same |
| `‖J(R_y(a)) − J(R_y(b))‖_F = 2√2 sin((a − b)/2)` | `abstraction_tests` derivation | tightness chain |
| tightness chain, `α = 0`, `α' = 0.3`, `β = 1.4`: `ε₁ = 4 sin 0.15`, `ε₂ = 4 sin 0.55`, measured `4√2 sin 0.7`, bound `4√2 (sin 0.15 + sin 0.55)`; with `pre = 1` `≈ 2.94`, with `post = 1` `≈ 3.55`, measured `≈ 3.64` | tensoring a Choi operator with `I` on a `d`-dimensional factor multiplies its norm by `√d`; the traced wires are fully depolarised below their trace so the post-composition attains `√2` | `test_the_bound_is_tight…` |
| the composite output entry of the concatenation lists eight low wires and `τ` is `256 → 4` | `π = π₁ ∘ π₂` over two blocks of four | `test_exact_links_compose_exactly…` |
| the inner `CZ̄` of `[[4,2,2]]` pairs a qubit of each block | the two homology representatives of the fixture | same |

## Corner-case rows

| Row | Case | Test |
|---|---|---|
| A empty | a law with no rows holds; an empty gadget noise family means no noise box | `chains::code_switching` (clean case) |
| B one | one query per chain | throughout |
| C boundary | `holds` admits the state tolerance; the norm refuses above the entry cap with the exact count | `test_frobenius_induced_norm…` |
| D symmetry-breaking | three distinct angles; `p = 0` against `p = 0.05` | tightness, distillation |
| E error text | a missing image names the query kind; a cross-block gate names both blocks | `test_composition_errors…`, `test_exact_links…` |
| F zero | exact links compose to zero on the concatenated code and the noiseless gadget | `test_exact_links…`, `test_code_switching…` |
| G partial | a two-sided entry over a sided alignment splits into two composite entries | `compose` (code chains) |
| H ill-typed | codes of unequal logical counts refused for switching; a narrower target refused | `chains.rs` |
| I classical | the norm is the largest block norm over classical blocks | `frobenius_induced_norm` doc |
| J order | composite low wires ascending `[0, 1, 2]` and `0..8` | tightness, concatenation |
| K precision | the three examples run the chains at `f32`, `f64` and `Float106` | examples |

## Findings the tests forced

| Finding | Fix |
|---|---|
| The first draft of the concatenation omitted the inner encoder from the low-level model: `ε₁ = 5.57` for `Z̄` | the inner encoder acts on the wires that stand for the middle qubits before the outer encoders |
| The first draft of the tightness oracle stated the bound as `4 (sin 0.15 + sin 0.55)`, dropping the constants | `4√2 (…)`; the code was right |
| An identity alignment on eight qubits fails its section check because the Choi operator has `2^32` entries | `frobenius_distance_by_gram`, exact through `Tr J(A)†J(B) = Σ|Tr A_a†B_b|²`, used by `TypeAlignment` |
| A composed `CZ̄` on the concatenated `[[4,2,2]]` crosses the outer blocks | refused by name; the spec scenario now says `Z̄` and `X̄` |

## Named-defect audit

Twelve defects across the law, the induced norm, the composite alignment and the chains; applied
by `scratchpad/audit_g6/audit.py` with the originals kept in the scratchpad, the composition and
alignment test binaries run under Bazel per defect.

| # | Defect | Class | Result |
|---|---|---|---|
| D1 | constants swapped in the bound | coefficient | caught (`bound = post · ε₁` when `ε₂ = 0`) |
| D2 | conjugate dropped from the natural representation | sign | caught (the phased two-operator family) |
| D3 | first block's norm instead of the largest | loop | caught (the two-block scalar morphism) |
| D4 | composite `τ` skips the first link | skipped case | caught |
| D5 | composite section in the wrong order | ordering | caught |
| D6 | measured residual taken from the second link | interpretation | caught |
| D7 | middle-wire map transposed | index | caught |
| D8 | outer program shifted by `k` instead of `n` | coefficient | **survived** on `Z̄(0)` and `X̄(1)`, whose representatives sit in block 0; caught once `Z̄(1)` and `X̄(0)` joined the test |
| D9 | depolarising amplitude without the square root | normalisation | caught (the Kraus box's trace check) |
| D10 | two-sided entries never split | branch | caught |
| D11 | `‖τ₁‖_pre` taken on the output side | interpretation | caught |
| D12 | Gram distance without the cross term | coefficient | caught |

Twelve of twelve caught after the one fixture widening, which the audit itself forced.

## Verification

`bazel test //deep_causality_quantum/... //lean:Quantum` green (81 targets); the three examples run
under Bazel at `f32`, `f64` and `Float106` with the law holding in every row; the exact links
compose to residual zero at `f64` (`≤ 6 · 10⁻¹⁵`) and exactly at `Float106`; the noisy gadget
gives `ε₁ = 0.4619`, `‖τ₂‖_post = 8` and a measured `0.4619` under the bound `3.695`; the
distillation round at `p = 0.05` gives `ε₁ = 0.768`, measured `0.767` under `1.536`, which says
the `[[4,2,2]]` ideal recovery removes little of a depolarising channel, as a distance-two code
must. The mutation run over the group 1 to 6 kernels is not started in this session.
