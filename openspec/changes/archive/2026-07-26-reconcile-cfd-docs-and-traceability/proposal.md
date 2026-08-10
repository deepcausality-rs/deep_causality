# Reconcile the CFD crate's documentation and constant/test traceability

## Why

The `deep_causality_cfd` pre-certification audit closed its two correctness phases and left two: Phase 3
(documentation truth-up) and Phase 4 (traceability). `AUDIT-REPORT.md` §9 items 16–24. Neither gates
correctness — the numerics are confirmed right against every closed-form reference available — but both
are what an engineer reads before trusting the crate, and both are the audit's largest surviving themes
by count: 87 doc-overclaims and 39 doc-gaps (§7). The risk the audit names is precise: an engineer reads
a docstring, a constant, or a green test and concludes something was established that was not.

This is the final audit change. It folds Phase 3 and Phase 4 into one workstream because they share a
root — a claim the crate makes about itself that nothing checks against an independent source: prose
against the code beneath it, a load-bearing constant against a publication, a test against a reference
other than the code's own prior output.

**The prose describes intent where the code marches a subset.** The DEC NS rate kernel's public
docstrings state the un-symmetrised convective operator `−i_u(du♭)`
(`src/solvers/dec/dec_ns_rate.rs:7,35`), while the code marches the skew-symmetrised
`½[G_ω u − G*_ω u]` introduced to stabilise it (`dec_ns_rate.rs:621-652`). Two spectral-projector
comments name the compact 5-point Laplacian eigenvalue `−(2−2cos(2πk/N))/Δ²`
(`src/tensor_bridge/projection.rs:157,169`) where the code applies the consistent `sin²(2πk/N)/dx²`
(`:158-163`). The DEC solver's module prose describes a first-order Chorin split (`src/solvers/dec/mod.rs:21-26`)
where the code projects inside each RK4 stage with no splitting error (`dec_ns_solver/step.rs:6-13`).
Each is a doc that would mislead a reader into distrusting correct code, or trusting the wrong reason for
it.

**Load-bearing constants carry no provenance.** `SMOOTH_CELLS = 2.0` sets the mask width that moves the
reported immersed-cylinder `C_d` by 6.1× (§4b) and carries only "Mask smoothing width in cells."
(`verification/qtt_cylinder_verification/config.rs:44`, and a duplicate at `qtt_park2t_blackout/config.rs:37`).
The Mach-1.05 shock floor (`src/types/flow/compressible_march_run.rs:326-327`) explains its branch but
not the 0.05 buffer above sonic. The `papers/` folder holds four PDFs and none for the penalization
method the immersed-body harness rests on — Angot / Bruneau & Fabrie (1999) is cited in harness text only
(`verification/qtt_cylinder_verification/print_utils.rs:122`).

**Load-bearing tests reference the code's own output, not an independent truth.** No test drives the
shipped QTT convection path (`rate_pair`, `src/solvers/qtt/incompressible_2d.rs:105`) with a nonzero
velocity: the two `scalar_rate` unit tests pass `u = v = 0`, and the full-solver Taylor–Green test drives
convection but the projection annihilates it, so a sign flip in the shipped convection would be invisible.
The Taylor–Green verification harness does check convection with `u,v ≠ 0` — but it re-assembles the
operator from `gradient_x`/`gradient_y` rather than calling the shipped `rate_pair`, so it gates a copy
(the pattern Phase 2 caught in gate BM-A, §5c).

**`Gates` is a dead, duplicate API.** It is exported from `lib.rs` and documented as the `[PASS]`/`[FAIL]`
block every self-verifying program prints, yet `Gates::new` is constructed only in its own unit test —
every shipped program uses `GateSeq` or `Verdict`'s `Display` (`Gates` finding, §7). It holds the only
five `println!` in `src/` (`src/types/flow/gates.rs:44-58`), and `Gates::finish()` returns success for an
empty gate set.

## What Changes

- **Reconcile the doc-overclaims against the code.** Where prose describes intent, mark it intent; where
  it asserts a property "by construction" that no check enforces, describe what the code does. Work the
  ACTION-LIST catalogue (86 actionable `doc-overclaim` rows), not a round number — the audit's "87" is a
  category estimate. A minority are already closed by Phases 1–2 (the `blended.rs` fold claim by B-4, the
  `boundary_zone` hook by item 15, the `penalization_heat_integral` rename by item 11) and are excluded.
- **Correct the DEC kernel and spectral-projector docs** to name the operator the code marches: the
  skew-symmetrised convective term, the consistent `sin²` projector eigenvalue, and the in-stage (no
  splitting) projection.
- **Document the undocumented capabilities** the audit lists as absent from the crate README —
  `DuctMarchRun`, `IgnitionCorridor`, snapshot/resume, `AcousticCoreInverse`, and the rest of the doc-gap
  catalogue — where a user looks for them, not only in rustdoc where they already exist.
- **Qualify the convergence claim.** State the QTT Taylor–Green order as second-order in space,
  first-order in time, and document the ladder's temporal-error floor (~1e-5 at fixed `dt`) and maximum
  usable length.
- **Resolve `Gates`** (owner decision, item 22): adopt it across the self-verifying programs so the
  README's claim becomes true, or retire it. Retirement is a deletion and needs owner approval. Until it
  is resolved, correct the README to name the type the programs use and make `Gates::finish()` refuse an
  empty gate set.
