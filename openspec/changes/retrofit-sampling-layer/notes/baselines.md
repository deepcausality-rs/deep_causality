<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Baselines

Measured on `main` at `eaa17aaf3` on 2026-09-14, before any task in this change ran. Every later
group compares against these numbers; a group that ends below its crate's row has regressed
something and is not done.

## Test counts

| Crate | Tests | Failing | Why it is here |
|---|---|---|---|
| `deep_causality_rand` | 154 | 0 | groups 1, 3, 5 |
| `deep_causality_stats` | 467 | 0 | groups 2, 3, 4A–4H |
| `deep_causality_uncertain` | 248 | 0 | group 5.3–5.5, the umbrella |
| `deep_causality_topology` | 1697 | 0 | group 5.7, the second crate to reach `stats` alone |

`stats` and `uncertain` were not in the proposal's baseline line, which said only "to be counted at
phase 1". They are counted here.

`topology` was not listed either. It is now, because 5.7 migrates it and 1 697 tests are the
evidence that the migration preserved behaviour rather than merely compiled.

## Source size

| Crate | Lines in `src/` |
|---|---|
| `deep_causality_rand` | 2 293 |
| `deep_causality_stats` | 2 788 |

Task 3.5 compares the `rand` figure after the per-type distribution files are deleted. The
prototype measured 129 lines net removed from `rand` for the split alone; the move to `stats` will
take considerably more out of it and put most of that into `stats`, so the interesting figure is
the pair, not either one alone.

## Reproducing

```bash
for c in rand stats uncertain topology; do
  cargo test -p deep_causality_$c 2>&1 | grep -E "^test result" \
    | awk -v c=$c '{p+=$4; f+=$6} END {print c": "p" passed, "f" failed"}'
done
```
