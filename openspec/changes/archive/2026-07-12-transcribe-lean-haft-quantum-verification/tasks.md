# Tasks: transcribe-lean-haft-quantum-verification

## 1. Correct the THEOREM_MAP quantum section (prerequisite)

- [x] 1.1 In `lean/THEOREM_MAP.md` `## Quantum`, fix the witness prose: `deep_causality_quantum/tests/kernels/{operator_linalg_tests,channel_tests}.rs` → `deep_causality_quantum/tests/formalization_lean/{partial_trace_tests,choi_tests}.rs`
- [x] 1.2 Fix the 8 `quantum.partial_trace*` rows' witness cells: `operator_linalg_tests.rs` → `partial_trace_tests.rs` (test names unchanged)
- [x] 1.3 Fix the 2 `quantum.choi.*` rows' witness cells: `channel_tests.rs :: test_apply_kraus_and_apply_choi_agree` → `choi_tests.rs :: test_apply_choi_is_linear`
- [x] 1.4 Verify the correction: every quantum witness cell names an existing file and `#[test]` in `deep_causality_quantum/tests/formalization_lean/`; ids, statements, and Lean cells byte-identical to before

## 2. Fill the Haft page

- [x] 2.1 Transcribe the 49 `haft.*` rows from the `### Haft layer` table into `website/docs/src/content/docs/formalization/haft.md`, columns `| id | statement | Lean proof | Test |`, Lean cells verbatim from the map's Lean-location column (`Haft/Functor.lean`), excluding the 2 "Not yet on the map" planned ids
- [x] 2.2 Rewrite the intro prose: "Forty-nine laws", GitHub link to `lean/DeepCausalityFormal/Haft/`, witness convention stated once (mirrored files in `deep_causality_haft/tests/formalization_lean/`, one test per id with a `THEOREM_MAP:` annotation), every row `proved`
- [x] 2.3 Delete the `:::caution` block and remove `draft: true` (keep `sidebar: order: 3`)

## 3. Fill the Quantum page

- [x] 3.1 Transcribe the 10 `quantum.*` rows from the corrected `## Quantum` section into `website/docs/src/content/docs/formalization/quantum.md`, columns `| id | statement | Lean proof | Rust witness | Test |`, Lean cells directory-qualified, witness cells bare filenames
- [x] 3.2 Complete the intro prose: "Ten laws", B1 headline framing (unconditional preservation false with witnessed counterexample; conditional boundary version holds), keep the `sorry`-exemption sentence, close with one sentence on the deferred targets (CJ reconstruction, QCM theorems)
- [x] 3.3 Delete the `:::caution` block and remove `draft: true` (keep `sidebar: order: 7`)

## 4. Complete the formalization index

- [x] 4.1 Add a `**[Haft](/formalization/haft/)**` bullet between Algebra and Core, and a `**[Quantum](/formalization/quantum/)**` bullet after Topology, each with a one-line summary
- [x] 4.2 Delete the "The Haft and Quantum layers are being documented. Filling them in is a [good first issue] …" sentence pair

## 5. Verify

- [x] 5.1 Haft cross-check: page ids set-equal to the 49 `### Haft layer` ids; every Lean filename exists under `lean/DeepCausalityFormal/`; every id present as a `THEOREM_MAP:` annotation in `deep_causality_haft/tests/formalization_lean/`; planned ids absent
- [x] 5.2 Quantum cross-check: page ids set-equal to the 10 map ids; every theorem in the exact named `.lean` file; every test in the exact named `_tests.rs` file
- [x] 5.3 Build the site (`npm run build` in `website/docs/`) and confirm exit zero with the `/formalization/haft/` and `/formalization/quantum/` routes present
