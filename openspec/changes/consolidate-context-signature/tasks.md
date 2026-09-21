## 1. Baseline and dependencies

- [ ] 1.1 Re-derive the impl census before changing anything, and record it: how many types
      implement `Coordinate`, `Spatial`, `Temporal`, `SpaceTemporal` and `Metric`, and at which
      value types. The change rests on no type implementing one of them twice, so confirm that
      still holds
- [ ] 1.2 Count `VS`/`VT` occurrences per crate as the before-number for task 4.5
- [ ] 1.3 Add `deep_causality_algebra` to `deep_causality_context/Cargo.toml` for the real-field
      bound, and confirm the tier order still places it below the context crate
- [ ] 1.4 Add `deep_causality_metric` for the signature enum, and confirm the same
- [ ] 1.5 Verify `deep_causality_num` is reachable for whatever the bound needs; add it only if the
      algebra bound does not already carry it

## 2. Associated types on the traits

- [ ] 2.1 `Coordinate` gains `type Coord`; every method that returned `V` returns `Self::Coord`
- [ ] 2.2 `Temporal` gains `type TimeUnit`; `time_unit` returns `Self::TimeUnit`
- [ ] 2.3 `Spatial` becomes `Identifiable + Coordinate` with no parameter
- [ ] 2.4 `SpaceTemporal` becomes `Identifiable + Spatial + Temporal`, with `t` returning
      `&Self::TimeUnit`
- [ ] 2.5 Rename `Metric<V>` to `Distance` with `fn distance(&self, other: &Self) -> Self::Coord`,
      and update its six implementors
- [ ] 2.6 Remove `MetricCoordinate`. Confirm first that it still has zero implementors and is named
      as a bound nowhere; if that changed, stop and re-decide
- [ ] 2.7 `cargo test -p deep_causality_context` green, still at `f64` throughout

## 3. The scalar parameter

- [ ] 3.1 Give each spatial node type a scalar parameter bounded once on the algebra real-field
      bound, and set `type Coord` to it
- [ ] 3.2 Same for the spacetime node types
- [ ] 3.3 Same for the real-valued temporal types; leave the integer-valued ones on their integer
      representations, which encode a tick rather than a precision
- [ ] 3.4 Propagate the parameter through the `Kind` enums
- [ ] 3.5 Remove every `f64` and `f32` from `deep_causality_context/src` outside the ready-made
      aliases, which deliberately fix a working scalar
- [ ] 3.6 Keep `BaseContext`, `BaseContextoid`, `UniformContext` and `UniformContextoid` concrete so
      alias-based call sites need no annotation they did not need before
- [ ] 3.7 Add a test building a context at the smallest shipped real field and another at the
      largest, reading coordinates back from both. This is the acceptance criterion for the whole
      group
- [ ] 3.8 `cargo test -p deep_causality_context` green

## 3b. Identifiers and the edge weight

- [ ] 3b.1 Move `ContextId` and `ContextoidId` out of `deep_causality_core` into
      `deep_causality_context`, with a docstring stating that the width is this crate's decision.
      A move, not a copy: core keeps no context identifier alias
- [ ] 3b.2 Route the 49 `id:` struct fields through them
- [ ] 3b.3 Route the 57 function signatures through them
- [ ] 3b.4 Confirm no raw unsigned integer remains in an identifier position in
      `deep_causality_context/src`
- [ ] 3b.5 Add `Default` to `RelationKind` on the `Datial` variant, documented as the graph bound's
      filler rather than a meaningful relation. A data relation is the least specific of the four,
      so its accidental appearance claims least
- [ ] 3b.6 Change the backing graph's weight to `RelationKind` and delete the `weight as u64` cast
- [ ] 3b.7 Add a test asserting an edge's relation is recoverable after storage, which it is not
      today
- [ ] 3b.8 Re-point `deep_causality`'s four source files and three test files that import the two
      from core, and drop them from its root re-export list — they are context items now, and the
      no-re-export contract already covers those
- [ ] 3b.9 Confirm each of the two has exactly one declaration site in the workspace
- [ ] 3b.10 Note that `TeloidTag` and `TeloidID` remain the same defect in core, unmoved here
      because this change does not touch ethos's vocabulary
- [ ] 3b.11 `bazel test //...` green

## 4. Six parameters to four

- [ ] 4.1 Drop `VS`/`VT` from `Context`, `Contextoid` and `ContextoidType`
- [ ] 4.2 Drop them from `Contextuable`, `ContextuableGraph` and `ExtendableContextuableGraph`
- [ ] 4.3 Fix `deep_causality`: `Model`, the generative interpreter, and the `Base*`/`Uniform*`
      aliases
- [ ] 4.4 Fix `deep_causality_ethos`: `Teloid`, `TeloidStore` and `EffectEthos` thread these
      parameters through their own signatures
- [ ] 4.5 Fix the example packages, then confirm `VS`/`VT` appear nowhere as a generic parameter and
      report the count against task 1.2
- [ ] 4.6 `bazel test //...` green. **Land this before starting group 6** — four parameters is
      independently releasable, and keeping the two failure modes apart is the point of the sequence

## 5. The coordinate families

- [ ] 5.1 Merge the two field-identical Cartesian spacetimes, keeping **`LorentzianSpacetime`** as
      the surviving name and its constructors. `Lorentzian` names the signature class and
      `Minkowski` names one flat member of it, so the general name survives a merge and the frame's
      constant carries the convention the two names used to carry
