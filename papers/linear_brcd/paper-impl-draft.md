# Near-Linear BRCD — Methodology & Implementation Differences

Author: Marvin Hansen

Status: Draft 

Sccope: methodology and implementation differences 

Companion: `paper-thesis.md`.

> Scope of this document: it records *only* how the method and its implementation
> differ from the original BRCD — in particular, how failure-period cost is reduced
> from worst-case exponential to near-linear, conceptually and theoretically first,
> then in code. It is not a full paper draft.

---

## 0. Provenance

Original method (the algorithm the deep_causality `brcd` module implements):

> Kenneth Lee, Zihan Zhou, Murat Kocaoglu. *Root Cause Analysis of Failures in
> Microservices via Bayesian Root Cause Discovery.* ICML 2026, PMLR 306.
> Code: https://github.com/kenneth-lee-ch/brcd

Two building blocks reimplemented in-tree (algorithm theirs; in-tree implementation
and BRCD adaptation ours):

> M. Wienöbst, M. Bannach, M. Liśkiewicz. *Polynomial-Time Algorithms for Counting
> and Sampling Markov Equivalent DAGs.* AAAI 2021 (arXiv:2012.09679).
>
> M. Wienöbst, M. Bannach, M. Liśkiewicz. *Polynomial-Time Algorithms for Counting
> and Sampling Markov Equivalent DAGs with Applications.* JMLR 24(213):1–45, 2023.

Original BRCD reaches this counting/sampling through the external `cliquepicking`
package. We reimplement it for performance reasons

---

## 1. Methodology difference: exponential → near-linear

### 1.1 Two exponentials as worst case 

BRCD's failure-period cost has two independent exponential factors:

- **(A) Configuration enumeration `Σ_V 2^{du(V)}`** — the per-candidate loop over all
  cut configurations of the candidate's undirected neighborhood (`du` = incident
  undirected edges). This is the worst case the original paper names (Appendix E).
- **(B) Per-configuration MEC sizing/sampling** — computing the I-MEC size `Q_i` and
  a representative DAG. In the *original paper* this is already polynomial (Wienöbst
  et al.). In the prior in-tree port it was an **exact AMO enumeration**, which is
  factorial on dense chordal residuals and was hard-capped (`MEC_ENUM_BOUND`).

This work removes both: 
(B) → polynomial (Section 2.1), 
(A) → near-linear

(Section 2.2). As a result, the worst-case complexity reduces to polynomial.

### 1.2 Conceptual foundation (the core thesis)

The expensive per-candidate quantity is `p(D | R) ∝ Σ_b L_b Q_b` over the `2^{du}`
cut orientations `b`. The thesis is that **this sum is dominated by a single
configuration per candidate whenever the anomaly is detectable** (the only regime in
which BRCD is itself trustworthy). The failure edge `F → R` makes the data strongly
prefer one orientation of `R`'s neighborhood; the other configurations carry
vanishing posterior mass. Exhaustive integration over all I-CPDAGs is, in practice,
an integral against a near-degenerate distribution. Hence ranking candidates by their
single dominant configuration reproduces the ranking by the full sum, and the
`2^{du}` enumeration can be replaced by *finding* that configuration — an `O(du)`
search — with no change to the decision.

### 1.3 Theoretical foundation

The concentration is driven by the same quantity that drives BRCD's published
guarantees: the strictly positive KL separation
`Δ_min = inf_{(G,R)≠(G⋆,R⋆)} E_{p⋆}[log p(X|G⋆,R⋆)/p(X|G,R)] > 0`.
Theorem 4.4 bounds the posterior mass on each wrong `(G,R)` by a term exponentially
small in `n·(Δ_min − 2Bε)`. The non-MAP configurations of the **true** candidate
`R⋆` are wrong-`G`/right-`R⋆` pairs, so their mass is bounded by that same
exponential. Thus **configuration concentration for the true candidate is a corollary
of Theorem 4.4** — the omitted configurations decay at the very rate that makes BRCD
consistent.

