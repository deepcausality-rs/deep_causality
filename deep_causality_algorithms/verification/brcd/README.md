# BRCD verification examples

These examples check the Rust BRCD port against the reference algorithm and against
captured outputs of the reference implementation.

The algorithm, the likelihood estimators, the BOSS structure learner, and the
data-preprocessing pipeline all come from:

> Kenneth Lee, Zihan Zhou, and Murat Kocaoglu. "Root Cause Analysis of Failures in
> Microservices via Bayesian Root Cause Discovery." International Conference on Machine
> Learning (ICML), 2026. <https://icml.cc/virtual/2026/poster/65359>

The datasets under `data/` are **derived, not raw**. The reference Python pipeline cleans
and windows the benchmark traces, and this repository stores the cleaned matrices together
with the reference ranking. The Rust port reads those matrices directly; it does not re-run
the Python preprocessing.

Each example is a standalone binary. It prints one `PASS`/`FAIL` line per check and exits
non-zero on any failure, so it runs in CI.

## Running

```bash
cargo run -p deep_causality_algorithms --example verification_base
cargo run -p deep_causality_algorithms --example verification_boss
cargo run --release -p deep_causality_algorithms --example verification_boss_sockshop
cargo run -p deep_causality_algorithms --example verification_online_boutique
cargo run -p deep_causality_algorithms --example verification_sockshop
```

`brcd_run` scores node families in parallel under the `parallel` feature. The result is
identical to the sequential path, so every example reproduces the same ranking either way.
Add `--features parallel` to any command to exercise that path.

## What gets checked, and against what

The examples test two separate things against two different references.

1. **The estimator, given a CPDAG.** The reference is the captured Python ranking
   (`expected.txt`), reproduced position for position.
2. **The structure learner (BOSS), which produces a CPDAG from data.** The reference is a
   known true graph on synthetic data. A learned CPDAG is a heuristic estimate, so an exact
   target does not exist on real data.

### `verification_base`: synthetic estimator smoke test

Generates a linear-Gaussian chain `X → Y → Z`, then an anomalous set that perturbs
`p(Y | X)`. Only Y's mechanism changes, so BRCD must rank Y first. The example is
self-contained and gates the heavier ones.

### `verification_boss`: BOSS structure learning

Runs BRCD with `cpdag = None`, so BOSS learns the CPDAG from the pre-failure data. It makes
two checks.

1. **Structural, on synthetic data with a known graph.** A chain `X → Y → Z` must yield the
   undirected path `X — Y — Z`; a collider `X → Z ← Y` must yield the directed v-structure.
   This is where BOSS has a true target.
2. **End to end, on real data.** On Online Boutique `adservice_cpu_1`, BOSS learns the
   structure and BRCD ranks against it. The injected fault lands at rank 1.

A learned CPDAG is not identical to the service-map CPDAG, and it need not be. BOSS is a
heuristic search, and the downstream ranking is robust to a Markov-equivalent graph. The
check is structural recovery plus fault recall, not an exact match to `expected.txt`.

### `verification_boss_sockshop`, `verification_boss_online_boutique`: supplied vs learned CPDAG

These run each case twice, once with the supplied CPDAG and once with `cpdag = None`, and
print both rankings side by side: the top five each, how deep they agree, the Spearman
correlation, and the fault rank under each.

The supplied CPDAG is the application's service map (the call graph), reversed: a metric of
service A points to a metric of service B whenever A calls B (paper, Appendix D). The map is
curated and fully directed; the paper calls this variant BRCD-C. BOSS instead learns a CPDAG
from statistical dependence in the metrics, and that graph carries undirected edges where the
data alone cannot orient them. The two are different objects. A call graph records who calls
whom; a learned CPDAG records what is conditionally dependent on what.

Sock Shop, both cases:

| case | supplied top-5 | learned top-5 | agree depth | Spearman | fault rank (supplied / learned) |
|---|---|---|---|---|---|
| carts_cpu_1 | `[42, 36, 26, 37, 17]` | `[42, 26, 36, 39, 37]` | 1 | 0.82 | 1 / 1 |
| carts_cpu_2 | `[37, 43, 22, 0, 24]` | `[39, 0, 43, 37, 22]` | 0 | 0.78 | 1 / 4 |

The two rankings agree at the top and diverge below. They agree where the mechanism shift is
large enough that any reasonable structure ranks the same variable: the fault and a few
strongly shifted metrics. They diverge below, because the order of moderate-importance
variables depends on the edges, and the learned edges are not the service map's. Spearman
stays near 0.8, so the rankings are broadly similar, but the leading agreement is shallow; on
`carts_cpu_2` the learned CPDAG pushes the fault from rank 1 to rank 4.

