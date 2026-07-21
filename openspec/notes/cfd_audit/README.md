<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# CFD Pre-Certification Audit

Audit of `deep_causality_cfd` for avionics **R&D** certification, 2026-07-21.

**Verdict: not yet certifiable.** The numerical core is sound — Rankine–Hugoniot exact, Ghia cavity
matched to four decimal places, Sod against the exact Riemann solution. What blocks certification is the
assurance layer: many advertised gates cannot fail, and none of them run in CI.

## Read in this order

| Document | What it is |
|---|---|
| [`AUDIT-REPORT.md`](AUDIT-REPORT.md) | **Start here.** Verdict, method and its limits, evidence, 10 confirmed blockers, readiness ranking, phased path to certification. |
| [`ACTION-LIST.md`](ACTION-LIST.md) | Every finding as a severity-ordered, actionable table. The input for the follow-up specification. |
| [`MODULE-INDEX.md`](MODULE-INDEX.md) | Per-module readiness at a glance. |
| [`RUN-LEDGER.md`](RUN-LEDGER.md) | What was executed, exit codes, gate counts, reproducibility check. |
| [`modules/`](modules/) | 16 per-module reports: findings with quoted code evidence, reference forms, and adversarial verdicts. |

## Headline numbers

| | |
|---|---|
| Source files read | 412 |
| Findings raised | 294 |
| Adversarially re-checked | 253 (168 confirmed, 83 partially, 2 refuted) |
| Unverified (verifier died on session limit) | 41 |
| Post-review severity | 10 critical, 76 major, 175 minor, 31 info |
| Positively confirmed correct against a reference | 176 |
| Blockers confirmed at source by the lead auditor | 10 / 10 |
| Unit tests | 813 pass, 0 fail, 2 ignored |
| Verification harnesses executed | 13 / 13, all exit 0 |
| Avionics examples executed | 7 / 7, all exit 0 |

## The three findings that decide the verdict

1. **The physics core is sound.** Every closed-form reference checked by hand matched to displayed
   precision.
2. **72 of 294 findings are tautology/circular-reasoning defects.** Several headline gates are
   algebraically incapable of failing.
3. **No CI job runs any of the 13 verification programs.** Every quantitative accuracy claim in the
   crate is unenforced.

## Caveats

- Two modules (QTT incompressible/immersed, crate-wide constants sweep) were **never adversarially
  verified** — their verifiers hit a session limit. Their reports carry a warning banner. The lead
  auditor independently confirmed their two most consequential criticals.
- Only the 10 blockers in `AUDIT-REPORT.md` §4 are asserted as certain. The rest are credible,
  located, triage-ready leads.
- Runtimes in the ledger are **not** comparable to the README's — these ran under heavy contention.

## Next step

`ACTION-LIST.md` plus `AUDIT-REPORT.md` §8 are intended to be converted into a follow-up OpenSpec
change and implemented as a dedicated remediation project. Phase 0 (re-verify the two unverified
modules) should precede scoping, since 6 of the 10 criticals sit in them.
