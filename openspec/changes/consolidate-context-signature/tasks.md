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

- [x] 6.1 Add `NoSpaceTime`, a zero-sized type implementing the spatial and spacetime traits
      trivially, with its own tests
      **`NoSpaceTime<R>` carries its scalar in `PhantomData`**, so it stays zero-sized at every
      scalar while still satisfying the frame's `Coord = Self::Scalar` bound. It can do that
      because its coordinate system has zero dimensions, so `coordinate()` always errors and no
      value of type `R` is ever produced. Its `TimeUnit` is `()`, not `R`: what it stands in for
      is a spacetime *position*, and such a context still keeps a real clock in the frame's `Time`
      member
      **Kept, and it outlived the frame that motivated it.** A four-parameter `Context` names a
      spatial and a spacetime type whether or not its graph holds either, so a clock-only context
      still has two slots to fill, and a zero-sized type fills them without naming something the
      graph never holds.
- [x] 6.2 Add `ContextFrame` with `Space`, `Time` and `SpaceTime` associated types
      **Plus a fourth, `Scalar: RealField`.** `Space` and `SpaceTime` are bound to it, so a frame
      cannot report a position in one scalar and an interval in another. `Time` is deliberately
      left free, because a tick is a count rather than a precision and a frame pairing a
      real-valued space with an integer clock is legitimate. Binding a supertrait's associated
      type through the subtrait (`Spatial<Coord = Self::Scalar>`) was verified to compile before
      the design was committed to
      **REVERSED by the user after review.** The trait landed, then came out again with group 7.
      It bundled three parallel node-payload slots and carried a scalar, a signature and a family.
      The signature moved to the nodes, the family is a function call the physics crate already
      makes with loose parameters, and the bundling is what a type alias does. Nothing consumed it:
      outside its own module it appeared in a re-export and a docstring.
- [x] 6.3 Add `SIGNATURE` as an associated constant of type `deep_causality_metric::Metric`
      **Reversed by the user during apply, and the reasoning is worth keeping.** A constant on the
      frame is a claim, and a variant frame's contents can contradict it: `SpaceTimeKind` holds the
      Newtonian `EuclideanSpacetime` at (+,+,+,+) beside the relativistic types at (−,+,+,+).
      Two attempts to spell the escape — `Option<Metric>` and a `MetricKind { Fixed, Deferred }`
      enum — were both rejected, correctly: they model *presence* when the real axis is
      *provenance*. A variant context is not a context with no signature; it is one whose
      signatures come from its nodes.
      **What landed instead:** a `MetricSignature` trait with `fn metric(&self) -> Metric`, made a
      supertrait of `SpaceTemporal` so every spacetime type must answer, implemented on
      `EuclideanSpacetime` (Euclidean(4)), `LorentzianSpacetime` and `TangentSpacetime`
      (Lorentzian(4)), `NoSpaceTime` (Euclidean(0)), and forwarded by `SpaceTimeKind`. The frame
      declares no signature at all; `metric_family` stays, because a node can say it is Lorentzian
      but not that it is Schwarzschild at a given mass.
      **This matches the rest of the stack.** `deep_causality_physics` reads a metric off a
      `CausalMultiVector` and compares it against the caller's expectation at ten sites across its
      em, relativity and dynamics kernels — its error text already says "Context expects". The
      node-reported signature is that same shape one layer up.
      **A correction I made along the way:** I argued the crate's own `MINKOWSKI_4D` constants
      having no consumers showed a compile-time metric had no place here. That was a miscalibrated
      negative control — the sample is examples that are variable-metric by nature, so their
      silence says nothing. The user caught it.
- [x] 6.4 Add `MetricFamily` as an enum in `deep_causality_metric`, naming the analytic forms with
      their parameters, and `metric_family` returning `None` where the metric has no closed form.
      An enum keeps that crate closed and checkable and matches what the physics crate computes
      **Four variants, derived from the closed forms in `deep_causality_physics`:** `Flat`,
      `Schwarzschild { mass }`, `Kerr { mass, spin }` and `Flrw { scale_factor, curvature_k }`.
      Each carries the parameters that fix the form across the manifold and none of the
      coordinates it is evaluated at — `schwarzschild_metric_at(mass, r)` takes both, and only the
      mass belongs here. `Flat` covers the Minkowski and Euclidean cases together, because the
      signature already separates them. `MetricFamily<R>` is unbounded in `R`, which keeps the
      metric crate a dependency-free leaf
