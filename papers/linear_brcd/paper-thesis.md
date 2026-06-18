# Follow-up paper

Author: Marvin Hansen

Status: First draft

Scope: Core thesis

Introducing commit: 0019b48260fde2bae06308797cbc544a474bd68f
---

## Core thesis

> **The failure-period cost of BRCD is not the configuration enumeration the worst
> case suggests. It is a single dominant configuration per candidate.** Under the
> same KL separation that drives BRCD's own posterior-consistency guarantee, the
> per-candidate configuration posterior concentrates, so scoring a small frontier
> (in the detectable-anomaly regime, the single MAP I-CPDAG) reproduces the exact
> candidate ranking. This turns BRCD's `Σ_V 2^{du(V)}` worst case into near-linear
> failure-period work with no loss of ranking accuracy, under a concentration
> guarantee that is a corollary of the original Theorem 4.4. We deliver the result
> as a deployable instrument: a compositional, type-safe Causal Discovery Language
> (CDL) with a dependency-free, type-safe implementation.

---

## 1. What the original paper established 

**BRCD is the first Bayesian root-cause discovery method that localizes the failed mechanism 
in a microservice system without learning the full causal graph and without post-failure interventions.

Its main contribution:

- Model a failure as a soft intervention with a proxy node `F` pointing at the
  root cause, and use a CPDAG learned from pre-failure observational data as the
  partial causal structure.
- Exploit that all DAGs in one interventional Markov-equivalence class (I-MEC)
  share a data likelihood, so a super-exponential set of DAGs collapses to one
  likelihood evaluation per I-CPDAG. Per candidate `R`, the method enumerates the
  cut configurations of `R`'s neighborhood (Algorithm 1, exhaustive by Corollary
  4.2), weights each I-CPDAG by its MEC size via the polynomial Wienöbst et al.
  (2023) sampler, and ranks candidates by the posterior `p(R | D)`.
- Provide the first nonparametric RCA guarantees: identifiability under
  interventional faithfulness (Lemma 4.1), and posterior consistency with an
  exponential finite-sample bound that survives an ε-accurate plug-in estimator
  (Theorems 4.3–4.4), governed by a KL separation `Δ_min`.
- Reach state-of-the-art top-`l` accuracy on Petshop, Online Boutique, and
  Sockshop, and on synthetic graphs, with as few as 5 anomalous samples.

The method is strong and the analysis is tight. The paper names its one worst-case
cost itself (Appendix E): the per-candidate configuration enumeration is
exponential in the local undirected degree, `Σ_V 2^{du(V)}`.

## 2. The problem to be solved

The original BRCD leaves two gaps open:

1. **The exponential runtime gates scalability.** BRCD's runtime grows with
   the undirected structure of the CPDAG. The paper caps synthetic runs at three
   minutes, and its scalability plot (Fig 2b) reaches roughly 150 seconds at
   `n = 1000`. On well-identified graphs this stays benign. On large or weakly
   identified structures the `2^{du}` enumeration becomes the blocker
   when time-to-diagnosis drives MTTR directly.

2. **The method is a research prototype.** It is a
   Python pipeline over `causal-learn` plus an external clique-picking package. It
   offers no compositional, type-safe surface for assembling a discovery workflow
   (load, clean, learn structure, rank, report), and no path to reproducible,
   dependency-free, inudstry grade production deployment.


## 3. Core thesis: the follow-up paper

> **The failure-period cost of BRCD is not the configuration enumeration the worst
> case suggests. It is a single dominant configuration per candidate.** Under the
> same KL separation that drives BRCD's own posterior-consistency guarantee, the
> per-candidate configuration posterior concentrates, so scoring a small frontier
> (in the detectable-anomaly regime, the single MAP I-CPDAG) reproduces the exact
> candidate ranking. This turns BRCD's `Σ_V 2^{du(V)}` worst case into near-linear
> failure-period work with no loss of ranking accuracy, under a concentration
> guarantee that is a corollary of the original Theorem 4.4. We deliver the result
> as a deployable instrument: a compositional, type-safe Causal Discovery Language
> (CDL) with a dependency-free, type-safe implementation.

Two coupled contributions:

