## Why

CDL ships one discovery algorithm, SURD. Issue [#598](https://github.com/deepcausality-rs/deep_causality/issues/598) asks for BRCD (Bayesian Root Cause Discovery): given a *normal* and an *anomalous* dataset over the same variables and a causal graph (CPDAG), rank the variables by their posterior probability of being the root cause of the regime change. The shared foundations are now in place — the archived `real-field-discovery`, `mixed-graph`, and `brcd-prep-foundations` changes deliver `RealField` genericity, the typed-endpoint `MixedGraph` (CPDAG), the Tier A numeric primitives (`gaussian_log_density`, `logsumexp`, `cg_solve`, sample covariance), and the Tier B causal-graph operations (Meek completion, unshielded-collider validity, trivial-case MEC sizing) rooted at `deep_causality_algorithms::causal_discovery::brcd`.

This change implements the **BRCD estimator itself** as a composition over those foundations, plus the two genuinely new components the foundations deliberately deferred, and — critically — a **verification suite that reproduces the reference algorithm's behaviour** against the authoritative implementation and its synthetic ground-truth experiment. The authoritative source is `ctx/next/brcd/` (per the algorithm's author; the experiment-repo copies are non-authoritative). The full reconciliation is `openspec/notes/rca/BRCD.md` §16.

The deliverable is the BRCD algorithm as a standalone, verified `deep_causality_algorithms` function operating on two tensors + a `MixedGraph` → a ranked posterior over root causes. Wiring it into the CDL typestate pipeline is a separate, later change (`cdl-discovery-pipeline`, held as `openspec/notes/cdl-integration.md`) so the algorithm is verified before the public-API seam is generalized around it.

## What Changes

All new code is rooted at `deep_causality_algorithms::causal_discovery::brcd`, beside the already-landed `meek`, `validity`, `mec` modules.

- **Plug-in ridge-Gaussian estimator** (`continuous`): per family `(node, parents)`, fit ridge least squares `β = solve(XᵀX + λI, Xᵀy)` (λ = 1e-4) via a direct SPD solve / `cg_solve`, residual variance `σ²` floored to 1e-12, then per-row log-density via the Tier A `gaussian_log_density` (byte-for-byte `_normal_logpdf_1d`). Optional monotone transform (none/log/log1p/yeojohnson) with the Jacobian on the original scale, auto-downgrading when the data invalidate a transform.
- **Mixture-of-experts F-integration** with a **logistic-regression gate** — the new numeric primitive. Three modes per family: F ∈ parents → a separate ridge-Gaussian per regime; F present ∉ parents → a two-expert mixture combined through a logistic gate `π(F=1 | X)` (empirical-prior fallback); F absent → a single expert. Mixtures combine via `logsumexp`.
- **Discrete estimator** (`discrete`): Dirichlet posterior-predictive (prequential), `α* = 5.0`.
- **Full uniform MEC sampler** — the second new component. Port the Wienöbst clique-picking `mec_size` + uniform `sample_dag` (or the `mcs_num` AMO/clique-tree enumeration) so configurations on the OB / Sock Shop path (BOSS/bootstrap CPDAGs carry undirected edges) are weighted and sampled correctly. This replaces the trivial arcs-only placeholder shipped by `brcd-prep-foundations` (MEC `RequiresUniformSampler` path).
- **F-node augmentation + cut-configuration enumeration**: concatenate normal + anomalous into a joint frame with an `FNODE` indicator (0/1); enumerate the `2^E` orientations of undirected edges incident on the root-candidate set (`getConfigurations_multi`), validating each by Meek completion + acyclicity + the no-new-unshielded-collider check (the landed `meek`/`validity` ops).
- **Posterior assembly + ranking** (`brcd_update` / `brcd_helper`): `sampleAugmentedGraphs` → cache each unique family's per-row log-likelihood once → per root, sum cached log-factors into `log P(D | G)`, add `log(mec_size / Σ)`, `logsumexp` over the root's DAGs, sum over rows, add `log(prior)`, normalize → posterior over roots → rank. The driver takes a supplied CPDAG directly; the no-CPDAG bootstrap-via-BOSS path is out of scope (below).
- **Verification suite** reproducing the reference behaviour at three tiers (golden unit fixtures, synthetic ground-truth recovery, authoritative-oracle cross-check). See the `brcd-verification` capability.

## Capabilities

### New Capabilities
- `brcd-algorithm`: the BRCD estimator — F-node augmentation, the plug-in ridge-Gaussian / mixture-of-experts / Dirichlet families, the logistic gate, the full uniform MEC sampler, cut-configuration enumeration with validity, and posterior ranking — as a standalone `deep_causality_algorithms::causal_discovery::brcd` function over two `CausalTensor`s + a `MixedGraph`, generic over `T: RealField`.
- `brcd-verification`: a tiered verification suite proving the Rust BRCD reproduces the authoritative algorithm — golden fixtures with reference posteriors, synthetic ground-truth root-cause recovery (top-k), and a cross-check against `ctx/next/brcd/brcd.py` on fixed seeds.

### Modified Capabilities
<!-- None. The `causal-graph` capability's MEC requirement is satisfied by an additive uniform-sampler implementation behind the existing API; no requirement text changes. -->

## Impact

- **Crates touched.** `deep_causality_algorithms` only (new modules under `causal_discovery::brcd`; the logistic gate and clique-picking sampler live here too unless a reusable home is justified during design). Consumes the already-public Tier A primitives (`deep_causality_tensor`, `deep_causality_sparse`) and `deep_causality_topology::MixedGraph` — no new cross-crate dependencies beyond those already added by `brcd-prep-foundations`.
- **Public API.** New `pub` BRCD entry point(s) in `deep_causality_algorithms`. No breaking change. The CDL pipeline (`deep_causality_discovery`) is untouched — that integration is the separate `cdl-discovery-pipeline` change.
- **Dependencies.** None added. No external numeric crates; repo-wide `unsafe_code = "forbid"` preserved; static dispatch (no `dyn`).
- **Out of scope (separate concerns).** (1) CDL pipeline integration — `cdl-discovery-pipeline`. (2) The no-CPDAG **BOSS** structure-learning bootstrap path (`BRCD/boss.py`) — BRCD here requires a supplied CPDAG; CPDAG discovery is upstream. (3) The Forest-KDE nonparametric family and the microservice call-graph adapter — the Gaussian path is the real-world target.
- **Verification data.** The synthetic generator and authoritative driver are at `ctx/next/brcd/`; real-world OB / Sock Shop datasets are downloaded by the reference harness (not bundled), so the oracle cross-check captures Python BRCD outputs as committed golden files on fixed seeds rather than re-downloading at test time.