Two scope statements (carried honestly):
1. **Estimand change.** Pruning replaces a marginal `Σ_b` (model averaging over
   I-CPDAGs) with a frontier/MAP estimate; these coincide *in ranking* under
   concentration, and the approximation degrades only in the sub-detection regime
   where full BRCD is already unreliable.
2. **True candidate vs. whole ranking.** Theorem 4.4 gives concentration for `R⋆`;
   preserving the entire candidate order additionally requires the per-candidate
   frontier estimate to rank wrong candidates consistently with the marginal. This
   holds with high fidelity empirically (Section 2.2); the formal extension to all
   candidates is the remaining theoretical item (Section 4).

### 1.4 The two reductions composed

```
                         original BRCD          prior in-tree port      this work
per-config MEC sizing    poly (Wienöbst, ext)   factorial (enumerate)   poly (in-tree Wienöbst)
config enumeration       Σ_V 2^{du}             Σ_V 2^{du}              O(du) greedy MAP finder
```

Per candidate: `2^{du} × (poly|factorial)` → `O(du) × poly`. For a single root cause
the candidate set is `O(n)`, so failure-period work becomes `O(n · du · poly(n,N))`
— polynomial, near-linear in `n` for bounded undirected degree. No exponential
remains in either factor.

---

## 2. Implementation difference

### 2.1 In-tree polynomial MEC counter + sampler (reimplementing Wienöbst)

New module `deep_causality_algorithms::dag_sampling`, a faithful translation of the
Wienöbst reference (clique tree, `ρ` inclusion–exclusion recurrence, memoized count
over flowers/separators, count-guided sampling), ported file-for-file: `chordal.rs`
(`mcs`), `clique_tree.rs`, `combinatorics.rs` (`factorial`, `ρ`), `count.rs`
(`count_amos`/`count_chordal`), `sample.rs`, plus `index_set`/`lazy_tokens`/
`memoization`/`graph`/`utils`. Public API: `count_amos`, `count_chordal`, `mec_size`
(over a `MixedGraph`'s undirected chordal components), `sample_dag` (uniform member),
`representative_dag` (deterministic member).

Adaptations from the reference (the code-level differences):
- **`BigUint` → generic `T: RealField + FromPrimitive`.** Counts are the same numeric
  type BRCD uses everywhere; instantiates at `f64` and `Float106`; no `num-bigint`.
  BRCD only uses ratios `Q_i/T`, so this is exact for realistic sizes and overflow-
  free in the log-space posterior.
- **`ρ` kept in linear space** — it subtracts terms, so log-space is invalid.
- **`Option<T>` memo sentinel** instead of `BigUint::ZERO`-as-uncomputed.
- **Vose alias table → exact inverse-CDF** weighted selection (no `gen_biguint_below`
  analogue under generic `T`); probabilities exactly proportional to AMO sub-counts.
- **`SliceRandom::shuffle` → dependency-free Fisher–Yates** over a generic
  `R: Rng`, with the caller's RNG threaded through (the reference quietly uses its own
  `thread_rng`; the port is deterministic under seeding).
- **`representative_dag` via MCS perfect-elimination order**, not raw vertex index
  (orienting by index can create a spurious collider, e.g. path `0−2−1 → 0→2←1`).

Correctness basis: validated against the retained *exact enumeration* counter
(`brcd_mec::mec_size`, an independent algorithm) — anchors (54, 108, K4=24, K5=120)
and 2000 random chordal graphs, zero mismatches; sampler validity / full-support /
chi-square uniformity; `Float106` instantiation. The enumeration counter is kept as
the test oracle and must never be the thing under test (else validation is circular).

### 2.2 MAP-configuration pruning (the near-linear step; beyond the paper)

New module `brcd_mapconfig` + a `ConfigStrategy ∈ {Full (default), MapPrune}` switch
on `BrcdConfig`. `MapPrune` replaces `get_configurations_multi`'s `2^{du}`
enumeration with `find_map_configs`: a greedy hill-climb over single-edge flips
(valid-start → improving flips until none), scoring each visited orientation by the
**production** weight `w(b) = Σ_node logL(node | parents in the representative DAG) +
ln(mec_size(b))` so the finder's ranking is consistent with BRCD Phases 2/3.
Cost: `O(du)` (`≤ du²+2du+2`) config evaluations vs `2^{du}`. For `du = 0` it returns
the single configuration, so `MapPrune ≡ Full` on directed CPDAGs.

