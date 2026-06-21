# BHPI Groundwork — Causal Hypergraph Learning for DeepCausality

Status: scoping note (pre-proposal). Date: 2026-06-20.
Reference source: `openspec/notes/causal_bhpi/BHPI-main/` (MATLAB).

## 1. Goal & framing

The objective is **not** a faithful port of BHPI. It is the first **causal hypergraph
learning algorithm that fits the Causaloid's recursive isomorphic structure** and that
produces a genuinely *causal* m-causes → n-effects mapping with overlapping relations.

BHPI ("Disentangling Latent Risk Pathways via Bayesian Hypergraph Inference", Ding et al.,
ICML 2026) is the closest existing structural template found to date — its overlapping
hyperedges, where a hyperedge ≅ a Causaloid Collection, are the natural fit. BHPI as
published is **associational** (a structured `P(Y|X)`); the causal content is a layer we add.

Endgame: integration into the Causal Discovery Language (`deep_causality_discovery`, CDL).

Distinction from existing discovery algorithms in the repo:
- **SURD** decomposes the joint influence of *known* sources on *one* target into
  redundant/unique/synergistic information. n→1, observed variables.
- **BRCD** learns an *oriented* causal graph over *observed* variables and ranks root causes.
  Pairwise edges + colliders.
- **This (CHL)** learns a *latent, overlapping* hypergraph: drivers → latent pathways →
  outcomes, m→n, with calibrated structural uncertainty and rare-outcome pooling. Neither
  SURD nor BRCD does this. The three are complementary legs of a diagnostic stack.

## 2. Reference source facts (established by reading the MATLAB)

Files under `BHPI-main/`: `simulate_design.m` (driver), `BHPI.m` (fit driver),
`helper/BHPI_single_iter_w_baseline.m` (the 7-block CAVI sweep), `…_wrapper.m`
(Robbins–Monro + ELBO), `helper/E_Overlap.m`, the data-gen chain
(`simu_data_gen` + `simulate_mixed_hypergraph` + `simulate_mechanisms` +
`compute_beta` + `calibrate_intercept`), `cavi_initialization.m` (NNMF), `repulsion_strength.m`.

Model: `y_{i,v} ~ Bernoulli(σ(α_v + xᵢᵀβ_v))`, `β_{j,v} = d⁻¹ Σ_e (z_e ρ_{v,e})(γ_{j,e} μ_{j,e})`.
Inference: structured CAVI with Polya–Gamma augmentation. Pinned details:

- **Update order (fixed coordinate-ascent; must preserve):** α → μ → γ → m → z → ω → hypers.
- **7-block sweep maps 1:1 onto the paper's Eqs 9–15 + A.4.2:**
  | Block | Updates | Paper eq |
  |---|---|---|
  | 1 | `q(α_v)` baseline | 10 |
  | 2 | `q(μ\|γ)` (`tau`, `B`) — **hotspot**, `O(P·E·N·V)` leave-one-out residual | 11–12 |
  | 3 | `q(γ\|z)` → ν | 13 |
  | 4 | `q(m)` → ρ (cross terms via `Xi_Xj`) | 14 |
  | 5 | `q(z)` → r (**carries structured-VI KL terms** `kl_z_rho/nu`) | 15 |
  | 6 | `q(ω)` PG: `E_Ω = tanh(η/2)/(2η)`, `η=√(max(E[η̃²],eps))` | 9 |
  | 7 | `σ²_μ, ν, ρ, r` Betas/IG via `psi` | A.4.2 |
- **Robbins–Monro damping wraps every step (CRITICAL).** The wrapper computes new values then
  blends each of ~22 returned quantities: `x ← (1−ρ_t)x_old + ρ_t x_new`, `ρ_t=(iter+t0)^(-0.3)`,
  `t0=10`. The equations in the sweep are the *target* of a damped step, not the state update.
  Miss this and the trajectory diverges on iteration 1. Full-batch in the default run
  (`batch_size=0`); the damping is always on (SVI machinery in full-batch mode).
