# brcd-algorithm Specification

## Purpose
TBD - created by archiving change brcd-estimator. Update Purpose after archive.
## Requirements
### Requirement: BRCD root-cause ranking from two datasets and a CPDAG

The system SHALL provide a BRCD entry point in `deep_causality_algorithms::causal_discovery::brcd` that takes a normal dataset, an anomalous dataset (both `CausalTensor<T>`, `T: RealField`, aligned columns), and a CPDAG (`MixedGraph`), and returns a posterior distribution over root-cause candidates ranked by probability. The entry point SHALL require a supplied CPDAG and SHALL NOT perform structure learning.

#### Scenario: Single-root continuous case is ranked

- **WHEN** BRCD runs on aligned normal/anomalous tensors and a supplied CPDAG with one true injected root cause
- **THEN** it returns a normalized posterior over the candidate variables and the true root cause is ranked first

#### Scenario: A CPDAG is required

- **WHEN** BRCD is called without a CPDAG
- **THEN** it returns an error indicating a CPDAG is required (no structure learning is performed)

#### Scenario: Misaligned datasets are rejected

- **WHEN** the normal and anomalous tensors do not share the same columns
- **THEN** BRCD returns an error rather than producing a ranking

### Requirement: F-node augmentation and cut-configuration enumeration

BRCD SHALL form a joint frame by concatenating the normal and anomalous datasets with an F-node indicator column (0 on normal rows, 1 on anomalous rows), and SHALL enumerate the orientations of the undirected edges incident on the root-candidate set. Each candidate orientation SHALL be validated by Meek completion, acyclicity, and the no-new-unshielded-collider check before it contributes to the posterior.

#### Scenario: Invalid orientations are excluded

- **WHEN** a candidate orientation introduces a cycle or a new unshielded collider at a target
- **THEN** that orientation is discarded and does not contribute to any root's likelihood

#### Scenario: Arcs-only CPDAG yields a single configuration

- **WHEN** the supplied CPDAG is already fully directed
- **THEN** enumeration produces exactly the one configuration and BRCD scores it directly

### Requirement: Plug-in ridge-Gaussian continuous estimator

For continuous variables BRCD SHALL score each family `(node, parents)` with a plug-in ridge-Gaussian: ridge least squares `β = solve(XᵀX + λI, Xᵀy)` with `λ = 1e-4`, residual variance `σ²` floored to `1e-12`, and per-row log-density equal to `deep_causality_tensor::gaussian_log_density`. The estimator SHALL support an optional monotone transform (none, log, log1p) with the Jacobian applied on the original scale and an auto-downgrade ladder when a transform is invalid for the data.

#### Scenario: Family log-density matches the plug-in Gaussian

- **WHEN** a family is fit on given rows
- **THEN** each row's contribution equals the plug-in Gaussian log-density at the ridge-fitted mean and floored variance, including the transform Jacobian when a transform is active

#### Scenario: Invalid transform auto-downgrades

- **WHEN** a configured transform (e.g. log) is invalid for the data (e.g. non-positive values)
- **THEN** the estimator downgrades along the ladder rather than producing NaN, and records which transform was used

### Requirement: Mixture-of-experts F-integration with a logistic gate

BRCD SHALL integrate the F-node in three modes per family: when F is a parent, fit a separate ridge-Gaussian per regime; when F is present but not a parent, combine two regime experts through a logistic-regression gate `π(F=1 | X)`; when F is absent, use a single expert. The logistic gate SHALL be an in-repo, `RealField`-generic, deterministic implementation with an empirical-prior fallback for a degenerate gate. Mixture combination SHALL use `deep_causality_tensor::logsumexp`.

#### Scenario: F-as-parent fits per regime

- **WHEN** F is a parent of the family's node
- **THEN** the family is scored by a separate ridge-Gaussian fit on the F=0 and F=1 rows respectively

#### Scenario: F-not-parent mixes through the gate

- **WHEN** F is present but not a parent of the family's node
- **THEN** the two regime experts are combined through the logistic gate probability and the log-densities are mixed via logsumexp

#### Scenario: Degenerate gate falls back to the empirical prior

- **WHEN** the logistic gate cannot be fit (singular/degenerate)
- **THEN** the gate probability falls back to the empirical F prior and scoring proceeds

### Requirement: Discrete Dirichlet estimator

For discrete variables BRCD SHALL score families with a Dirichlet posterior-predictive (prequential) estimator using concentration `α* = 5.0`.

#### Scenario: Discrete family is scored prequentially

- **WHEN** a family over discrete variables is scored
- **THEN** each row's contribution is the Dirichlet posterior-predictive probability with `α* = 5.0`

### Requirement: Exact MEC sizing and uniform sampling

BRCD SHALL compute the exact Markov-equivalence-class size of each validated configuration and SHALL draw a uniform representative DAG, via in-repo AMO enumeration (no external dependency). The configuration's log-weight SHALL be `log(mec_size / Σ)` over the enumerated class. The enumeration SHALL be bounded and SHALL report an explicit error or log when a configuration exceeds the bound, never silently truncating.

#### Scenario: Undirected CPDAG is weighted by class size

- **WHEN** a validated configuration has undirected edges
- **THEN** its contribution is weighted by its exact equivalence-class size and one representative DAG is sampled uniformly with the seeded RNG

#### Scenario: Enumeration bound is explicit

- **WHEN** a configuration's equivalence class exceeds the enumeration bound
- **THEN** BRCD surfaces an explicit error or log rather than silently scoring a truncated class

### Requirement: Posterior assembly, caching, and ranking

BRCD SHALL assemble the posterior by caching each unique family's per-row log-likelihood once, summing cached log-factors into `log P(D | G)` per DAG, adding the MEC log-weight, combining a root's DAGs via `logsumexp`, summing over rows, adding the log-prior, and normalizing into a posterior over roots. The result SHALL be deterministic given the configured seed.

#### Scenario: Repeated families are computed once

- **WHEN** two DAGs share a family `(node, parents)`
- **THEN** that family's per-row log-likelihood is computed once and reused

#### Scenario: Result is deterministic given a seed

- **WHEN** BRCD runs twice with the same inputs and the same seed
- **THEN** the posterior and ranking are identical