- **C1 (method): MAP-configuration BRCD.** Replace per-candidate enumeration of
  all valid cut configurations with evaluation of a small, cheaply-found frontier
  (top-1 in the detectable regime). Prove, as a configuration-level corollary of
  Lemma 4.1 and Theorem 4.4, that under `Δ_min > 0` the omitted configurations
  carry vanishing posterior mass with high probability, so the candidate ranking
  is preserved. The accuracy/compute trade-off degrades gracefully exactly where
  full BRCD is itself unreliable, that is, on sub-detection-threshold anomalies.

- **C2 (system): the dual-purpose CDL.** A type-state pipeline language over an
  effect monad (error short-circuit plus warning accumulation) that composes the
  discovery workflow with compile-time stage-ordering guarantees, hosting BRCD and
  SURD as peer sub-pipelines on a native, `unsafe`-free, deterministic engine. The
  contribution is that one pipeline serves both phases of the BRCD cost split.
  Offline, during normal operation, it runs with no CPDAG: it learns the structure
  with BOSS and persists the result. Online, during a failure, it runs with that
  CPDAG supplied: it skips structure learning and pays only the ranking cost, the
  work that governs time-to-diagnosis. The same code path, configured by whether a
  CPDAG is present, is the offline analytics tool and the production RCA service.
  This requires one small change to the CDL, which today learns the CPDAG and
  discards it; persisting it closes the loop (see
  `openspec/notes/cdl-cpdag-cache.md`). With that change the offline/online split
  the paper only describes becomes an executable, measurable property of the
  system: a cold run (learn plus rank) against a warm run (load plus rank) on
  identical data isolates the structure-learning cost the production path avoids.

## 4. Findings that substantiate the thesis

All counts are implementation-independent (configurations, likelihood
evaluations, posterior mass), not timings, so they survive review regardless of
language.

- **F1. The worst case is real only on dense undirected structure, and absent on
  the real data.** (`brcd_config_census`) All four committed microservice CPDAGs
  are fully directed: `du = 0`, so `Σ 2^{du} = n`, and the configuration
  exponential never fires on the deployed-style benchmarks. The `2^{du}` cost is a
  property of undirected (synthetic or weakly identified) structure, which is
  where C1 earns its keep (the Fig 2b regime). The consequence is direct: "lower
  the worst case" is the wrong claim, and "the failure-period work is dominated by
  one configuration" is the right one.

- **F2. Validity already collapses the easy cases; cliques are the only tight
  worst case.** (`brcd_config_census`) For a star neighborhood the
  no-unshielded-collider rule collapses `2^{du}` valid configurations to `du + 1`
  exactly. For a path it stays constant. Only a clique neighborhood realizes the
  full `2^{du}`. Formally, the valid-configuration count equals the number of
  cliques in the mutual-adjacency graph of the candidate's undirected neighbors,
  a clean supporting lemma.

- **F3. No exact sub-exponential summation exists (a dead end we close
  honestly).** (`brcd_factorization_probe`) On the clique worst case the log-weight
  `w(b) = log L_b + log Q_b` carries dense, high-order interactions. Order-≥4
  energy rises from 9% to 43%, and the effective interaction order climbs from 3
  to 5, as `du` grows from 3 to 6. So `exp(w)` does not contract in
  `exp(treewidth)`, and a junction-tree or tensor-train DP buys nothing. The
  coupling comes from the global Meek closure and the MEC-size weight. Exact
  acceleration is impossible, which is precisely what motivates an approximate
  (pruned) method.

- **F4. The posterior concentrates on one configuration, and top-1 preserves the
  ranking.** (`brcd_topk_pruning`) On the clique worst case, at any detectable
  anomaly strength, the participation ratio is 1.00, and a single configuration
  captures at least 99.9% of each candidate's mass (`k* = 1`) for every size up to
  `2^{du} = 64`. Scoring only that MAP configuration reproduces the full candidate
  ranking exactly: the true root cause stays on top, and the full order is
  identical. At sub-detection anomaly strength the posterior is diffuse
  (`k* ≈ 2^{du}`) and the ranking degrades, but that is the regime where full BRCD
  is already untrustworthy, so nothing operable is lost. This is the empirical
  backbone of C1, and it ties the achievable budget to `Δ_min`.