- **Priors all Jeffreys** `Beta(1/2,1/2)`; `sigma2_alpha=10`; slab init var 100.
- **Convergence:** `max|Δβ| < tol  AND  Δexpected_log_lik < tol`, `tol=1e-4`, `max_iter=2000`.
  NOT ELBO-based (ELBO tracked for logging only).
- `dv_inv = 1/sqrt(E_hat)` is a **scalar**, not per-disease.
- **Special functions:** inner loop needs **`digamma` (psi) only**; ELBO additionally needs
  `gammaln` (via `log(beta(a,b))`), and ELBO is deferrable → `gammaln` deferrable.
- **`E_Overlap.E_O_m_diff` is `O(E²·V²)` as written**; rank-1 reduces it to `O(E²·V)` but
  perturbs the low bits → keep literal form for golden-match, optimize for production.
- **Data generation and NNMF init are NOT reproducible in Rust** (MATLAB MT RNG, `randperm`,
  `fzero`, `binornd`, `nnmf` ALS ×200). Export them as fixtures; do not regenerate.
- **`cavi_initialization` has a `"true"` path** (init from padded ground-truth) — deterministic,
  no RNG/NNMF. This is the key to deterministic CAVI verification (see §7).
- Staged warm-up (`staged`) is OFF in the default run → implement only Stage 3 for v1.

## 3. Capability match (num / tensor / haft / uncertain)

| Crate | Provides | Gap |
|---|---|---|
| `deep_causality_num` | `Float` (exp/ln/tanh/sqrt/powf) on f64/f32/**Float106**; `erf` (Float106) | **`digamma` absent** (required); `lgamma`/`gamma` absent (ELBO only) |
| `deep_causality_tensor` | N-D `CausalTensor` (row-major, strided); `matmul`, `permute_axes`, `sum_axes`/`mean_axes`, NumPy-style broadcasting, einsum AST, SVD/Cholesky; `ext_stats` (logsumexp, gaussian_log_density, sample_covariance) | no public row/col view, no range-slice (perf) |
| `deep_causality_haft` | HKT (GAT witness), Functor/Monad/Applicative/Foldable/Traversable, `morphism_endo::iterate_to_fixpoint` (bounded), NaturalIso, Arrow/Morphism; tensor/sparse/**topology(GraphWitness)**/core implement the FP traits | **no Fix/Algebra/Coalgebra recursion scheme** |
| `deep_causality_uncertain` | `Uncertain<T>` (point/normal/uniform/bernoulli), **`MaybeUncertain::from_bernoulli_and_uncertain` = exact spike-and-slab**, MC+QMC samplers, SPRT collapse, `expected_value`/`std` | no Beta/Gamma (see §10 — out of scope) |

**Layering rule (do not violate):** the solver is deterministic closed-form moments
(`tensor` + `num`); `uncertain` is the *output-representation* layer only. Running CAVI
through `Uncertain` (sampling) would forfeit the closed forms and the `O(N·E·(P+V))` speed.

## 4. Infrastructure changes required

### 4.1 `digamma` in `deep_causality_num`
- New module `src/special_functions/` (mirror `float_106/erf.rs` structure, A&S citations).
- `digamma(x)` generic over `Real` (asymptotic series + recurrence), unit-tested to ULP
  against known values and against MATLAB `psi`. ~30–60 LOC + tests.
- `lgamma` deferred (only needed if/when ELBO parity is wanted).
- Workspace lint: new module inherits crate lints; no new exemption.

### 4.2 Strided views in `deep_causality_tensor`
- Extend `tensor_view`: `col(j) -> view`, `row(i) -> view`, `slice_range(axis, start, end) -> view`.
- Views, not copies (crate already stores strides). Row is contiguous; column is strided.
- Justification: the CAVI leave-one-out residuals (block 2) are column-indexed work that does
  not express cleanly as a single `matmul`. Route N-heavy contractions through `matmul`/`contract`;
  use views for the structure-indexed loops.