Differences in behavior/scope:
- **Default is `Full`** (exact enumeration) — existing behavior, tests, and the
  reference rankings are unchanged; near-linear is opt-in.
- **No degree ceiling on the pruned path.** Since the finder never materializes the
  `2^{du}` space (it manipulates a `du`-bit label), it is bounded only by the `usize`
  width (`MAX_MAPPRUNE_EDGES = 62`), not the full path's `MAX_CONFIG_EDGES = 16`.
- Fidelity (measured): `du = 0` exact to `1e-9`; realistic detectable-anomaly CPDAGs
  → top-1 100%, full-order Kendall-τ ≈ 0.997; pathological cliques → top-1 100%, tail
  τ ≈ 0.76 (the expected frontier approximation). Detail belongs to the open
  Evaluation section.

### 2.3 Production integration (ranking-preserving)

`brcd_algo` (the `brcd_run` loop) and `brcd_boss_bootstrap` call
`dag_sampling::mec_size` / `sample_dag` instead of the capped enumeration; the
enumeration functions are retained untouched as the dag_sampling oracle. This swap is
**exact and ranking-preserving** — it changes only *how* the MEC size and a
representative DAG are computed, not *what* is scored: the MEC size is exact and the
sampled member's likelihood is Markov-equivalence-invariant, so which member is drawn
cannot change a candidate's score. It is independent of, and composes with, the
opt-in MAP-pruning of §2.2 (the only source of approximation).

Two distinct consequences, not to be conflated:
- **Same rankings, reproduced exactly.** On the captured reference (supplied-CPDAG,
  `du = 0`) the full ranking reproduces position-for-position: Online Boutique 45/45
  (both cases), Sock Shop 44/45.
- **Newly tractable, still exact.** The learned-CPDAG path (`brcd_run(None)` → BOSS →
  undirected hubs) that the prior factorial MEC step could not finish now completes —
  with the **default `Full`** strategy, i.e. the exact marginal. "Completes" here is a
  tractability win from the polynomial counter/sampler, **not** an approximation; the
  ranking is the exact BRCD marginal (the fault is recovered at rank 1). The
  tail-reordering caveat belongs only to opt-in `MapPrune` (§2.2, §2.5), not here.

### 2.4 Learn-once CPDAG cache (CDL layer)

`deep_causality_discovery` gains an opt-in `cpdag_cache_path` that persists the
BOSS-learned CPDAG keyed to its input (FNV-1a over the normal tensor + BOSS seed,
sidecar key; supplied → keyed-cache → learn resolution; stale/missing/corrupt →
re-learn + overwrite, never a silently-wrong graph; learn uses the identical BOSS
config as `brcd_run`, so warm == cold). This operationalizes BRCD's
offline-learn / online-rank split, which the original describes but the prior pipeline
discarded (it re-learned every run).

### 2.5 What is reconstructed, by configuration × scenario

The **reference value** is the original paper's captured Python BRCD ranking where one
exists — the supplied-CPDAG Online Boutique / Sock Shop cases (`expected.txt`). For
`du > 0` graphs (learned or synthetic) no external number exists, so the reference is
**full-enumeration BRCD** (the exact marginal `Σ_b L_b Q_b`), and `MapPrune` is
measured against it.

| Configuration | `du = 0` — directed / service-map CPDAG (supplied) | `du > 0` — realistic learned / asymmetric, detectable anomaly | `du > 0` — pathological symmetric clique |
|---|---|---|---|
| **`Full`** (default, exact) | **Exact** vs the Python reference — OB 45/45 ×2, SS 44/45, position-for-position | Exact BRCD marginal (now tractable; no external oracle for a learned CPDAG) | Exact BRCD marginal |
| **`MapPrune`** (opt-in, `O(du)`) | **Exact** — one valid config, identical to `Full`/the Python reference (== `Full` to 1e-9) | **top-1 100%**; full order Kendall-τ ≈ 0.997 vs `Full` | **top-1 100%**; full order τ ≈ 0.76 vs `Full` (tail reorders) |

