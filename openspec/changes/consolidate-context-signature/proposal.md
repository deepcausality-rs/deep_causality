## Why

`Context`, `Contextoid` and `ContextoidType` each carry six generic parameters. Two of them, `VS`
and `VT`, say what a coordinate and a time unit are made of, and they are threaded by hand through
seven types: `Context`, `Contextoid`, `ContextoidType`, `Contextuable`, `ContextuableGraph`,
`ExtendableContextuableGraph` and `SpaceTemporal`. They appear 318 times in
`deep_causality_context/src` and 648 times across the workspace.

They buy nothing. A trait *parameter* earns its place where one type implements the trait at
several value types. Counting every impl in the crate shows that never happens:

| Trait | Impls | Value types used |
|---|---|---|
| `Coordinate<V>` | 13 | `f64` only |
| `Spatial<V>` | 12 | `f64` only |
| `Metric<V>` (renamed `Distance`) | 6 | `f64` only |
| `SpaceTemporal<VS, VT>` | 5 | `<f64, f64>` only, `VS == VT` every time |
| `Temporal<VT>` | 11 | `f64` ×8, `u64` ×2, `i64` ×1 |

Only `Temporal` varies at all, and it varies by implementor rather than within one. That is what an
associated type is for, and `Datable` in the same directory already does it: `type Data`.

The second cost is precision. Every context node type spells `f64` into its fields —
`EuclideanSpace { x: f64, y: f64, z: f64 }`, `MinkowskiSpacetime { x, y, z, t: f64 }`. The rest of
the workspace treats precision as a parameter, so a program names its scalar once and the
arithmetic follows. The context layer is the exception, and a model cannot run its context at
`BFloat16` or `Float106` however the rest of it is written.

These are one change, not two. Making `EuclideanSpace` generic in its scalar is precisely what an
associated type carries: `impl<R> Spatial for EuclideanSpace<R> { type Coord = R; }`. Parameterise
the struct and the trait parameter has nothing left to do.

## What Changes

- **`Coordinate`, `Spatial`, `Temporal`, `SpaceTemporal` and the distance trait take associated
  types.**
  `Coordinate { type Coord; }`, `Temporal { type TimeUnit; }`, and the supertrait chain reads
  `Spatial: Identifiable + Coordinate` with `Self::Coord` flowing through.
- **BREAKING: `Context`, `Contextoid` and `ContextoidType` drop `VS` and `VT`**, going from six
  parameters to four, and the same two come off `Contextuable`, `ContextuableGraph`,
  `ExtendableContextuableGraph` and `SpaceTemporal`.
- **BREAKING: every context node type becomes generic in its scalar.** `EuclideanSpace<R>`,
  `EcefSpace<R>`, `GeoSpace<R>`, `NedSpace<R>`, the spacetime family and the `Kind` enums, each
  bounded once on the tower's real-field bound from `deep_causality_algebra`. No context type names
  a concrete float. The acceptance test is a context running at `BFloat16` with no line added to
  the crate.
- **BREAKING: `ContextFrame` bundles space, time and spacetime**, taking `Contextoid<D, F>` to two
  parameters. It carries the manifold's signature as an associated constant and names the analytic
  metric family where one exists.
- **BREAKING: `MinkowskiSpacetime` and `LorentzianSpacetime` merge, and `Lorentzian` survives.**
  They are field-identical today — `id, x, y, z, t, time_scale` — and differ only in a signature
  convention carried in the type's name. `Lorentzian` names the signature class where `Minkowski`
  names one flat member of it, so the general name survives and the frame's constant carries the
  convention. `SpaceTimeKind::Minkowski` goes with it.
- **`NoSpaceTime`, a zero-sized type**, so a frame can state that its context has no spatial
  extent. Rust's associated-type defaults are unstable, so the member cannot simply be omitted, and
  the measured usage makes this the common case rather than an edge: across the workspace the
  contextoid variants are used Root 45, Datoid 47, Tempoid 15, Spaceoid 5, SpaceTempoid 5. Most
  contexts hold no spatial node at all.
- **BREAKING: `Metric<V>` is renamed `Distance`.** It computes `fn distance(&self, other: &Self)`,
  and `deep_causality_metric` already exports an enum called `Metric` that is a manifold signature.
  The frame needs the signature, so the distance trait gives up a name that was never describing
  what it does.
- **BREAKING: `MetricCoordinate<V>` is removed.** The marker `Coordinate<V> + Metric<V>` has zero
  implementors and is named as a bound nowhere. With associated types the combination it expressed
  is written inline in one line where anything needs it.
- **BREAKING: `QuaternionSpace` and `SpaceKind::Quaternion` are removed.** Its fields are
  `id, w, x, y, z`: a rotation operator acting on a space rather than a space of its own. Removing
  only the `Spatial` impl is insufficient, because the variant enum is the ordinary frame member,
  so the arm would leave an orientation reachable as a space exactly where most frames meet it. The
  type is self-contained — twelve files, no consumer outside the crate.
- **The space and spacetime families are misaligned, and the change writes that down.** Five
  spaces against three Cartesian spacetimes, with no geodetic or NED counterpart, so a `GeoSpace`
  frame has no spacetime to name. Stating the restriction is in scope; adding the missing
  spacetimes is not.
