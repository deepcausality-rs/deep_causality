# BRCD Port — Findings, Scope Decision & Author-Call Questions

**Issue:** [#598 "Investigate BRCD for addition to CDL"](https://github.com/deepcausality-rs/deep_causality/issues/598)
**Status:** Pre-design investigation. No code written. Scope decided (§1). Provenance + per-system CPDAG source pending author call (§9).
**Mandate (from owner):** Full capability port of the chosen target — no MVP, no algorithmic reduction *of the target*. "Reproduce the computational results"; bit-exact floats are negotiable, *decision/semantic equivalence* (same root-cause rankings) is the bar.

---

## 0. Source material (poster abstract — most authoritative artifact we have)

**Title:** *Root Cause Analysis of Failures in Microservices via Bayesian Root Cause Discovery*
**Authors:** Kenneth Lee, Zihan Zhou, Murat Kocaoglu (Purdue — **no Adobe** on BRCD; the Adobe collaboration was the separate 2022 *RCD* paper).
**Availability:** No public paper found as of 2026-05-31 — consistent with under-review / pre-camera-ready for a 2026 venue. The manuscript is the canonical spec; **request it (even under-review/NDA).**

Abstract claims that anchor the design:
- Bayesian inference *"without enumerating all DAGs from each I-MEC"* via *"uniform DAG sampling (Wienöbst et al., 2023)"* (= `cliquepicking`). Sampling is deliberate, for scalability, with ε-vanishing posterior bounds.
- *"first consistency guarantees for nonparametric RCA"* — the nonparametric (Forest-KDE) claim is the **synthetic** theory headline (see §3); the real-world implementation we target uses linear-Gaussian.
- Evaluated on **public** systems: Online Boutique, Sock Shop, Petshop.

---

## 1. SCOPE DECISION — port the real-world model, skip synthetic

**Target = the real-world BRCD implementation** (`experiments/real-world/RCAEval/e2e/brcd.py` → `brcd_helper` → linear-Gaussian path). **Skip the synthetic benchmark and its Forest-KDE / oracle-BN path.**

Rationale (owner): the synthetic benchmark is a workbench for testing ideas; only those that pan out get refined and run on the real systems. The real-world code is also (a) **internally coherent** (no stale-harness bug — §3), (b) a **simpler estimator** (Gaussian, not KDE), and (c) more approachable. Replicating the real-world findings is the goal.

Skipping KDE/discrete/multi-root/etc. is **not a reduction** — they are not part of the real-world configuration (§5). **Open sub-decision:** the real-world suite is *three* systems, and Petshop pulls BOSS back in (§4a). Decide whether the target is **OB + Sock Shop only** (supplied call graph, no BOSS) or **all three incl. Petshop** (BOSS required).

---

## 2. What BRCD is

Shared framing (RCD lineage): a failure = a soft intervention; add a binary **F-node** (0 = normal, 1 = anomalous); a root cause = a node whose conditional law changes between regimes = a neighbor of the F-node. BRCD does Bayesian model averaging over augmented DAGs (CPDAG + `FNODE→root`) and ranks candidates by posterior. The provided `RCD_NeurIPS22.pdf` is the *different* constraint-based RCD method — background only.

---

## 3. Reference-repo reality: a multi-paper benchmark dump

`ctx/next/brcd/` is a multi-year aggregation across RCD, RCG, BARO, AERCA, CausalRCA, RUN, and BRCD — not a clean method release. Verified consequences:

- **Synthetic harness is STALE (a real refactoring bug).** `experiment_synthetic.py:161` calls `brcd.brcd(df_obs, df_int, cpdag, obs_bn, int_bn, obs_ground_truth, int_ground_truth=False, version=...)`, but the imported `brcd()` (`code/models/brcd.py`) is `brcd(normal_df, anomalous_df, cpdag, isdiscrete, node_transform, transform_parents, version)` — `obs_bn`/`int_bn` bind to `isdiscrete`/`node_transform` and `int_ground_truth` is not a parameter → **`TypeError`**. The committed synthetic harness cannot run the committed synthetic model; the published synthetic numbers came from an uncommitted tree. (Hence we skip synthetic — provenance unrecoverable.)
- **Real-world harness is COHERENT.** `main-ss.py`/`main-ob.py` → `from RCAEval.e2e import brcd` → `brcd(data, inject_time, dataset, graph, **kwargs)` → `brcd_helper(...)` in the same file, matching signature. (Coherence ≠ provenance — still confirm it's the published code; §9.)
- **Three `brcd` generations + one adapter**, not rivals: top-level `brcd.py` (older, BN/ground-truth machinery, exposes `brcd_helper`); `code/models/brcd.py` (newer, Forest-KDE, U/C prior); `code/models/brcd_k.py` (newer + multi-root); `RCAEval/e2e/brcd.py` (the real-world adapter — **our target**).
- **`boss.py` is vendored three times** (`./BRCD/boss.py`, `synthetic/code/models/boss.py`, `real-world/RCAEval/e2e/BRCD/boss.py`) — copy-paste dep vendoring, classic research-repo pattern (each experiment dir self-contains its deps). All three import `local_score_BIC_from_cov` from a **0-byte `LocalScoreFunction.py`**, so BOSS relies on causal-learn's score at runtime.
- **The two experiment families used different estimators:** synthetic = Forest-KDE + oracle-BN CPDAG + `node_transform="log"`; real-world = **linear-Gaussian + `node_transform="none"`**, CPDAG per-system (§4a). We target the latter.

---

## 4. The real-world pipeline to port (verified spec)

Entry → `brcd(data, inject_time, dataset, graph)` [[adapter](../../ctx/next/brcd/experiments/real-world/RCAEval/e2e/brcd.py#L2324)]:
1. **Data prep.** Split rows by `inject_time` into normal/anomalous; `preprocess(...)` (domain step, §9); intersect columns.
2. **CPDAG** (per-system — see §4a). In the OB/Sock Shop adapter it is the **domain service-call graph**, arcs-only: `df_to_prefix_graph(normal_df, graph)` maps the service-level graph onto metric columns by name-prefix; `PDAG(nodes, arcs=…, edges=[])`.
3. → `brcd_helper(normal, anomal, cpdag, isdiscrete=False, node_transform="none", transform_parents=True, num_root_causes_candidates=1)`. CPDAG supplied ⇒ single `brcd_update` (no bootstrap).
4. **`brcd_update`:**
   - Concatenate + add `FNODE` (0/1). Candidates = each single column (k=1). Prior = uniform.
   - `sampleAugmentedGraphs`: per candidate root `r`, orient F-incident undirected edges (**none** for arcs-only input), add `FNODE→r`, Meek-complete, validate (acyclic, no new unshielded collider at `r`); `cliquepicking` `mec_size` + `sample_dag`. **Arcs-only ⇒ already a DAG ⇒ `mec_size=1`, single DAG = the graph itself.**
   - Score each unique `(node, parents)` family via `continuous_likelihood_fn_gaussian`:
     - **root (`FNODE ∈ parents`):** `F='FNODE'` → **per-regime ridge** (separate fit on normal vs anomalous rows) [[L2142](../../ctx/next/brcd/experiments/real-world/RCAEval/e2e/brcd.py#L2142)].
     - **others (`FNODE ∉ parents`):** no `F` ⇒ `F=None` → **single ridge** [[L2152](../../ctx/next/brcd/experiments/real-world/RCAEval/e2e/brcd.py#L2152)].
     - `node_transform="none"` ⇒ identity, zero Jacobian; `transform_parents` no-op under "none".
   - Per root: `logsumexp` over sampled DAGs weighted by `log(mec_size/Σ)` (trivial — one DAG). Sum row log-likelihoods + log-prior, normalize → posterior over candidates → ranked list.

**Estimator core:** ridge regression (per-regime for the root family, plain otherwise) → conditional-Gaussian log-density → sum over rows → uniform-prior posterior → rank. All functions of sample means/covariances + small SPD solves.

### 4a. BOSS is used — but per-system (corrects an earlier claim)

BOSS is **not** dead code. Which systems run it:
- **OB / Sock Shop** (`main-ss.py`/`main-ob.py` via the adapter): CPDAG = supplied call graph; BOSS is **commented out** in the adapter ([L2349](../../ctx/next/brcd/experiments/real-world/RCAEval/e2e/brcd.py#L2349)). **BOSS not used.**
- **Petshop** (`main-petshop.py`): does **not** use the adapter; it imports BOSS directly (`from RCAEval.e2e.BRCD.boss import boss`, [L49](../../ctx/next/brcd/experiments/real-world/main-petshop.py#L49)) and calls `boss(normal_metrics_new.to_numpy())` ([L582](../../ctx/next/brcd/experiments/real-world/main-petshop.py#L582)) to **learn** the CPDAG (no reliable call graph for Petshop). **BOSS used.**
- **Bootstrap path** (`brcd_helper` with `cpdag=None`): also calls BOSS ([L1886](../../ctx/next/brcd/experiments/real-world/RCAEval/e2e/brcd.py#L1886)) — not triggered by the OB/SS adapter (which supplies a CPDAG).

⇒ The "BOSS looks present but unused" impression comes from reading only the OB/SS adapter. Replicating **Petshop** requires porting BOSS (permutation search + BIC-from-cov + Meek `dag2cpdag`). Replicating **OB + Sock Shop** does not.

---

## 5. What the real-world target does NOT use (skipping = correct scope, not reduction)

| Component | Status in real-world target |
|---|---|
| **BOSS** | Needed **only for Petshop** / bootstrap; OB + Sock Shop supply the call graph. (§4a) |
| **Forest-KDE** | Absent — Gaussian path active; KDE commented out. |
| **Logistic-gated mixture-of-experts** | **Never reached** — root family uses per-regime Branch 1 (gating unused); non-roots pass `F=None` (single expert). No logistic regression to port. |
| **log/log1p/Yeo-Johnson transforms** | `node_transform="none"`. |
| **Discrete Dirichlet path** | `isdiscrete=False`. |
| **Multi-root (k>1)** | `num_root_causes_candidates=1`. |
| **Nontrivial uniform MEC sampling** | Arcs-only CPDAG ⇒ `mec_size=1`. Implement trivial case; full Wienöbst sampler likely unneeded for OB/SS (confirm §9). Petshop via BOSS→`dag2cpdag` may yield undirected edges ⇒ nontrivial MEC. |

---

## 6. In-repo capability coverage (no external numeric crates)

| Need | In-repo primitive | Verdict |
|---|---|---|
| Ridge / conditional-Gaussian (`XᵀX+λI`, SPD) | `cg_solve` ([cg_solver.rs:48](../../deep_causality_topology/src/utils/cg_solver.rs#L48)) + `CausalTensor::{matmul, inverse}` | **Covered.** CG is `pub(crate)` → lift to `deep_causality_sparse` or reimplement (~50 lines). |
| Sample mean / covariance | `Manifold::covariance_matrix` ([covariance.rs:20](../../deep_causality_topology/src/types/manifold/api/covariance.rs#L20)) or a free function | Covered. |
| DAG: parents, topo-order, acyclicity, Meek, no-new-collider | topology `Graph` (sparse CSR) + small standard algorithms | **Build** (small). |
| BOSS (only if Petshop in scope) | permutation search + BIC-from-cov (covariance + CG) + Meek | **Build** — substantial; gate on the §1 sub-decision. |
| RNG | `deep_causality_rand` | Needed only if MEC sampling turns nontrivial (Petshop) or BOSS shuffle. |

**Topology crate:** useful narrowly (CG solver + graph container) — not its TDA/Hodge/gauge machinery. **Multivector crate: not a fit.** Do not force geometric algebra in.

---

## 7. CDL integration constraints (deep_causality_discovery)

- **Two datasets.** The typestate flow carries one `CausalTensor`; BRCD needs **normal + anomalous**. ([cdl_with_features.rs](../../deep_causality_discovery/src/types/cdl/cdl_with_features.rs))
- **Output type hardcoded to `SurdResult<f64>`** in the trait, the typestate method, and `WithCausalResults`. BRCD returns a ranked posterior → generalize the trait, `CausalDiscoveryConfig`, the state, the analyzer, the formatter. ([causal_discovery.rs:34-38](../../deep_causality_discovery/src/traits/causal_discovery.rs#L34-L38), [causal_discovery_config.rs:14-16](../../deep_causality_discovery/src/types/config/causal_discovery_config.rs#L14-L16))
- BRCD needs a **user-supplied domain graph** as the CPDAG input (OB/SS) — CDL has no such notion today.

---

## 8. Validation contract

- **Oracle:** replicate the **real-world** results on the public benchmarks. Requires the RCAEval datasets + per-system service-call graphs + run protocol (§9). Start with **OB + Sock Shop** (supplied graph, no BOSS, deterministic `mec_size=1`); add **Petshop** only if BOSS is in scope.
- **Metric:** the reference's top-k / AC@k root-cause recall against labeled injected faults.
- **Pass:** Rust port's recall matches the reference within reported variance (the OB/SS path is essentially deterministic given `mec_size=1`).
- **Synthetic:** out of scope (workbench; provenance unrecoverable).

---

## 9. Author-call questions — ordered by leverage to a validatable real-world port

> If only the top three are answered, those three unblock design.

1. **Is `RCAEval/e2e/brcd.py` (+ `main-*.py`) the actual code behind the real-world paper results, and at which commit?** Internal coherence ≠ provenance (the synthetic harness proved that). Ideally: manuscript + that commit. — *Highest leverage; defines the spec.*
2. **Can we get the RCAEval datasets + per-system service-call graphs + run protocol for OB / Sock Shop / Petshop?** The call graph *is* the CPDAG input for OB/SS; without it there is nothing to port against. — *Decides whether a validatable target exists.*
3. **Per-system CPDAG source / BOSS scope:** confirm OB + Sock Shop use the supplied call graph (no BOSS) and Petshop learns it via BOSS. ⇒ decides whether BOSS is in scope at all. — *Largest swing in port size (§1 sub-decision).*
4. **What does `preprocess(data, dataset, dk_select_useful=...)` do, and is `dk_select_useful` (domain-knowledge feature selection) on the critical path for the reported numbers?** — *Hidden input-shaping dependency.*
5. **Does the OB/SS granularized call graph ever yield undirected edges / nontrivial I-MECs, or always arcs-only (`mec_size=1`)?** Decides whether the full Wienöbst sampler is needed or only the trivial single-DAG case. — *Decides MEC machinery scope.*
6. **Confirm the real-world config:** Gaussian likelihood, `node_transform="none"`, `transform_parents=True`, `k=1`, ridge λ (`1e-4`?), per-regime split for the root family. — *Pins the estimator.*
7. **Confirm the F-node detection mechanism:** per-regime scoring for the root vs single-expert for non-roots is the intended signal. — *Confirms the theoretical core.*
8. **Determinism / seed / run protocol** for the published real-world curves; expected run-to-run rank stability. — *Sets the validation tolerance band.*

---

## 10. Dependency graph — real-world folder

Annotated for the **OB / Sock Shop Gaussian target**. `🟢 LIVE` = on the target path · `🟡 PETSHOP` = only if Petshop in scope · `🔴 DEAD` = imported but never executed on the Gaussian path · `⬛ EXT` = external lib to port/replace · `📦 vendored` = copied into the bundle.

```
main-ob.py / main-ss.py ─────────────┐                         main-petshop.py ──┐
                                      ▼                                           │
                          RCAEval/e2e/brcd.py  (adapter + brcd_helper             │
                          │                     + brcd_update                     │
                          │                     + continuous_likelihood_fn_gaussian│
                          │                     + sampleAugmentedGraphs)          │
                          │                                                       │
   🟢 LIVE (OB/SS) ───────┼──► graphical_models.PDAG   ⬛  Meek to_complete_pdag, parents_of,
                          │                                chain_components, replace_edge_with_arc → PORT (L1+L2)
                          ├──► networkx                ⬛  DiGraph, predecessors, topo_sort,
                          │                                is_DAG, connected_components            → PORT (L1)
                          ├──► cliquepicking (cp)        ⬛  mec_size, MecSampler — TRIVIAL (arcs-only ⇒ =1) → L4
                          ├──► scipy.special.logsumexp ⬛  → reimpl (L0)
                          ├──► numpy.linalg solve       ⬛  ridge/Schur — → CG (L0/L3)
                          ├──► .BRCD.utils.gm_to_nx_Digraph 📦  (1 helper of a 975-line file)
                          └──► RCAEval.io.time_series   ⬛  data loading → replace w/ our loader
                                                                                  │
   🟡 PETSHOP only ──────────► .BRCD.boss.boss 📦  ◄───────────────────────────────┘ (imported directly)
                                  ├──► causallearn.search…gst.GST         ⬛  Grow-Shrink Trees  → PORT (L4b, the crux)
                                  ├──► causallearn.utils.dag2cpdag         ⬛  Meek               → reuse L2
                                  ├──► causallearn.graph.GeneralGraph/GraphNode ⬛  DAG container → reuse L1
                                  ├──► causallearn.graph.Endpoint          ⬛  edge endpoint enum → reuse L1
                                  └──► score.LocalScoreFunctionClass 📦 + local_score_BIC_from_cov 📦
                                          → BIC-from-cov = covariance Schur complement → reuse L3

   🔴 DEAD on the Gaussian path (imported, never executed):
        sklearn {KernelDensity, ExtraTreesRegressor}        → Forest-KDE backend
        sklearn.LogisticRegression                          → mixture gate (never reached)
        sklearn {PowerTransformer, StandardScaler,
                 GridSearchCV, KBinsDiscretizer}            → transforms / discretize
        pyAgrum (gum)                                       → discrete BN inference
        BRCD/mcs_enum.py 📦                                 → exact AMO enum (synthetic alt; not imported here)
        ~974/975 lines of BRCD/utils.py 📦                  → only gm_to_nx_Digraph used
        multiprocessing / concurrent.futures / tqdm         → parallelism / progress UX
```

**Key reading:** the OB/SS live path touches **zero causal-learn** — causal-learn (`GeneralGraph`, `GraphNode`, `GST`, `dag2cpdag`, `Endpoint`) is entirely inside the **BOSS/Petshop branch**. The OB/SS graph machinery comes from `graphical_models.PDAG` + `networkx` instead. So causal-learn replication is a *Petshop-gated* work item, cleanly separable from the OB/SS core.

---

## 11. causal-learn API surface (small — worth replicating in-repo)

Every causal-learn symbol the real-world BRCD path touches, all in the BOSS branch:

| causal-learn symbol | What the code uses | In-repo replacement | Cost |
|---|---|---|---|
| `GraphNode(name)` | node object | our DAG node (index) | trivial (L1) |
| `GeneralGraph(nodes)` + `add_directed_edge` + `get_nodes` + `node_map` + `get_graph_edges` + `edge.node1/2`, `endpoint1/2` | DAG container + edge-endpoint readout | our `Dag`/`Pdag` (L1) | small |
| `Endpoint.TAIL` / `Endpoint.ARROW` | encode arc vs undirected edge | our edge-kind enum (L1) | trivial |
| `dag2cpdag(G)` | DAG → CPDAG | our Meek rules (L2) | medium |
| `GST(i, score)` + `.trace(prefix[, parents_out])` | best parent-subset of `prefix` for node `i` under the local score, cached in a trie | grow-shrink parent search (L4b) | **the only nontrivial port** |
| `LocalScoreClass` (📦 53 lines) + `local_score_BIC_from_cov` (📦) | BIC score = `n·log(condVar) + log(n)·|PA|·λ`, condVar via covariance Schur complement | covariance + CG (L3) | small (vendored, readable) |

**Assessment (supports replicating, not depending):**
- The whole surface is **6 symbols**, and only **GST** carries real algorithmic weight. `GeneralGraph`/`GraphNode`/`Endpoint` are a thin graph ADT we build anyway for OB/SS; `dag2cpdag` and the BIC score reuse the Meek (L2) and covariance/CG (L3) layers we build anyway.
- `GST.trace` returns "optimal parents of `i` among `prefix`, + score." The **trie caching is an optimization** — a correctness-equivalent grow-shrink (or, for small graphs, exhaustive) parent search reproduces the *result*. We port the result, not the data structure. GST's algorithm is public BSD causal-learn source if we want a faithful reference.
- Net: a **single small Rust module** (the L1+L2+L3 layers, plus L4b grow-shrink) replaces causal-learn entirely and is **Petshop-gated**. For OB/SS we replicate *nothing* from causal-learn.

---

## 12. Dependency-ordered build roadmap (lowest dep → algorithm)

Each layer compiles and is unit-testable before the next. **Minimal first-compilable unit = L0 + L1** (independent; can be built in parallel). The OB/SS critical path is L0→L1→L2→L3→L4→L5→L6; **L4b (BOSS / causal-learn replication) is a Petshop-gated side branch.**

```
L0  Numeric foundation        cg_solve (SPD; lift from topology or reimpl ~50 LOC),
    deps: CausalTensor, num   covariance/mean, logsumexp, Gaussian log-density
        │
        ├───────────────┐
        ▼               ▼
L1  Graph ADT        L3  Scoring  (the unified covariance/CG primitive)
    Dag/Pdag:            conditional variance = Schur complement of Σ via CG  ⇒
    arcs+undirected,       (a) BRCD Gaussian family log-likelihood (per-regime + single-expert)
    parents/children,      (b) [Petshop] BIC-from-cov local score = n·log(condVar)+log(n)·|PA|·λ
    topo-order,          deps: L0
    acyclicity,
    chain components
    (replaces GeneralGraph/networkx/PDAG skeleton)
        │
        ▼
L2  Meek orientation   (ONE impl serves both dag2cpdag AND PDAG.to_complete_pdag)
    deps: L1
        │
        ├──────────────────────────────────────────────┐
        ▼                                                ▼
L4  Augmented-DAG + MEC                            L4b  BOSS  🟡 Petshop-gated
    add FNODE→root, orient F-incident edges,            permutation search +
    Meek-complete (L2), validate                        grow-shrink best-parents (≈GST) +
    (acyclic, no-new-collider);                         dag2cpdag (=L2)  → learned CPDAG
    mec_size + sampling (trivial =1 for arcs-only;       deps: L1, L2, L3
    Wienöbst sampler only if nontrivial)
    deps: L1, L2
        │
        ▼
L5  BRCD estimator
    brcd_update: candidates → augmented DAGs (L4) → family scores (L3, cached)
                 → BMA logsumexp ×mec_size → Σ rows → +prior → posterior → rank
    brcd_helper: data prep (FNODE, combos, uniform prior), dispatch
                 (supplied CPDAG → single update;  [cpdag=None → L4b bootstrap])
    deps: L1, L2, L3, L4 (+L4b if Petshop)
        │
        ▼
L6  CDL integration (deep_causality_discovery)
    two-dataset pipeline state; BrcdConfig + ranked-result type;
    generalize CausalDiscovery trait / CausalDiscoveryConfig / WithCausalResults /
    analyzer / formatter; user-supplied domain-graph input
    deps: L5
```

**Scope dial:** OB + Sock Shop ship with **L0–L6 minus L4b** — a few hundred lines of clean numerics over `CausalTensor` + the CG solver, **no causal-learn, no external numeric crate**. Adding **Petshop** turns on **L4b** (grow-shrink + the causal-learn surface from §11), the single largest increment.

---

## 13. Component generality & crate placement

Most of what the port needs is **not** BRCD-specific. Under issue #598's goal (CDL as a *collection* of discovery algorithms: PC/GES/BOSS/RCD/RCG/BRCD…), the layers split into three generality tiers. Building the shared tiers as shared code — not inside a BRCD module — is what makes this work compound across the collection.

**Tier A — general numerics (repo-wide reuse; not causal at all).** Net cleanup, not BRCD debt.
| Component | Home | Note |
|---|---|---|
| `cg_solve` (SPD solver, L0) | promote to `deep_causality_sparse` (or `deep_causality_num`), expose publicly | currently `pub(crate)` in topology; its own doc invites the lift; topology consumes it back |
| sample covariance / mean (L0) | `deep_causality_tensor` (method/util on `CausalTensor`) | **de-duplicates** the covariance already computed inside topology's `Manifold` |
| `logsumexp`, Gaussian log-density, conditional-variance / partial-correlation Schur complement (L0/L3) | `deep_causality_num` | numerically-stable stats primitives reusable anywhere |

**Tier B — general causal-discovery infrastructure (reused by the whole algorithm family, not BRCD).**
| Component | Home | Note |
|---|---|---|
| directed-graph storage + topo-sort + acyclicity (L1) | **`ultragraph` (already exists)** | reuse Kahn's `topological_sort`; add a `parents/predecessors` accessor if missing |
| PDAG/CPDAG type (mixed directed+undirected edges) (L1) | shared `causal_discovery::graph` (algorithms crate) | equivalence-class representation; algorithm-agnostic |
| Meek orientation rules (DAG↔CPDAG) (L2) | shared `causal_discovery::graph` | one impl serves `dag2cpdag` *and* PDAG completion; every score/constraint learner needs it |
| MEC size + uniform sampling (Wienöbst) (L4) | shared `causal_discovery::mec` | general causal primitive; trivial for arcs-only |
| BOSS structure learning (L4b) | `causal_discovery::boss` (sibling of `surd`) | a **standalone algorithm**, not BRCD-specific — BRCD only *consumes* a CPDAG |

**Tier C — BRCD-specific (only this algorithm).**
| Component | Home |
|---|---|
| F-node augmented-DAG construction; `brcd_update`/`brcd_helper`; U/C prior (L4–L5) | `deep_causality_algorithms::brcd` (sibling of `surd`) |
| two-dataset pipeline state + polymorphic discovery-result type (L6) | `deep_causality_discovery` — pipeline-specific, but a general improvement future algorithms reuse |

**Module-first vs new crate.** Tier B is genuinely shared, but today nothing *outside* `deep_causality_algorithms` needs it (discovery consumes algorithms; topology doesn't need PDAGs). So house Tier B as **shared submodules inside `deep_causality_algorithms`** (`causal_discovery::{graph, mec, boss}`), beside `surd` — lowest ceremony, fits the existing layout. **Promote to a dedicated crate** (e.g. `deep_causality_causal_graph`) **only when** a non-algorithm crate needs the PDAG/Meek types directly. That trigger doesn't exist yet; adopting a new crate now buys 19→20 crates of Cargo/lints/SBOM/Bazel ceremony for no current consumer.

```
Tier A  numerics            deep_causality_num / _tensor / _sparse   (general; cleanup + de-dup)
Tier B  causal infra        ultragraph (digraph)  +  algorithms::causal_discovery::{graph,mec,boss}
Tier C  BRCD                algorithms::brcd   +   discovery (two-dataset + result generalization)
```

**Takeaway:** of the whole port, only **Tier C** is BRCD-specific. Tier A is reusable everywhere (and pays down existing duplication), Tier B is the foundation every future causal-discovery algorithm in CDL will reuse. Build A and B as shared layers and the next algorithm (RCD, RCG, PC, GES…) starts from L4, not L0.

---

## 14. VERIFIED specification — real-world OB / Sock Shop path (no-assumptions pass)

Full line-by-line trace of `RCAEval/e2e/brcd.py` + drivers + `RCAEval/io/time_series.py` + cliquepicking docs. Everything below is **read from source**, not inferred. (Petshop/BOSS path not in this pass — separate.)

### 14.1 End-to-end pipeline (verified, with line refs)

**Driver** (`main-ss.py` / `main-ob.py`): download dataset via RCAEval (`download_*_dataset()`); per fault case read `simple_data.csv` → `data` (has a `time` column); `inject_time` from `inject_time.txt`. Build the hard-coded service call graph (`create_sock_shop_graph` 18 edges / `create_online_boutique_graph` 14 edges — both acyclic `nx.DiGraph`), then **`G = G.reverse()`** for *both* datasets (main-ss.py L307-311). Dispatch `func = brcd` with `graph=G`, `dk_select_useful=False`; consume `out["ranks"]` vs the labeled true root cause (top-k).

> ⚠️ **`G.reverse()` is load-bearing and easy to miss.** The CPDAG arcs are the **reverse** of the service call graph (failure propagates dependency→dependent, opposite to call direction). The port MUST reverse.

**`brcd(data, inject_time, dataset, graph)`** [L2324]: split normal=`time<inject_time` / anomalous=`time>=inject_time`, drop `time`; `preprocess` each (§14.3); intersect columns; `granular = df_to_prefix_graph(normal, graph)` expands reversed service edges to metric columns via `col.startswith(prefix)`, all-u×all-v [L46]; `cpdag = PDAG(nodes=cols, arcs=granular.edges, edges=[])` — **arcs-only by construction** [L2347]; → `brcd_helper(…, isdiscrete=False, node_transform="none", transform_parents=True, k=1)`.

**`brcd_helper`** [L2227]: `joint = concat(normal, anomal)`; `FNODE = [0]*|normal| + [1]*|anomal|` [L2247]; `combos` = each single column; `prior = uniform` [L2252] (**BRCD-U; the χ²/C prior is NOT in the real-world path**); CPDAG supplied → single `brcd_update`; rank columns by posterior desc → `{"ranks": …}`.

**`brcd_update`** [L2104]:
- `sampleAugmentedGraphs` [L1537]: per candidate root `r`, `getConfigurations_multi` → arcs-only ⇒ no incident undirected edges ⇒ **E=0 ⇒ exactly one configuration**; `to_complete_pdag()` (Meek) is a no-op on a DAG; convert to single-directed nx edges; add `FNODE` node + `FNODE→r`; `cp.mec_size` + `cp.MecSampler.sample_dag()`.
- Score each unique `(node, parents)` family once [L2131] via `continuous_likelihood_fn_gaussian`:
  - **root `r` (`FNODE∈parents`):** `F='FNODE'` → **Branch 1** [L407]: split rows by F; per regime, ridge-regress `node` on continuous parents, density `N(y; Xβ, σ²)`.
  - **every other node (`FNODE∉parents`):** called with **no F ⇒ `F=None`** → **Branch 2 single-expert** [L469]: ridge-regress on all rows. (The mixture/logistic-gate code at L496-566 is **unreachable** — `F=None` never enters it.)
- Aggregate [L2190]: `log_joint(DAG)=Σ_node famLL`; `logsumexp` over the single DAG `+ log_p_G`; sum over rows; `+ log(prior)`; `posterior = exp(logpost − max)`.

### 14.2 Verified simplifications (now facts, not assumptions)

| Claim | Status | Evidence |
|---|---|---|
| MEC layer is **deterministic**, `mec_size=1`, `sample_dag`=identity | **CONFIRMED** | arcs-only CPDAG + FNODE out-only ⇒ pure DAG; cliquepicking docs: "single `(a,b)` = directed; DAG input ⇒ `mec_size`=1, `sample_dag` returns the input." **⇒ cliquepicking replaceable by pass-through; no uniform sampler needed for OB/SS.** Resolves §9 Q5. |
| Logistic-gated mixture-of-experts | **DEAD** | non-root families pass `F=None` → single-expert sub-branch; mixture code never reached |
| log/log1p/Yeo-Johnson transforms | **INERT** | `node_transform="none"` ⇒ `_transform_and_jacobian` returns identity + zero Jacobian [L295]; `transform_parents` guarded by `eff_transform!="none"` |
| Prior | **uniform (BRCD-U)** | `prior = np.ones(len(combos))/len(combos)` [L2252] |
| Ridge λ | **`1e-4`** | default in both score fns [L345/L575]; `_fit_ridge` = `(XᵀX+λI)⁻¹Xᵀy`, `σ²=RSS/dof` [L326] |
| Root scoring | **per-regime ridge**; others **pooled ridge** | Branch 1 vs Branch 2 single-expert |
| `k` | **1** (single root) | `num_root_causes_candidates=1`; final `[t[0] for t in …]` |
| **Whole OB/SS path is deterministic** | **CONFIRMED** | mec_size=1 (no sampling) + `np.linalg.solve` + closed-form means/vars ⇒ no RNG anywhere ⇒ validation is **exact-comparable**, not distributional |

### 14.3 `preprocess` (RCAEval/io/time_series.py L96) — verified

With `dk_select_useful=False` (the driver's value) and `dataset` set: `preprocess = drop_time → drop_constant → convert_mem_mb`. The `select_useful_cols`/`drop_extra`/`drop_near_constant` domain-knowledge block is **skipped**. ⇒ candidate columns = all metrics minus `time` minus zero-variance columns, with memory rescaled to MB. **No feature selection.** Resolves §9 Q4 (dk is OFF in the published driver).

### 14.4 Estimator in one sentence

For each candidate metric `r`: build DAG = (reversed call graph) + `FNODE→r`; score the joint Gaussian log-likelihood of the concatenated normal+anomalous data where `r`'s conditional is fit **separately per regime** and every other node's is fit **pooled**; rank `r` by `Σ_rows logP + log(uniform prior)`. The metric whose per-regime split best explains the anomalous rows wins. Pure means/covariances + `1e-4`-ridge solves; deterministic.

### 14.5 Remaining genuine unknowns (small, precisely scoped)

- **Provenance** (still §9 Q1): is this file the published code? The code is now *fully understood* regardless, so this only affects whether our spec == the paper's spec.
- **Petshop/BOSS path**: not traced in this pass (out of OB/SS scope; turns on the causal-learn surface §11 + L4b).
- **Minor numeric details**: `drop_constant` threshold, `convert_mem_mb` factor, `intersect` column ordering — all in `time_series.py`, deterministic, trivially portable.
- `df_to_prefix_graph` uses crude `startswith` prefix matching — fine for these service names, but a faithful port should replicate the exact matching (not a "smarter" version).

**Conclusion:** the OB/Sock Shop target is now an **implementation-grade, fully-deterministic spec** with no external sampler, no causal-learn, no sklearn, no transforms, and reproducible public data + hard-coded graphs. The roadmap (§12) for OB/SS is confirmed sound; the only open item that matters is provenance confirmation with the author.
