<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# TDD record, group 7: the decoder as an abstraction

## Provenance of the literals

| Literal | Source | Where used |
|---|---|---|
| two mechanisms `p₁ = 0.1` on `D0 D1`, `p₂ = 0.2` on `D1 L0`: `P(000) = 0.72`, `P(110) = 0.08`, `P(011) = 0.18`, `P(101) = 0.02`; firing the second shifts by `011` | independent Bernoulli products by hand | `dem_model_tests` |
| the memory experiment's nominal string: `(1 − p)³(1 − p_c) + p²(1 − p) p_c` | the empty subset and `{E0, E1, E01}`, the one subset whose flips cancel | `dem_model_tests` |
| record `(a0, a1, m0, m1, m2)` with `a0 = q0 ⊕ q1`, `a1 = q1 ⊕ q2`, `m_i = q_i` | only `X` noise and `Z` readout: the circuit is a classical stochastic process | fixture doc, `decoder_abstraction_tests` |
| the omitted-mechanism residuals: nominal mismatch `0.024` on every square, `1.19` at the injected location | `‖P ∘ f − Q ∘ f‖ = ‖P − Q‖` for the modelled locations; a shifted record against an unshifted prediction otherwise | `decoder_abstraction_tests` |
| attribution: the correlated fault ranks first with `p = 0.05`, `p_c = 0.02` | the model assigns first-order weight `p` to each single-qubit pattern and second-order `p²` to `D1 L0`, so `p_c < 2p` makes the correlated fault the largest surprise; `Z` faults leave a `Z`-basis record unchanged | `decoder_abstraction_tests` |
| Stim: `error(0.01) D0 D1`, `error(0.02) D1 L0`, `detector D0` gives two mechanisms, detectors `D0 D1`, observable `L0` | the spec scenario | `dem_model_tests` |

## Corner-case rows

| Row | Case | Test |
|---|---|---|
| A empty | a decoder abstraction with no locations has the `Io` query alone; an empty gadget family | `test_the_decoder_enters…` |
| B one | one observable; one phantom | `dem_model_tests` |
| C boundary | `DEM_MAX_MECHANISMS + 1` refused; probability `1.5` refused; a row summing to `1.5` refused | `dem_model_tests`, `decoder_abstraction_tests` |
| D symmetry-breaking | `D0 D1` against `D1 L0` mechanisms; `(1, 0, 1, 1, 0)` reads as `(1, 0, 1)` and not `(0, 0, 0)` | both |
| E error text | `line 2: \`repeat 3 {\``, `mechanism 5`, `two roles`, `node 9`, `row 3`, `must be X`, `latent` | both |
| F zero | the complete model passes at `10⁻¹²`; a phantom changes nothing | both |
| G partial | comments, blank lines and detector coordinates in Stim text | `dem_model_tests` |
| H ill-typed | a matrix of the wrong shape; a fault with a node on a latent | both |
| I classical | the model is classical; its `Io` morphism is `1 → 1` with eight blocks | `dem_model_tests` |
| J order | attribution descending throughout; the worst failure first among equals | `decoder_abstraction_tests` |
| K precision | `f64`; the amplitudes are `√p` lifted through `from_f64` | throughout |

## Findings the tests forced

| Finding | Fix |
|---|---|
| The first test claimed the three modelled locations still commute when a mechanism is omitted; they carry the nominal mismatch, since a fault shifts both sides alike | the test asserts equal residuals to `10⁻¹²` at the modelled locations and a worst failure at the injected one; the spec scenario now says so |
| The first nominal-string oracle omitted the subset `{E0, E1, E01}` whose flips cancel | `+ p² (1 − p) p_c` |
| `detector(1, 2) D3` split at the space inside the parenthesis | the argument is read off the line before tokenising |
| The record opens `2^9` branches and `2^14` traced operators, above the operator cap, though most are exactly zero | exact pruning of zero branches and zero traced operators; the fixture raises the cap |

## Named-defect audit

Twelve defects across the model, the parser, the decoder lift, the alignment's classical map, the
attribution and the pruning; applied by `scratchpad/audit_g7/audit.py` with the originals in the
scratchpad, three test binaries run under Bazel per defect.

| # | Defect | Class | Result |
|---|---|---|---|
| D1 | mechanism weight `p` for the non-firing branch | coefficient | caught |
| D2 | the fault shift dropped | skipped case | caught |
| D3 | observables land on detector bits | index | caught |
| D4 | phantom fires with probability one | interpretation | caught |
| D5 | graph edges read backwards | interpretation | caught |
| D6 | unknown directives accepted | branch | caught |
| D7 | stochastic rows not checked | branch | caught |
| D8 | matrix read transposed | interpretation | caught |
| D9 | the classical output map ignored | skipped case | caught |
| D10 | attribution ascending | ordering | caught |
| D11 | pruning inverted | branch | caught |
| D12 | worst failure the smallest | ordering | caught |

Twelve of twelve caught on the first pass. The mutation run over the kernels is not started in
this session.
