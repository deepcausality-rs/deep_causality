# BRCD verification examples

Each verification is a standalone Rust example, runnable individually:

```bash
cargo run -p deep_causality_algorithms --example verification_base
cargo run -p deep_causality_algorithms --example verification_online_boutique
```

Each example prints a `PASS`/`FAIL` line per check and exits non-zero if any check
fails, so it runs directly in CI.

## `verification_base` — synthetic recovery (self-contained)

Generates a linear-Gaussian chain `X → Y → Z`, then an anomalous dataset that
perturbs `p(Y | X)`. Because only Y's conditional mechanism changes, BRCD must
rank **Y** as the top root cause. No Python or external data is required; this is
the base smoke verification that gates the real-world examples.

## `verification_*` — real-world replication (acceptance bar)

These replay the real-world experiments (Online Boutique, Sock Shop, …) and assert
the Rust ranking reproduces the authoritative Python BRCD. The data layout is one
subdirectory per case:

```
data/<dataset>/<case>/{normal.csv, anomalous.csv, cpdag.txt, expected.txt, notes.txt}
```

### Data provenance — what these files are

The committed files are the **processed reference**, not the raw download:

```
raw download                         data/<dataset>/<case>/         Rust example
data/online-boutique/                  (committed reference)
  adservice_cpu/1/data.csv   ──►       normal.csv      ◄─┐
  (51 cols, full length,               anomalous.csv     ├─ INPUTS the Rust BRCD runs on
   raw units)                          cpdag.txt       ◄─┘
        │                              expected.txt    ◄── OUTPUT the Rust must reproduce
  [main-*.py pipeline +                notes.txt           (provenance: names + config)
   preprocess + brcd_helper()]
```

* `normal.csv` / `anomalous.csv` — the **exact** `df_obs` / `df_a` fed into
  `brcd_helper`, already cleaned on the Python side (time + constant columns
  dropped, mem → MB, windowed to ~600 rows, columns intersected). That is why OB
  shows 45 vars / 600 rows, not the raw `data.csv`'s 51 cols × full length.
* `cpdag.txt` — the exact CPDAG passed in (from the reversed service map).
* `expected.txt` — the ranking Python BRCD produced; the target the Rust port
  must match. `notes.txt` holds the same ranking as metric names plus the config.

**Implication:** preprocessing (windowing, constant-column dropping, unit
conversion) happens on the Python side and is baked into `normal.csv` /
`anomalous.csv`, so the Rust port does **not** reimplement it to pass verification
— it starts from the already-clean matrices. Having the Rust side own preprocessing
too is a separate, larger decision.

### Success criterion

The check is **the Rust ranking reproduces Python's `expected.txt` exactly**,
position-for-position, for every case.

> **Reference note.** The first OB capture was produced by a BRCD build whose
> ranking step exponentiated the log-posterior (`np.exp(lp − max)`) before
> `argsort`; on a dominant fault that underflows every non-top candidate to `0.0`,
> so the secondary ranks collapsed to index order. The Rust port ranks on the
> log-posterior directly (the paper's `p(R|D)`, Eq. 3), which is underflow-free.
> Once the reference was re-captured **with that ranking fix**, Python and Rust
> agree on the **entire** ranking. See `openspec/notes/brcd_python_ranking_bug.md`.

#### Verified results (vs the ranking-fixed reference)

| dataset / case | positions | exact match |
|---|---|---|
| online-boutique/adservice_cpu_1 | 45/45 | ✓ |
| online-boutique/adservice_cpu_2 | 45/45 | ✓ |
| sock-shop-2/carts_cpu_1 | 44/44 | ✓ |
| sock-shop-2/carts_cpu_2 | 45/45 | ✓ |

### Workflow (per dataset)

1. **Run the Python reference.** In `ctx/next/brcd/`, set up the conda env from its
   README and run BRCD on the dataset, capturing both the inputs and the output:

   ```python
   from brcd.brcd import brcd_helper as brcd
   result = brcd(df_obs, df_a, cpdag=cpdag, isdiscrete=False,
                 node_transform="none", transform_parents=True,
                 num_root_causes_candidates=1)
   print(result["ranks"])           # the expected ranking
   df_obs.to_csv("normal.csv", index=False)
   df_a.to_csv("anomalous.csv", index=False)
   ```

2. **Extract the output to a text file.** Save the printed `ranks` (and any console
   output) so the expected result is auditable, then **derive the expected ranking**
   as 0-based variable indices.

3. **Commit the artifacts** under `examples/verification/brcd/data/<dataset>/<case>/`:

   | File | Contents |
   |---|---|
   | `normal.csv` | `df_obs` — numeric columns, optional header row |
   | `anomalous.csv` | `df_a` — same columns as `normal.csv` |
   | `cpdag.txt` | line 1: `num_vars`; then one edge per line, `U i j` (undirected `i — j`) or `D i j` (directed `i → j`), 0-based |
   | `expected.txt` | expected ranking, one 0-based variable index per line, best first |
   | `notes.txt` *(optional)* | the raw Python console output for provenance |

4. **Replicate the config.** Edit the `BrcdConfig` in the example to match the
   Python run (`num_root_causes`, `node_transform`, `transform_parents`, seed).

5. **Run the Rust example** and confirm it matches the expected ranking:

   ```bash
   cargo run -p deep_causality_algorithms --example verification_online_boutique
   ```

## Note on exactness

The success criterion is the **reproduced ranking**, not bit-exact
posteriors (numpy PCG64 sampling and the Python numeric stack are not reproducible
in Rust). The `verification_base` example verifies the recovery *principle* on
Rust-generated data; the `verification_*` real-world examples verify the *ranking*
against the committed Python reference on identical input data.