This gap is expected, not a BOSS defect. The paper offers BRCD-C (use the map) and the
bootstrap variants BRCD-B10/B100 (average over learned CPDAGs) for exactly this reason. To
judge BOSS itself, compare it to a known true graph on synthetic data, which
`verification_boss` does.

Only Sock Shop has a table because the learned run does not always finish. A fully directed
CPDAG needs no cut-configuration enumeration (`mec_size = 1`), so the supplied run is instant.
A learned CPDAG has undirected edges, and BRCD enumerates `2^(undirected edges incident on the
candidate)` configurations; the cost is exponential in the local undirected degree (paper,
Appendix E). Online Boutique's larger cases (about 50 variables, 2100 rows) learn a CPDAG with
a high-degree undirected hub, and the learned run does not complete in reasonable time. That
intractability is itself an argument for a directed service map. The Online Boutique comparison
example is kept but flagged, and the table above is from Sock Shop, which completes.

### `verification_online_boutique`, `verification_sockshop`: real-world acceptance

These replay the captured Python results on the supplied CPDAG. Each reproduces the reference
ranking position for position.

| dataset / case | positions | exact match |
|---|---|---|
| online-boutique/adservice_cpu_1 | 45/45 | ✓ |
| online-boutique/adservice_cpu_2 | 45/45 | ✓ |
| sock-shop-2/carts_cpu_1 | 44/44 | ✓ |
| sock-shop-2/carts_cpu_2 | 45/45 | ✓ |

## Data layout and provenance

One subdirectory per case:

```
data/<dataset>/<case>/{normal.csv, anomalous.csv, cpdag.txt, expected.txt, notes.txt}
```

The committed files are the processed reference. The reference Python pipeline drops time and
constant columns, converts memory to MB, windows to about 600 rows, and intersects columns;
this repository stores the result. That is why Online Boutique shows 45 variables and 600 rows
rather than the raw 51 columns at full length.

| file | contents |
|---|---|
| `normal.csv` | `df_obs`, the cleaned pre-failure data fed to `brcd_helper` |
| `anomalous.csv` | `df_a`, the cleaned failure data, same columns |
| `cpdag.txt` | line 1 `num_vars`, then one edge per line: `U i j` undirected, `D i j` directed, 0-based |
| `expected.txt` | the reference ranking, one 0-based index per line, best first |
| `notes.txt` | the raw Python console output, for provenance |

The Rust port reads `normal.csv` and `anomalous.csv` as they are; it does not reimplement the
Python preprocessing.

## Adding a dataset

1. Run the reference in `ctx/next/brcd/` (set up its conda env from that README):

   ```python
   from brcd.brcd import brcd_helper as brcd
   result = brcd(df_obs, df_a, cpdag=cpdag, isdiscrete=False,
                 node_transform="none", transform_parents=True,
                 num_root_causes_candidates=1)
   print(result["ranks"])
   df_obs.to_csv("normal.csv", index=False)
   df_a.to_csv("anomalous.csv", index=False)
   ```

2. Derive `expected.txt` from `result["ranks"]` as 0-based indices.
3. Commit `normal.csv`, `anomalous.csv`, `cpdag.txt`, `expected.txt`, and optional `notes.txt`
   under `data/<dataset>/<case>/`.
4. Match the example's `BrcdConfig` to the Python run.
5. Run the example and confirm the ranking matches.

## Notes on the reference

**Exactness.** The bar is the reproduced ranking, not bit-identical posteriors; numpy's PCG64
sampling and the Python numeric stack are not reproducible in Rust. `verification_base` checks
the recovery principle on Rust-generated data; the real-world examples check the ranking against
the captured reference on identical inputs.

**Ranking underflow, fixed in the capture.** The first Online Boutique capture exponentiated the
log-posterior before sorting (`np.exp(lp − max)`). When one fault dominates, that step underflows
every other candidate to zero and collapses the lower ranks to index order. The Rust port sorts on
the log-posterior, which the paper's `p(R | D)` (Eq. 3) implies and which does not underflow.
Re-captured with that fix, Python and Rust agree on the full ranking. See
`openspec/notes/brcd/brcd_python_ranking_bug.md`.

**BOSS score sign.** The port uses the higher-is-better BIC sign, the convention of causal-learn
and of the BOSS the paper runs ("default setting of BOSS from causal-learn", Appendix D). The
vendored Python BOSS is sign-inverted: it learns the empty graph on a clean chain and one
spurious edge on a collider, so its learned-CPDAG outputs can be wrong. A divergence from the
Python learned-CPDAG result is evidence against the reference, not the port; confirm it by
temporarily restoring the bug behind a test-only switch, never by shipping the wrong sign. See
`openspec/notes/brcd/brcd_boss_sign_bug.md`.
