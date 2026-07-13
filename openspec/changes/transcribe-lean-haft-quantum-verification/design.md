# Design: transcribe-lean-haft-quantum-verification

## Context

Third and final change in the formalization-docs series. Two stubs remain, with different shapes:

- **Haft** (`sidebar: order: 3`): 49 rows in the `### Haft layer` table. The map's own columns differ from other layers — `| id | statement | Lean | Lean location | Test | Kani |` with **no Rust-witness column**, and Lean-location cells are directory-qualified filenames without theorem names (`Haft/Functor.lean`). The stub mandates the `topology.md` shape: `| id | statement | Lean proof | Test |`. Witnesses: `deep_causality_haft/tests/formalization_lean/` mirrors the Lean tree one-to-one; every id is carried by a `THEOREM_MAP:` doc-comment on a test whose name is `test_<id minus the "haft." prefix, dots → underscores>` (verified: all 49 annotations present; the map prose's "name pattern `test_<id>`" is loose, and the stub's "near 45 laws" undercounts).
- **Quantum** (`sidebar: order: 7`): 10 rows in the `## Quantum` section, all `proved`, columns `| id | statement | Lean | Lean location | Rust witness | Test |` (no Kani). The stub mandates the `num.md`/`core.md` shape. The section's headline is a *negative* result: unconditional `partial_trace_preservation` is **false** with a witnessed counterexample; the conditional boundary version holds. The `/Quantum/` tree is exempt from the CI `sorry` gate.

**Discovered defect**: the map's quantum witness pointers are stale post `quantum-crate-scaffold` migration. Reality (verified): witnesses live in `deep_causality_quantum/tests/formalization_lean/partial_trace_tests.rs` (8 partial-trace ids) and `choi_tests.rs` (2 choi ids, test `test_apply_choi_is_linear`). The map claims `tests/kernels/`, names `operator_linalg_tests.rs`/`channel_tests.rs` in every witness cell, and names a nonexistent test for the choi rows. The similarly-named files under `tests/types/qgates/` are behavioral tests whose comments explicitly defer to the `formalization_lean` witnesses. All Lean theorem names in the map verified correct.

## Goals / Non-Goals

**Goals:**

- Correct the `## Quantum` witness pointers in `THEOREM_MAP.md`, then publish both pages transcribed from the corrected map.
- Complete the formalization index: all seven layers listed, pending-documentation sentence gone.

**Non-Goals:**

- No Lean or Rust changes; no CI workflow changes; no new laws.
- No transcription of the "Not yet on the map (blocked / scaling)" table (`haft.traversable.composition`, `haft.effect_unbound.laws`) — planned ids, not proved laws.
- No rendering of the quantum deferred targets (CJ reconstruction, QCM theorems) as table rows; they get one prose sentence, mirroring the map.
- The map's haft prose ("name pattern `test_<id>`") is loose but harmless; fixing it is out of scope (quantum cells are wrong, haft prose is merely imprecise).

## Decisions

1. **Fix the map before transcribing, in the same change** — the alternative (transcribe the stale cells verbatim, per "the map is the source of truth") would publish witness pointers that provably don't exist; the alternative of a separate prerequisite change adds ceremony for a 10-cell factual correction with no spec impact. The map remains the single source of truth precisely because it gets corrected when it drifts. **This is the decision to scrutinize in review**: it touches `lean/THEOREM_MAP.md`, which the two prior changes treated as read-only.

2. **Correction is surgical** — witness prose line + 10 witness cells (filename swap ×8, filename+testname swap ×2). Ids, statements, Lean cells, and the section's narrative prose are untouched, so the CI id-linkage (`formalization.yml`) is unaffected.

3. **Haft page: topology shape, Lean cells verbatim** — `| id | statement | Lean proof | Test |`; Lean cells keep the map's directory-qualified filenames (`Haft/Functor.lean`) with no theorem names, exactly as the map has them (consistent with `topology.md`, which also omits theorem names). The intro prose states the witness convention once: mirrored test files, one test per id, `THEOREM_MAP:` annotations — so readers can find witnesses without a column.

4. **Haft page prose states "Forty-nine laws"** — corrects the stub's "near 45" against the actual row count; the count doubles as the transcription checksum.

5. **Quantum page: num/core shape, cells relative to the corrected locations** — Lean cells directory-qualified (`Quantum/PartialTrace.lean :: partialTraceRight_add`, matching the map and the complex-dual precedent), witness cells bare filenames (`partial_trace_tests.rs :: test_partial_trace_linearity`). Intro keeps the stub's sorry-exemption sentence, adds the B1 headline framing (the stub instructions require it), and closes with one sentence on the deferred targets, mirroring the map's closing paragraph.

6. **Shared witnesses transcribed as-is** — `test_partial_trace_linearity` covers add+smul, `test_partial_trace_bimodule_law` covers both bimodule rows, `test_partial_trace_nonpreservation_counterexample` covers the counterexample and its value, `test_apply_choi_is_linear` covers both choi rows. Same convention as the generic-monoid rows on the algebra page.

7. **Index completion** — Haft bullet between Algebra and Core (sidebar order 3), Quantum bullet after Topology (order 7). The sentence pair "The Haft and Quantum layers are being documented. Filling them in is a good first issue for new contributors." is deleted entirely; nothing remains pending. The index's Scope section already documents the quantum caveats and stays as-is.

## Risks / Trade-offs

- [Editing `THEOREM_MAP.md` breaks the CI id-linkage check] → Only witness-cell text changes, ids untouched; the real witness files carry matching `THEOREM_MAP:` annotations for all 10 ids (verified), so a check keyed on either ids or annotations still passes. CI runs on the PR regardless.
- [Transcription typo across 59 rows] → Same mechanical verification as the prior pages, adapted per layer: haft — page ids diffed against the map AND against the `THEOREM_MAP:` annotations in `deep_causality_haft/tests/formalization_lean/`, Lean filenames checked to exist under `lean/DeepCausalityFormal/`; quantum — per-file theorem and test checks against the corrected cells.
- [The `tests/types/qgates/` near-duplicates mislead future readers] → Out of scope to reorganize; the corrected map plus the qgates files' own comments (which already point to `formalization_lean`) disambiguate.
- [49-row table is large for one page] → Accepted; `core.md` already ships 41 rows in the same format. The stub's "partial table is still useful" escape hatch is not needed.

## Open Questions

None blocking. The one decision needing reviewer sign-off is Decision 1 (map correction in-scope); if rejected, the quantum page must wait on a separate map-fix change, and only the haft page and index edits proceed.
