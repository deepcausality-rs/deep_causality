# Near-Linear BRCD — Measured Results

Author: Marvin Hansen

Companion to `paper-thesis.md` and `paper-impl-draft.md`. This file holds the raw
result tables and the exact commands that produce them, so every number in the
thesis/impl drafts is reproducible from a clean checkout of the `brcd-next` branch.

## How to read these numbers

Two classes of quantity appear:

- **Configuration-evaluation counts are exact and deterministic** — `2^{du}` for the
  `Full` strategy, `du + 1` for `MapPrune`. These do not vary by machine or seed and
  are the robust headline.
- **Wall-clock is indicative only** — medians over ≥5 seeds (accuracy/compute) or
  ≥3 reps (cache), measured on a single developer machine in `--release`. Absolute
  milliseconds will differ elsewhere; the *shape* of the curve and the relative
  ordering are the point.

Accuracy metrics: **top-1** = the planted root cause is ranked first; **top-3** = it
is in the first three; **top-1 agree** = `Full` and `MapPrune` choose the same top
candidate set on the same graph.

## Commands

```bash
# Accuracy / compute: MapPrune vs Full (Sweeps A, B, C below). With no args every
# sweep runs; pass a selector to run a subset, e.g. `-- c` for only the large-n sweep.
cargo run --release -p deep_causality_algorithms --example brcd_eval_accuracy_compute
cargo run --release -p deep_causality_algorithms --example brcd_eval_accuracy_compute -- c

# Same harness with multicore likelihood scoring (rayon) enabled:
cargo run --release --features parallel -p deep_causality_algorithms --example brcd_eval_accuracy_compute -- c

# Cold (learn) vs warm (cached) CPDAG resolution through the CDL pipeline
cargo run --release -p deep_causality_discovery   --example brcd_cache_cold_vs_warm
```

Sources: `deep_causality_algorithms/verification/brcd/brcd_eval_accuracy_compute.rs`
and `deep_causality_discovery/examples/brcd_cache_cold_vs_warm.rs`.

---

## 1. Accuracy vs compute — `MapPrune` vs `Full`

### Sweep A — controlled degree (planted cliques, `du = c−1`)

Strong separation (perturb = 4.0), 5 seeds, 150 rows/regime. This is the regime where
the `2^{du}` wall actually bites. `Full` is enumerated up to `MAX_CONFIG_EDGES = 16`
(`du ≤ 16`); past that it refuses ("—") and only `MapPrune` runs.

| c | du | Full cfg (`2^{du}`) | MAP eval (`du+1`) | Full top-1 | MAP top-1 | Full top-3 | MAP top-3 | Full ms | MAP ms |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 4 | 3 | 8 | 4 | 100% | 100% | 100% | 100% | 0.89 | 2.11 |
| 5 | 4 | 16 | 5 | 100% | 100% | 100% | 100% | 2.86 | 4.81 |
| 6 | 5 | 32 | 6 | 100% | 100% | 100% | 100% | 8.58 | 12.52 |
| 7 | 6 | 64 | 7 | 100% | 100% | 100% | 100% | 25.56 | 22.24 |
| 8 | 7 | 128 | 8 | 100% | 100% | 100% | 100% | 73.66 | 43.61 |
| 9 | 8 | 256 | 9 | 100% | 100% | 100% | 100% | 204.01 | 71.33 |
| 10 | 9 | 512 | 10 | 100% | 100% | 100% | 100% | 547.46 | 108.21 |
| 11 | 10 | 1024 | 11 | 100% | 100% | 100% | 100% | 1487.63 | 173.46 |
| 12 | 11 | 2048 | 12 | 100% | 100% | 100% | 100% | 4054.15 | 314.47 |
| 13 | 12 | 4096 | 13 | 100% | 100% | 100% | 100% | 10905.14 | 401.54 |
| 18 | 17 | — | 18 | — | 100% | — | 100% | — | 2332.59 |
| 22 | 21 | — | 22 | — | 100% | — | 100% | — | 7558.10 |
| 26 | 25 | — | 26 | — | 100% | — | 100% | — | 19436.11 |

**Reading.** Top-1 *and* top-3 are identical (both 100%) everywhere `Full` is
feasible. `Full`'s config count is exactly `2^{du}` and its wall-clock explodes
(10.9 s at `du = 12`); past `du = 16` it refuses entirely, while `MapPrune` stays at
`du + 1` evaluations and completes through `du = 25`. At this strong separation even
the clique *tail* is preserved (top-3 100%); the looser clique-tail figure
(Kendall-τ ≈ 0.76) appears only at weaker separation — configuration concentration
tightens with detectability, exactly as the thesis predicts.

#### Sequential vs parallel (`--features parallel`)

Same machine, **16 logical cores**, same Sweep A graphs, median ms. Speed-up = seq / par.