- **F5. Amortization is structural (supporting).** (`brcd_config_census`) A
  family-level cache makes the failure-period cost a single base pass plus one
  `F`-augmented family per candidate. On a 45-variable directed CPDAG, 2070 naive
  family evaluations collapse to 91 unique ones, roughly 22×. Failure-period work
  is `O(n)` family evaluations on directed CPDAGs, independent of density.

- **F6. A cheap `O(du)` finder reproduces the full ranking without enumeration
  (the load-bearing test, passed).** (`brcd_heuristic_mapconfig`) On realistic
  asymmetric CPDAGs (a random DAG through `dag_to_cpdag`), a greedy finder matches
  the full-enumeration top-1 candidate on 100% of trials. On a planted-clique
  stress test where `2^{du}` reaches 256, a one-pass greedy search (`du + 1`
  evaluations) and a hill-climb (about `1.5·du`) both reproduce the oracle top-1
  on 100% of trials, against `2^{du}` for full enumeration. The ranking holds even
  though the finder does not always locate the exact MAP configuration: the
  one-pass exact-MAP rate falls to 84% at `du = 8`, yet its top-1 ranking stays at
  100%. The candidate posterior is concentrated enough (F4) that any near-MAP
  configuration yields the candidate's rank, so the method is forgiving of the
  search heuristic. This is the central robustness property, and it lowers the
  algorithmic risk: the contribution rests on the ranking being stable under
  near-optimal configs, not on a search reaching a global optimum.

  F6 evidence (`brcd_heuristic_mapconfig`, linear-Gaussian, single root cause,
  H1 = one-pass greedy, H2 = hill-climb; "top-1" = heuristic top candidate equals
  the full-enumeration oracle's; "MAP-hit" = the heuristic found the exact best
  config; "evals" = configuration evaluations per candidate):

  | regime        | du   | 2^du | oracle→rc | H1 top-1 | H2 top-1 | H1 MAP-hit | H2 MAP-hit | evals H1 / H2 |
  |---------------|------|------|-----------|----------|----------|------------|------------|---------------|
  | random CPDAG  | ~1   | 2.2  | 99–100%   | 100%     | 100%     | 89–94%     | 98%        | 1.7 / 2.0     |
  | clique        | 5    | 32   | 100%      | 100%     | 100%     | 91%        | 98%        | 6.0 / 7.8     |
  | clique        | 7    | 128  | 100%      | 100%     | 100%     | 86%        | 96%        | 8.0 / 12.2    |
  | clique        | 8    | 256  | 100%      | 100%     | 100%     | 84%        | 95%        | 9.0 / 14.1    |

  Top-1 ranking stays at 100% across the sweep while the evaluation budget grows
  linearly (H1 = `du + 1`) against the exponential `2^{du}`; the MAP-hit column
  drops without disturbing the ranking, which is the robustness point above. The
  clique stress used a strong anomaly (perturbation 4.0); the random-CPDAG block
  covers a weak anomaly (1.0) as well.

**Open algorithmic gap: resolved by F6.** The earlier concern was that F4 only
validated the target with an oracle MAP config, leaving open whether the
configuration can be found cheaply. F6 closes it: a greedy `O(du)` search
reproduces the full ranking, and it does not even need the exact MAP. Two
hardening steps remain for submission, as coverage rather than open risk: a
weak-anomaly clique sweep, and the full Fig-2b protocol at `n` up to 1000 on the
real datasets.

## 5. How C1 (method) and C2 (CDL) work together

- **The CDL is what makes the failure-period argument operable.**
  C1's value proposition is time-to-diagnosis during an incident. That claim is
  only credible if the ranker runs in-process, deterministically, with no cluster.
  C2 provides exactly that. The same CDL pipeline realizes the split C1 formalizes,
  selected by whether a CPDAG is present: with none, it learns the structure
  offline and persists it; with the persisted CPDAG supplied, it ranks online and
  pays only the cheap step. The split is a property of how the pipeline is run, not
  a claim about its types; persisting the learned CPDAG is the one change that
  makes it real (see `openspec/notes/cdl-cpdag-cache.md`).

- **C1 is what gives the CDL a reason to exist beyond engineering.** A DSL alone is
  a systems artifact. Hosting a method with a new
  approximation guarantee, pruned BRCD with preserved ranking, becomes the method
  through which a correct-by-construction, compute-bounded discovery pipeline is assembled and run.