### 4.3 Recursion schemes in `deep_causality_haft` (+ Causaloid rework, DEFERRED)
- The Causaloid's three isomorphic forms are the fixpoint of a base functor:
  `CausaloidF<X> = Singleton(CausalFn) | Collection(Vec<X>, AggLogic) | Graph(HyperGraph<X>)`,
  `Causaloid ≅ Fix<CausaloidF>`.
- Effect propagation (evaluation) is a **catamorphism** `cata(eval_algebra)` — the universal
  property makes it the *unique* fold, i.e. a determinism/verifiability guarantee, not decoration.
- Structure learning (the learner emitting a hypergraph) is an **anamorphism** `ana(build)`.
  Build-then-evaluate fuses to a **hylomorphism** with a proven fusion law.
- HAFT already has the prerequisites (HKT witness + Functor). Add `src/recursion/`:
  `Fix<F>` (Box-backed; `dyn`-free, within policy), `Algebra<F,A> = F::Type<A>→A`,
  `Coalgebra<F,A> = A→F::Type<A>`, `cata`/`ana`/`hylo`. Bounded depth (fits `morphism_endo` ethos).
- **Blast-radius decision:** the legacy `Causaloid` is the load-bearing core; migrating it is a
  separate, deliberate effort — NOT part of this project. Sequencing:
  1. Add `recursion/` to HAFT as an isolated greenfield module (no existing-code change).
  2. Make the **learner the first client**: emit its hypergraph via `ana → Fix<CausaloidF>`,
     evaluate via `cata`. Greenfield, low blast radius.
  3. Migrate the legacy Causaloid later, once the machinery is proven.
- Rationale recorded for the record: the 2021 Causaloid design used recursive FP data
  structures as "closest next best." Recursion schemes are the categorical name for exactly
  that intuition — this *rigorizes and validates* the original design, it does not replace it.

## 5. The learner architecture (built from DeepCausality's own parts)

The learner is expressed in DeepCausality's causal monad, not bolted on. Pattern lifted from
`examples/causal_correction_examples/corrective_ddos_detector` (the stateful loop) and
`examples/causal_counterfactual_examples/counterfactual_treatment_effect` (the intervene-compare):

```
CausalFlow::from(initial_process())
    .iterate_n(MAX_ITER, |it| it.bind(cavi_sweep)
                                .update_state(|s,_| robbins_monro_blend(s_old, s_new)))
    .into_process()
```

| CAVI concept | DeepCausality mechanism (exists) |
|---|---|
| variational state `{r, ρ, ν, μ, α, hypers}` | the threaded **`State`** of `PropagatingProcess` |
| data `X, Y` + hyperparameters | read-only **`Context`** |
| one CAVI sweep (Eqs 9–15) | the `.bind(cavi_sweep)` closure (numerics in tensor+num) |
| Robbins–Monro damped update | **`.update_state(blend)`** |
| iterate to fixpoint | **`iterate_n`** / an `iterate_until(Δβ<tol)` — the HAFT `morphism_endo` shape |
| per-iteration ELBO / Δβ / convergence | the **`EffectLog`** → the *learning itself is auditable* |
| interventional / invariance environments | **`.intervene(...)` / `.alternate_context(env)`** + compare |

The CAVI fixpoint IS the corrective-control loop; the invariance test IS the counterfactual
comparison; the numerics live inside the bind closures; the output is `ana → Fix<CausaloidF>`.

## 6. Making it causal (the contribution, additive to BHPI)

BHPI's latent space is associational (a structured `P(Y|X)` factorization). Calibrated
uncertainty about an associational object stays associational; UQ does not change the rung.
To make the m→n overlapping mapping causal:

- **Multi-environment invariance in the objective.** Add an environment index; require a
  hyperedge's induced effect to be **invariant across environments/interventions** (ICP-style);
  keep only stable edges. A second prior alongside repulsion: repulsion buys *structural*
  identifiability, invariance buys *causal* identifiability. This operationalizes the EPP's
  mechanism-autonomy/stability postulate (a pathway is causal iff its effect survives
  perturbation of the others).
