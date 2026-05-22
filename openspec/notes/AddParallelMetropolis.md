# add-parallel-metropolis — forward-looking design note

**Status:** Forward-looking design note. Sketches a future change set that layers multi-core parallelism on top of the R6 single-edge Metropolis primitive shipped by `add-cubical-regge-calculus-analytical`. Not yet a proposal — this document scopes the parallelism architecture so the eventual change set can be authored against a clear set of decisions.

**Prerequisites:** all already shipped.
- R6 `metropolis_update` + `regge_gradient_at_edge` (single-thread, mutate-in-place).
- `LatticeComplex::edge_id_to_position_axis` (used to identify edge-conflict sets).
- `deep_causality_rand::Rng` + `Normal` + `StandardUniform` for per-thread RNGs.

**Target hardware:** 16-core workstation (current dev environment). The design must scale cleanly from 1 to ~16 cores; super-linear claims would be dishonest, sub-linear is acceptable, but the goal is *useful* parallelism — within 2–3× of perfect linear on the saturating path.

---

## 1. The three orthogonal parallelism axes

A serial Markov chain `(x_0, x_1, x_2, …)` is sequential by construction: `x_{n+1}` depends on `x_n`. But real-world MCMC sampling has three independent parallel dimensions that do not fight each other:

| Axis | What it parallelises | Detailed balance | Speedup ceiling | Code cost |
|---|---|---|---|---|
| **A. Independent chains** | Multiple chains, each with its own RNG, on different cores. | Trivially preserved (each chain is its own valid MC). | Linear in `K` cores, capped by memory bandwidth. | Trivial — wrap existing `metropolis_update` in `rayon::scope`. |
| **B. Within-chain checkerboard** | Multiple edges per "step", batched by hinge-conflict colouring. | Preserved per colour batch (within-colour proposals are mutually independent). | Linear in `num_cores`, *per chain*. | Moderate — need conflict-graph colouring helper + batch commit. |
| **C. Replica exchange / parallel tempering** | N replicas at different β values; periodic Metropolis swap on action difference. | Preserved by symmetric swap proposal. | N replicas in parallel × per-replica axes A/B. | ~200–300 LOC for the swap protocol. |

The three axes compose multiplicatively. On a 16-core box: e.g. 4 independent chains × 4-colour checkerboard within each chain = 16-way saturation.

---

## 2. Why each axis matters

**Axis A — Independent chains** is the cheapest win and the *correct* way to estimate observables with statistical error bars. MCMC theory tells you that `K` independent chains give `√K` reduction in observable variance after equilibration. With 16 cores you can run 16 chains, each requiring its own equilibration; the *sampling* phase parallelises trivially. Catch: equilibration cost is paid per chain — 16 chains × 10⁴ burn-in = 1.6×10⁵ burn-in steps in wall-clock time, not the same as a 10⁴-step single chain.

**Axis B — Within-chain checkerboard** speeds up *equilibration* of a single chain, which axis A can't help with. On axis-aligned cubical lattices the hinge-conflict structure is benign:

- **D=3**: hinges *are* edges (1-cells), so two edges share a hinge iff they're the same edge. *Every edge is independent of every other edge.* Full parallel batch update per step, no colouring needed. Effective speedup ≈ `num_cores`, capped only by `num_edges / num_cores`.
- **D=4**: hinges are squares (2-cells). Two edges conflict iff some square contains both. Each edge conflicts with ~24 others (6 squares × 4 edges per square, minus self and shared edges). Standard greedy graph colouring gives a 3- or 4-colouring; update one colour set in parallel per "batch step". Effective speedup ≈ `num_cores / num_colours`.
- **D=2**: gradient is zero everywhere (vertex hinges, vol=1), so the per-edge action is constant. Metropolis on edge lengths has uniform equilibrium; no acceleration needed.

**Axis C — Replica exchange** is the production-grade pattern for crossing free-energy barriers. Required for near-critical-point studies (where naive Metropolis gets trapped) and multicanonical sampling. Not needed for smoke tests; useful for any phase-diagram scan.

---

## 3. Proposed implementation order

Three change sets in sequence, each independently shippable and reviewable. Effort budgets are deep estimates for one developer on this codebase.

### Phase P1 — Independent-chain parallelism (Axis A)

Smallest possible win, largest immediate benefit for the test suite and any near-term research use.

**Crate affected:** `deep_causality_topology` only.

**New API (additive):**

```rust
/// Run K independent Metropolis chains in parallel across `num_threads`
/// cores. Each chain owns a cloned starting geometry and a distinct RNG.
/// Returns the K final geometries plus accept/reject statistics.
pub fn metropolis_parallel_chains<G, F>(
    initial: &CubicalReggeGeometry<D, R, Euclidean>,
    complex: &LatticeComplex<D, R>,
    config: ChainConfig<R>,
    rng_factory: F,
) -> Vec<ChainResult<R>>
where
    G: Rng + Send,
    F: Fn(usize) -> G + Sync,
    /* + bounds for R: Send + Sync */
```

