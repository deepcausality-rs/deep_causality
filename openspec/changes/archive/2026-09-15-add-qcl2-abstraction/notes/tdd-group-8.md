<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# TDD record, group 8: the crosstalk consumer over circuits, and close-out

## The consumer

`examples/quantum_examples/qcl_examples/qcl_crosstalk_circuits/` re-expresses `H₁`, `H₂` and the
cyclic `H₄` as `CircuitModel` values and keeps `H₃` as the v1 factorization (design D18). The run
asserts what the spec scenario asks: the cycle refused at `build()` as
`CyclicStructureUnsupported`, two circuits screened through their dilations (`check_markov`
accepted, `check_decomposable` vacuous on a two-node boundary), three candidates admitted on the
plant subject, the plan `E1 do(Q1)` and `E2 do(Q2)` at cost 2 against tomography at 200, and
`H1 Q1->Q2` the survivor at 100.1 bits. The v1 example stands beside it unchanged.

| Literal | Source |
|---|---|
| leg dimension 16 per single-wire node, conditional factor 256 entries | D3's `(d_in · d_out)²` with `d_in = d_out = 2` |
| a two-output bath node's leg 256, its single-wire children's factors `2^24` entries, the Markov union `2^32` | the same convention with `d_in = d_out = 4` for the bath, then `(256 · 16)²` and `(256 · 16 · 16)²` |
| plan cost 2, tomography 200, survivor at 100.1 bits | the v1 example's probes and floor, unchanged |

## Close-out checks

| Check | Result |
|---|---|
| `bazel test //...` | 1407 targets green |
| `bazel test //deep_causality_quantum/... //lean:Quantum` | 82 test targets and the Quantum proofs green |
| `make check_examples` | every Cargo example has a Bazel target, 46 manifests |
| `cargo fmt --all --check` | clean |
| `openspec validate --specs` | 201 passed; the change validates |
| default build of `deep_causality_quantum` | green |
| `cargo clippy --workspace --all-targets` | clean, no warnings |
| `no-std` build of `deep_causality_quantum` | **blocked outside the crate**: `deep_causality_stats/src/utils_tests/sampling.rs` uses `std::thread` and `Vec` without the `alloc` import under `no-std`; the quantum crate's abstraction layer itself is ungated and `alloc`-only |

## What is left after the change

The `Observe(Ō)` query on a code (D16), the common-cause candidate as a circuit (D18), and the
`cargo mutants` runs over the kernels, whose named-defect audits ran for every group but whose
mutation tables were not completed in the sessions that built them.
