# Raw information: DeepCausality Quantum

Step 0 of `docs/writing_guides/TurnRawMaterialintoWriting.pdf`. This file holds facts and their sources
and no prose for the reader. Every fact carries the letter of the file it comes from. Section 02 turns
these facts into the home page.

Read on 2026-09-30: every `.rs` file under `deep_causality_quantum/src/` (87 files), `README.md`,
`LEAN_QUANTUM.md`, `CHANGELOG.md` and `Cargo.toml` of the crate, and the `main.rs`, `model.rs` and
`README.md` of all 14 examples under `examples/quantum_examples/`. Six QCL examples were run in
release mode and their output is quoted below.

Not read: the 21,646 lines under `deep_causality_quantum/tests/`, the seven PDFs under
`deep_causality_quantum/papers/`, and `utils_print.rs` of the non-QCL examples. Claims about the
papers rest on the crate's own citations of them.

## Sources

| Letter | File or run |
|---|---|
| A | `deep_causality_quantum/src/types/pipeline/{config,validate,control,ledger,spec}.rs` |
| B | `deep_causality_quantum/src/types/design/{experiment_design,adjudicate}.rs` |
| C | `deep_causality_quantum/src/types/decision/{check,tolerance}.rs` |
| D | `deep_causality_quantum/src/types/qcm/{hypothesis,markov_freeze,faithfulness,process_factors,dilation}.rs` |
| E | `deep_causality_quantum/src/types/{qpu,verdict,carriers}/*` and `density_matrix.rs` |
| F | `deep_causality_quantum/src/types/{abstraction,circuit_model,qcode}/*` |
| G | `deep_causality_quantum/{README.md,LEAN_QUANTUM.md,Cargo.toml}` |
| H | `examples/quantum_examples/qcl_examples/qcl_crosstalk/{main,model,constants}.rs` |
| H-run | `cargo run --release -p quantum_examples --example qcl_crosstalk`, 2026-09-30 |
| I | the other QCL examples: `qcl_qcm_freeze`, `qcl_geometric_qec`, `qcl_code_switching`, `qcl_concatenated_code`, `qcl_distillation_round`, `qcl_crosstalk_circuits` |
| I-run | the same command for each of the first five of those, 2026-09-30 |
| J | `qcm_freeze_check`, `quantum_counterfactual`, `quantum_geometric_tensor`, `gauge_electroweak`, `topological_insulator`, `hopf_fibration_multivector`, `ikkt_matrix_model` |

## What the library decides, from Sources A, B, C, D

- A candidate is either a structural candidate, a factorization of a process operator with the Hilbert legs each factor acts on, or a mechanism candidate, a channel applied to the plant. (D)
- A structural candidate reaches `control` only through `validate`; a plant configuration with structural candidates has no `ControlSource` implementation, so the compiler refuses the shortcut. (A)
- `build()` refuses a structural candidate whose supports contain a directed cycle, as `CyclicStructureUnsupported`, before any check runs. (A)
- The cycle refusal is a scope decision. The C₃ criterion does not reject cyclic structures. (A, `error/quantum_error.rs`)
- The Markov check tests every pair of factors whose supports intersect. It compares ‖[ρⱼ, ρₖ]‖_F with a threshold C·(‖ρⱼ‖·bₖ + ‖ρₖ‖·bⱼ + 2γₙ‖ρⱼ‖‖ρₖ‖), where C is 8 by default and the unit roundoff is `R::epsilon()`. (D)
- The Markov check stops at the first rejecting pair. On a plant a rejecting candidate is dropped from the admitted set and no error is raised; on a model subject `finalize` returns `CommutatorNonZero` naming the pair. (A, D)
- The decomposability check derives a causal structure from the supports and rejects a C₃ sub-relation, the structure of two commuting CNOTs. (D)
- A check returns a report. Each record holds the measured quantity, the threshold, the margin and whether it was accepted. The report also holds the number of items examined, and a report that examined nothing reads `Vacuous`. (C)
- `design` finds the minimum-cost set of experiments that separates every pair of candidates by at least `floor_bits` of separation. It solves a dynamic program over 2^C(n,2) subsets, so it refuses more than 7 candidates by default. Pairs that no experiment separates are listed as uncovered. (B)
- Separation between two predicted read-outs p and q over n shots is n · (−log₂(√(pq) + √((1−p)(1−q)))), the Bhattacharyya distance of n independent draws, in bits. (B, E `qpu/shot_estimate.rs`)
- `adjudicate` names a survivor when exactly one world's verdict holds and that world's read-out separates from every other world's by at least the floor. Otherwise it returns an ambiguity: no survivor, several survivors, one survivor that is not separated, non-commuting projections, or a single world. (B)
- Forked worlds keep separate ledgers of shots, experiments, predictions, device time, cost and bits. The ledgers are compared and never summed. (A)
- Counts are natural numbers on a caller-chosen width; real quantities are on a caller-chosen `RealField`. Every tolerance derives from that field's epsilon. (A, C)