**Dependency add:** `rayon` (workspace-wide already? if not, this change set lands it under `deep_causality_topology/Cargo.toml`).

**Implementation:** `rayon::scope` with `K` chains, each calling the existing `metropolis_update` in a tight loop. The per-chain Hodge ⋆ / gradient computations are *already* allocation-free in the hot path (per R6.6.1 single-edge gradient optimisation), so there's no shared-state contention.

**Property tests:**

- Acceptance rate of pooled chains within ±1σ of single-chain rate (sanity).
- Final-configuration mean across chains stable across reruns (per-thread RNG determinism).
- Per-chain final lengths all strictly positive, finite.

**Effort:** ~120 LOC + ~5 tests + `rayon` dep. **~3 hours.**

**Expected speedup on 16-core:** ~14× (some overhead from rayon scheduling + per-chain `geometry.clone()`).

---

### Phase P2 — Within-chain checkerboard (Axis B)

Multiplies P1's speedup *per chain*. Most useful for single-chain equilibration; combined with P1, gives K × M-way parallelism.

**Crate affected:** `deep_causality_topology` only.

**New API (additive):**

```rust
/// Conflict-set partitioning of edges by hinge incidence. Two edges in the
/// same set are guaranteed to share no (D−2)-hinge, so updating them
/// simultaneously preserves detailed balance.
///
/// Result is a vector of colour classes; iteration over classes is sequential,
/// within-class updates are parallel-safe.
pub fn edge_conflict_colouring<const D, R>(
    complex: &LatticeComplex<D, R>,
) -> Vec<Vec<usize>>; // [colour_id][edge_ids]

/// Parallel batch Metropolis: pick one colour class per "batch step", update
/// every edge in the class concurrently via `rayon::par_iter`.
pub fn metropolis_batch_update<G: Rng + Send>(
    &mut self,
    complex: &LatticeComplex<D, R>,
    colouring: &[Vec<usize>],
    rng_pool: &mut [G],
    sigma: R,
    beta: R,
) -> BatchOutcome<R>;
```

**Key design point — D=3 is the easy case.** Every edge is its own colour class, so `edge_conflict_colouring(d3_lattice)` returns `vec![vec![0], vec![1], ...]` — one edge per class. The natural parallelism is: ignore the colouring abstraction, just `par_iter` over all edges directly. The colouring machinery only matters for D≥4.

**D=4 colouring:** a greedy first-fit graph colouring on the edge-edge conflict graph. Vertices = edges; edge in conflict graph iff some (D-2)-cell contains both. Greedy gives ≤ Δ+1 colours where Δ is the max degree (~24 for D=4 on regular cubical). Practical colour count: 3 or 4 on regular lattices.

**Property tests:**

- `edge_conflict_colouring` produces a valid colouring: every pair within the same colour shares no hinge.
- Single-thread `metropolis_batch_update` produces the same accept/reject sequence as a serial loop over the same edges (commutativity within a colour class).
- Detailed balance: 50K-step batch run vs serial run, same final-distribution moments to within statistical tolerance.

**Effort:** ~250 LOC + ~10 tests. **~6 hours.**

**Expected speedup on 16-core:** for D=3, near-linear in cores (small lattice = limited concurrency, large lattice = full saturation). For D=4 with 4-colouring, ~4× per chain × 14× from P1 = ~50× combined.

---

### Phase P3 — Replica exchange / parallel tempering (Axis C)

The production-grade barrier-crossing tool. Optional but high-value for phase-diagram studies.

**Crate affected:** `deep_causality_topology` only, but the public API broadens to handle replica-array configuration.

**New API (additive):**

```rust
pub struct ReplicaArray<const D, R> {
    replicas: Vec<CubicalReggeGeometry<D, R, Euclidean>>,
    betas: Vec<R>,
}

impl ReplicaArray<D, R> {
    /// Run one parallel-tempering step: each replica does `inner_steps`
    /// Metropolis updates in parallel (via P1+P2), then adjacent-β replica
    /// pairs propose a swap with Metropolis criterion on the action
    /// difference.
    pub fn step<G: Rng + Send>(
        &mut self,
        complex: &LatticeComplex<D, R>,
        rngs: &mut [G],
        inner_steps: usize,
        sigma: R,
    ) -> TemperingOutcome<R>;
}
```

**Property tests:**

- Swap criterion produces valid Metropolis acceptance: `min(1, exp((β_i − β_j)(S_i − S_j)))`.
- Detailed balance under swap: swap-only run with no inner updates produces a stationary distribution that respects the joint product measure.
- Observable estimates: replica at β converges to the same observable mean as a pure-β single chain (within statistical tolerance).

**Effort:** ~300 LOC + ~12 tests. **~10 hours.**

**Expected speedup on 16-core:** N replicas × per-replica P1+P2 speedup. For 4 replicas + 4-way per-replica parallelism = ~16-way saturation, *with the additional scientific benefit of barrier crossing*.

---

## 4. Hard architectural constraints inherited from R6

These are non-negotiable for any parallel-Metropolis change set:

1. **Per-thread RNG.** Sharing a single `Rng` across threads kills reproducibility and serialises on the RNG state. Each thread owns its `Rng`; the parallel API takes an `Rng` *pool*.
2. **Per-thread geometry clone for axis A.** Independent chains need independent state. The `CubicalReggeGeometry<D, R, S>: Clone` (already shipped) makes this free.
3. **No locks on the hot path.** Axis B requires that within-colour edge updates touch disjoint state. The `PerEdge` length buffer is the *only* mutable per-step state; per-edge writes within a colour class are guaranteed-disjoint by the colouring.
4. **`Send + Sync` bounds at the API boundary.** Required for `rayon::par_iter`. `R: RealField + Send + Sync` and `K: ChainComplex + Send + Sync` are the new bounds; both are satisfied by all current concrete types (`f32` / `f64` / `DoubleFloat` are `Send + Sync`; `LatticeComplex<D, R>` is `Send + Sync` provided `R` is).
5. **Single-edge gradient is the right primitive.** Axis B's `par_iter` over edges calls `regge_gradient_at_edge(e)` per edge. That's O(D · 2^D) per call, no allocation, no shared cache. The optimisation we landed in R6.6.1 is precisely what makes axis B viable.

---

## 5. Honest caveats

- **P1 speedup is bounded by equilibration cost.** If your science needs `N_eq = 10⁵` burn-in steps, `K = 16` independent chains needs `16 × 10⁵` total burn-in steps — same wall-clock as `16 × 10⁵` serial steps, no gain on equilibration. P1 gains only on the *sampling* phase after burn-in.
- **P2 small-lattice degradation.** On a 3D 3×3×3 lattice (54 edges), 16 cores means each thread updates ~3 edges per batch. Thread-creation / scheduling overhead may dominate the actual work. Useful threshold: roughly `num_edges ≥ 4 × num_cores`. For a 16-core box that's `num_edges ≥ 64` — the current 3D 3×3×3 test lattice is right at the edge of usefulness. Production-scale lattices (16³ = 12K edges) are firmly in the saturating regime.
- **P3 swap-rate tuning is its own science.** Replica spacing in β-space governs swap acceptance; too-far-apart → no swaps → no barrier crossing benefit; too-close → many swaps but no temperature reach. This requires per-system tuning that lives outside the library code itself.
- **`rayon` introduces a workspace dependency** that the AGENTS.md rule "avoid the introduction of external crates unless it is necessary for testing" must explicitly accept. `rayon` is mature, well-maintained, and effectively a standard component of any Rust data-parallel pipeline; the rule's spirit (no random one-off crates) is honoured by reaching for the obvious community choice. Document this as a deliberate exception in the P1 change set's proposal.

---

## 6. Validation strategy on the 16-core target hardware

For the eventual `add-parallel-metropolis` change set, the validation gate should include:

- **Strong scaling on a fixed-size lattice.** Run 16³ in 3D with `K ∈ {1, 2, 4, 8, 16}` chains; plot wall-clock vs `K`. Expect ~14× at K=16 with rayon scheduling overhead.
- **Weak scaling on a fixed-per-core problem.** Hold per-core lattice size constant, scale total lattice size with `K`. Expect ~constant wall-clock.
- **Statistical validity.** Pooled-chain observable estimates must match analytic expectations (where available) and serial-chain estimates (otherwise) within `√N` standard error.
- **Production smoke.** A single 4-replica × 4-checkerboard run (16-way saturation) at 10⁸ Metropolis steps on a 3D 16³ lattice. Should complete in under an hour wall-clock if the design is sound — exactly the size of run that the 16-core box can comfortably handle and that's stretching enough to expose real bottlenecks.

10⁸ steps with 16-way saturation is effectively 6.25×10⁶ serial-step-equivalents per core, or ~6 minutes per core in release mode given the R6 single-edge gradient cost of ~16 μs / step. So total wall-clock ≈ 6 minutes for a 10⁸-step run — within the budget for a CI smoke test.

---

## 7. Bottom line

Three orthogonal parallelism axes, three change sets, all additive on top of R6. Each ships independently with linear speedup on its own axis; composed, they reach 16-way saturation on the target workstation with room to spare. The single-edge gradient optimisation from R6.6.1 is what makes axis B viable — without it, `par_iter` would serialise on the full-gradient allocation/compute and the speedup would collapse.

If only one of the three were shipped, P1 (independent chains, ~3h effort) gives the largest delivered-value-per-hour. If two, P1 + P2 gives the production-grade per-chain saturation. P3 is the research-grade upgrade for barrier-crossing studies.

**Naming:** the eventual change set is `add-parallel-metropolis` (single change set covering all three phases, gated R-style as P1 → P2 → P3) or split into `add-parallel-metropolis-chains` / `add-parallel-metropolis-checkerboard` / `add-parallel-tempering` (three separate change sets). Recommendation: single change set with three gated phases, mirroring the `add-cubical-regge-calculus-analytical` R4/R5/R6 pattern that worked well in practice.
