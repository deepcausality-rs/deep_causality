<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Lorenz gap: QCL against Robin Lorenz's quantum causal models

**What this is.** A comparison of QCL in `deep_causality_quantum` with Robin Lorenz's published work
on quantum causal models, with particular attention to Barrett, Lorenz & Oreshkov, *Cyclic quantum
causal models* (Nat. Commun. 12, 885, 2021). Most of what the crate builds on monadic structure
comes from this line of work. The papers are in [`Lorenz/`](Lorenz/).

**Status.** Assessment, 2026-10-08, against `main` plus the working tree. No code written. Read in
full: the cyclic paper, and earlier this session the abstraction paper. Read in part: van der Lugt
& Lorenz v2 (§1 to §4 and the discussion), Barrett, Lorenz & Oreshkov 2019 (the outline, §11 and
§13.2), the thesis (the outline and chapter 8), Lorenz & Barrett 2021 (the abstract and its
hypothesis), and the Synthese paper (the definitions and §5). The 2023 and 2024 papers were
collected as context and not read.

**Finding.** QCL implements the acyclic core of the framework: the commuting-factor Markov
condition, the C₃ screen, the mechanism-level `do`, the dilation of circuits, and abstraction. It
implements none of the cyclic extension, and only part of the acyclic framework around the Markov
check. The cyclic paper also changes what a check must establish: once cycles are admitted, a
product of commuting, locally normalised channels is no longer automatically a valid process. That
has a direct consequence for `qcl_crosstalk`. Its cyclic candidate H₄ is not a valid process at all
(§3.1), so it can be rejected on physical grounds rather than refused by scope.

---

## 1. The corpus

Every causal-model publication on Lorenz's Google Scholar profile, plus the two compositional-
framework papers the 2026 abstraction paper builds on.

| Year | Paper | File |
|---|---|---|
| 2019 | Barrett, Lorenz & Oreshkov, *Quantum causal models*, arXiv:1906.10726v2 | [`2019_…_Quantum-causal-models_…pdf`](Lorenz/2019_Barrett-Lorenz-Oreshkov_Quantum-causal-models_arXiv-1906.10726v2.pdf) |
| 2020 | Lorenz & Barrett, *Causal and compositional structure of unitary transformations*, Quantum 5, 511 (2021), arXiv:2001.07774v2 | [`2020_Lorenz-Barrett_…pdf`](Lorenz/2020_Lorenz-Barrett_Causal-and-compositional-structure-of-unitary-transformations_arXiv-2001.07774v2.pdf) |
| 2020 | Barrett, Lorenz & Oreshkov, *Cyclic quantum causal models*, Nat. Commun. 12, 885 (2021), arXiv:2002.12157v3 | [`2020_…_Cyclic-quantum-causal-models_…pdf`](Lorenz/2020_Barrett-Lorenz-Oreshkov_Cyclic-quantum-causal-models_arXiv-2002.12157v3.pdf) |
| 2020 | Lorenz, *Quantum causal structure*, DPhil thesis, University of Oxford | [`2020_Lorenz_…_DPhil-thesis-Oxford.pdf`](Lorenz/2020_Lorenz_Quantum-causal-structure_DPhil-thesis-Oxford.pdf) |
| 2022 | Lorenz, *Quantum causal models: the merits of the spirit of Reichenbach's principle for understanding quantum causal structure*, Synthese 200:424, doi:10.1007/s11229-022-03887-5 | [`2022_Lorenz_…_Synthese-200-424.pdf`](Lorenz/2022_Lorenz_Quantum-causal-models-Reichenbach_Synthese-200-424.pdf) |
| 2023 | Lorenz & Tull, *Causal models in string diagrams*, arXiv:2304.07638 | [`2023_Lorenz-Tull_…pdf`](Lorenz/2023_Lorenz-Tull_Causal-models-in-string-diagrams_arXiv-2304.07638v1.pdf) |
| 2024 | Tull, Lorenz, Clark, Khan & Coecke, *Towards compositional interpretability for XAI*, arXiv:2406.17583 | [`2024_…_XAI_…pdf`](Lorenz/2024_Tull-Lorenz-Clark-Khan-Coecke_Towards-compositional-interpretability-for-XAI_arXiv-2406.17583v1.pdf) |
| 2025 | van der Lugt & Lorenz, *Unitary causal decompositions: a characterisation via lattice theory*, Quantum (accepted 2026-09-09), arXiv:2508.11762v2 | [`2025_vanderLugt-Lorenz_…pdf`](Lorenz/2025_vanderLugt-Lorenz_Unitary-causal-decompositions_arXiv-2508.11762v2.pdf) |
| 2026 | Lorenz & Tull, *Causal and compositional abstraction*, arXiv:2602.16612 | [`2026_Lorenz-Tull_…pdf`](Lorenz/2026_Lorenz-Tull_Causal-and-compositional-abstraction_arXiv-2602.16612v1.pdf) |

