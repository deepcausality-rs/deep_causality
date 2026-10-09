<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Quantum backlog

**What this is.** The open items carried out of the quantum notes archived on 2026-10-08, one row
per item. Each row names its source note, now under
[`../archive/quantum/`](../archive/quantum/), and the section the item comes from.

**Status.** Rows marked *verified* were checked against the tree on 2026-10-08. Rows marked *per
source* repeat what the source note says and were not re-checked.

**Live notes keep their own open items**, and this file points at them rather than copying them:

- [`dynamic-qcm.md`](dynamic-qcm.md): tracks Q5 and Q6, T3 to T6, G2 and G3, D1 and D2, C1 to C4,
  F1 to F6.
- [`example-quantum-control-loop.md`](example-quantum-control-loop.md): the calibration example,
  designed and not built.
- [`qcl-next.md`](qcl-next.md): the ranked next steps and the correlated-burst gap of its §11.

---

## 1. QCL pipeline and campaign

| # | Item | State | Source |
|---|---|---|---|
| B-1 | **Plant path on bare metal.** Split the graph-dependent half of `Hypothesis` from the rest, so the plant path compiles without `qcm` and `std`. | open, verified: `pipeline/mod.rs` names it a follow-up | `qcl-design-note.md` §9.2 |
| B-2 | **Instrument-level intervention**, `intervene_instrument(node, instrument)`. A probe is modelled as a mechanism replacement today, and `predict` says so. | open, verified: the name is reserved in `qcm/hypothesis.rs` | `qcl-design-note.md` §9.2; `qcl-corrections.md` X-4 |
| B-3 | **Induced factorization of a composite**, built from the parts' dilations, so a `CertificateNotInherited` failure can be resolved instead of only reported. | open, per source | `qcl-design-note.md` §9.2; `qcl-corrections.md` X-2 |
| B-4 | **`Observe(Ō)` on a code.** The logical measurement needs a classical coarse-graining in the alignment (D16 of the QCL-2 change). | open, verified: no `Observe` in `abstraction/code_abstraction.rs` | `qcl-design-note.md` §9.4 |
| B-5 | **A common-cause candidate as a circuit.** Under the dilation's leg convention a two-output node has a leg of 256 and its children's factors 2^24 entries (D18). | open, verified: `qcl_crosstalk_circuits` keeps H₃ as a factorization | `qcl-design-note.md` §9.4 |
| B-6 | **Mutation runs over the QCL-2 kernels**, with equivalent mutants recorded in `.cargo/mutants.toml` as `AGENTS.md` requires. | open, verified: no quantum entries in `.cargo/mutants.toml` | `qcl-design-note.md` §9.4 |
| B-7 | **Carry the amplification after marginalisation.** A Markov report could survive a trace degraded by `√(d_B)` instead of being discarded. Optional. | open, per source | `qcl-design-note.md` §5.1 |
| B-8 | **A Markov-rejecting plant candidate**, with non-commuting factors on a shared support, so the plant path exercises that gate too. Only if a physically honest case exists. | open, per source | `example-crosstalk-attribution.md` §11 Q4 |
| B-30 | **Cyclic causal structures.** Admit cyclic candidates instead of refusing them at `build()`. Needs a validity condition for cyclic quantum causal models (Barrett, Lorenz & Oreshkov, arXiv:2002.12157, in `deep_causality_quantum/papers/`); a screen to replace C₃-exclusion, which the crate applies to acyclic influence relations only; and the dilation and abstraction layers extended to cycles, which Lorenz & Tull leave as future work (arXiv:2602.16612 §8, with a trace sketch in App. D and Ferradini, Gitton & Vilasini, arXiv:2502.04168, as a starting point). | open, verified: `build()` returns `CyclicStructureUnsupported` | `qcl-design-note.md` §7.6; `example-crosstalk-attribution.md` §7.1 |

## 2. Benchmarks

`deep_causality_quantum` has no benchmarks (verified). The workspace reference machine is the M3 Max,
16 cores, 128 GB.

| # | Item | State | Source |
|---|---|---|---|
| B-9 | **Tick latency.** `bind` on `PropagatingEffect<FloatType>`; `bind` on `PropagatingProcess` with the `Ledger` as state; the log on, bounded and off; a full observe → gate → `alternate_value` tick. Report allocations per tick beside wall clock, read against the 235 µs and 0.2–10 µs code cycles. | open, verified | `qcl-design-note.md` §10.2; `positioning.md` §5.3 |
| B-10 | **Stage cost.** `check_markov` per intersecting pair; `find_c3` swept over the declared system count; `Projection::range_projector`; `design` swept over the hypothesis count `n` to locate the `2^C(n,2)` cliff. | open, verified | `qcl-design-note.md` §10.3 |
| B-11 | **The price of precision.** B-9 and B-10 at `f32`, `f64` and `Float106`, with each scalar's tolerances recorded beside its time. | open, verified | `qcl-design-note.md` §10.4 |
| B-12 | **A bounded-time log for a fast loop.** `EffectLog::add_entry` allocates a `String` per entry. A fixed-capacity ring of event codes written by the fast loop and drained into the full record by the slow loop. | open, verified: `log_effect.rs` pushes `message.to_string()` | `positioning.md` §5.3 |

## 3. Code checks and QEC