- **Interventional data where available** (fault injection, A/B, feature flags, simulators)
  makes edges causal by construction. This is the RCAEval/Sock-Shop validation path
  (`examples/causal_discovery_examples/data/sock-shop-2/`).
- Surviving edges are *candidate mechanisms* → Causaloids the EPP can `intervene` on meaningfully.

Open research question (the thing that makes this novel rather than another structured
regression): whether the invariance objective reliably promotes associational edges to stable
mechanisms on real multi-environment data.

Trap to avoid (recorded): toggling an input of a model fit to *observational* data is **ablation**,
not intervention. The toggle must be in the world (or a faithful simulator), or the structure
must prove invariant across genuine environments.

## 7. Verification strategy (golden fixtures)

The correctness oracle is the paper's **simulation** (public; ground truth `H/β/γ`), not UK
Biobank (access-gated) and not RCAEval (which carries root-cause-node ground truth, the wrong
shape — that validates BRCD, and serves here only as the *causal/interventional* testbed).

Fixture set, exported from MATLAB once:
1. `{X, Y, α, Beta, H, γ, μ}` from `simu_data_gen` (frozen artifact; do not regenerate in Rust).
2. The `"true"`-init `initials` struct → deterministic CAVI start (no NNMF/RNG to reproduce).
3. Per-iteration intermediates (`r, ρ, ν, μ, α, E_Ω`) after iters 1, 5, 20.

Acceptance: feed identical data + `"true"` init; **replicate the Robbins–Monro schedule
exactly**; diff each of the 7 blocks against its MATLAB intermediate (a divergence localizes to
one equation). Judge by tolerance-level structure/metric agreement (identical PIP>0.5 structure,
H-AUC/γ-AUC/cor(β,β̂) within tolerance), not bit-identity — transcendentals + summation order
differ across MATLAB/Rust. Test `digamma` to ULP first; it is the fidelity hinge.

## 8. CDL integration

`deep_causality_discovery` already exposes `CdlConfigBuilder → CdlBuilder.build_brcd()
.brcd_load_input().brcd_discover()`. Mirror it: `build_chl(config) → chl_load_input(csv/parquet)
→ chl_discover()` returning (a) the posterior hypergraph and (b) a ready-to-compose Causaloid
Collection/Graph (`ana → Fix<CausaloidF>`). Engine lives in
`deep_causality_algorithms/causal_discovery/chl/` (beside `brcd`, `surd`); CDL is the DSL skin.
Same shape as the `ml_rca` detect→explain chain, extended to detect → **group** → orient.

## 9. Sequencing

1. `digamma` in `num` (§4.1) + strided tensor views (§4.2).
2. `recursion/` module in HAFT (§4.3 steps 1–2), greenfield.
3. Learner as a `PropagatingProcess` fixpoint loop (§5), numerics in the bind closures,
   emitting `Fix<CausaloidF>`. Verify against golden fixtures (§7).
4. Invariance/intervention causal layer via `intervene`/`alternate_context` (§6).
5. CDL `chl_*` surface (§8).
6. Deferred: legacy-Causaloid migration; ELBO + `gammaln`; staged warm-up; batch/SVI.

## 10. Out of scope / decided against

- **Beta/Gamma distributions in `deep_causality_uncertain`.** Considered and rejected. The
  solver computes closed-form Beta/Gamma *moments* (via `digamma`) and never samples them; the
  exposed output marginals are Bernoulli PIPs (`MaybeUncertain`) and Gaussian effects
  (`Uncertain::normal`). Adding Beta/Gamma adds nothing to either layer. (The only case it would
  serve is second-order uncertainty about a PIP, and even then the minimal move is a
  hierarchical Bernoulli with an `Uncertain<f64>` presence parameter — not a new distribution
  family. Not pursued.)
- **Faithful BHPI replication as an end in itself.** The MATLAB is a template and a verification
  oracle, not the deliverable.
- **Legacy Causaloid migration to `Fix<CausaloidF>`.** Real and desirable, but a separate
  large-blast-radius effort; this project only introduces the machinery at the new edge.
