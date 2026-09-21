## 1. Baseline and dependencies

- [x] 1.1 Re-derive the impl census before changing anything, and record it: how many types
      implement `Coordinate`, `Spatial`, `Temporal`, `SpaceTemporal` and `Metric`, and at which
      value types. The change rests on no type implementing one of them twice, so confirm that
      still holds
- [x] 1.2 Count `VS`/`VT` occurrences per crate as the before-number for task 4.5
- [x] 1.3 Add `deep_causality_algebra` to `deep_causality_context/Cargo.toml` for the real-field
      bound, and confirm the tier order still places it below the context crate
- [x] 1.4 Add `deep_causality_metric` for the signature enum, and confirm the same
- [x] 1.5 Verified: `deep_causality_num` arrives transitively through `deep_causality_algebra`, so
      no direct dependency was added. `deep_causality_metric` needed `features = ["default"]`,
      because the workspace entry sets `default-features = false` and its error type needs `alloc`

## 2. Associated types on the traits

- [x] 2.1 `Coordinate` gains `type Coord`; every method that returned `V` returns `Self::Coord`
- [x] 2.2 `Temporal` gains `type TimeUnit`; `time_unit` returns `Self::TimeUnit`
- [x] 2.3 `Spatial` becomes `Identifiable + Coordinate` with no parameter
- [x] 2.4 `SpaceTemporal` becomes `Identifiable + Spatial + Temporal`, with `t` returning
      `&Self::TimeUnit`
- [x] 2.5 Rename `Metric<V>` to `Distance` with `fn distance(&self, other: &Self) -> Self::Coord`,
      and update its six implementors
- [x] 2.6 Remove `MetricCoordinate`. Confirm first that it still has zero implementors and is named
      as a bound nowhere; if that changed, stop and re-decide
- [x] 2.7 `cargo test -p deep_causality_context` green, still at `f64` throughout: 431 unit tests
      and 22 doctests pass

## 3. The scalar parameter

- [x] 3.1 Give each spatial node type a scalar parameter bounded once on the algebra real-field
      bound, and set `type Coord` to it
      **`RealField` supplies less than the code assumed.** It carries `is_finite`, `sqrt`, `abs`,
      `atan2`, `sin`, `cos`, `pi()`, `zero()`, `one()`, arithmetic, ordering and `Copy`, but not
      `Default`, `Debug`, `powi` or `to_radians`. `Adjustable` impls therefore bound
      `R: RealField + Default`, because `ArrayGrid<T, ..>` needs `Default` to initialise its
      backing array; `Display` impls that render with `{:?}` bound `+ Debug` rather than switching
      to `{}`, which would change how whole numbers print; `powi(2)` became a multiplication; and
      `GeoSpace`'s Haversine writes its own degrees-to-radians from `R::pi()`. Numeric literals in
      kernels cross onto the real axis through `deep_causality_num::lift`, which is why that crate
      is now a direct dependency — Tier 0 against this crate's Tier 6, so the tier order holds
- [x] 3.2 Same for the spacetime node types
- [x] 3.3 Same for the real-valued temporal types; leave the integer-valued ones on their integer
      representations, which encode a tick rather than a precision
      **`DiscreteTime` and `EntropicTime` are untouched**, as are the `time_unit` accessors that
      return their ticks
- [x] 3.4 Propagate the parameter through the `Kind` enums
      **`TimeKind<R>`'s tick-based variants carry no `R`**, so constructing one from `DiscreteTime`
      or `EntropicTime` alone leaves the scalar unpinned and needs an annotation. Its `time_unit`
      crosses those ticks onto the real axis with `lift_count` rather than an `as` cast
- [x] 3.5 Remove every `f64` and `f32` from `deep_causality_context/src` outside the ready-made
      aliases, which deliberately fix a working scalar
      **Done: zero `f64`/`f32` remain on a code line outside `src/alias/`.** Three traits welded a
      scalar and were changed with the node types: `SpaceTemporalInterval` and `MetricTensor4D` now
      read theirs from `Coordinate::Coord` rather than declaring a second associated type that
      could disagree with the coordinates, and `ScalarValue` became one blanket impl in place of
      seven named primitives — a list decides for the caller which scalars exist, so `BFloat16`
      could not have been projected without an entry being added for it
- [x] 3.6 Keep `BaseContext`, `BaseContextoid`, `UniformContext` and `UniformContextoid` concrete so
      alias-based call sites need no annotation they did not need before