- **Framing for the venue.** Lead with C1, the statistical and algorithmic result
  with its concentration corollary, as the contribution that clears the ML bar.
  Present C2 as the realization that makes the result deployable and reproducible,
  and that generalizes beyond BRCD. 

- **One-sentence synthesis.** We show that BRCD's failure-period cost collapses to
  a single dominant configuration per candidate, provably, under its own
  separation condition, and we deliver this as a compositional, dependency-free
  Causal Discovery Language that turns a cluster-scale research
  prototype into a fast, reproducible, in-process diagnostic instrument.

## 6. Deployment assumptions
 
BRCD needs two inputs at ranking time: a normal pool and an anomalous pool over the same
variables, tagged by the indicator `F` (`F = 0` normal, `F = 1` anomalous).

**"Aligned" is schema alignment, not row pairing.** BRCD treats the two pools as
i.i.d. samples (the paper assumes i.i.d.), concatenates them, and appends the `F`
column. Alignment means the same metric columns and a correct pre/post label, not
synchronized or paired rows. This lowers the online bar: no matched sampling is
required, only a shared schema and a known failure boundary.

**BRCD is localization downstream of detection, not a standalone monitor.** It
presumes the failure-onset time is known so it can form `F` (the paper notes the
onset time is "usually available"). It does not detect the failure, and it does
not need the trigger-point metric, which is a point in its favor against methods
like ST and IDI. The deployed role is incident-triggered: a detector fires,
supplies the time boundary, and BRCD ranks.

For the online diagnostic to be real, all of the following must hold true:

1. **A detector supplies the onset time.** Online RCA is detector → window → BRCD.
   Every comparable method sits downstream of an anomaly signal; BARO, for
   instance, supplies its own change-point detection.
2. **A regime-matched normal baseline.** The `F = 0` pool must reflect the
   system's pre-fault state under comparable load and time-of-day, not a stale
   training window.
3. **Anomalous samples from the post-onset window.** Automatic during an incident,
   provided the failing service keeps emitting metrics.
4. **Schema alignment and missingness handling.** Same metric set; impute or drop
   missing and zero-variance metrics, as the paper does. A service that stops
   emitting during a fault is a real online wrinkle.
5. **Distributional invariance modulo the fault.** The two windows must differ
   only because of the fault. A concurrent deploy, an autoscaling event, a traffic
   spike, or seasonality confounds the `F` contrast and localizes the confound,
   not the root cause. This is the assumption most at risk in production, mitigated
   by baseline matching and by excluding windows with known concurrent changes.
6. **A CPDAG valid for the current regime.** The offline structure must still
   describe current normal behavior; regime drift forces a re-learn, which is the
   invalidation criterion for the persisted CPDAG.

Two degenerate cases must be stated openly. A brand-new service or a never-seen
load regime has no matched baseline, and a fault coincident with another change
violates invariance. In both, the diagnostic degrades, and BRCD's own consistency
guarantee weakens with it, since the KL separation `Δ_min` shrinks when the
contrast is confounded.

## 7. What still must be done before this is a submission

1. ~~**Heuristic MAP-config finder versus oracle.**~~ Done (F6): a greedy `O(du)`
   finder recovers the full ranking without enumeration, and does not need the
   exact MAP. Remaining hardening: a weak-anomaly clique sweep.
2. **Validation on the synthetic Fig-2b regime** at `n` up to 1000 through the
   real ranker (not the small `n = 10` probe), to demonstrate the speed-up where
   `du > 0` bites at scale.
3. **The concentration corollary**, written formally as an extension of Theorem
   4.4: top-`k` mass at least `1 − ε` implies the ranking is preserved with high
   probability.
4. **Accuracy/compute trade-off curves.** Top-`l` accuracy and configurations
   scored against budget, on Petshop, OB, Sockshop, and synthetic data,
   reproducing the original accuracy at a fraction of the configuration work.
5. **Honest scope statement.** On directed service-map CPDAGs the config pruning is
   a no-op (`du = 0`). The win is on undirected-heavy and large synthetic graphs,
   and the amortized failure-period `O(n)` result carries the directed case.

## 8. Next steps

1. Verify the the weak-anomaly clique sweep (the one hardening step F6 left open)
2. Implement the proposed Heuristic MAP as a secondary implementation.
3. Compare both methods via appropriate benchmarks.
4. Decide work division for the write-up.
5. Write the paper.


