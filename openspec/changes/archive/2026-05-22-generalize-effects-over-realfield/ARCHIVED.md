# Archived: not implemented — crate retired

**Archived on:** 2026-05-22
**Status:** Abandoned. Specs were never implemented.

## Reason

During feasibility assessment of this change set, an audit of the
workspace revealed that `deep_causality_effects` had **zero downstream
consumers**:

- No other workspace crate listed it as a dependency in any
  `Cargo.toml` or `BUILD.bazel`.
- Neither `deep_causality` nor `deep_causality_ethos` consumed
  `EffectData` or `NumericValue`.
- The "temporary `::<f64>` pins" the proposal claimed were installed by
  R0 (`generalize-topology-over-realfield`) and
  `generalize-physics-over-realfield` did **not exist** in the tree
  (verified by workspace-wide grep for the tag
  `// TEMP: removed by generalize-effects-over-realfield` — zero hits).

The crate's functionality (heterogeneous `EffectData` containers as a
unified payload type for causal graphs) had been superseded by the
propagating-effect model inside `deep_causality`'s effect propagation
process. The decision to fold effect representation into the
propagation process — rather than keep it in a dedicated container
crate — predated this proposal but was not surfaced in the change
set's framing.

Given the crate was unused and its design intent had been overtaken
by a different architectural choice, generalizing it over
`R: RealField` would have been work in service of a hypothetical
consumer that never materialized.

## Resolution

The crate was **retired** rather than generalized:

- `deep_causality_effects/src/` reduced to an empty stub library.
- `deep_causality_effects/README.md` rewritten as a deprecation
  notice marking the crate name as effectively free for reuse.
- Published versions on crates.io (0.0.1 – 0.0.7) remain available
  for any existing external dependents. Version 0.0.8 was published
  as the empty-stub deprecation marker.
- The ethos `README` line claiming independence from
  `deep_causality_effects` was removed (it was no longer informative
  once the crate carried no functionality).

The specs in this proposal were never written; only `proposal.md` and
`design.md` exist. They are preserved here as a record of the design
exploration and the audit findings that led to the retirement decision.
