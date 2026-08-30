<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `deep_causality_linear` design prototype

Throwaway code kept for its evidence. It answers two questions that
`openspec/changes/archive/2026-08-30-add-linear-algebra-crate/` states as requirements — now the
`linear-*` specs under `openspec/specs/` — and the requirements are only
as good as these measurements, so the measurements are kept runnable.

This is a **separate Cargo workspace**, deliberately not a member of the repository workspace. It
builds against the real `deep_causality_num` and `deep_causality_algebra` by relative path, so it
tracks the tower as the tower changes. Nothing here ships.

## What it answers

**Does an access trait preserve word-parallel XOR?** One generic `rref` in `linear_b`, written
against a row-operation trait, runs over four representations of the same 𝔽₂ matrix. If the trait
leaked per-element access into the inner loop, packed and unpacked would run at the same speed.
They do not, and the generic packed path also beats a hand-written non-generic elimination over
`&mut [u64]`.

**Does bit-packing earn its complexity?** `linear_a/src/gf2_scalar.rs` builds the alternative the
algebra tower alone permits — a `Gf2` scalar with all eight operator impls plus every marker needed
to satisfy `deep_causality_algebra::Field`, stored one byte per bit. `consumer_b` runs the same
algorithm over that and over `PackedGf2<u64>`.

**Where can the impl live?** `tensor_impl` holds an orphan-rule probe. Under
`--features orphan_probe` it must fail with E0117, showing that a third crate cannot broker
`impl MatrixView for CausalTensor<f64>`.

## Layout

| crate | role |
|---|---|
| `linear_a` | design A — the `Matrix<F2>`-over-a-new-`Field`-impl route, built to be priced, not endorsed |
| `linear_b` | design B — algorithms only, generic over `MatrixView` / `RowOps` / `MatrixBuild`. This is the design the change adopts |
| `linear_c` | design C — free functions over `&mut [Vec<F>]` |
| `consumer_b` | implements design B's traits in a crate that did not define them — the position `deep_causality_tensor` and `deep_causality_topology` would be in |
| `tensor_impl` | orphan-rule probe against the real `CausalTensor` and `CsrMatrix` |

## Running it

```bash
cd openspec/notes/linear/prototype

# the benchmark, at any size
cargo run --release --example f2_word_parallel -p consumer_b 1024

# the orphan probe — this MUST fail with E0117
cargo build -p tensor_impl --features orphan_probe
```

## Recorded results

M3 Max, 16 cores, 128 GB; `--release`. Reproduced from this location on 2026-08-23.

| n | packed `u64`, generic | packed, hand-written | `Gf2` byte scalar | packed vs byte | seam cost | memory |
|---|---|---|---|---|---|---|
| 128 | 103.25 µs | 110.50 µs | 170.58 µs | 1.7× | 0.93× | 2 vs 16 KiB |
| 256 | 410.50 µs | 447.79 µs | 702.50 µs | 1.7× | 0.92× | 8 vs 64 KiB |
| 512 | 1.803 ms | 1.896 ms | 3.341 ms | 1.9× | 0.95× | 32 vs 256 KiB |
| 1024 | 7.65 ms | 8.23 ms | 18.23 ms | 2.4× | 0.93× | 128 vs 1024 KiB |
| 2048 | 34.67 ms | 37.43 ms | 111.93 ms | 3.2× | 0.93× | 512 vs 4096 KiB |

The three columns above are stable across runs. The benchmark also prints a `Vec<Vec<Gf2>>` figure
for design C; that one varies by up to 1.5× between runs on the same input and nothing in the change
depends on it.

## Caveats

- The benchmark times one run per size with no warm-up and no statistical treatment. It is sized to
  separate a 2–3× effect, not to resolve a 5% one. The production benchmark is task 3.6, in
  `benches/` under `criterion`.
- `Dense<Gf2>` uses the exact-field pivot rule. A float dense matrix pivots on magnitude, which the
  row-operation trait supports through an overridable method and which this prototype does not
  exercise.
- `rows_of_rows.rs` and `DensePivoted` in `consumer_b` are unused by the benchmark and carry dead-code
  warnings. Left as written rather than tidied, so the measured artifact stays the measured artifact.