- [x] 3.7 Add a test building a context at the smallest shipped real field and another at the
      largest, reading coordinates back from both. This is the acceptance criterion for the whole
      group
      **`BFloat16` (8-bit significand) and `Float106` (~106-bit)**, in
      `tests/.../context_graph/scalar_parameter_tests.rs`. Both build a context through one generic
      helper and read axis 0 back. A third test is the negative control: it stores 0.1 at each
      scalar, widens both readings to `f64`, and asserts the narrow one carries more error. Without
      it the first two would pass even if the parameter never reached the node
- [x] 3.8 `cargo test -p deep_causality_context` green

## 3b. Identifiers and the edge weight

- [x] 3b.1 Move `ContextId` and `ContextoidId` out of `deep_causality_core` into
      `deep_causality_context`, with a docstring stating that the width is this crate's decision.
      A move, not a copy: core keeps no context identifier alias
- [x] 3b.2 Route the 49 `id:` struct fields through them
- [x] 3b.3 Route the 57 function signatures through them
- [x] 3b.4 Confirm no raw unsigned integer remains in an identifier position in
      `deep_causality_context/src`
- [x] 3b.5 Add `Default` to `RelationKind` on the `Datial` variant, documented as the graph bound's
      filler rather than a meaningful relation. A data relation is the least specific of the four,
      so its accidental appearance claims least
- [x] 3b.6 Change the backing graph's weight to `RelationKind` and delete the `weight as u64` cast
- [x] 3b.7 Add a test asserting an edge's relation is recoverable after storage, which it is not
      today
      **`ContextuableGraph` had no way to read an edge back**, only `contains_edge` returning a
      bool, so the assertion needed `get_edge(a, b) -> Option<&RelationKind>` added beside it,
      mirroring the existing `contains_node`/`get_node` pair. The test puts two different relations
      on two edges out of one node, so it fails if the weight collapses to a constant.
      `ExtendableContextuableGraph` gained the matching `extra_ctx_get_edge` on the user's
      instruction, so an extra context recovers its edge relations too, under the same test shape
- [x] 3b.8 Re-point `deep_causality`'s four source files and three test files that import the two
      from core, and drop them from its root re-export list — they are context items now, and the
      no-re-export contract already covers those
- [x] 3b.9 Confirm each of the two has exactly one declaration site in the workspace
- [x] 3b.10 Note that `TeloidTag` and `TeloidID` remain the same defect in core, unmoved here
      because this change does not touch ethos's vocabulary
      **Premise corrected during apply: neither is in core.** Both are declared in
      `deep_causality_ethos/src/alias/mod.rs`, which is the crate that owns them, so there is no
      parallel defect to leave behind. Nothing to do
- [x] 3b.11 `bazel test //...` green

## 4. Six parameters to four

**Sequencing corrected during apply.** Tasks 4.1 and 4.2 landed inside group 2. The plan had the
trait rewrite and the parameter removal as separate steps, and they cannot separate: `Context`
bounds `S: Spatial<VS>`, so the moment `Spatial` loses its parameter, `VS` has nothing to bind to
and the crate stops compiling. Group 2 therefore ends at four parameters in the context crate, and
group 4 is the downstream fan-out that follows from it.

- [x] 4.1 Drop `VS`/`VT` from `Context`, `Contextoid` and `ContextoidType` (landed with group 2)
- [x] 4.2 Drop them from `Contextuable`, `ContextuableGraph` and `ExtendableContextuableGraph`
      (landed with group 2)
- [x] 4.3 Fix `deep_causality`: `Model`, the generative interpreter, and the `Base*`/`Uniform*`
      aliases
- [x] 4.4 Fix `deep_causality_ethos`: `Teloid`, `TeloidStore` and `EffectEthos` thread these
      parameters through their own signatures
- [x] 4.5 Fix the example packages, then confirm `VS`/`VT` appear nowhere as a generic parameter.
      Against task 1.2's baseline of 648 (deep_causality 50, context 318, ethos 280, examples 0):
      now zero in every crate, including the three stale doc references that described parameters
      the types no longer take
- [x] 4.6 `bazel test //...` green: 1409/1409. Bazel caught one ethos test file that
      `cargo test --no-run` did not, because Bazel compiles each test file as its own crate.
      **Land this before starting group 6** — four parameters is
      independently releasable, and keeping the two failure modes apart is the point of the sequence

## 5. The coordinate families

