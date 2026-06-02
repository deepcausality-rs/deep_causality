## ADDED Requirements

### Requirement: Golden fixtures reproduce the reference posterior

The change SHALL include golden test fixtures — small `(normal, anomalous, CPDAG)` cases with the posterior over root causes captured from the authoritative `ctx/next/brcd/brcd.py` on a fixed seed — and SHALL assert that the Rust BRCD produces the **same ranking** and per-root log-posteriors within a pinned tolerance `ε`. The fixtures SHALL cover every estimator mode: F-as-parent, F-not-parent mixture, F-absent, continuous and discrete, and a CPDAG carrying undirected edges (exercising MEC weighting).

#### Scenario: Ranking matches the reference exactly

- **WHEN** the Rust BRCD runs on a golden fixture
- **THEN** the ordering of root-cause candidates is identical to the captured reference ranking

#### Scenario: Log-posteriors match within tolerance

- **WHEN** the Rust BRCD runs on a golden fixture
- **THEN** each per-root log-posterior is within the pinned tolerance `ε` of the captured reference value

#### Scenario: Every estimator mode is covered

- **WHEN** the golden fixture suite runs
- **THEN** it includes at least one case each for F-as-parent, F-not-parent mixture, F-absent, a discrete-variable case, and an undirected-edge CPDAG

#### Scenario: The X→Y→Z toy reproduces the author's ranking

- **WHEN** BRCD runs on the committed `df_obs`/`df_a` CSV inputs of the author's `X → Y → Z` toy (undirected CPDAG `edges=[(X,Y),(Y,Z)]`, `node_transform="none"`, anomaly perturbing `p(Y|X)`)
- **THEN** the returned `ranks` equal `['Y','X','Z']`

### Requirement: Synthetic ground-truth root-cause recovery

The change SHALL include a self-contained synthetic experiment — data generated in-repo with a known injected root cause under a known graph and a fixed seed, mirroring the reference `experiments/synthetic` generator — and SHALL assert that BRCD ranks the true root cause first (single-root) and within top-k (multi-root).

#### Scenario: True root is recovered top-1

- **WHEN** BRCD runs on seeded synthetic data with a single known injected root cause
- **THEN** the true root cause is ranked first

#### Scenario: Multiple roots are recovered within top-k

- **WHEN** BRCD runs on seeded synthetic data with a known set of injected root causes
- **THEN** every true root cause appears within the top-k ranked candidates

### Requirement: Deterministic, dependency-free verification

The verification suite SHALL be deterministic (fixed seeds, no live data download, no external numeric crates) and SHALL run as part of the crate's standard test suite. Reference oracle outputs SHALL be committed golden data captured offline, not regenerated at test time.

#### Scenario: Suite is reproducible offline

- **WHEN** the verification suite runs on a clean checkout with no network access
- **THEN** it completes using only in-repo synthetic generation and committed golden fixtures, with identical results across runs