## What the evidence is, from Sources E, G

- In the default build, shots come from a seeded Bernoulli draw at the Born probability Tr(Pρ) of a density matrix. A fixed seed reproduces the histogram. (E `qpu/born_sampler.rs`, `qpu/prng.rs`)
- The `qpu` feature adds a `QpuSampler` trait, an in-process state-vector simulator `SimQpu` capped at 24 qubits, and `qpu_effect`, which lifts a sampler call into the causal monad. No vendor adapter ships. (E, G)
- Naming a shot budget compiles only under `qpu`. (E `qpu/evidence.rs`)
- The pipeline module is behind the `qcm` feature, which implies `std`. (A, G)

## What the crosstalk run does, from Sources H and H-run

- The plant is two qubits in |00⟩, with two observables: qubit 2 excited (e2) and qubit 1 excited (e1). (H `model.rs`)
- Four candidates: H1 Q1→Q2, H2 Q2→Q1, H3 a shared bath B driving both, and H4 the cycle Q1→Q2→B→Q1. (H `model.rs`)
- All factors are diagonal, so every acyclic candidate is a legal quantum causal model and the discrimination rests on the interventions. (H `model.rs`)
- Three structures fit the passive observation exactly because they are Markov equivalent. (H `main.rs`)
- Five experiments with a cost each: E0 passive (1), E1 hold Q1 excited and read Q2 (1), E2 hold Q2 excited and read Q1 (1), E3 echo both (2), E4 process tomography (200). Costs are "in the same arbitrary unit throughout". (H `constants.rs`)
- The read-out each experiment predicts under H1, H2, H3 is typed into `constants.rs` from a table in the design note: E0 0.04, 0.04, 0.04; E1 0.40, 0.10, 0.10; E2 0.10, 0.40, 0.10; E3 0.01, 0.01, 0.04; E4 0.90, 0.50, 0.10. The example does not derive them from the factors. (H `constants.rs`, `model.rs`)
- Shots per experiment 1024, floor 5 bits, agreement window 3 standard errors, seed 20260821. (H `constants.rs`)
- Run output: `build()` refused H4; `check_markov` accepted, examined 5; `check_decomposable` accepted, examined 3; H1, H2 and H3 admitted. (H-run)
- Run output: the plan is E1 and E2 at total cost 2. E1 resolves the pairs (H1, H2) and (H1, H3). E2 resolves (H1, H2) and (H2, H3). Tomography alone would cost 200, 100 times more. (H-run)
- Run output: E1 observed 0.377 ± 0.015 over 1024 shots, drawn from the Born sampler at H1's prediction. H1 predicts 0.40 and is consistent; H2 and H3 predict 0.10 and are rejected. (H-run)
- Run output: H1 survives, 100.1 bits from its nearest rival. The example then checks that the survivor is the structure the observation was drawn from, and it is. (H-run)
- The observation is sampled from H1's own prediction, and only E1, the first planned experiment, is observed. (H `main.rs`)
- E1 predicts 0.10 under both H2 and H3, so an E1 read-out near 0.10 would leave both standing; E2 predicts 0.40 under H2 and 0.10 under H3 and separates them. (derived from H `constants.rs`; not run)
- The example's closing line for the survivor H1 reads: "a scheduling or echo fix applies; frequency reallocation is not required." (H `main.rs`)
- Checked by hand from the table: 100.1 bits is 1024 · the Bhattacharyya distance between 0.4004 and 0.0996, the predictions rounded to whole shots (410 and 102 of 1024). For exact predictions 0.40 and 0.10 the separation is 99.5 bits. (Python check, 2026-09-30)
- Checked by hand: the printed "tightest pair 164.8 bits" is the smallest, over the three pairs, of the best separation any offered experiment reaches, and process tomography E4 reaches it. Under the chosen plan the tightest pair separates at 99.5 bits. (Python check; B `experiment_design.rs`, the `best` array ranges over all offered experiments)

