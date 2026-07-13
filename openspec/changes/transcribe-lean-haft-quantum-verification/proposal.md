# Proposal: transcribe-lean-haft-quantum-verification

## Why

The last two formalization pages are still `draft: true` stubs: Haft (49 proved higher-kinded laws, the largest layer) and Quantum (10 proved partial-trace/Choi laws, including the headline B1 counterexample). Publishing them completes the formalization section — every verification layer on the public site. Additionally, exploration for this change found the `## Quantum` section of `lean/THEOREM_MAP.md` is stale against the Rust tree (wrong witness directory and filenames in all 10 rows, one wrong test name), which must be corrected first so the page transcribes truth. The originating note is `openspec/notes/transcribe-lean-haft-quantum-verification.md`.

## What Changes

- **Correct the `## Quantum` section of `lean/THEOREM_MAP.md`** (prerequisite): witness prose `deep_causality_quantum/tests/kernels/{operator_linalg_tests,channel_tests}.rs` → `deep_causality_quantum/tests/formalization_lean/{partial_trace_tests,choi_tests}.rs`; the 8 partial-trace rows' witness cells `operator_linalg_tests.rs` → `partial_trace_tests.rs`; the 2 `choi.*` rows' witness cells `channel_tests.rs :: test_apply_kraus_and_apply_choi_agree` → `choi_tests.rs :: test_apply_choi_is_linear`. Verified against the actual test tree (all 10 ids carry `THEOREM_MAP:` annotations there). No id, statement, or Lean cell changes.
- Fill `formalization/haft.md` with the 49 `haft.*` rows (topology-style shape: `| id | statement | Lean proof | Test |`, no per-row witness column), publish it (remove draft scaffolding).
- Fill `formalization/quantum.md` with the 10 `quantum.*` rows (num/core-style shape with witness column), keeping the B1 framing — unconditional preservation is false, the conditional boundary version holds — and the `sorry`-gate exemption notice; publish it.
- Update `formalization/index.md`: add Haft (between Algebra and Core) and Quantum (after Topology) bullets under "The layers", and delete the "being documented" / good-first-issue sentence — all layers are then live.

The quantum stub's embedded instructions repeat the stale `tests/kernels/` location; the page will name the corrected location instead.

## Capabilities

### New Capabilities

- `haft-formalization-docs`: the website formalization page for the Haft layer — a complete rendering of the `haft.*` rows of `lean/THEOREM_MAP.md` (excluding the "Not yet on the map" planned ids), published and linked from the index.
- `quantum-formalization-docs`: the website formalization page for the Quantum layer — a complete rendering of the `quantum.*` rows with the B1 counterexample framing, published and linked from the index, backed by a corrected `THEOREM_MAP.md` quantum section.

### Modified Capabilities

None. The witness-cell corrections in `THEOREM_MAP.md` are factual doc fixes; no spec pins those paths (checked `quantum-formalization` and `quantum-crate-scaffold`), and no Lean, Rust, or CI behavior changes.

## Impact

- `lean/THEOREM_MAP.md` — quantum section witness pointers corrected (10 rows + prose). This file is CI-referenced (`formalization.yml` id-linkage), so CI must stay green; ids are untouched.
- `website/docs/src/content/docs/formalization/haft.md`, `quantum.md` — tables filled, prose completed, draft flags removed.
- `website/docs/src/content/docs/formalization/index.md` — two layer bullets added; pending-documentation sentence removed.
- Read (not modified): `lean/DeepCausalityFormal/Haft/*.lean`, `lean/DeepCausalityFormal/Quantum/*.lean`, `deep_causality_haft/tests/formalization_lean/*.rs`, `deep_causality_quantum/tests/formalization_lean/*.rs`.