- [x] 6.5 Provide frames for the shipped configurations: one over the variant `Kind` enums as the
      ordinary case, one fixed-type frame as the specialisation, and one with no spatial part that
      names `NoSpaceTime`. Document which to reach for when the world is not known at compile time
      **`UniformFrame`, `BaseFrame` and `ClockFrame`**, with the selection table on the
      `context_frames` module.
      **Member and signature are independent choices, corrected during apply.** An earlier note
      here called it a defect that `SpaceTimeKind` admits both `LorentzianSpacetime` (−,+,+,+) and
      `EuclideanSpacetime` (+,+,+,+) while `UniformFrame` declares one constant. It is not: the
      member says which coordinate types a context may hold, the signature says which manifold the
      model claims, and the frame is where the two are fixed together. A probe moving between the
      coordinate and tangent spacetimes stays Lorentzian throughout, which is the variance the
      enum is for. A Newtonian or particle-physics model declares its own frame over the same enum
      with `Metric::Euclidean` or `Metric::Minkowski` and keeps the variance; only a fixed frame
      gives it up. Two tests pin this — the regime-change test uses two variants inside the
      signature class, and a second asserts `EastCoastFrame` and `WestCoastFrame` name the same
      member type and differ only in their constant
      **REVERSED with 6.2.** `UniformFrame`, `BaseFrame` and `ClockFrame` landed and came out
      again. The distinction they encoded — variant members against fixed ones — is the one the
      `BaseContext` and `UniformContext` aliases already draw.
- [x] 6.5b Add a test holding two spacetime contextoids of different concrete variants in one
      context on a variant frame, which is what a regime-changing model needs
      **Kept, and moved.** It lives in `tests/.../context_graph/mixed_spacetime_tests.rs` with the
      type parameters spelled out, alongside a second test holding a Newtonian node beside a
      relativistic one so the two report different signatures, and a clock-only context that names
      `NoSpaceTime` in both empty slots.
- [x] 6.6 Add a test asserting a frame's signature is readable without constructing a context, and
      one asserting two frames with different conventions are distinguishable through the metric
      crate's existing convention checks
      **Half reversed.** There is no frame signature to read. The convention half survives as
      per-type tests: each spacetime type pins its own signature, and one asserts a node's
      signature feeds `detect_convention` and `is_lorentzian` with nothing new in between.
- [x] 6.7 `cargo test -p deep_causality_context` green
      414 tests and 22 doctests. `bazel test //...` 1408. The two new test directories each needed
      their own `rust_test_suite` in `BUILD.bazel`, because the context crate globs per directory
      rather than per tree — without them Bazel compiled neither file

## 7. Four parameters to two — REMOVED

**Dropped by the user after review, with the frame it depended on.** The group would have replaced
`Context<D, S, T, ST>` with `Context<D, F>`.

It does not pay for itself. Measured across the workspace: 218 call sites already reach the context
through the ready-made aliases and would not have changed either way, 16 sites spell four arguments,
and 24 generic declarations in `deep_causality` and `deep_causality_ethos` would have collapsed from
four bounds to one. That is the whole benefit, and it is ergonomic rather than a capability.

The design's stated justification was false. It said that naming the three separately "lets a
context pair a space with a spacetime that cannot contain it". There is no containment:
`ContextoidType` makes them parallel arms — `Spaceoid(S)` beside `SpaceTempoid(ST)` — and a frame
would not have validated the triple either, since any three types can be written into an impl. A
frame makes the choice once; it does not make a wrong choice impossible.

**The user's reason for keeping the parameters explicit:** a library that handles four generic
parameters should show them, because an opaque bundle can encode incorrect assumptions and hides
them from the reader and the maintainer.

Everything the frame carried is expressible without it: bundling by a type alias, which
`BaseContext` and `UniformContext` already do in one line each; the signature by the node, which
landed in group 6; the family by calling the physics function with its parameters, which the
examples already do; and the scalar coherence between space and spacetime by a where clause on
`Context` itself, if it is wanted.

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