## Other subjects, from Sources A, F, I, I-run

- A model subject is a factorization over a frozen graph. The commuting pair (σz and diag(3, −1)) screens with one pair examined and worst margin 0. The pair (σx, σz) fails `validate` with `CommutatorNonZero` naming nodes 0 and 1, and the frozen subject stays frozen. On a dynamic graph the same pair aborts `freeze_quantum` and the graph rolls back. (I-run `qcl_qcm_freeze`)
- `check_decomposable` on a two-node chain examines no 3 × 3 block and reads `Vacuous`. (I-run)
- A code subject is a chain complex read as a CSS code. The 4 × 4 square torus gives [[32, 2]] with 16 Z checks and 16 X checks of weight 4. Four checks accept: `derive_code` (32 examined), `check_ldpc_weights` (96), `check_class_invariance` (6), `check_clifford_action` (2). (I-run `qcl_geometric_qec`)
- A weight bound of 3 rejects at the first Z check with margin 4/3; a bound of 4 accepts. (I-run)
- Every code check is an 𝔽₂ or exact-rational computation over supports. The in-process simulator caps at 24 qubits and the code has 32. (I-run, E)
- An abstraction links a low-level circuit model to a high-level one with a type alignment (τ, E) and a query map. Its naturality check compares Choi operators in Frobenius norm. (F)
- The composition law bounds the composite residual: ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂. The exact case is proved in Lean; the approximate law is the crate's own statement. (F `composition.rs`, G)
- Code switching from [[4,2,2]] to [[8,2,2]] with depolarising noise 1/10 on one logical wire: ε₁ = 0.4619, ε₂ = 0, ‖τ₂‖_post = 8, bound 3.695, measured 0.4619. Without noise every residual is 0. (I-run `qcl_code_switching`)
- Concatenating [[4,2,2]] with itself composes Z̄ and X̄ exactly. A CZ̄ across two outer blocks is refused by name. (I-run `qcl_concatenated_code`)
- A distillation round on [[4,2,2]] with depolarising p = 1/100: ε₁ = 0.1639, recovery leaves 0.1638, bound 0.3277. At p = 5/100: ε₁ = 0.7680, measured 0.7667, bound 1.536. (I-run `qcl_distillation_round`)
- `DecoderAbstraction` validates a detector error model against a circuit by naturality squares and ranks faults by residual. The crate contains no decoding algorithm. (F `decoder_abstraction.rs`)
- `check_fault_tolerance` and `haruna_filter` decide which Table 1 logical gates tolerate every weight-one Pauli fault, exactly, by pushing the fault through the program. (F `fault_tolerance.rs`)
- Five of the 14 examples are quantum in subject and use other crates: `quantum_counterfactual`, `quantum_geometric_tensor`, `gauge_electroweak`, `topological_insulator`, `hopf_fibration_multivector`. The other nine import `deep_causality_quantum`. (J imports; G `README.md`)

## Boundaries, from Sources A, B, E, F, G

- Evidence is simulated. Shots come from a seeded sampler on a density matrix or from `SimQpu`. (E)
- The crate contains no decoder. (F)
- Cyclic structures are refused by decision. (A)
- The example's predictions and costs are supplied by the caller. (H)
- `design` covers at most 7 candidates by default. The ideal recovery is dense and stops at 10 qubits. The numeric semantics refuses Choi operators above 2^24 entries. The dilation caps a factor at 2^24 entries, which is why H3 stays a factorization in `qcl_crosstalk_circuits`. (B, F, I)
- The pipeline needs `std`, through the `qcm` feature. (A, G)
- Lean: 14 rows named `quantum.*` are marked proved in `lean/THEOREM_MAP.md` (counted with grep). `LEAN_QUANTUM.md` names 7 deferred targets: the Choi reconstruction isomorphism and six model targets. (G, `lean/THEOREM_MAP.md`)