Left out of the folder: the QNLP papers, lambeq, the two particle-physics works, and arXiv:2609.00372
(August 2026, quantum representation learning, with no causal content). The Scholar entry *Quantum
causal loops* (2021) is the press release for the cyclic paper, not a separate paper.

---

## 2. Lorenz's position today

- **Causal structure is the no-influence structure of an underlying unitary.** A process is the
  marginal of a unitary process; a model is Markov when the process factorises into pairwise
  commuting channels; and for acyclic graphs, Markov is equivalent to compatibility with a unitary
  (2019, Theorem 4.10).
- **The cyclic extension admits any directed graph** (2021, Definitions 1 and 2), requires that the
  product of the factors be a valid process operator, and shows the quantum SWITCH and the
  Araújo–Feix process to be faithful cyclic models. For a unitary process, causal nonseparability is
  equivalent to a cyclic causal structure (Theorem 4). Whether Markov implies compatibility for
  cyclic graphs is open (Hypothesis 1).
- **The compositional side.** van der Lugt & Lorenz v2 prove that C₃-exclusion characterises
  exactly which constraint sets traditional unitary circuits can represent (Theorem 3.2). Routed,
  direct-sum decompositions are the open frontier, and Lorenz & Barrett's hypothesis that every
  unitary has a causally faithful extended decomposition is still unproved.
- **Abstraction.** Lorenz & Tull leave genuine quantum causal abstraction, cyclic structures and
  approximate abstraction to future work (§8).

---

## 3. The cyclic paper against QCL

