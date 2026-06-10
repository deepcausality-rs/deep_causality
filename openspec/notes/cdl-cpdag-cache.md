# CDL CPDAG persistence (learn-once, rank-many)

Design note for a small, non-breaking change to `deep_causality_discovery`: when
the BRCD sub-pipeline learns a CPDAG (because none was supplied), persist it, so
later runs load the cached graph and skip the expensive structure-learning step.
The change is minor to implement (roughly a day) but it closes a real gap and
makes the CDL a dual-purpose instrument: offline structure discovery and online
root-cause ranking through the same pipeline.

## The gap (verified against the code)

BRCD's cost splits cleanly. Learning the CPDAG with BOSS is the expensive part and
depends only on the pre-failure observational data. Ranking candidates against a
known CPDAG is the cheap part and is the only work that must happen during a
failure. The original paper states this directly (Appendix E: "many of these
computations can be performed during normal operation. The only computational
expense during the failure period is the likelihood evaluation that involves F as
a parent"; Section 5.2: "other computational costs can be amortized during normal
operation").

The current CDL does not exploit the split. The learned CPDAG is computed and
thrown away:

- `brcd_run` returns only `BrcdResult`. In the `cpdag = None` branch it computes
  `let learned = boss_learn(...)?` and passes it to `run_with_cpdag(...)`; the
  `learned` graph is a local that drops at end of scope. The caller cannot recover
  it. (`deep_causality_algorithms/src/causal_discovery/brcd/brcd_algo.rs`)
- The CDL discover stage stores only `brcd_result`. When no CPDAG path was
  supplied, BOSS runs inside `brcd_run` and the CDL never sees the graph.
  (`deep_causality_discovery/src/types/cdl/brcd_loaded.rs:24`)
- A saver already exists and has zero call sites in the pipeline:
  `save_cpdag_csv` (`src/types/data_loader/cpdag_csv.rs:32`), exported from the
  crate root. The matching `load_cpdag_csv` is already wired for the
  supplied-CPDAG path (`src/types/data_loader/brcd.rs:49`).

So every run with no supplied graph re-learns the structure from scratch, even
when the observational window has not changed. The loop is most of the way built
and simply not closed.

## The change

Wire a learn-then-persist-then-reuse path into the BRCD sub-pipeline. No change to
`brcd_run`; the orchestration belongs in the CDL, and the pieces it needs are
already public (`boss_learn`, `save_cpdag_csv`, `load_cpdag_csv`).

1. Extend the BRCD loader config with an optional CPDAG cache path (distinct from
   the existing supplied-CPDAG path, or unify the two under one resolved source).
2. In the discover stage, resolve the CPDAG source in order:
   - supplied path present and the file exists, load it;
   - cache path present and the file exists and its key matches, load it;
   - otherwise call `boss_learn` on the normal data, then `save_cpdag_csv` to the
     cache path, then rank with the learned graph.
3. Rank with `brcd_run(Some(&graph), ...)` in every case, so structure learning
   runs at most once per observational window.

### Cache validity is the only real design decision

The file write is trivial; correctness is not. A saved CPDAG is valid only for the
same normal dataset and the same BOSS configuration. If either changes, the cache
is stale and would silently produce a wrong graph. Key the cache to invalidate
correctly:

- compute a key from a hash of the normal dataset plus the BOSS settings, and
  store it alongside the CPDAG (a header line or a sidecar file);
- on load, recompute the key and reject the cache on mismatch, falling back to
  re-learning;
- alternatively, leave the path user-managed and document that the operator
  invalidates it when the observational window is refreshed.

This matches the operational model: the structure is re-learned periodically as
the system evolves during normal operation, not once per incident.

### Optional: make the source explicit in the type-state

The CDL's type-state currently encodes stage ordering, not the structure source.
A small addition would make the offline/online boundary explicit in the API: a
`CpdagSource` distinction (`Supplied`, `Cached`, `Learned`) carried through the
loaded state. This is optional and can follow the functional change; the
persistence path above is the load-bearing part.

## Why it matters beyond convenience

Persisting the CPDAG is ordinary caching on its own. Its value here is that it
operationalizes the offline/online split the paper only describes, through one
pipeline:

- **Offline (normal operation):** run the pipeline on observational data with no
  CPDAG; it learns the structure with BOSS and persists it.
- **Online (failure period):** run the same pipeline with the cached CPDAG
  supplied; it skips structure learning and pays only the ranking cost, which is
  the work that governs time-to-diagnosis.

The same CDL serves both roles. That gives a clean cold-versus-warm latency
measurement (learn + rank versus load + rank on identical data), which
demonstrates the amortization the paper asserts but never realizes in code.

## Scope

- In: optional cache path, source resolution, `save_cpdag_csv` wired on the
  learned graph, cache keying/invalidation, tests for the three resolution paths.
- Out: changes to `brcd_run`'s signature; changes to the SURD sub-pipeline; the
  `CpdagSource` type-state (optional follow-up).
