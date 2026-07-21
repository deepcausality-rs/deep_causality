<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# CFD Audit — Execution Ledger

Every verification harness, study and avionics example was built in `--release` and executed.

> **This ledger is the pre-remediation record (2026-07-21, before Phase 1).** It is kept as measured;
> the post-Phase-1 state is summarised below and is materially different. In particular the gate
> counts here are what made blocker B-3 visible — three harnesses emitted no gate marker at all.

## Post-remediation state (after Phase 1)

| | This ledger (before) | After Phase 1 |
|---|---|---|
| Harnesses executed by CI | **0** | 9 per PR, 4 nightly |
| Harnesses with no gate at all | 3 (`dec_cylinder`, `mms_taylor_green`, `dec_graded_mms`) | 0 |
| Gate lines across the PR suite | — | 38, all evidence-classed |
| Harnesses exiting non-zero | 0 / 13 | 1 / 13 — `qtt_cylinder`, **known-failing by design** |
| Baselines carrying a verdict | 6 / 12 | 13 / 13 |
| PR-suite wall-clock (uncontended) | — | 12.7 s |

The single non-zero exit is not a regression: `qtt_cylinder_verification` gained η and mask-smoothing
ladders that measure a real non-convergence in the Brinkman configuration (audit report §5b). It runs
nightly and is routed to Phase 2.

**Every "exit 0" in the tables below should be read against that.** At the time of this ledger a
zero exit meant only that the program ran — several came from gates that could not fail, and one
harness exited 0 even after a solver error.

> **Timings are not comparable to the README.** These runs executed while 16 audit agents were
> running concurrently on the same machine. Contention inflates wall-clock substantially
> (the lid cavity ran 1407 s against a documented ~28 s at a different grid). Exit codes and
> gate counts are unaffected and are the meaningful columns.

Gate columns count literal `[PASS]` / `[FAIL]` markers in each program's stdout+stderr.
A harness with 0 gates emits no marker — see the audit report, blocker B-3.

## Verification harnesses (13)

| Program | Exit | Wall-clock (s, contended) | README (s) | `[PASS]` | `[FAIL]` |
|---|---|---|---|---|---|
| `mms_taylor_green_verification` | 0 | 1 | ~1 | 0 | 0 |
| `dec_graded_mms_verification` | 0 | 1 | ~1 | 0 | 0 |
| `dec_taylor_green_re1600_verification` | 0 | 1 | <1 | 0 | 0 |
| `qtt_taylor_green_verification` | 0 | 1 | <1 | 0 | 0 |
| `qtt_cylinder_verification` | 0 | 5 | ~1 | 0 | 0 |
| `qtt_park2t_blackout` | 0 | 4 | ~1 | 6 | 0 |
| `qtt_sod` | 0 | 1 | ~1 | 3 | 0 |
| `qtt_ramc_stagline` | 0 | 0 | ~1 | 7 | 0 |
| `qtt_blunt_body_2d` | 0 | 1 | ~2 | 0 | 0 |
| `qtt_reentry_3d` | 0 | 1 | ~3 | 0 | 0 |
| `dec_lid_cavity_re1000_verification` | 0 | 1407 | ~28 (33²) | 0 | 0 |
| `dec_cylinder_wake_verification` | 0 | 187 | ~155 | 0 | 0 |
| `dec_cylinder_verification` | 0 | 578 | ~510 | 0 | 0 |

**Totals:** 13 programs, 0 non-zero exits, 16 PASS markers, 0 FAIL markers.

## Studies (9)

| Program | Exit | Wall-clock (s, contended) | `[PASS]` | `[FAIL]` |
|---|---|---|---|---|
| `qtt_rank_study` | 0 | 28 | 0 | 0 |
| `qtt_rank_dynamic` | 0 | 9 | 0 | 0 |
| `qtt_rank_nonlinear` | 0 | 84 | 0 | 0 |
| `qtt_rank_3d` | 0 | 97 | 0 | 0 |
| `qtt_rank_fitted_dynamic` | 0 | 235 | 0 | 0 |
| `qtt_acoustic_precond` | 0 | 6 | 0 | 0 |
| `qtt_blend_metric` | 0 | 5 | 0 | 0 |
| `qtt_repin_marcher` | 0 | 196 | 0 | 0 |
| `qtt_rank_plume` | 0 | 149 | 0 | 0 |

**Totals:** 9 programs, 0 non-zero exits, 0 PASS markers, 0 FAIL markers.

## Avionics examples (7)

| Program | Exit | Wall-clock (s, contended) | `[PASS]` | `[FAIL]` |
|---|---|---|---|---|
| `turbulence_flow` | 0 | 3 | 0 | 0 |
| `flight_envelope_placard` | 0 | 3 | 2 | 0 |
| `nozzle_operating_map` | 0 | 4 | 4 | 0 |
| `viv_resonance_margin` | 0 | 160 | 3 | 0 |
| `plasma_blackout_corridor` | 0 | 55 | 13 | 0 |
| `plasma_blackout_weather` | 0 | 276 | 8 | 0 |
| `plasma_blackout_retropulsion` | 0 | 378 | 16 | 0 |

**Totals:** 7 programs, 0 non-zero exits, 46 PASS markers, 0 FAIL markers.

## Unit tests

```
cargo test -p deep_causality_cfd --release
→ 813 passed; 0 failed; 2 ignored
```

## Committed-output reproducibility

| Example | Fresh run vs committed `output.txt` |
|---|---|
| `viv_resonance_margin` | byte-identical |
| `flight_envelope_placard` | identical but for trailing newline |
| `nozzle_operating_map` | identical but for trailing newline |
| `plasma_blackout_corridor` | identical but for the wall-clock line (40.9 s committed vs 48.2 s under load) |