- **BREAKING: `ContextId` and `ContextoidId` move from `deep_causality_core` into
  `deep_causality_context`**, and the 111 raw `u64` in that crate — 49 `id:` fields, 57 signatures
  — route through them. Core declared both and never used them, so a width decision belonging to
  contexts sat in a crate that owns no context. `deep_causality` stops re-exporting them, as it
  already does for every other context item. Precision and range are two axes; parameterising the
  first while hardcoding the second unwelds half the crate.
- **BREAKING: the context graph stores `RelationKind` instead of its discriminant.** It holds
  `UltraGraphWeighted<Contextoid<..>, u64>` and writes `weight as u64`, flattening a
  `#[repr(u8)]` four-variant enum into eight bytes and discarding the type, so an edge's relation
  cannot be read back. The backing graph bounds its weight on clone and default only, and the
  context graph calls no weighted algorithm, so the enum qualifies.
- **A frame over the variant `Kind` enums is the primary case**, with a fixed-type frame as the
  specialisation. A causal model can change regime within one run, and a context assembled from
  stored records carries its frame per record, so a read spanning several returns more than one.
  Several physics examples predate this crate and reason about regime change outside any typed
  context; `event_horizon_probe` switches from Newtonian to Minkowski within one model. Enabling
  those migrations is part of what this change is for.
- **BREAKING: `GeoSpace` carries the vertical datum its altitude is measured from.** The field is
  documented today as metres above one specific ellipsoid, which fixes one reference for every
  value the type will ever hold. The surfaces differ by tens of metres, so the same number is a
  different height against a geoid model or a standard atmosphere.
- **Out of scope:** curvature. A metric tensor is a field, `g_μν(x)`, and
  `deep_causality_physics::theories::general_relativity::metrics` already computes it —
  `schwarzschild_metric_at`, `kerr_metric_at`, `flrw_metric_at` — returning a tensor and importing
  `deep_causality_metric` for the convention. Nothing here duplicates that.

## Capabilities

### New Capabilities
- `context-scalar-parameter`: precision is a parameter of the context layer. Every node type is
  generic in its scalar under one blanket bound, no context type names a concrete float, and a
  context runs at any of the shipped real fields.
- `context-associated-value-types`: `Coordinate`, `Spatial`, `Temporal`, `SpaceTemporal` and
  `Distance` carry associated types rather than value parameters, and `VS`/`VT` come off the seven
  types that thread them. Includes the `Metric` → `Distance` rename and the removal of the unused
  `MetricCoordinate`.
- `context-frame`: the `ContextFrame` trait bundling space, time and spacetime, the signature it
  carries, `NoSpaceTime` for a frame with no spatial extent, and the two-parameter
  `Contextoid<D, F>` that follows.
- `context-coordinate-families`: which spatial and spacetime types exist and what each one is —
  the merged coordinate spacetime, orientation excluded from `Spatial`, and the stated gap where a
  geodetic frame has no spacetime.
- `context-id-parameter`: the crate owns its identifier aliases, no context type spells a raw
  integer for an identifier, and the graph stores the relation rather than its discriminant.

### Modified Capabilities
- `context-crate-identity`: its requirement that no moved item changes its signature needs the
  exception this change makes, and its list of the traits the crate owns changes with the rename.
- `core-shared-vocabulary`: it fixes core as the single home for eight primitive aliases. Two of
  them are context identifiers that core never used, so the count becomes six.
- `context-symbolic-dimension-removed`: it fixes the parameter count at six. This change takes it
  to four and then two, so the requirement's arity claim is restated.

## Impact

**Breaking, and wider than the extraction that preceded it.** That change moved types between
crates and left their shapes alone. This one changes the shape of every context type.

| Consumer | What changes |
|---|---|
| `deep_causality_context` | 17 types shed `VS`/`VT`; every node type gains a scalar parameter; `Metric` becomes `Distance`; `MetricCoordinate` is removed; two spacetimes merge; `QuaternionSpace` loses a trait |
| `deep_causality` | `Model`, the generative interpreter, and the `Base*` / `Uniform*` aliases all name the context parameters |
| `deep_causality_ethos` | `Teloid`, `TeloidStore` and `EffectEthos` thread them through their own signatures |
| examples | every package that builds a context |

New dependency: `deep_causality_algebra` for the scalar bound, and `deep_causality_metric` for the
signature. Both sit far below the context crate, so neither introduces a cycle.

**One thing this leaves behind.** `TeloidTag` and `TeloidID` stay declared in
`deep_causality_core` with no user there, naming a concept `deep_causality_ethos` owns and
independently declares. Core should declare only vocabulary core uses; moving that pair is a
separate change, because nothing here touches ethos's vocabulary.

**Sequencing is load-bearing.** Four parameters is a pure win that no usage pattern argues against.
Two is a trade: a frame with three required members asks most contexts to name two types their
graph never holds, which is what `NoSpaceTime` and the family findings exist to answer. The
coordinate-family work lands before the frame, not after.

**Versions are release-plz's.** The `BREAKING CHANGE:` footers drive the bump; nothing here sets a
version by hand.
