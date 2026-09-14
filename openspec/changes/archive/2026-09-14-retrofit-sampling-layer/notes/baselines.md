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

## Group 4 close-out, measured 2026-09-14

| Crate | Baseline | Now | Delta |
|---|---|---|---|
| `deep_causality_stats` | 467 | **623** | +156 |
| `deep_causality_rand` | 154 | 101 | −53, in groups 1–3 |

Group 4 touched only `stats`: `git status` over `deep_causality_rand` is empty for the whole group,
so the `rand` row is carried from the group-3 close-out rather than re-argued here. The 53 tests it
lost left with the code they test — the per-type distribution files that moved to `stats` in
`cec19c870` — and the pair is up by 103 over the baseline.

One test was **removed** in the close-out rather than added: `cauchy_tests` carried an empty
`no_moment_assertion_exists_in_this_module`, a body with nothing in it, standing as a marker for
the module's prohibition on moment assertions. A test that cannot fail reads as coverage and is
not, so the prohibition was moved into the module documentation where it is actually read.

Source size, for the same reason, is now the pair: `rand` 1 398 lines against 2 293 at baseline,
`stats` 4 303 against 2 788. The sampling layer as a whole grew by 620 lines, and what it bought is
seven distributions that did not exist and one that was written by hand at every call site.

### A test cargo ran and Bazel did not

`all_scalars_tests.rs` was first written at `tests/types/distr/all_scalars_tests.rs`. Cargo ran it —
it is registered in `tests/types/distr/mod.rs` like any other module. Bazel did not: the crate's
`rust_test_suite` globs `tests/types/distr/*/*_tests.rs`, exactly one directory deep, and the file
sat at the top of that directory.

`bazel test //...` came back green over 1 384 tests without ever compiling it. The evidence that it
was missing is a `bazel query` for the target, not a passing run, and that is the check worth
keeping: a green workspace says nothing about a target that does not exist.

Moved to `tests/types/distr/all_scalars/all_scalars_tests.rs`, which is where every other
distribution test already lives. The layout was not decoration.
