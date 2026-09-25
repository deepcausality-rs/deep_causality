## MODIFIED Requirements

### Requirement: Parallelism is an opt-in parameter threaded through the solver bounds
The crate SHALL carry an opt-in `parallel` feature that forwards to `deep_causality_unified_math/deep_causality_topology/parallel`
and `deep_causality_par/parallel`, and fans out itself only through `deep_causality_par`. The
`CfdScalar` bound and the theory/solver trait bounds SHALL include `MaybeParallel`, so the inner
topology operator loops fan out under the feature with no solver code change. Serial execution SHALL
be the default. The crate SHALL parallelize at a single granularity per run — coarse-grained over
independent cases (counterfactual branches, ensembles, sweeps) or fine-grained per cell — and SHALL
NOT nest the two, to avoid oversubscription.

#### Scenario: The feature is off by default and results are unchanged
- **WHEN** the crate is built without the `parallel` feature
- **THEN** the solver runs serially and reproduces the same results as a parallel build to tolerance

#### Scenario: Independent cases fan out under the feature
- **WHEN** several independent counterfactual branches or ensemble members run under `--features parallel`
- **THEN** they execute concurrently and each result matches its serial counterpart
