<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# CFD Audit — Execution Ledger

Every verification harness, study and avionics example was built in `--release` and executed.

> **Timings are not comparable to the README.** These runs executed while 16 audit agents were
> running concurrently on the same machine. Contention inflates wall-clock substantially
> (the lid cavity ran 1407 s against a documented ~28 s at a different grid). Exit codes and
> gate counts are unaffected and are the meaningful columns.

Gate columns count literal `[PASS]` / `[FAIL]` markers in each program's stdout+stderr.
A harness with 0 gates emits no marker — see the audit report, blocker B-3.

## Verification harnesses (12)

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

**Totals:** 12 programs, 0 non-zero exits, 16 PASS markers, 0 FAIL markers.

## Studies (0)

_Not yet complete at time of writing._

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