| Cyclic paper | QCL today, checked in the tree | Gap |
|---|---|---|
| **Definition 1**: a model over any directed graph, cycles included | `build()` refuses a cycle with `CyclicStructureUnsupported` ([`config.rs`](../../../deep_causality_quantum/src/types/pipeline/config.rs)) | Admit directed graphs. The next three rows are prerequisites. |
| **Validity is global.** For a cyclic graph, "even when a faithful QCM exists it is not in general the case that any set of commuting channels … defines a process operator"; in the acyclic case any such product is valid. Methods characterises a process operator by positivity, its trace, and which Hilbert–Schmidt terms may appear. | `check_normalization` tests `Tr_A ρ_{A\|Pa(A)} = 1_{Pa(A)}` node by node ([`validate.rs`](../../../deep_causality_quantum/src/types/pipeline/validate.rs)). That is sufficient for acyclic graphs only. | A process-validity check on the joint operator: positivity, trace, and the allowed term types, which is a linear projection. |
| **Each node has an input and an output space**, and `ρ_{A\|Pa(A)}` acts on `A`'s input and its parents' outputs | `FactorSupports` gives a single-system node one leg, the node index, with `support(A) = {A} ∪ Pa(A)` ([`process_factors.rs`](../../../deep_causality_quantum/src/types/qcm/process_factors.rs)). Only the dilation path pairs input and output on a leg (QCL-2, V-7). | Split each node's leg into input and output on the plant and model paths. A 2-cycle cannot be written without it. Acyclic models gain as well: today a node's own factor and its children's factors share one leg and must commute, where the framework puts them on different spaces. The split removes that source of incompleteness. |
| **Faithfulness** (Eq. 4): every channel signals from every parent. Proposition 1: there is no faithful cyclic model with two nodes. | Absent. `check_decomposable` decides the circuit-decomposition sense of "faithful", a different property. | A per-parent signalling check. It also screens 2-cycles structurally. |
| **Causal structure from no-influence** (Definitions 3 and 4; 2019 Theorem 4.3) | `CausalStructure` comes from graph reachability or from circuit wiring, never from an operator | Derive the structure from a unitary or a process operator. The next two rows need it. |
| **Causal discovery from a process operator.** 2019 §11 builds the simple induced graph from `n(n−1)` linear no-signalling checks and tests Markovianity; the 2021 version "does not halt anymore when encountering a cyclic graph". | Absent. QCL decides between named candidates only. | A candidate generator, fed by a tomography experiment the campaign can price. |
| **Causal separability** (Definition 8); **for a unitary process, nonseparable ⇔ cyclic** (Theorem 4) | Absent | For unitary processes, a structural test. The general case needs an SDP witness, and the workspace has no SDP. This is `dynamic-qcm.md` SPEC-Q6. |
| **Unitary extendibility and compatibility** (Definitions 5 and 6; Hypothesis 1) | The dilation is acyclic only | Research. QCL could check compatibility against a given extension. |
| **Cycles become circuits through direct sums.** The SWITCH decomposes as `U = (1 ⊗ T ⊗ 1)(⊕ᵢ Vᵢ ⊗ Wᵢ)(1 ⊗ S ⊗ 1)` (Eq. 14), and every unitary process with that node layout is a direct sum of summands in which influence follows acyclic paths. The Baumeler–Wolf extension of the Araújo–Feix process has a direct-sum decomposition too (Figs. 9 and 10). | The C₃ screen covers traditional circuits; the `⊕` generator is backlog B-26 | The route by which a cyclic candidate gets a circuit form. The C₃ screen does not apply to cyclic candidates. |

### 3.1 H₄ is not a valid process

`qcl_crosstalk` builds H₄ as the cycle Q1 → Q2 → B → Q1. Each child is excited with probability 0.4
when its driver is excited and with `0.1 · 0.6 / 0.9 = 1/15` otherwise. Read as a classical
split-node process (cyclic paper, Definition 9), H₄ is

```text
κ = P(Q2_in | Q1_out) · P(B_in | Q2_out) · P(Q1_in | B_out)
```

A process is valid when its outcome probabilities sum to 1 under every choice of local operations.
Evaluated on 2026-10-08:

| Local operations | Total probability |
|---|---|
| Identity at every node | **1.037037** |
| NOT at Q1, identity elsewhere | **0.962963** |
| Constant 0 at Q1, identity elsewhere | 1.000000 |

Each link is a 2 × 2 stochastic matrix whose second eigenvalue is `0.4 − 1/15 = 1/3`. With identity
at every node the total is the trace of the loop's product, `1 + (1/3)³`; with a NOT at one node it
is `1 − (1/3)³`. Only cutting the loop restores 1. A loop of signalling channels with no break is
the grandfather-type inconsistency the validity condition excludes.

So H₄ can be refused for a stated physical reason: it is not a valid process. That reason replaces
"outside v1's scope by decision", and the computation is the acceptance test for the validity check
of §3.

### 3.2 What admitting cycles takes

1. **The input/output leg split**, on the plant and model paths.
2. **The global validity check**, run on the joint operator after normalisation. Its cost is
   exponential in the node count, like the joint operator itself, and it needs a cap stated before
   allocation.
3. **The faithfulness check**, which also rejects every 2-cycle (Proposition 1).
4. **A positive case.** A SWITCH-type candidate (nodes A, B, a root P and a leaf F) is a valid,
   faithful cyclic model, and gives the admitted path a test as concrete as H₄ gives the refused
   one.