- [x] 5.1 Merge the two field-identical Cartesian spacetimes, keeping **`LorentzianSpacetime`** as
      the surviving name and its constructors. `Lorentzian` names the signature class and
      `Minkowski` names one flat member of it, so the general name survives a merge and the frame's
      constant carries the convention the two names used to carry
      **Verified before merging:** the two are field-identical (`id`, `x`, `y`, `z`, `t`,
      `time_scale`) with identical constructors, and eight of nine trait impls are identical
      modulo the name. `Display` differs, and `LorentzianSpacetime`'s is the richer of the two
      since it prints `time_scale`, so it survives unchanged. The 25 tests paired 1:1 except
      `test_adjust_with_infinity_should_fail`, which was ported to the Lorentzian suite so the
      merge costs no coverage. `MinkowskiSpacetime` derived `Copy` and `LorentzianSpacetime` does
      not; no consumer relied on it. The removal is 14 files, 10 under
      `src/.../space_time/minkowski_spacetime/` and 4 under `tests/.../space_time/minkowski/`.
      **Deletion permission granted by the user, 2026-09-21.** `deep_causality_physics` has its
      own `minkowski_metric()` function, which is unrelated and untouched
- [x] 5.2 Delete the `SpaceTimeKind::Minkowski` arm and update every consumer of the removed name
- [x] 5.3 Remove `QuaternionSpace` **and** the `SpaceKind::Quaternion` arm, not only the `Spatial`
      impl. A quaternion is a rotation operator acting on a space rather than a space of its own,
      and an orientation belongs beside a position in a pose. Removing the trait alone leaves the
      type reachable as a space through the enum, and the frame spec makes `SpaceKind` the ordinary
      frame member — so the defect would survive exactly where most frames meet it
- [x] 5.4 The removal deletes 12 files under `types/context_node_types/space/quaternion_space/` and
      its test directory. **Deletion permission granted by the user, 2026-09-21**, so the golden
      rule's ask is already answered for these files and no further prompt is needed. Confirm the
      count against the tree first; verified self-contained, with no consumer in `deep_causality`,
      `deep_causality_ethos` or the examples
      **Count corrected during apply: 11 files, not 12** — 8 under `src/.../space/quaternion_space/`
      and 3 under the matching test directory. Consumers updated: the `SpaceKind` arm and its
      `Coordinate`/`Identifiable`/`Display` match arms, four module declarations, the root export,
      two tests, and the two doc lines naming the type as a supported space
- [x] 5.5 Add the vertical datum to the geodetic spatial type as a field, and remove the claim from
      the altitude field's doc comment that fixed one reference for the whole type. If it is an
      enum, start from the five ISO 19111-aligned members `WGS84`, `EGM96`, `EGM2008`, `ISA` and
      `Terrain`, each classified by the height or datum type it belongs under. `ISA` and `Terrain`
      are the two that catch people out: a pressure altitude is not a height at all, and a terrain
      height depends on which model produced it
      **Landed as `VerticalDatum`** in `types/context_types/vertical_datum/`, beside `TimeScale`,
      which is the existing home for a vocabulary enum the node types reference. `WGS84` carries
      the `Default`. `GeoSpace::new` takes the datum as a fifth argument and `Display` prints it,
      because an altitude printed without its reference names no point
- [x] 5.6 Add a test asserting two geodetic positions with equal altitudes against different datums
      are distinguishable
- [x] 5.7 Document in the crate which spatial types pair with a spacetime and which require a
      conversion first, so the gap is recorded rather than discovered. Note that the geodetic type
      also carries a datum the Cartesian types do not, so a conversion out of it loses it
      **Written as the module doc on `types/context_node_types/space/`.** All three spacetimes
      store Cartesian `x, y, z`, so `EuclideanSpace` pairs directly, `EcefSpace` pairs numerically
      but loses the earth-fixed frame, and `NedSpace` and `GeoSpace` each need something neither
      type stores — a local origin in one case, a geodetic conversion and a datum decision in the
      other
- [x] 5.8 Leave every `TimeScale` variant spelled exactly as it is, including the inconsistent
      `Nanoseconds` and `Microseconds` beside `Millisecond` and `Second`. The names are a contract a
      projection matches on, so tidying one is a data migration elsewhere rather than a rename here.
      If they should be tidied, that is its own decision and not a silent one
      **Verified: `time_scale/` has an empty diff against the branch point.** `Nanoseconds`,
      `Microseconds`, `Millisecond` and `Second` are spelled exactly as before
- [x] 5.9 `bazel test //...` green

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
