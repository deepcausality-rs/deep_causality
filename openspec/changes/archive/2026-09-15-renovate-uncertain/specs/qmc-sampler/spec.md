<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## MODIFIED Requirements

### Requirement: QmcSampler as an alternative Sampler
`deep_causality_uncertain` SHALL provide a `QmcSampler` implementing `Sampler<R>` for every scalar the crate's blanket bound admits, alongside the existing `SequentialSampler`. `QmcSampler` SHALL evaluate an `Uncertain<R>` computation graph by drawing each stochastic leaf through the inverse-CDF transforms (never through rejection sampling), using one coordinate of a Sobol point per leaf. All deterministic node kinds (arithmetic, comparison, logical, function, negation) SHALL be evaluated identically to `SequentialSampler`. `SequentialSampler` SHALL remain the default.

The bound was "every supported `ProbabilisticType`", a trait implemented for three named types.
That trait is removed and the bound is now the blanket scalar bound, so the QMC sampler serves every
scalar the crate serves — including ones it does not name.

The fmap and apply node kinds leave the list of deterministic kinds because the arms are removed;
no builder ever produced them. QMC's guard against data-dependent structure is unchanged.

`SequentialSampler` remains the default. The previous clause "seeded MC bit-output SHALL be
unchanged" is dropped: leaf draws are re-addressed by seed, index and leaf ordinal, so a seeded
Monte-Carlo sequence recorded before this change does not reproduce after it. That is stated as
breaking rather than silently violated.

#### Scenario: QMC reduces error at equal sample count
- **WHEN** the mean of a low-dimension static `Uncertain<R>` integrand is estimated with `QmcSampler` and with `SequentialSampler` at the same sample count `N`
- **THEN** the QMC estimate's error against the analytic value is no larger than the MC estimate's, and converges at a faster rate as `N` grows

#### Scenario: Deterministic node handling matches the MC sampler
- **WHEN** a tree mixing distributions with arithmetic, comparison, and logical operators is evaluated under QMC
- **THEN** the deterministic operators produce the same composition of leaf values as `SequentialSampler` would for those same leaf values

#### Scenario: QMC serves a scalar the crate does not name
- **WHEN** a `QmcSampler` is built over an `Uncertain<R>` at a scalar named nowhere in the crate
- **THEN** it compiles and draws, with no implementation added for that scalar

## REMOVED Requirements

### Requirement: Sampler-discriminated sample cache
**Reason**: There is no cache to key. The global sample cache is removed, and with it the
`SamplerKind` discriminant that existed only to keep Monte-Carlo and Quasi-Monte-Carlo entries
apart inside it. A `SampleSession` is Monte-Carlo or Quasi-Monte-Carlo by construction, so two
sampling strategies cannot share an address in the first place and there is nothing to cross-serve.

**Migration**: Code holding a `SamplerKind` drops it. Code that relied on a Monte-Carlo and a QMC
draw at the same index being distinct gets that property from the session: construct one session
with `SampleSession::seeded` and one with `SampleSession::qmc`, and draw from each. Code that
relied on a second call at the same index returning the stored value gets it from the addressing
scheme instead, which is stronger — it holds across roots that share a leaf, where the cache held
only per root.