- **Give every load-bearing constant a source, units, and — where a publication backs it — a `papers/`
  entry.** `SMOOTH_CELLS`, the `qtt_park2t_blackout` `ETA`, and the Mach-1.05 floor gain provenance; the
  penalization reference (Angot / Bruneau & Fabrie 1999) is added to `papers/` and cited from the harness.
- **Give a load-bearing test an independent reference.** Add a test that drives the shipped `rate_pair`
  convection path with `u,v ≠ 0` against an analytic reference, and route the Taylor–Green harness's
  convection check through the shipped path rather than a re-assembly.

Explicitly **not** in scope: any change to a marcher, kernel, or gate *bound* — this change corrects prose,
constant provenance, and test references, and adds one behavioural test; it does not alter what the code
computes. The three items Phases 1–2 already delivered are excluded and re-verified, not re-done: the
RAM-C order-of-magnitude framing (item 19), the lid-cavity 65²-default summary row (item 20), and the
cylinder `ETA` re-derivation (item 23, cylinder site).

## Capabilities

### New Capabilities

- `documentation-code-parity`: the cross-cutting contract that the crate's prose describes the code that
  ships — a kernel docstring names the operator the kernel marches, a comment does not contradict the
  code beneath it, a "by construction" claim names an enforcing check or is marked as intent, every
  load-bearing public capability is documented where a user looks for it, a convergence claim states its
  order and regime, and the crate ships no dead duplicate gate API. This capability owns the doc-overclaim
  and doc-gap repairs across `src/`, the READMEs, `verification/`, and `studies/` that no behavioural spec
  mandates, mirroring how `verification-gate-integrity` owns the gate repairs.
- `constant-and-test-traceability`: the contract that a load-bearing constant carries a source, its units,
  and a `papers/` entry where a publication backs it, and that a load-bearing test references an
  independent truth rather than a value pinned from the code's own prior output.

### Modified Capabilities

None. The doc-overclaims sit across many behavioural capabilities whose *behaviour* is confirmed correct
and unchanged — this change corrects the prose, constants, and tests that describe that behaviour, not the
behaviour. Threading the edits through every existing spec would sprawl the change across dozens of specs
for prose that no single one owns; two cross-cutting capabilities own it instead, as Phase 1 did for the
gate theme.

## Impact

**Docs / prose (the bulk — mechanical, low-risk)**
- `deep_causality_cfd/src/solvers/dec/{dec_ns_rate.rs,mod.rs}`, `src/theories/incompressible_dec.rs` — the
  convective-operator docstrings (item 17a) and the Chorin/in-stage contradiction (`mod.rs:21-26`).
- `deep_causality_cfd/src/tensor_bridge/projection.rs:157,169` — the two spectral-projector comments (item 17b).
- `deep_causality_cfd/README.md` and per-module docstrings — the ~86 ACTION-LIST doc-overclaim rows
  (item 16) and the doc-gap capability sections (item 18: `DuctMarchRun`, `IgnitionCorridor`,
  snapshot/resume, `AcousticCoreInverse`, …).
- `deep_causality_cfd/verification/qtt_taylor_green_verification/{README.md,print_utils.rs}`,
  `verification/README.md` — the convergence-order qualification and the temporal floor/max-length note
  (item 21).

**Constants / traceability**
- `verification/qtt_cylinder_verification/config.rs`, `verification/qtt_park2t_blackout/config.rs` —
  `SMOOTH_CELLS` provenance (item 23a).
- `src/types/flow/compressible_march_run.rs:326-327` — the Mach-1.05 floor justification (item 23c).
- `deep_causality_cfd/papers/` — add the Angot / Bruneau & Fabrie (1999) penalization reference; cite it
  from the immersed-body harness (item 23d). Note the two orphan PDFs (`mittal2005.pdf`, `mohamed2016.pdf`,
  cited by author name nowhere) — index or cite, do not delete without owner approval.

**Code (small, and one new test)**
- `deep_causality_cfd/src/types/flow/gates.rs` — `Gates::finish()` refuses an empty set; adoption or
  retirement per the owner's item-22 decision.
- `deep_causality_cfd/tests/solvers/qtt/incompressible_2d_tests.rs` — a new behavioural test driving
  `rate_pair` with `u,v ≠ 0` against an analytic reference (item 24). Public-API behavioural only — no
  source-text scraping, per the standing rule (§5c).
- `verification/qtt_taylor_green_verification/main.rs` — route the convection check through the shipped
  `rate_pair` rather than a `gradient_x`/`gradient_y` re-assembly (item 24).

**Evidence**
- No example figure moves — this change touches no marcher, kernel, or gate bound. The one behavioural
  addition (item 24) is a new test, verified by `bazel test //...`; the harness re-route (item 24) is
  checked to leave the harness's reported convection error unchanged.

**Risk**
- **Deletion sensitivities.** Retiring `Gates` (item 22) and removing an orphan PDF (item 23) are
  deletions; both are owner decisions and neither is taken without approval. The change proceeds on the
  no-deletion path (correct the README, adopt or defer) unless the owner directs otherwise.
- **The audit's counts are estimates, not manifests.** "87 doc-overclaims" is 86 actionable rows in the
  ACTION-LIST and 91 raw occurrences across the module reports; "39 undocumented capabilities" is the
  doc-gap category total, a mix of README capability sections and scattered docstring gaps. The change
  works the catalogue and records the true count it reconciled, rather than asserting a round number.