- [ ] 5.2 Delete the `SpaceTimeKind::Minkowski` arm and update every consumer of the removed name
- [ ] 5.3 Remove `QuaternionSpace` **and** the `SpaceKind::Quaternion` arm, not only the `Spatial`
      impl. A quaternion is a rotation operator acting on a space rather than a space of its own,
      and an orientation belongs beside a position in a pose. Removing the trait alone leaves the
      type reachable as a space through the enum, and the frame spec makes `SpaceKind` the ordinary
      frame member — so the defect would survive exactly where most frames meet it
- [ ] 5.4 The removal deletes 12 files under `types/context_node_types/space/quaternion_space/` and
      its test directory. **Deletion permission granted by the user, 2026-09-21**, so the golden
      rule's ask is already answered for these files and no further prompt is needed. Confirm the
      count against the tree first; verified self-contained, with no consumer in `deep_causality`,
      `deep_causality_ethos` or the examples
- [ ] 5.5 Add the vertical datum to the geodetic spatial type as a field, and remove the claim from
      the altitude field's doc comment that fixed one reference for the whole type. If it is an
      enum, start from the five ISO 19111-aligned members `WGS84`, `EGM96`, `EGM2008`, `ISA` and
      `Terrain`, each classified by the height or datum type it belongs under. `ISA` and `Terrain`
      are the two that catch people out: a pressure altitude is not a height at all, and a terrain
      height depends on which model produced it
- [ ] 5.6 Add a test asserting two geodetic positions with equal altitudes against different datums
      are distinguishable
- [ ] 5.7 Document in the crate which spatial types pair with a spacetime and which require a
      conversion first, so the gap is recorded rather than discovered. Note that the geodetic type
      also carries a datum the Cartesian types do not, so a conversion out of it loses it
- [ ] 5.8 Leave every `TimeScale` variant spelled exactly as it is, including the inconsistent
      `Nanoseconds` and `Microseconds` beside `Millisecond` and `Second`. The names are a contract a
      projection matches on, so tidying one is a data migration elsewhere rather than a rename here.
      If they should be tidied, that is its own decision and not a silent one
- [ ] 5.9 `bazel test //...` green

## 6. The frame

- [ ] 6.1 Add `NoSpaceTime`, a zero-sized type implementing the spatial and spacetime traits
      trivially, with its own tests
- [ ] 6.2 Add `ContextFrame` with `Space`, `Time` and `SpaceTime` associated types
- [ ] 6.3 Add `SIGNATURE` as an associated constant of type `deep_causality_metric::Metric`
- [ ] 6.4 Add `MetricFamily` as an enum in `deep_causality_metric`, naming the analytic forms with
      their parameters, and `metric_family` returning `None` where the metric has no closed form.
      An enum keeps that crate closed and checkable and matches what the physics crate computes
- [ ] 6.5 Provide frames for the shipped configurations: one over the variant `Kind` enums as the
      ordinary case, one fixed-type frame as the specialisation, and one with no spatial part that
      names `NoSpaceTime`. Document which to reach for when the world is not known at compile time
- [ ] 6.5b Add a test holding two spacetime contextoids of different concrete variants in one
      context on a variant frame, which is what a regime-changing model needs
- [ ] 6.6 Add a test asserting a frame's signature is readable without constructing a context, and
      one asserting two frames with different conventions are distinguishable through the metric
      crate's existing convention checks
- [ ] 6.7 `cargo test -p deep_causality_context` green

## 7. Four parameters to two

- [ ] 7.1 Put the frame on both `Contextoid` and `Context`: `Contextoid` holds its variant payloads
      so it must name their types, and it takes them from the frame's associated types
- [ ] 7.2 Reduce `Contextoid` and `ContextoidType` to the data type and the frame
- [ ] 7.3 Reduce `Context` and the three graph traits
- [ ] 7.4 Rewrite the ready-made aliases against frames
- [ ] 7.5 Fix `deep_causality`, `deep_causality_ethos` and the examples again
- [ ] 7.6 `bazel test //...` green

## 8. Verify

- [ ] 8.1 `make format && make fix` — fix clippy findings by rewriting, not by `#[allow]`
- [ ] 8.2 `bazel test //...` green across the workspace
- [ ] 8.3 Assert no context type names a concrete float outside the aliases
- [ ] 8.4 Assert `Contextoid` takes two parameters and `deep_causality_context::Metric` does not
      resolve
- [ ] 8.5 Confirm `cargo build -p deep_causality_core --no-default-features --features no-std` still
      succeeds
- [ ] 8.6 Re-derive the tier blocks in `AGENTS.md` from the manifests, since the context crate
      gained two dependencies
- [ ] 8.7 Regenerate the SBOM for every crate whose dependencies changed
- [ ] 8.8 Run every example and confirm output is unchanged
- [ ] 8.9 Confirm the crate's identifier width can be changed at its one declaration and the crate
      still builds, which is the acceptance criterion for group 3b
- [ ] 8.10 Sanity-check that a physics example could now be migrated to a typed context: pick the
      one that switches regime and confirm a variant frame expresses it. Do not migrate it here;
      that is its own change

## 9. Release

- [ ] 9.1 Leave versions to release-plz; the `BREAKING CHANGE:` footers drive the bump. Do not edit
      a `version =` line and do not hand-write a changelog
- [ ] 9.2 Draft the migration note under `docs/drafts/`: the parameter reduction, the scalar
      parameter with a worked example, the identifier aliases, the `Distance` rename, the removed
      marker, the merged spacetime, the quaternion correction and the edge-weight change
- [ ] 9.3 Put the breaking-change detail in the commit messages, one footer per breaking item
- [ ] 9.4 Prepare the commit messages and hand them to the user to commit