5. **A stated scope for the C₃ screen.** It is skipped for cyclic candidates, and the report says
   so.

---

## 4. The rest of the framework against QCL

| Lorenz | QCL | Gap |
|---|---|---|
| An intervention at a node is a quantum instrument (2019 §3.2); the do-intervention of §6 is the special case that fixes the output state | `intervene_instrument` is reserved and not built; QCL's mechanism `do` covers §6 | Backlog B-2 |
| The quantum d-separation theorem, sound and complete for quantum relative independence (2019 Theorem 8.3; thesis §5.5), and quantum analogues of the three do-calculus rules (2019 Theorems 10.7, 10.17, 10.26) | Absent | Each candidate implies equality constraints on marginals that can be tested without full tomography: an experiment family for `design`. The thesis outlook asks for "exploring whether there is a problem for which the generalised do-calculus rules are useful"; QCL's campaign is one. |
| Partial interventional data as the practical problem (2019 §13.2; Synthese §5; the thesis outlook asks for a discovery problem "where the given data is not a (full) process operator"). §13.2 reads the three rule analogues as equality constraints on the marginal over the observed nodes, and hypothesises that they are complete. | The campaign plans experiments over partial data | QCL works on the problem Lorenz names as practical. What it lacks is the d-separation and do-calculus constraints that would make it principled beyond named candidates. |
| Latent common causes when a process is not Markov for its induced graph (thesis §8.2) | Absent | Research, after the discovery row of §3 |
| A meaningful notion of quantum counterfactual (thesis §8.2) | The causal monad's `alternate_value` (`deep_causality_core`) substitutes a classical value; nothing quantum | Research |
| Genuine quantum causal abstraction (Lorenz & Tull §8) | Abstractions over circuits | Open upstream |

---

## 5. Citation hygiene

**van der Lugt & Lorenz v2 changed two things the crate cites.** The published version (Quantum,
accepted 2026-09-09; arXiv v2, 14 September 2026) drops Theorem 4.9(v) without comment. v1's item
read "for all `b₁, b₂, b₃ ∈ B`, either `p({b₁,b₂}) ∩ p({b₂,b₃}) = ∅` or `p({b₁,b₂}) ⊆ p({b₂,b₃})` or
`p({b₁,b₂}) ⊇ p({b₂,b₃})`", included as "algorithmically the most straightforward to verify"; v2's
Theorem 4.9 has items (i) to (iv). The proof of Theorem 3.2's (iii) ⇒ (i), v1's Proposition 6.1,
is Proposition C.1 in v2's Appendix C.

The crate cites both:

- `deep_causality_quantum/src/types/qcm/faithfulness.rs`, lines 22 (Proposition 6.1), 195 and 198
  (Theorem 4.9(v));
- `deep_causality_quantum/tests/types/qcm/faithfulness_tests.rs`, lines 17, 65, 150, 153 and 198
  (Theorem 4.9(v) as the test oracle).

Behaviour is unaffected: the check implements Definition 3.1 directly. The test oracle, though, now
rests on a criterion the published paper no longer states. The fix cites v2, uses Theorem 4.9(iii)
(at most one path between each input and output of `L_G`) or Definition 3.1 itself as the oracle,
and keeps v1's item only if it is labelled as v1's. The crate's `papers/` folder holds v1; v2 is in
[`Lorenz/`](Lorenz/).

**The Markov-condition citation is correct.** "Lorenz 2022, Definition 3.3, footnote 11" matches the
Synthese paper. Synthese states the condition for DAGs; the directed-graph form is the cyclic
paper's Definition 2.

---

## 6. Order of work

1. **The input/output leg split, the global validity check and the faithfulness check.** Medium.
   Together they turn the cyclic refusal into a physical one, with H₄ as the worked counterexample.
2. **Admit cyclic structural candidates** behind those checks, with a SWITCH-type candidate as the
   positive case and the C₃ screen's scope stated.
