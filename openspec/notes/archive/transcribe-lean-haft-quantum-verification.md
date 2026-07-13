# Note: transcribe the Haft and Quantum Lean verification to the website

## What

Publish the last two formalization stubs — `website/docs/src/content/docs/formalization/haft.md` and `quantum.md` — completing the series after `transcribe-lean-algebra-verification` and `transcribe-lean-num-complex-dual-verification` (both archived 2026-07-12). Haft follows the `topology.md` shape (no per-row Rust-witness column); Quantum follows the `num.md`/`core.md` shape (witness column, no Kani).

## Context

- **Haft**: 49 `haft.*` rows in the `### Haft layer` table (the stub's "near 45" undercounts; the 51-count from a naive grep includes 2 planned ids in the "Not yet on the map" table that must NOT be transcribed: `haft.traversable.composition`, `haft.effect_unbound.laws`). Page column shape `| id | statement | Lean proof | Test |`; the map's Lean-location cells are already directory-qualified file names without theorem names (`Haft/Functor.lean`). Witnesses: `deep_causality_haft/tests/formalization_lean/` mirrors the Lean tree one-to-one; the naming pattern is `test_<id minus the haft. prefix>` (e.g. `haft.functor.laws` → `test_functor_laws`), NOT the literal `test_<id>` the map's prose claims. All 49 ids verified present as `THEOREM_MAP:` annotations in the witness tree.
- **Quantum**: 10 `quantum.*` rows in the `## Quantum` section, all `proved`; all 8 sampled Lean theorems verified in `lean/DeepCausalityFormal/Quantum/`. Headline framing to keep: unconditional `partial_trace_preservation` is **false** (witnessed counterexample `partial_trace_nonpreservation`); the conditional boundary version holds. The `/Quantum/` tree is exempt from the CI `sorry` gate.
- **Discovered staleness in `THEOREM_MAP.md`'s quantum section** (the reason this change is not purely a website edit): the witnesses actually live in `deep_causality_quantum/tests/formalization_lean/{partial_trace_tests,choi_tests}.rs` (all 10 ids annotated, verified), but the map's prose says `tests/kernels/`, every row's witness cell names the wrong file (`operator_linalg_tests.rs`/`channel_tests.rs` — those exist under `tests/types/qgates/` and even point to the formalization_lean tests in comments), and the two `choi.*` rows name a nonexistent test (`test_apply_kraus_and_apply_choi_agree`; the real witness is `test_apply_choi_is_linear`). Likely fallout from the quantum-kernel migration (`quantum-crate-scaffold`). No spec pins these paths, so correcting them is a factual doc fix, not a requirement change.

## Constraints

- The map correction must be minimal: the quantum section's witness prose + the 10 witness cells. No Lean, no Rust, no CI changes.
- The website pages transcribe from the **corrected** map, so page and map stay consistent.
- Same mechanical verification as the prior two changes, adapted: haft rows checked via the `THEOREM_MAP:` annotations in the mirrored test files; quantum rows checked per-file against theorems and test names.
- Site must build with both routes; the index's "being documented" sentence disappears entirely — all layers are then live.
