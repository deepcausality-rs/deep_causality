## Context

The foundations for BRCD are landed and archived: `RealField` genericity (`real-field-discovery`), the typed-endpoint `MixedGraph` CPDAG (`mixed-graph`), the Tier A numeric primitives and the Tier B causal-graph operations (`brcd-prep-foundations`, rooted at `deep_causality_algorithms::causal_discovery::brcd::{meek, validity, mec}`). This change implements the BRCD estimator over them, ported faithfully from the authoritative `ctx/next/brcd/` (the author's canonical version; experiment-repo copies are non-authoritative). The reconciliation of record is `openspec/notes/rca/BRCD.md` §16; line references below are into `ctx/next/brcd/brcd.py` and `ctx/next/brcd/BRCD/mcs_num.py`.

BRCD is built and verified as a **standalone algorithm** — two `CausalTensor`s (normal, anomalous) + a `MixedGraph` (CPDAG) → a ranked posterior over root-cause candidates — *before* it is wired into the CDL typestate pipeline. That wiring is the separate `cdl-discovery-pipeline` change. Verifying the algorithm against the reference oracle first means the public-API seam is later generalized around a known-correct estimator, not a reconstructed one.

Constraints carried from the repo: no external numeric crates, `unsafe_code = "forbid"` workspace-wide, static dispatch (no `dyn`), one-type-one-module layout, full test coverage of new code, deterministic given a seed (seeded RNG from `deep_causality_rand`; no ambient randomness).

## Goals / Non-Goals

**Goals:**
- A faithful, `RealField`-generic BRCD estimator composing the landed foundations: F-node augmentation, cut-configuration enumeration with Meek/validity, the plug-in ridge-Gaussian / mixture-of-experts / Dirichlet families, the logistic gate, exact MEC sizing + uniform sampling, and posterior ranking — all under `causal_discovery::brcd`.
- The two new components the prep deferred: a uniform MEC engine (exact AMO enumeration) and a logistic-regression gate.
- A tiered verification suite that reproduces the authoritative algorithm's behaviour: golden fixtures (reference posteriors), synthetic ground-truth recovery, and an oracle cross-check on fixed seeds.

**Non-Goals:**
- CDL pipeline integration (`DiscoveryOutcome<T>`, two-dataset carriage, formatter). Separate change `cdl-discovery-pipeline`.
- The no-CPDAG **BOSS** structure-learning bootstrap path. BRCD here requires a supplied CPDAG; discovery of the CPDAG is upstream.
- The Forest-KDE nonparametric family and the microservice call-graph adapter. The Gaussian/Dirichlet path is the real-world target; the KDE family is the reference's *option*, not its canonical path.

## Decisions

**D1. Standalone algorithm, verified before CDL wiring.**
BRCD is a `pub fn` in `deep_causality_algorithms::causal_discovery::brcd` taking `(&CausalTensor<T>, &CausalTensor<T>, &MixedGraph<…>, BrcdConfig)` and returning a `BrcdResult<T>` (ranked posterior over candidates). It does not depend on `deep_causality_discovery`. The CDL `DiscoveryOutcome::Brcd(BrcdResult<T>)` wiring is the later change. *Why:* the estimator's correctness is the project risk; isolating it lets the verification suite be the gate, and lets `cdl-discovery-pipeline` be designed against the real `BrcdResult<T>`.

**D2. MEC engine = exact AMO enumeration (port `mcs_num.py`), not the external clique-picking package.**
`brcd.py` calls the external `cliquepicking` (`cp.mec_size`, `cp.MecSampler`, Wienöbst 2023), but the authoritative tree also ships `BRCD/mcs_num.py` — a pure enumerator (`enumerate_amos`, `enumerate_dags`) that is the normative in-tree alternative. Port *that*: it has no external dependency, gives the **exact** MEC size (`= |enumerate_dags|`) and a uniform sample (pick one enumerated DAG with the seeded RNG), and reproduces the reference weights `log(mec_size / Σ)` identically. It replaces the trivial arcs-only placeholder behind the existing `mec` API (`mec_size`/`representative_dag` extend to a `mec_sample_dag`), so call sites do not change shape. *Trade-off:* enumeration is exponential in the worst case; the reference's clique-picking samples without enumerating. For the experiment scale (root-incident undirected subgraphs) enumeration is tractable; the implementation **bounds the enumeration and `log`s/erros explicitly if a configuration exceeds the bound** — no silent truncation. A future change can swap in a true clique-picking sampler behind the same API if scale demands it. *Alternative rejected:* port the Wienöbst clique-picking algorithm now — substantially more complex, and unnecessary for faithful reproduction at the target scale.

**D3. Logistic-regression gate as an in-repo primitive (the one new numeric component).**
The mixture gate `π(F=1 | X)` (brcd.py L534/543, `sklearn.LogisticRegression`, L2, lbfgs) is implemented in-repo as ridge-penalized logistic regression solved by **Newton/IRLS** over the SPD primitives already available, generic over `T: RealField`, deterministic, no external crate. It lives under `causal_discovery::brcd` (a `gate` module) unless design review surfaces a clearly reusable home. *Why IRLS:* it reuses the SPD solve, converges in a few iterations on these low-dimensional gates, and matches lbfgs's optimum (same convex objective) closely enough for ranking reproduction. Empirical-prior fallback (degenerate/singular gate) mirrors the reference.

**D4. Ridge fit via a direct SPD (Cholesky) solve; estimator composes Tier A.**
`_fit_ridge` (brcd.py L312) is `β = solve(XᵀX + λI, Xᵀy)`, `σ² = resid·resid / max(n−p, 1)` floored to 1e-12, λ = 1e-4. Implement with the in-place Cholesky already proven in `deep_causality_tensor` (`conditional_variance`'s private solver) on the small `p×p` normal-equations system; per-row density via Tier A `gaussian_log_density` (exact `_normal_logpdf_1d`); mixtures combine via Tier A `logsumexp`. *Alternative considered:* `cg_solve`. Rejected as the default: the normal-equations system is tiny and dense, so a direct Cholesky is simpler and exact; `cg_solve` stays available if an ill-conditioned case warrants it.

**D5. Family log-likelihood caching keyed on `(node, sorted parents)`.**
`brcd_update` (L1756) caches each unique family's per-row log-likelihood once and reuses it across every DAG that contains that family. Replicate this: a `BTreeMap<(usize, Vec<usize>), Vec<T>>` cache keyed on the node and its sorted parent indices. This is both the reference's behaviour and the dominant performance lever.

**D6. Supplied-CPDAG only; BOSS bootstrap out of scope.**
`brcd_helper` (L1863) branches: CPDAG supplied → `brcd_update` directly; none → bootstrap CPDAGs via BOSS + `get_top_k_cpdags_with_ratio` + `parallel_weighted_posterior`. This change implements the **supplied-CPDAG** branch only. The bootstrap/BOSS branch is a separate concern (CPDAG discovery is upstream; CDL produces the graph). The API accepts an optional list of weighted CPDAGs so the bootstrap path can be added later without a signature change.

**D7. Transform ladder implemented with `yeojohnson` deferrable behind the same API.**
The continuous estimator's optional transform (none/log/log1p/yeojohnson, brcd.py L279/L708/L752) with Jacobian + auto-downgrade affects which density a family reports, so it is part of faithful reproduction. Implement none/log/log1p fully; `yeojohnson` (with its λ search) may ship in a follow-up behind the same `Transform` enum + auto-downgrade ladder if it proves heavy — the verification fixtures pin which transform each family selects so a deferral is visible, not silent.

**D8. `RealField`-generic numerics; structural code precision-free; deterministic.**
Every numeric component (ridge fit, gate, densities, mixtures, Dirichlet) is generic over `T: RealField`. The enumeration/validity/graph layer carries no scalars. All randomness (uniform DAG sampling) is from a seeded `deep_causality_rand` RNG threaded through `BrcdConfig`, so runs are reproducible and the oracle cross-check is deterministic.

## Verification strategy (the `brcd-verification` capability)

Three tiers, strongest guarantee first where feasible:

1. **Golden unit fixtures.** Small hand-built `(normal, anomalous, CPDAG)` cases with the reference posterior over roots captured from `ctx/next/brcd/brcd.py` on a fixed seed, committed as golden data. Assert the Rust ranking is **identical** and each log-posterior is within a tolerance `ε`. Covers each estimator mode (F∈parents, F∉parents mixture, F absent; continuous and discrete) and the MEC weighting on a CPDAG with undirected edges.
2. **Synthetic ground-truth recovery.** Mirror `experiments/synthetic/data_generation.py`: generate data with a *known injected* root cause under a known graph (seeded), run Rust BRCD, assert the true root is ranked top-1 (and within top-k for the multi-root case) — the paper's synthetic success metric, self-contained in-repo.
3. **Authoritative oracle cross-check.** On a handful of fixed synthetic datasets + CPDAGs, the committed Python-BRCD posteriors (rankings + log-posteriors) are the golden reference; the Rust output matches rankings exactly and log-posteriors within `ε`. Real-world OB / Sock Shop datasets are downloaded by the reference harness and not bundled, so this tier uses committed Python outputs on fixed seeds, not live re-download.

The tolerance `ε` and seed set are pinned in the verification spec; ranking equality is exact (the success criterion is reproduced rankings/top-k, not bit-exact floats — per the owner's full-capability-port mandate).

## Risks / Trade-offs

- **AMO enumeration blow-up (D2).** → Bounded with an explicit error/log on exceeding the cap; tractable at experiment scale; swappable for true clique-picking later behind the same API.
- **IRLS gate vs lbfgs optimum (D3).** → Same convex objective; verification fixtures assert ranking reproduction, which is robust to small gate-coefficient differences; empirical-prior fallback matches the reference on degenerate gates.
- **Oracle data is captured, not live (D7/verification).** → Committed golden files on fixed seeds; the synthetic generator is in-repo and re-runnable, so tier 2 is independent of any captured artifact.
- **Scope temptation toward BOSS / Forest-KDE / CDL wiring.** → Hard-bounded by D1/D6 and Non-Goals; the APIs (optional weighted-CPDAG list, `Transform` enum, `mec_sample_dag`) absorb the deferred pieces without churn.

## Open Questions

- Home of the logistic gate: `causal_discovery::brcd::gate` (local) vs a reusable `deep_causality_algorithms` regression module. Settled in design review / tasks; default local (D3) until a second consumer exists.
- Exact `ε` and the seed/fixture set for the oracle cross-check — pinned in the verification spec during implementation, once the Python oracle outputs are captured.
- Whether `yeojohnson` ships in this change or a fast-follow (D7) — gated on its weight relative to the rest; the API is shaped to absorb either way.