| # | Item | State | Source |
|---|---|---|---|
| B-13 | **`check_degeneracy` on a detector error model.** Group mechanisms by identical target set; a group of two or more cannot be separated by syndrome data alone. Report the class and name the experiment that breaks it. | open, verified: no such function | `example-geometric-qec.md` §8.4 |
| B-14 | **DEM export.** Emit Stim-readable text from a derived code plus a noise model. | open, verified: `DemModel` imports only | `example-geometric-qec.md` §8.5 |
| B-15 | **Code distance by enumeration** on small complexes. `qcl_geometric_qec` prints "no distance claimed". | open, verified | `example-geometric-qec.md` §1.1, §11 item 3 |
| B-16 | **Executed genericity.** A hand-built non-lattice `SimplicialComplex` in the code example, so more than one `ChainComplex` implementor runs. | open, per source | `example-geometric-qec.md` §12 Q2 |
| B-17 | **A failing class-invariance case**, physically honest and not manufactured. Optional. | open, per source | `example-geometric-qec.md` §12 Q4 |
| B-18 | **The gate catalogue.** Higher cup products and Steenrod squares, then the addressable-generator inventory. | open, verified: no Steenrod code in topology; tracked in `dynamic-qcm.md` SPEC-T3 to T6 | `example-geometric-qec.md` §6.4 |
| B-31 | **A reference decoder for SPEC-D1.** An exact maximum-likelihood decoder over a `DemModel`, by enumerating mechanism subsets within `DEM_MAX_MECHANISMS`, as a function rather than a trait. It decodes under the correlated model a candidate names, which is what assumption A7 of `dynamic-qcm.md` asks to measure against an independent-error decoder. A real-time decoder stays out: its budget is microseconds on dedicated hardware, and QCL runs at supervisory rate (`positioning.md` boundary 8). | open, verified: the crate holds no decoding algorithm (`decoder_abstraction.rs`, `dem_model.rs`) | `dynamic-qcm.md` Track D; `qcl2-roadmap.md` D2-5 scoped decoding out of QCL-2 only |

## 4. Formalization and bounds

| # | Item | State | Source |
|---|---|---|---|
| B-19 | **Lean: the dilation of a circuit is Markov for its induced DAG** (Barrett–Lorenz–Oreshkov). Asserted in Rust at Q-TOL on every fixture. | open, per source | `qcl2-roadmap.md` §2, §10 |
| B-20 | **Diamond norm as an optional `Tolerance` member**, an SDP. No SDP exists in the workspace; the Frobenius-to-diamond bound ships. Optional. | open, per source | `qcl2-roadmap.md` D2-2, §11; `qcl2-roadmap-verification.md` V-9 |
| B-21 | **A write-up of the ε-abstraction**, its definition and composition law, reviewable outside the code. | not checked whether the archived change's notes cover it | `qcl2-roadmap.md` §11 |
| B-22 | **The deferred Lean targets** of `deep_causality_quantum/LEAN_QUANTUM.md`: the Choi reconstruction, `quantum.markov_commutativity`, `quantum.verdict.orthomodular` and the rest. | open; tracked in `LEAN_QUANTUM.md` | `qcl-design-note.md` §9.1 |

## 5. Beyond the quantum crate

| # | Item | State | Source |
|---|---|---|---|
| B-23 | **`deep_causality_do_calculus`**: do-operator, counterfactuals, d-separation and identification over the hypergraph. Identification is `dynamic-qcm.md` SPEC-F2. | open, verified: the crate does not exist | `full-stack.md` §7; `positioning.md` §4 (C6) |
| B-24 | **Relativistic order.** Derive causal order from invariants (Minkowski interval → timelike or spacelike → `bind` or `∇`), and check frame covariance and no-signalling at freeze. | open, per source | `full-stack.md` §3 level 2, §4 invariant 2 |
| B-25 | **Classical indefinite causal order**, a convex mixture over orders through `Uncertain` and `continue_with`. | open, per source | `full-stack.md` §3 level 3 |
| B-26 | **The direct-sum generator `⊕`** for causally faithful decompositions beyond sequential and tensor composition (Lorenz–Barrett 2021). Open upstream. | open, per source | `full-stack.md` §3 level 4a, §7 |
| B-27 | **A worked audit trail** for quantum-derived evidence: which backend, calibration, shots and post-processing a decision rested on. | open, per source | `positioning.md` §4 (C5) |
| B-28 | **A fusion example**: MHD plasma confinement with a disruption gate, derived from `grmhd`. CFD scope. | open, per source | `quantum-epp.md` §9.6 |
| B-29 | **A QPU-calling solve**: a linear-solve or QLBM causaloid for one CFD timestep. Hardware-gated. | open, per source | `quantum-epp.md` §9.6 |

## 6. Settled by decision

Listed so they are not re-added as open work.

- **Nesting by abstraction across a partial trace** is refused: preservation is false, with a Lean
  counterexample. Composition and the boundary warrant ship instead (`qcl-design-note.md` §5.1).
- **Vendor QPU adapters**: the `qpu` feature is a typed seam with `SimQpu` behind it (`quantum-epp.md`
  §5).
- **Native dagger-compact quantum semantics** and coherence across causaloids are outside the
  classical substrate (`full-stack.md` §6).
- **`positioning.md` §11's site decisions** were overtaken by the site rebuilt around
  `qcl_crosstalk`; see [`../quantum-site/`](../quantum-site/).
