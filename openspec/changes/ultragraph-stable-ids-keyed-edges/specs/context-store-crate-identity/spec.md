## ADDED Requirements

### Requirement: The crate depends on ultragraph_traits alone
`deep_causality_context_store` SHALL declare exactly one dependency, `ultragraph_traits`, in `[dependencies]`, and no build dependency. Dev-dependencies are permitted only where a test needs one.

`ultragraph_traits` declares no dependency itself, so a backend that links the store crate links two crates and nothing else. The dependency exists so that `RelationKind` can implement `EdgeKey` where it is declared.

#### Scenario: The manifest declares one dependency
- **WHEN** the `[dependencies]` table of `deep_causality_context_store/Cargo.toml` is read
- **THEN** it lists `ultragraph_traits` and nothing else

#### Scenario: The tier block records the position
- **WHEN** the workspace dependency tiers are re-derived from the manifests
- **THEN** `ultragraph_traits` appears in Tier 0 and `deep_causality_context_store` in Tier 1 of the `AGENTS.md` tier block, the root crate list and crate count in `AGENTS.md` include both, and `deep_causality_context`'s entry lists the store crate as a direct dependency

### Requirement: RelationKind is an edge key and ordered
`RelationKind` SHALL implement `EdgeKey` with `Key = RelationKind`, returning itself, and SHALL derive `PartialOrd` and `Ord`, ordering its variants by declaration: `Datial < Temporal < Spatial < SpaceTemporal`.

#### Scenario: The key is the relation
- **WHEN** `edge_key` is called on `RelationKind::Spatial`
- **THEN** it returns `RelationKind::Spatial`

#### Scenario: The order is declaration order
- **WHEN** the four variants are sorted
- **THEN** the order is `Datial`, `Temporal`, `Spatial`, `SpaceTemporal`

## REMOVED Requirements

### Requirement: The crate has no dependencies
**Reason**: `RelationKind` implements `EdgeKey` from `ultragraph_traits` in the crate that declares it, which the orphan rule requires and which needs the dependency.
**Migration**: The requirement "The crate depends on ultragraph_traits alone" replaces it; the crate moves from Tier 0 to Tier 1.