Read across: the published reference rankings are reproduced **exactly by both
configurations** — the real datasets are `du = 0`, where pruning is a no-op. The only
approximation lives in `MapPrune` on `du > 0` graphs, and there it preserves the
decision (top-1) always and the whole order almost exactly, fraying only on
adversarial symmetric structure.

### 2.6 Three-way comparison: reconstruction and runtime complexity

Notation: `du` = a candidate's undirected degree, `dmax = max_V du(V)`, `n` =
variables, `N` = samples; single-root-cause case.

| Aspect | Original paper (Python + `cliquepicking`) | Previous Rust port | New Rust port |
|---|---|---|---|
| MEC count / sample | poly (Wienöbst, external pkg) | exact AMO enumeration — **factorial**, capped at 100k | in-tree poly Wienöbst, generic `T`, **uncapped** |
| Cut-config handling / candidate | enumerate all `2^{du}` | enumerate all `2^{du}` | `Full`: `2^{du}` (default) · `MapPrune`: `O(du)` greedy finder |
| Config evaluations / candidate | `2^{du}` | `2^{du}` | `Full`: `2^{du}` · `MapPrune`: `du + 1` (= `log₂` of the `Full` space) |
| Failure-period complexity (k=1) | `O(n · 2^{dmax} · poly)` — exp. in degree | `O(n · 2^{dmax} · factorial)` — two exponentials; often does **not** complete | `Full`: `O(n · 2^{dmax} · poly)` · `MapPrune`: `O(n · dmax · poly)` — **near-linear** |
| Reconstruction vs Python ref (`du = 0`, OB/SS) | the reference | exact *when it completes* | **exact** (both `Full` and `MapPrune`): OB 45/45 ×2, SS 44/45 |
| Reconstruction on dense / learned (`du > 0`) | exact marginal | factorial MEC → cap/abort (e.g. learned OB did not finish) | `Full`: exact marginal (completes) · `MapPrune`: top-1 100%, τ ≈ 0.997 realistic / 0.76 clique |
| Degree ceiling | none | `MEC_ENUM_BOUND` + `2^16` config cap | none on `MapPrune` (only `usize` width, 62) |
| Numeric type | `BigUint` | `usize` (capped) | generic `RealField` — `f64` / `Float106` |
| Dependencies | causal-learn + cliquepicking + bignum | none | none, `unsafe`-free |

**Headline.** For the *same* rankings — reproduced exactly on the published
benchmarks, and top-1-identical with ≈ 0.997 full-order fidelity on realistic
`du > 0` graphs — the per-candidate configuration work drops from `2^{du}`
evaluations to `du + 1`, the **binary logarithm of the space it used to enumerate**.
In absolute terms the configuration factor goes from exponential to *linear* in the
degree, and the failure-period cost from exponential to *near-linear* in `n`. That is
the practical win: order-of-magnitude-or-more fewer evaluations for a ranking that is
exact where it can be checked and decision-identical where it cannot.

---

## 3. References

1. K. Lee, Z. Zhou, M. Kocaoglu. *Root Cause Analysis of Failures in Microservices
   via Bayesian Root Cause Discovery.* ICML 2026, PMLR 306.
2. M. Wienöbst, M. Bannach, M. Liśkiewicz. *Polynomial-Time Algorithms for Counting
   and Sampling Markov Equivalent DAGs.* AAAI 2021. arXiv:2012.09679.
3. M. Wienöbst, M. Bannach, M. Liśkiewicz. *Polynomial-Time Algorithms for Counting
   and Sampling Markov Equivalent DAGs with Applications.* JMLR 24(213):1–45, 2023.
4. C. Meek. *Causal inference and causal explanation with background knowledge.* UAI 1995.

