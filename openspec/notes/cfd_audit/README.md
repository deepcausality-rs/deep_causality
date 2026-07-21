<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# CFD Pre-Certification Audit

Audit of `deep_causality_cfd` for avionics **R&D** certification, 2026-07-21.

**Verdict: not yet certifiable. Phase 1 complete; Phase 2 is next.** The numerical core is sound —
Rankine–Hugoniot exact, Ghia cavity matched to four decimal places, Sod against the exact Riemann
solution. What blocked certification was the assurance layer: many advertised gates could not fail or
could not discriminate, and none of them ran in CI.

## Remediation status

**Phase 1 is implemented and archived** as
[`2026-07-21-make-cfd-evidence-enforceable`](../../changes/archive/2026-07-21-make-cfd-evidence-enforceable/),
43/43 tasks, with its four capability specs synced into `openspec/specs/`.

| Blocker | Status |
|---|---|
| B-1 Millikan–White reduced mass mislabelled | open — **Phase 2** |
| B-2 No CI executes the verification suite | **resolved** |
| B-3 `dec_cylinder_verification` has no gate | **resolved** |
| B-4 `BlendedMap` documents an absent fold check | open — **Phase 2** |

| | Before | After Phase 1 |
|---|---|---|
| CI executes the verification suite | no | 9 per PR, 4 nightly |
| Harnesses with no gate at all | 3 | 0 |
| Gate lines declaring an evidence class | 0 | 38 / 38 |
| Baselines carrying a verdict | 6 / 12 | 13 / 13 |
| Unit tests | 813 | 828 |

Phase 1 also **found three things this report missed**, all recorded in the report itself: two further
ungated harnesses beyond B-3, six baselines with no verdict rather than one truncated, and a new
physics finding from a gate that did not previously exist (§5b). It also **refuted one finding** —
the energy-budget gate is not a tautology (§5), and the correct action was to change nothing.

## Read in this order

| Document | What it is |
|---|---|
| [`AUDIT-REPORT.md`](AUDIT-REPORT.md) | **Start here.** Verdict, method and its limits, evidence, 4 blockers, what the adversarial stage overturned, readiness ranking, phased path. |
| [`ACTION-LIST.md`](ACTION-LIST.md) | Every surviving finding as a severity-ordered, actionable table. The input for the follow-up specification. |
| [`MODULE-INDEX.md`](MODULE-INDEX.md) | Per-module readiness at a glance. |
| [`RUN-LEDGER.md`](RUN-LEDGER.md) | What was executed, exit codes, gate counts, reproducibility check. |
| [`modules/`](modules/) | 16 per-module reports: findings with quoted code evidence, reference forms, and adversarial verdicts. |

## Headline numbers

| | |
|---|---|
| Source files read | 412 |
| Findings raised | 294 |
| Adversarially re-checked | **294 (100 %)** — 190 confirmed, 100 partially, 4 refuted |
| Post-review severity | **4 critical**, 72 major, 179 minor, 35 info |
| Positively confirmed correct against a reference | 176 |
| Unit tests | 813 pass, 0 fail, 2 ignored |
| Verification harnesses executed | 13 / 13, all exit 0 |
| Avionics examples executed | 7 / 7, all exit 0 |

## The three findings that decide the verdict

1. **The physics core is sound.** Every closed-form reference checked by hand matched to displayed
   precision.
2. **72 of 294 findings are tautology/circular-reasoning defects.** Several headline gates are
   algebraically incapable of failing; for the QTT immersed cylinder, no gate constrains drag
   correctness at all.
3. **No CI job runs any of the 13 verification programs.** Every quantitative accuracy claim in the
   crate is unenforced.

## The four blockers

| | Finding | Location |
|---|---|---|
| B-1 | Millikan–White reduced mass is 7.0 amu for a pair documented as N₂–N₂ (correct: 14). Duplicated in two places; propagates into all three plasma-blackout examples. Correcting it likely removes the headline RAM-C agreement. | `fitting.rs:72`, `config.rs:37`, `constants.rs:128` |
| B-2 | No CI or Bazel target executes the verification suite. | `.github/workflows/` |
| B-3 | `dec_cylinder_verification` has zero gates and exits 0 after a solver error. | `dec_cylinder_verification/main.rs` |
| B-4 | `BlendedMap` documents a fold check it does not perform; unguarded ÷det J. | `coordinate/blended.rs:17,171` |

## On method

The adversarial stage was re-run to completion, including two modules whose first verifiers hit a
session limit. It cut criticals from 26 to 4 and majors from 131 to 72, and refuted 4 findings outright
— including **one the lead auditor had personally endorsed** (see §5 of the audit report). Two verifiers
executed probes against the shipped code to reproduce or disprove the auditors' numbers rather than
reasoning about them.

Findings were not accepted because they sounded plausible. Where the evidence did not survive, it is
recorded as overturned rather than quietly dropped.

## Caveats

- Severity labels were not normalized across modules — compare within a module, not across.
- Runtimes in the ledger are **not** comparable to the README's; these ran under heavy contention.
- The 4 criticals are asserted as certain. The 72 majors are verified but should be triaged.

## Next step

**Phase 2 — close the physics defects** (`AUDIT-REPORT.md` §9, items 7–15). Propose it the same way
Phase 1 was proposed, drawing its scope from §9 and its per-finding evidence from `ACTION-LIST.md`.

Two things make Phase 2 easier than Phase 1 was:

- **Item 10 already has a failing acceptance test.** `qtt_cylinder_verification` fails nightly on
  exactly the Brinkman-envelope condition item 10 describes; the fix is done when its η ladder
  converges. No new harness is needed.
- **Every other item is now detectable.** Phase 1's job was to make the evidence layer capable of
  catching these; a Phase-2 fix that regresses will be caught by a gate that can fail.

Item 7 (`REDUCED_MASS_AMU`) is the one to sequence first and carefully: correcting it likely *removes*
the headline RAM-C agreement rather than preserving it, and the report is explicit that the gates must
be re-derived from the corrected physics rather than re-tuned to restore the old number.
