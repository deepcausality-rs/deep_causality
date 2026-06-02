# BRCD Port — Findings, Scope Decision & Author-Call Questions

**Issue:** [#598 "Investigate BRCD for addition to CDL"](https://github.com/deepcausality-rs/deep_causality/issues/598)
**Status:** Pre-design investigation. No code written. Scope decided (§1). Provenance + per-system CPDAG source pending author call (§9).
**Prerequisite changes (must land first, in order):** `real-field-discovery` (generify `deep_causality_algorithms` + `deep_causality_discovery` over `RealField`) → `brcd-prep-foundations` (shared numerics + causal-graph layer + `DiscoveryOutcome<T>` + two-dataset carriage). The BRCD estimator change composes both.
**Genericity requirement:** the BRCD estimator MUST itself be fully generic over `T: RealField`. Its numeric parts (per-regime/pooled Gaussian scoring, conditional-variance, log-likelihood, posterior, ranking) are generic over `T`, returning `BrcdResult<T>` carried as `DiscoveryOutcome<T>`. The augmented-DAG construction reuses the precision-agnostic causal-graph layer (no scalar, no `T`). So BRCD composes seamlessly with the math stack at `f32`/`f64`/`Float106`/`f16`/`f128` — genericity follows the data: real where there are reals, absent where there are none.
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

Entry → `brcd(data, inject_time, dataset, graph)` [[adapter](../../../ctx/next/brcd/experiments/real-world/RCAEval/e2e/brcd.py#L2324)]:
1. **Data prep.** Split rows by `inject_time` into normal/anomalous; `preprocess(...)` (domain step, §9); intersect columns.
2. **CPDAG** (per-system — see §4a). In the OB/Sock Shop adapter it is the **domain service-call graph**, arcs-only: `df_to_prefix_graph(normal_df, graph)` maps the service-level graph onto metric columns by name-prefix; `PDAG(nodes, arcs=…, edges=[])`.
3. → `brcd_helper(normal, anomal, cpdag, isdiscrete=False, node_transform="none", transform_parents=True, num_root_causes_candidates=1)`. CPDAG supplied ⇒ single `brcd_update` (no bootstrap).
4. **`brcd_update`:**
   - Concatenate + add `FNODE` (0/1). Candidates = each single column (k=1). Prior = uniform.
   - `sampleAugmentedGraphs`: per candidate root `r`, orient F-incident undirected edges (**none** for arcs-only input), add `FNODE→r`, Meek-complete, validate (acyclic, no new unshielded collider at `r`); `cliquepicking` `mec_size` + `sample_dag`. **Arcs-only ⇒ already a DAG ⇒ `mec_size=1`, single DAG = the graph itself.**
   - Score each unique `(node, parents)` family via `continuous_likelihood_fn_gaussian`:
     - **root (`FNODE ∈ parents`):** `F='FNODE'` → **per-regime ridge** (separate fit on normal vs anomalous rows) [[L2142](../../../ctx/next/brcd/experiments/real-world/RCAEval/e2e/brcd.py#L2142)].
     - **others (`FNODE ∉ parents`):** no `F` ⇒ `F=None` → **single ridge** [[L2152](../../../ctx/next/brcd/experiments/real-world/RCAEval/e2e/brcd.py#L2152)].
     - `node_transform="none"` ⇒ identity, zero Jacobian; `transform_parents` no-op under "none".
   - Per root: `logsumexp` over sampled DAGs weighted by `log(mec_size/Σ)` (trivial — one DAG). Sum row log-likelihoods + log-prior, normalize → posterior over candidates → ranked list.

**Estimator core:** ridge regression (per-regime for the root family, plain otherwise) → conditional-Gaussian log-density → sum over rows → uniform-prior posterior → rank. All functions of sample means/covariances + small SPD solves.

### 4a. BOSS is used — but per-system (corrects an earlier claim)

BOSS is **not** dead code. Which systems run it:
- **OB / Sock Shop** (`main-ss.py`/`main-ob.py` via the adapter): CPDAG = supplied call graph; BOSS is **commented out** in the adapter ([L2349](../../../ctx/next/brcd/experiments/real-world/RCAEval/e2e/brcd.py#L2349)). **BOSS not used.**
- **Petshop** (`main-petshop.py`): does **not** use the adapter; it imports BOSS directly (`from RCAEval.e2e.BRCD.boss import boss`, [L49](../../../ctx/next/brcd/experiments/real-world/main-petshop.py#L49)) and calls `boss(normal_metrics_new.to_numpy())` ([L582](../../../ctx/next/brcd/experiments/real-world/main-petshop.py#L582)) to **learn** the CPDAG (no reliable call graph for Petshop). **BOSS used.**
- **Bootstrap path** (`brcd_helper` with `cpdag=None`): also calls BOSS ([L1886](../../../ctx/next/brcd/experiments/real-world/RCAEval/e2e/brcd.py#L1886)) — not triggered by the OB/SS adapter (which supplies a CPDAG).

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
| Ridge / conditional-Gaussian (`XᵀX+λI`, SPD) | `cg_solve` ([cg_solver.rs:48](../../../deep_causality_topology/src/utils/cg_solver.rs#L48)) + `CausalTensor::{matmul, inverse}` | **Covered.** CG is `pub(crate)` → lift to `deep_causality_sparse` or reimplement (~50 lines). |
| Sample mean / covariance | `Manifold::covariance_matrix` ([covariance.rs:20](../../../deep_causality_topology/src/types/manifold/api/covariance.rs#L20)) or a free function | Covered. |
| DAG: parents, topo-order, acyclicity, Meek, no-new-collider | topology `Graph` (sparse CSR) + small standard algorithms | **Build** (small). |
| BOSS (only if Petshop in scope) | permutation search + BIC-from-cov (covariance + CG) + Meek | **Build** — substantial; gate on the §1 sub-decision. |
| RNG | `deep_causality_rand` | Needed only if MEC sampling turns nontrivial (Petshop) or BOSS shuffle. |

**Topology crate:** useful narrowly (CG solver + graph container) — not its TDA/Hodge/gauge machinery. **Multivector crate: not a fit.** Do not force geometric algebra in.

---

## 7. CDL integration constraints (deep_causality_discovery)

- **Two datasets.** The typestate flow carries one `CausalTensor`; BRCD needs **normal + anomalous**. ([cdl_with_features.rs](../../../deep_causality_discovery/src/types/cdl/cdl_with_features.rs))
- **Output type hardcoded to `SurdResult<f64>`** in the trait, the typestate method, and `WithCausalResults`. BRCD returns a ranked posterior → generalize the trait, `CausalDiscoveryConfig`, the state, the analyzer, the formatter. ([causal_discovery.rs:34-38](../../../deep_causality_discovery/src/traits/causal_discovery.rs#L34-L38), [causal_discovery_config.rs:14-16](../../../deep_causality_discovery/src/types/config/causal_discovery_config.rs#L14-L16))
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

## 15. Full-paper algorithm & estimator detail (supersedes the source-level reconstruction where they differ)

The complete BRCD paper draft is now available (unpublished — **no citation until publication**). It confirms the architecture in §§12–14 and pins down three things our source-level study left approximate: the two driver algorithms, the exact per-configuration operation list (which validates the Tier B causal-graph layer), and the Bayesian linear-Gaussian estimator the real-world path actually uses (which refines the BRCD-change scope beyond a plug-in Gaussian).

### 15.1 Algorithm 1 — Augmented Graphs Sampling

```
input : CPDAG C(G*) over V = (V, E); number of root causes k
output: set of augmented DAGs S_Gaug, and T = Σ_i Q_i
S_Gaug = ∅ ; T = 0
for each R ⊆ V with |R| = k:
    G_p ← add F → R to C(G*)                      # F-node augmentation
    for each cut configuration of the neighborhood of R:
        C_R(G*) ← apply Meek rules (Meek 1995) to G_p     # → I-CPDAG
        (G_aug, Q_i) ← sample one DAG from C_R(G*) AND
                       compute |C_R(G*)| via clique-picking (Wienöbst 2023)
        add (G_aug, Q_i) to S_Gaug
return S_Gaug, T = Σ_i Q_i
```

The "cut configuration of the neighborhood of R" enumerates the orientations of the **edge cut** `E[R, V∖R]` (edges with one end in R, one in V∖R); Meek closure turns each valid orientation into an I-CPDAG. Corollary 4.2 (paper): this enumerates *all and only* the I-CPDAGs compatible with `(C(G*), R)`.

### 15.2 Algorithm 2 — BRCD

```
input : data D = D_obs ∪ D_int ; root-cause prior p(R) ; CPDAG C(G*)
output: posterior p(R | D)
S_Gaug, T ← Algorithm 1 on C(G*)
for each candidate R' ⊆ V:
    for each (G_aug, Q_i) ∈ S_Gaug with F → R' in G_aug:
        p(D | G, R') ← Π_{X ∈ V ∪ {F}} p(X | Pa(X))   # factorize per G_aug
        M_i = p(D | G, R') · p(G | R'),  with p(G | R') = Q_i / T
    p(D | R') ← Σ_i M_i
p(R | D) ← p(D | R) p(R) / Σ_{R'} p(D | R') p(R')
return p(R | D)
```

Key efficiency identity (paper §4, Lemma 4.5): all DAGs in one I-MEC share the same interventional likelihood, so scoring one I-CPDAG (one sampled DAG, weighted by MEC size `Q_i/T`) stands in for super-exponentially many DAGs. Ranking the `(G, R)` posteriors under a uniform prior is equivalent to ranking I-CPDAGs.

### 15.3 Appendix E — per-configuration operation list (validates Tier B)

For a CPDAG with `n` nodes, `m` edges, `u` undirected edges, `d_max = max_V` undirected-degree, the algorithm enumerates `Σ_V 2^{d_u(V)} ≤ n·2^{d_max}` configurations; **per configuration** it runs exactly:

| Operation | Cost | Our prep mapping |
| --- | --- | --- |
| Apply **Meek rules** | `O(n · d_max²)` | `brcd-prep` task 2.4 |
| **Acyclicity check via DFS** | `O(n + m)` | task 2.3 (our self-contained cycle/topo check — DFS/Kahn, **no** `ultragraph` round-trip; confirms decision to keep it in-type) |
| **New-unshielded-collider check** | `O(n · d_max²)` | task 2.5 (validity check) |
| **DAG sample + MEC size** via clique-picking | `O(n⁴)` | deferred — the Wienöbst uniform sampler (Petshop-gated); prep ships only the trivial arcs-only MEC (task 2.6) |
| Likelihood eval (per family) | `O(Σ_i r_i q_i)` discrete / Gaussian solve continuous | the BRCD-change scoring layer, built on Tier A |

So the Tier B causal-graph layer (Meek + DFS acyclicity + unshielded-collider validity + MEC sizing) is exactly the per-configuration machinery the paper specifies. The only Tier B piece the paper runs that prep defers is the **full clique-picking MEC sampler**; prep's trivial arcs-only case (size 1, representative = input) is sufficient for the OB/Sock Shop target.

The setting assumes **no latent variables** (paper §3), so BRCD lives entirely in CPDAG-space — directed + undirected edges only. The `MixedGraph` two-mark `{Tail, Arrow}` set is sufficient; the reserved `Circle` mark is forward-looking (PAGs) and never exercised by BRCD.

### 15.4 Appendix F — the *paper's* conjugate linear-Gaussian *(SUPERSEDED by §16.1: the authoritative code uses a plug-in ridge-Gaussian, not NIG/Student-t)*

The real-world (continuous) path does **not** score with a plug-in Gaussian log-density. §14.4's "joint Gaussian log-likelihood … per-regime vs pooled" is the point-estimate sketch; the paper's *default* scoring rule is the **integrated (prequential) marginal likelihood** of a Bayesian linear-Gaussian family, chosen for finite-sample robustness with very few anomalous samples. Per family `p(X | Pa(X))`:

- **Optional monotone transform** `z = T(X)` (e.g. `log`, `log1p`) on child (and optionally parents) for linearity; the density on the original scale carries the **Jacobian** `|dT(x)/dx|`.
- **Per-regime linear-Gaussian model**, `F = f ∈ {0,1}`:  `z = β_{f,0} + β_fᵀ U + ε_f`, `ε_f ~ N(0, σ_f²)`, where `U` are the continuous parents other than `F`.
- **Normal-Inverse-Gamma conjugate prior** on `(β_f, σ_f²)` with hyperparameters `(m_0, Λ_0, α_0, β_0)`; posterior updates:
  - `Λ_f = Λ_0 + X_fᵀ X_f`
  - `m_f = Λ_f⁻¹ (Λ_0 m_0 + X_fᵀ y_f)`
  - `α_f = α_0 + n_f / 2`
  - `β_f^post = β_0 + ½ (y_fᵀ y_f + m_0ᵀ Λ_0 m_0 − m_fᵀ Λ_f m_f)`
- **Student-t posterior predictive** per new row `x*` (covariates incl. intercept):
  - `z* | D_f ~ Student-t( ν_f = 2α_f,  μ_f = x*ᵀ m_f,  s_f² = (β_f^post / α_f)(1 + x*ᵀ Λ_f⁻¹ x*) )`
  - per-row density on the untransformed scale: `t_{ν_f}(T(x*); μ_f, s_f²) · |dT(x*)/dx|`
- The **family marginal likelihood** is the product of per-row predictives (order-independent). The root-cause node `r`'s family is fit **per regime** (`F=0` and `F=1` separately); all other nodes' families are fit **pooled**. (Discrete data uses the analogous Dirichlet/BDeu prequential score — Appendix F — but the OB/Sock Shop target is continuous.)

**Implication for scope:** prep's Tier A primitives (`conditional_variance` = covariance Schur complement → residual variance; `gaussian_log_density`; `cg_solve`; sample mean/covariance) are the *necessary numeric building blocks* but are **not** the BRCD scoring layer. The BRCD change must add the NIG → Student-t conjugate machinery (`Λ`/`m`/`α`/`β` posterior updates with ridge-stabilized SPD solves, the Student-t log-density, the optional transform + Jacobian, and the per-regime/pooled assembly) on top of them. The theoretical guarantees (Theorems 4.3/4.4) are estimator-agnostic — they need only an `ε`-accurate plug-in — so the conjugate model is a robustness/finite-sample choice, not a correctness requirement; a simpler plug-in Gaussian would still satisfy the consistency results but underperform at the 5-anomalous-sample regime the paper targets.

## 16. Authoritative source reconciliation (`ctx/next/brcd/`) — supersedes earlier reconstructions

The algorithm's author provided the **authoritative** BRCD under `ctx/next/brcd/`; the versions in the experiment repos (RCAEval / the reference bundle studied in §§3–4 and §14) were **modified for each experiment's specifics and are NOT authoritative**. This section records what the authoritative code pins down and corrects. Where it conflicts with §§0–15, **§16 wins**.

### 16.0 Authoritative files — this is the source of truth

Port against these files only. Anything under the experiment repos is reference/illustrative, not normative.

| Authoritative file | Role |
| --- | --- |
| **`ctx/next/brcd/brcd.py`** | The BRCD algorithm itself: the linear-Gaussian / Dirichlet / Forest-KDE estimators, the F-node augmentation, cut-configuration enumeration + validity (`getConfigurations_multi`, `has_new_unshielded_collider_at`), augmented-DAG sampling (`sampleAugmentedGraphs`), the posterior assembly (`brcd_update`), the bootstrap-CPDAG path, and the driver (`brcd_helper`). **This is the primary spec.** |
| **`ctx/next/brcd/BRCD/boss.py`** | BOSS observational structure learning (BIC-scored, with optional background-knowledge required edges) — the upstream step that produces the input CPDAG when one is not supplied. |
| **`ctx/next/brcd/BRCD/LocalScoreFunction.py`**, **`LocalScoreFunctionClass.py`** | The local score functions (BIC etc.) BOSS optimizes. |
| **`ctx/next/brcd/BRCD/mcs_num.py`** | A local MEC enumerator (`enumerate_amos`, `enumerate_dags`) — the AMO/clique-tree route, an in-tree alternative to the external `cliquepicking` package used in `brcd.py`. |
| **`ctx/next/brcd/BRCD/utils.py`** | PDAG ↔ `causal-learn` graph conversion and CPT / Bayesian-network helpers. |

**Non-authoritative (do not port from):** the experiment-repo `brcd.py` and harness studied in §§3–4 (`experiments/real-world/RCAEval/e2e/…`) and the verified-spec pass in §14 — useful as cross-checks, but the per-experiment edits there diverge from the canonical algorithm above. Treat §§3–4 and §14 as *historical reconstruction*; trust §16 + `ctx/next/brcd/` for the port.

### 16.1 Estimator — plug-in ridge-Gaussian, NOT NIG/Student-t *(corrects §15.4)*

§15.4 described the paper's Appendix F (Normal-Inverse-Gamma → Student-t conjugate). The **authoritative code does not do that.** The continuous family `continuous_likelihood_fn_gaussian` → `gaussian_conditional_postpred_rowwise` is a **plug-in linear-Gaussian**:

- **Ridge least squares** `_fit_ridge`: `β = solve(XᵀX + λI, Xᵀy)`, `σ² = (resid·resid)/max(n−p,1)`, floored to `1e-12`; ridge `λ = 1e-4`. (A plain SPD solve — our `cg_solve` fits, or a direct solve — **not** the covariance Schur complement; `conditional_variance` computes the same population quantity but is not the path the code takes.)
- **Plug-in Gaussian log-density** `_normal_logpdf_1d(x, μ, var) = −½(log(2π·var) + (x−μ)²/var)` with `var` floored to `1e-12` — **byte-for-byte our Tier A `gaussian_log_density`.** Confirmed primitive.
- **F integration, three modes:** F ∈ parents → fit a separate ridge-Gaussian **per regime** (F=0/F=1); F present but ∉ parents → **mixture of two experts** with a **logistic-regression gate** `π(F=1|X)` (fallback: empirical prior); F absent → a single expert.
- **Optional monotone transform** none/log/log1p/yeojohnson with the Jacobian on the original scale, auto-downgrading `log → log1p → yeojohnson` when the data make a transform invalid.
- **Discrete family** `discrete_likelihood_fn_dirichlet`: **Dirichlet posterior-predictive (prequential)**, `α* = 5.0` — this *does* match the paper. A Forest-KDE family exists as the nonparametric option.

**Scope impact (future BRCD change):** the continuous estimator is simpler on the variance side than §15.4 implied (plug-in ridge, not conjugate), but adds three components our prep does not cover: a **mixture-of-experts** assembly, a **logistic-regression gate**, and **optional transforms + Jacobian**. The logistic gate is the one genuinely new numeric primitive (everything else composes from `gaussian_log_density` + an SPD/ridge solve + `logsumexp`).

### 16.2 MEC — the FULL clique-picking sampler is on the main path *(refines §15.3 / brcd-prep D6)*

The authoritative code calls `cliquepicking` (`cp.mec_size(edges)`, `cp.MecSampler(edges).sample_dag()` — Wienöbst 2023) for **every** root configuration in `sampleAugmentedGraphs` **and** every bootstrap CPDAG in `get_top_k_cpdags_with_ratio`. There is also a local enumerator `BRCD/mcs_num.py` (`enumerate_amos`, `enumerate_dags`). Either way, **the full uniform MEC sampler is used on the OB/Sock Shop path, not only Petshop.** The trivial arcs-only MEC case scoped in `brcd-prep-foundations` (D6) is therefore a **placeholder**: a faithful BRCD port must port the full clique-picking sampler (`mec_size` + uniform `sample_dag`) or the `mcs_num` AMO-enumeration. This is a substantial new component for the BRCD change, not a deferred Petshop concern.

### 16.3 Graph layer — validates Tier B and `MixedGraph`

The code operates on `graphical_models.PDAG`: `to_complete_pdag()` (Meek), `parents_of`, `replace_edge_with_arc`, `_undirected_neighbors`, `arcs`/`edges`, adjacency via `has_edge`/`has_arc`. `getConfigurations_multi` enumerates the `2^E` orientations of the undirected edges **incident on the root-candidate set**, validating each by Meek completion + acyclicity (`networkx.is_directed_acyclic_graph`) + the **no-new-unshielded-collider** check (`has_new_unshielded_collider_at`). This maps 1:1 onto `topology::MixedGraph` (arcs/undirected/parents/`set_endpoint`/`topological_sort`) and the `brcd-prep` Meek + validity tasks (2.4, 2.5). No change needed to those.

### 16.4 Driver structure (for the future BRCD change)

- `brcd_helper(normal_df, anomalous_df, cpdag=None, …)`: concatenate normal+anomalous, add an `FNODE` indicator column (0 on normal rows, 1 on anomalous), enumerate root-cause candidate combos, uniform prior. If no CPDAG is supplied → bootstrap CPDAGs via **BOSS** (`BRCD/boss.py`, BIC-scored, with optional background-knowledge edges) + weight per `get_top_k_cpdags_with_ratio` + `parallel_weighted_posterior`; else call `brcd_update` directly.
- `brcd_update`: `sampleAugmentedGraphs` → cache each unique `(node, parents)` family's per-row log-likelihood once → per root, sum cached log-factors into a per-DAG `log P(D|G)`, add `log(mec_size/Σ)`, `logsumexp` over the root's DAGs, sum over rows, add `log(prior)`, normalize → posterior over roots → rank.

### 16.5 External-dependency → in-repo replacement map (BRCD change)

| Reference (authoritative) | In-repo replacement | Status |
| --- | --- | --- |
| `graphical_models.PDAG` | `deep_causality_topology::MixedGraph` | ✅ built (archived `mixed-graph`) |
| `to_complete_pdag` (Meek) | `brcd-prep` `causal_discovery::meek` over `MixedGraph` | scoped (task 2.4) |
| `networkx` acyclicity | `MixedGraph::topological_sort`/`has_cycle` | ✅ built |
| `has_new_unshielded_collider` | `brcd-prep` `causal_discovery::validity` | scoped (task 2.5) |
| `cliquepicking` `mec_size`/`MecSampler` | **port the Wienöbst clique-picking sampler** (or `mcs_num` AMO enumeration) | **NEW — substantial; brcd-prep ships only the trivial case** |
| `scipy.special.logsumexp` | `deep_causality_tensor::logsumexp` | ✅ built |
| `_normal_logpdf_1d` | `deep_causality_tensor::gaussian_log_density` | ✅ built (exact match) |
| `_fit_ridge` (`solve(XᵀX+λI)`) | `deep_causality_sparse::cg_solve` or a direct SPD solve | ✅ primitive available |
| `sklearn.LogisticRegression` (F-gate) | **logistic regression** | **NEW — needed for mixture gating** |
| `PowerTransformer`/`KernelDensity` | yeojohnson transform / Forest-KDE | optional, defer (Gaussian path is the target) |
| `BRCD/boss.py` (structure learning) | upstream CPDAG discovery (CDL produces the CPDAG) | separate concern |

### 16.6 Author update (small) — driver contract, local scores, and a golden toy *(2026-06-02 repo refresh)*

The author refreshed `ctx/next/brcd/`: added `BRCD/LocalScoreFunction.py`, updated the README, and committed a toy run in `ctx/next/example.txt`. None of it changes the estimator; it pins the public contract and gives a reproducible end-to-end reference.

- **Driver contract (README).** `from brcd.brcd import brcd_helper as brcd`, called as
  `brcd(df_obs, df_a, cpdag=…, isdiscrete=False, node_transform="none", transform_parents=…, num_root_causes_candidates=k, bootstrap_samples=…) -> {"ranks": [...]}`.
  The output is `result["ranks"]` — a **ranked list of root-cause candidates** (best first). `num_root_causes_candidates = k` controls how many simultaneous root causes are scored. `cpdag=None` triggers the BOSS bootstrap path (out of scope here). This refines §16.4: the in-repo `BrcdResult<T>` exposes a `ranks` ordering.
- **`LocalScoreFunction.py` (BOSS only).** The local scores BOSS optimizes: `local_score_BIC` / `local_score_BIC_from_cov` (linear-Gaussian BIC, `n·log(σ²_resid) + log(n)·|PA|·λ`, with ridge `eps=1e-6` on `XX` and a `lambda_value` penalty discount), `local_score_BDeu` (discrete), and CV / marginal RKHS scores. **All of this belongs to the deferred BOSS / no-CPDAG bootstrap change, not `brcd-estimator`** (which requires a supplied CPDAG, D6). No effect on stages 1–9.
- **Golden toy (`example.txt`) — the primary acceptance fixture.** Linear-Gaussian chain `X → Y → Z` (`X=εx`, `Y=1.5X+εy`, `Z=2Y+εz`); normal `df_obs` (seed 1), anomalous `df_a` (seed 2) **perturbs `p(Y|X)`** via `y_intercept=2.0`. CPDAG = nodes `[X,Y,Z]`, **`arcs=[]`, `edges=[(X,Y),(Y,Z)]`** (a fully *undirected* chain — one chain component, the path-of-3 → 3 AMOs, exactly our stage-1 MEC). `node_transform="none"`, `num_root_causes_candidates=1`. **Expected `ranks: ['Y','X','Z']`** (Y is the root cause; its mechanism changed). `transform_parents=True` is passed but is a **no-op** here because `node_transform="none"` (`transform_parents` only acts when a transform is active), so the fixture needs no transform work — it is reproducible by stages 1–6 alone.
  - *Data capture:* the data uses numpy `default_rng` (PCG64), **not bit-reproducible from the seed in Rust**, so the verification commits the generated `df_obs`/`df_a` as CSV golden inputs and asserts Rust BRCD returns `['Y','X','Z']` on them (verification tiers 1/3; `transform_parents` only acts when a transform is active).