3. **Causal structure from no-influence, and induced-graph discovery** as a candidate generator.
4. **d-separation predictions** as an experiment family for `design`.
5. **The v2 citation fix** in `faithfulness.rs` and its tests. Small, and independent of the rest.
6. **Research**: Hypothesis 1 of the cyclic paper, routed decompositions, latent common causes,
   quantum counterfactuals, genuine quantum causal abstraction.

Backlog B-30 in [`quantum-backlog.md`](quantum-backlog.md) points at cyclic structures; items 1 to
5 above are its expansion.

---

## 7. Open questions

1. **The leg split is a breaking change** to `FactorSupports` and every caller of `from_graph`. Make
   it in one change, or introduce a typed in/out leg beside the current one and migrate?
2. **Validity check cost.** The joint operator is exponential in the node count. Which cap, and
   does the check run on every candidate or only on cyclic ones, where it is not implied?
3. **The positive case.** A SWITCH-type candidate needs root and leaf nodes the crosstalk plant does
   not have. A new example, or an extension of `qcl_crosstalk`?
4. **Discovery's input.** Induced-graph discovery needs a full process operator. Is a priced
   tomography experiment in the campaign the right source, or should discovery wait for the
   d-separation constraints that work on partial data?

---

## 8. Sources

Papers, in [`Lorenz/`](Lorenz/) unless noted:

- Barrett, J., Lorenz, R. & Oreshkov, O. (2019). *Quantum causal models.* arXiv:1906.10726.
- Lorenz, R. & Barrett, J. (2021). *Causal and compositional structure of unitary
  transformations.* Quantum 5, 511; arXiv:2001.07774.
- Barrett, J., Lorenz, R. & Oreshkov, O. (2021). *Cyclic quantum causal models.* Nat. Commun. 12,
  885; arXiv:2002.12157.
- Lorenz, R. (2020). *Quantum causal structure.* DPhil thesis, University of Oxford,
  https://www.cs.ox.ac.uk/people/aleks.kissinger/theses/lorenz-thesis.pdf; record at
  https://ora.ox.ac.uk/objects/uuid:ae92d193-29cd-46ac-993b-4dd6568da435
- Lorenz, R. (2022). *Quantum causal models: the merits of the spirit of Reichenbach's principle
  for understanding quantum causal structure.* Synthese 200:424.
- Lorenz, R. & Tull, S. (2023). *Causal models in string diagrams.* arXiv:2304.07638.
- Tull, S., Lorenz, R., Clark, S., Khan, I. & Coecke, B. (2024). *Towards compositional
  interpretability for XAI.* arXiv:2406.17583.
- van der Lugt, T. & Lorenz, R. (2026). *Unitary causal decompositions: a characterisation via
  lattice theory.* Quantum (accepted 2026-09-09); arXiv:2508.11762v2. v1 is in
  `deep_causality_quantum/papers/`.
- Lorenz, R. & Tull, S. (2026). *Causal and compositional abstraction.* arXiv:2602.16612.

Publication list: Google Scholar,
https://scholar.google.jp/citations?hl=en&user=ljdbFqMAAAAJ&view_op=list_works&sortby=pubdate, and
the arXiv author listing.

In this repository:

- [`quantum-backlog.md`](quantum-backlog.md), B-2, B-26 and B-30
- [`dynamic-qcm.md`](dynamic-qcm.md), SPEC-Q6
- [`config.rs`](../../../deep_causality_quantum/src/types/pipeline/config.rs),
  [`validate.rs`](../../../deep_causality_quantum/src/types/pipeline/validate.rs),
  [`process_factors.rs`](../../../deep_causality_quantum/src/types/qcm/process_factors.rs),
  [`faithfulness.rs`](../../../deep_causality_quantum/src/types/qcm/faithfulness.rs)
- `examples/quantum_examples/qcl_examples/qcl_crosstalk/model.rs`, the H₄ tables of §3.1
