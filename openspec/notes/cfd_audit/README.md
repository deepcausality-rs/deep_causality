<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# CFD Pre-Certification Audit

Audit of `deep_causality_cfd` for avionics **R&D** certification, 2026-07-21.

**Verdict: not yet certifiable.** The numerical core is sound — Rankine–Hugoniot exact, Ghia cavity
matched to four decimal places, Sod against the exact Riemann solution. What blocks certification is the
assurance layer: many advertised gates cannot fail or cannot discriminate, and none of them run in CI.

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

`ACTION-LIST.md` plus `AUDIT-REPORT.md` §9 are intended to be converted into a follow-up OpenSpec change
and implemented as a dedicated remediation project.