| du | Full seq | Full par | Full × | MapPrune seq | MapPrune par | MapPrune × |
|--:|--:|--:|--:|--:|--:|--:|
| 3 | 1.66 | 1.01 | 1.6× | 3.82 | 3.00 | 1.3× |
| 4 | 3.30 | 1.52 | 2.2× | 6.26 | 4.59 | 1.4× |
| 5 | 8.20 | 4.27 | 1.9× | 11.95 | 10.12 | 1.2× |
| 6 | 24.31 | 13.60 | 1.8× | 21.89 | 17.46 | 1.3× |
| 7 | 71.92 | 43.81 | 1.6× | 43.88 | 33.49 | 1.3× |
| 8 | 201.34 | 133.80 | 1.5× | 69.34 | 54.96 | 1.3× |
| 9 | 544.23 | 383.03 | 1.4× | 105.43 | 83.17 | 1.3× |
| 10 | 1482.92 | 1107.97 | 1.3× | 169.69 | 134.03 | 1.3× |
| 11 | 3992.03 | 3153.77 | 1.3× | 316.29 | 255.26 | 1.2× |
| 12 | 10654.55 | 8856.11 | 1.2× | 394.25 | 317.83 | 1.2× |
| 17 | — | — | — | 2378.36 | 1955.06 | 1.2× |
| 21 | — | — | — | 7751.47 | 6677.45 | 1.2× |
| 25 | — | — | — | 19608.48 | 16994.33 | 1.2× |

**Reading — and it refutes the obvious guess.** Parallelism gives only a *modest*
speed-up on 16 cores (≤ ~2.2×, never close to linear), and for `Full` it **shrinks as
`du` grows** — 2.2× at du=4 down to 1.2× at du=12. That is the opposite of "more configs
⇒ more parallel win." The reason: the `2^{du}` blow-up lives in the **sequential**
per-configuration loop (augment → MEC-size → sample, once per configuration), *not* in
the parallelized family-scoring map. As `du` grows the sequential config loop dominates,
Amdahl's serial fraction → 1, and the speed-up collapses toward 1× exactly where compute
hurts most. `MapPrune` sits at a flat ~1.2–1.4× (few configs; the parallel scoring share
is small and roughly constant).

### Sweep B — scaled `n` (random linear-Gaussian CPDAGs)

perturb = 3.0, ≥10 graphs per `n`, restricted to graphs where `Full` is feasible so
both strategies run on the same graphs.

| n | Full top-1 | MAP top-1 | Full top-3 | MAP top-3 | Full ms | MAP ms | top-1 agree |
|--:|--:|--:|--:|--:|--:|--:|--:|
| 10 | 100% | 100% | 100% | 100% | 0.55 | 2.19 | 100% |
| 25 | 100% | 100% | 100% | 100% | 1.79 | 13.35 | 100% |
| 50 | 100% | 100% | 100% | 100% | 9.28 | 75.55 | 100% |
| 75 | 100% | 100% | 100% | 100% | 26.36 | 212.63 | 100% |
| 100 | 100% | 100% | 100% | 100% | 76.29 | 558.21 | 100% |

**Reading (honest — not a uniform speed-up).** Accuracy is identical
(top-1/top-3 100%, 100% top-1 agreement). But here `MapPrune` is *slower* in
wall-clock than `Full`: on low-`du` random CPDAGs most candidates have only a handful
of valid configs, so `Full`'s direct enumeration is already trivial and the finder's
hill-climb bookkeeping (clone, Meek, MEC sizing per visited orientation) costs more.
The compute win is **specifically the high-local-degree regime** (Sweep A); on
low-`du` graphs full enumeration is cheap and pruning buys nothing in time. The
portable claim is the deterministic `2^{du} → du + 1` reduction and the `du > 16`
feasibility cliff, not a blanket wall-clock win.

### Sweep C — large `n` to 1000 (bounded-degree CPDAGs)

The axis of the original paper's Fig-2b (runtime vs number of variables, out to
n = 1000). Expected in-degree is held ~constant (`p_edge = 2/n`) so local undirected
degree stays bounded as `n` grows; both strategies run on the same graphs. perturb =
3.0, 150 rows/regime, 2–4 graphs per `n`. This is a **Rust-native continuous**
generator, *not* the paper's discrete pyAgrum protocol.

| n (graphs) | Full top-1 | MAP top-1 | Full top-3 | MAP top-3 | Full ms | MAP ms | top-1 agree |
|--:|--:|--:|--:|--:|--:|--:|--:|
| 50 (4) | 100% | 100% | 100% | 100% | 7.8 | 43.4 | 100% |
| 100 (4) | 100% | 100% | 100% | 100% | 24.8 | 153.2 | 100% |
| 250 (3) | 100% | 100% | 100% | 100% | 265.8 | 1088.9 | 100% |
| 500 (2) | 100% | 100% | 100% | 100% | 1512.6 | 5114.0 | 100% |
| 1000 (2) | 100% | 100% | 100% | 100% | 11088.2 | 29390.3 | 100% |

**Reading (honest).** Both strategies **complete at n = 1000** (Full ≈ 11.1 s,
MapPrune ≈ 29.4 s), accuracy identical (top-1/top-3 100%, 100% agreement), and **no
exponential in n** — the bounded-degree regime removes the `2^{du}` wall, which is the
point. Two caveats stated plainly:

- **Wall-clock is super-linear, not near-linear.** From n = 100 to n = 1000 (10×) Full
  grows 24.8 ms → 11.1 s (≈ 447×, empirical exponent ≈ 2.6) — roughly cubic, dominated
  by per-candidate graph augmentation and likelihood scoring over all `n` nodes. The
  achievement is **exponential → polynomial** (both exponential factors removed); it is
  *not* linear in `n`. (The "near-linear" headline elsewhere refers to the per-candidate
  configuration factor `2^{du} → du + 1`, not to total runtime in `n`.)
- **MapPrune is ≈ 2.6× slower than Full here**, consistently — the same bounded-`du`
  finder overhead seen in Sweep B. Pruning wins only where `du` is large (Sweep A).

**Comparison to the original Fig-2b.** The original Python BRCD reports ≈ 150 s at
n = 1000; our Rust ranker completes the same axis at ≈ 11 s (Full) / ≈ 29 s (MapPrune).
This is **not a controlled head-to-head** — different language, a different (continuous,
Rust-native) generator vs the paper's discrete pyAgrum BNs, a different machine, and the
paper's figure bundles a heavier discrete pipeline. Read it as: our implementation
reproduces the Fig-2b *scaling shape* and stays comfortably within practical latency at
n = 1000 — not as a literal speed-up factor.

#### Sequential vs parallel (`--features parallel`)

The crate's `parallel` feature parallelizes BRCD's likelihood **family-scoring** map
(`rayon` `par_iter` in `brcd_algo`); the per-candidate structural work (augmentation,
config enumeration, MEC sizing) stays sequential. Same machine, **16 logical cores**,
same Sweep C graphs, median ms:

| n | Full seq | Full par | MapPrune seq | MapPrune par |
|--:|--:|--:|--:|--:|
| 50 | 7.5 | 7.1 | 40.2 | 42.3 |
| 100 | 24.4 | 22.8 | 145.5 | 140.4 |
| 250 | 257.6 | 257.1 | 1085.1 | 1099.8 |
| 500 | 1502.9 | 1488.9 | 5044.4 | 5060.4 |
| 1000 | 10956.8 | 10895.3 | 29263.8 | 28452.2 |

**Reading (honest).** In this large-`n`, bounded-degree regime **parallel ≈ sequential**
(≈ 1.0× on 16 cores, within run-to-run noise). By Amdahl's law that means the
parallelized family-scoring map is a *tiny* fraction of total cost here; the bottleneck
is the **sequential per-candidate structural work** (graph augmentation / config / MEC
sizing — the cubic term), which the `parallel` feature does not touch. The feature is
matched to the *opposite* regime: many configurations per candidate (high `du`,
Sweep-A-like), where family scoring dominates. So on Fig-2b-style wide, sparse graphs
multicore buys ~nothing — the lever is reducing per-candidate structural overhead, not
adding cores.

### When does parallelism help? (synthesis of Sweeps A + C)

The `parallel` feature parallelizes exactly **one** thing — the likelihood family-scoring
map. Whether that matters is entirely a question of how large a share of total cost that
map is in a given regime:

| Regime | Dominant cost (sequential unless noted) | Parallel speed-up (16 cores) |
|---|---|--:|
| Sweep A, low `du` | small config loop; scoring is a real share | ~1.5–2.2× |
| Sweep A, high `du` (`Full`) | `2^{du}` config loop (augment/MEC/sample) | ↓ to ~1.2× |
| Sweep A, any `du` (`MapPrune`) | few configs; little to parallelize | ~1.2–1.4× |
| Sweep C, large `n` sparse | per-candidate structural work | ~1.0× |

**Bottom line.** Multicore helps only in the narrow band where the scoring map is a
meaningful fraction of work (small/moderate graphs), and it never reaches near-linear
scaling. The two regimes that actually dominate runtime — the `2^{du}` configuration
enumeration (high `du`) and the per-candidate structural work (high `n`) — are **both
sequential** and untouched by the current feature, so parallelism does *least* exactly
where it would matter *most*. Real parallel speed-up would require parallelizing the
**configuration-enumeration loop** (high-`du` path) or the **candidate loop** (high-`n`
path), not the family-scoring map.

---

## 2. Learn-once cache — cold vs warm

CDL pipeline driven end to end (`brcd_load_input → brcd_discover`) with a
`cpdag_cache_path` and no supplied CPDAG, on a synthetic 30-variable dataset
(800 normal + 800 anomalous rows, planted root cause `v25`). Median of 3 reps, fresh
cache per cold rep.

| Run | What it does | Time (ms) |
|---|---|--:|
| Cold | BOSS learns + persists + rank | 290.0 |
| Warm | cache load + rank | 12.3 |
| Δ | structure learning avoided | 277.8 |

Speed-up ≈ **23.6×**. The warm ranking is asserted **equal** to the cold ranking
(both top `[v25]`) — the cache is correct, not merely fast. The warm number is the
failure-period cost the production (online) path actually pays once structure learning
is amortized away.
