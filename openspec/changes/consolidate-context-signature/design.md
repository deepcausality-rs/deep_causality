## Context

`deep_causality_context` was extracted from `deep_causality` in the preceding change. That move
changed where the types live and deliberately left their shapes alone, except for dropping the
`SYM` parameter and relaxing `Data<T>`. This change takes on the shapes.

Three things are wrong with them, and they are the same thing seen from three sides.

**The value parameters do no work.** An impl census over the whole crate:

| Trait | Impls | Value types |
|---|---|---|
| `Coordinate<V>` | 13 | `f64` ×13 |
| `Spatial<V>` | 12 | `f64` ×12 |
| `Metric<V>` | 6 | `f64` ×6 |
| `SpaceTemporal<VS, VT>` | 5 | `<f64, f64>` ×5 |
| `Temporal<VT>` | 11 | `f64` ×8, `u64` ×2, `i64` ×1 |

No type implements any of these at two different value types. A trait parameter is the right tool
only when one type needs several instantiations; otherwise it is an associated type spelled the
long way, and the cost is that every consumer must name and forward it.

**The scalar is welded in.** `EuclideanSpace { x: f64, y: f64, z: f64 }`. The rest of the workspace
is generic in its scalar under the tower's bounds, so a program names its working type once and
everything follows. Context is the layer that cannot.

**Nothing separates a signature from a tensor.** `deep_causality_metric` owns the first — a
manifold invariant, one constant. `deep_causality_physics` owns the second — `g_μν(x)`, a field.
The context crate has a trait called `Metric` that is neither: it computes a distance.

## Goals / Non-Goals

**Goals:**

- `Contextoid<D, F>`: two parameters, down from six.
- Precision is a parameter of the context layer. A context runs at `BFloat16`, `f32`, `f64` or
  `Float106` with no line added to the crate.
- A frame states what kind of world a context describes: one that has no spatial part, one whose
  regime changes at runtime, or one fixed at compile time. The variant case is the ordinary one.
- The integer axis is a parameter too: a context's id width is changeable at one declaration.
- The signature lives where it is constant and is checked once, at a boundary.

**Non-Goals:**

- **Making `TangentSpacetime.metric` a tensor.** Settled against during design: the field is
  load-bearing — `interval_squared` contracts it as `g_μν v^μ v^ν`, which is the only thing
  separating this type from the flat spacetimes — but `TangentSpacetime` has no consumer outside
  the context crate and its own tests. The stated benefit of a tensor, removing a conversion at
  every boundary, is worth nothing when there is no boundary. It stays `[[R; 4]; 4]` under the
  scalar parameter.
- **Curvature.** `deep_causality_physics::theories::general_relativity::metrics` already computes
  `g_μν` at a point for Schwarzschild, Kerr and FLRW, returning a tensor and importing
  `deep_causality_metric` for the convention. Duplicating that here would be a second
  implementation of a module that works.
- **Adding the missing spacetimes.** F2 says a `GeoSpace` frame has no spacetime to name. This
  change writes the restriction down; closing it is a modelling decision about whether a geodetic
  spacetime is a type or a conversion.
- **Reworking `Datable` or `D`.** `Datable` already carries its payload associatively, so `D`
  varies the implementor rather than the payload, and a deployment's own `Datable` is a real
  extension point.
- **Reintroducing `Copy` to `Data<T>`.** It bounds `T: Default + Clone + PartialEq` and stays that
  way.

## Decisions

### 1. Associated types, because no implementor needs two

`Coordinate` gains `type Coord`, `Temporal` gains `type TimeUnit`, and the supertrait chain carries
them:

```rust
pub trait Coordinate { type Coord; fn dimension(&self) -> usize; /* … */ }
pub trait Spatial: Identifiable + Coordinate {}
pub trait Temporal: Identifiable { type TimeUnit; fn time_scale(&self) -> TimeScale; /* … */ }
pub trait SpaceTemporal: Identifiable + Spatial + Temporal { fn t(&self) -> &Self::TimeUnit; }
```

`Temporal` is the one that genuinely varies — `f64`, `u64`, `i64` across eleven implementors — and
that is exactly the case an associated type serves: the variation is *between* implementors, never
within one.

*Alternative considered: keep the parameters and default them.* Rejected. Defaults on type
parameters do not propagate through supertrait bounds, so every consumer still names them.

### 2. The scalar parameter and the associated type are one change

```rust
pub struct EuclideanSpace<R> { id: u64, x: R, y: R, z: R }
impl<R: RealField> Spatial for EuclideanSpace<R> {}
impl<R: RealField> Coordinate for EuclideanSpace<R> { type Coord = R; }
```

Parameterising the struct is what gives the associated type something to say. Doing either alone
leaves the other half-done: a generic struct with a parameterised trait forces `Spatial<R>` back
into every signature, and an associated type over a hardcoded `f64` fixes `type Coord = f64`
forever.

The bound is one blanket real-field bound from `deep_causality_algebra`, named once per type. No
context type spells a concrete float.

### 3. `Metric<V>` becomes `Distance`; `MetricCoordinate` is deleted

Three traits in the crate carry "Metric" in the name and the census is lopsided: `Metric<V>` has
six implementors and no caller outside tests, `MetricCoordinate<V>` has **zero** implementors and
is named as a bound nowhere, and `MetricTensor4D` has one.

`Metric<V>` computes `fn distance(&self, other: &Self) -> V`, so `Distance` says what it does. The
name it gives up is needed: `deep_causality_metric::Metric` is the signature enum the frame
carries, and two different things called `Metric` cannot both be imported into the frame's module.

`MetricCoordinate` goes. With associated types the combination it expressed is one inline bound.

*Alternative considered: path-qualify one of them.* Rejected — `deep_causality_metric::Metric` at
every use site, to preserve a name that was describing the wrong concept.

### 4. The frame carries the signature, not the tensor

```rust
pub trait ContextFrame {
    type Space:     Spatial;
    type Time:      Temporal;
    type SpaceTime: SpaceTemporal;
    const SIGNATURE: Metric;
    fn metric_family() -> Option<MetricFamily>;
}
```

A signature is constant across a manifold by continuity: a Lorentzian manifold has signature
(−,+,+,+) at every point, and Schwarzschild, Kerr and FLRW are all Lorentzian. One constant per
frame, so an associated constant, checkable at compile time.

A metric family is a choice of analytic form with parameters, which varies and cannot be a
constant. `None` marks the numerically evolved case, where no closed form exists and the
coordinate carries the tensor itself.

Three regimes and what each costs:

| Regime | Signature | Tensor |
|---|---|---|
| Flat — Minkowski, Euclidean | frame constant | constant, derivable |
| Curved analytic — Schwarzschild, Kerr, FLRW | frame constant | computed from parameters by the physics crate |
| Numerically evolved — GRMHD, ADM | frame constant | per point, no closed form |

`TangentSpacetime.metric: [[f64; 4]; 4]` is the third regime stored on every point. For the first
two it is 128 bytes of waste, and the first two are the common case. Keeping the field on that one
type states the regime in the type system.

### 5. `NoSpaceTime` rather than an optional member

Rust's associated-type defaults are unstable, so `type SpaceTime` cannot be omitted. A zero-sized
`NoSpaceTime` implementing the traits trivially lets a frame name it.

The measured usage says this is the common case, not an escape hatch. Across the workspace the
contextoid variants are used Root 45, Datoid 47, Tempoid 15, Spaceoid 5, SpaceTempoid 5. Most
contexts hold a root, some data and a clock. Without `NoSpaceTime` every one of them names two
types its graph never holds.

*Alternative considered: make the frame's spacetime member `Option`-shaped at the type level.*
Rejected — there is no such thing; the ZST is how Rust spells it.

### 6. `MinkowskiSpacetime` and `LorentzianSpacetime` merge

Both are `{ id, x, y, z, t, time_scale }`. The only difference between them is the sign convention,
which lives in the type's *name* today and moves to the frame's `SIGNATURE`. One coordinate
spacetime then serves both, and `sign_convention_mismatch` — which already exists in
`deep_causality_metric` — becomes answerable once at a boundary rather than by picking a type.

### 6b. `Lorentzian` is the name that survives the merge

Both types carry the Lorentzian signature and differ only in sign convention. `Lorentzian` names
the signature class; `Minkowski` names one flat member of that class. A merge keeps the general
name, and the frame's constant carries the convention the two names used to carry.

### 7. `QuaternionSpace` leaves the crate

Its fields are `id, w, x, y, z`: a rotation operator acting on a space rather than a space of its
own. An orientation sits beside a position in a pose, and a context locates things while a rotation
locates none.

The type goes, and so does the `SpaceKind::Quaternion` arm. Removing only the `Spatial` impl was
the earlier plan and it is not enough: Decision 8b makes the variant enum the ordinary frame member,
so an arm carrying the type leaves it reachable as a space precisely where most frames will meet
it. Half the fix would sit one level below where the problem now lives.

An orientation belongs in a deployment's own vocabulary, attached to a position rather than
standing beside one.

### 8. Sequence: four before two

Six to four is a pure win — no usage pattern argues against it and the impl census proves it safe.
Four to two is a trade, and the trade is only acceptable once a frame can say it has no spatial
part and once the family it selects from is coherent. So `NoSpaceTime`, the merge and the
`QuaternionSpace` correction land before `ContextFrame`, not after.

### 8b. The frame sits on both `Contextoid` and `Context`, and admits the `Kind` enums

`Contextoid` holds its variant payloads, so it must name their types; it gets them from
`F::Space`, `F::Time` and `F::SpaceTime`. `Context` holds a graph of contextoids, so it names the
same `F`. Both carry it, and both land at two parameters.

*Alternative considered: the frame on `Context` only, with `Contextoid<D, S, T, ST>` underneath.*
Rejected on inference. There are 85 standalone `Contextoid::new` call sites; one type argument to
infer is better than three, and the ready-made aliases absorb it exactly as they do now.

**A frame fixes the types, not the physics.** Moving `F` between the two changes nothing about
homogeneity: the graph is already one node type, so every contextoid in a context already shares
its space, time and spacetime types. Variation at runtime is what the `Kind` enums are for, and
they satisfy the frame's bounds today — `SpaceKind: Spatial`, `TimeKind: Temporal`,
`SpaceTimeKind: SpaceTemporal`. `UniformContext` is built on exactly that.

So a frame whose members are `Kind` enums is a first-class case, not a workaround, and it is what a
context that changes regime needs. The physics examples are the concrete reason: several of them
predate this crate and reason about regime changes outside any typed context —
`event_horizon_probe` runs Newtonian far-field and switches to Minkowski near the Schwarzschild
radius within one causal model. Migrating it to a typed context needs a spacetime that varies at
runtime, which is a frame over `SpaceTimeKind`. The frame must not make that harder than a fixed
frame, because enabling those migrations is part of what this change is for.

### 8c. A geodetic altitude carries its datum

`GeoSpace.alt` is documented as "Altitude in meters above the WGS84 ellipsoid". The reference is in
a doc comment, which fixes it for every value the type will ever hold. Ellipsoidal, geoid-model and
barometric references differ by tens of metres, so the same number is a different height against
each, and a type carrying the number without the reference carries an ambiguity that reads as a
fact.

The datum becomes a field. A constant on the frame would model it better — a vertical datum is a
property of the reference frame in the same way the metric signature is — but a constant cannot
vary, and two positions differing only in datum belong to one context. The same reasoning sends the
frame itself to the variant enums: whatever varies per node has to live in the node.

This deepens F2 rather than resolving it. The geodetic type already had no spacetime counterpart;
now it also carries a reference the Cartesian types do not, so converting out of it discards
something the others never held.

### 9. The edge weight carries `RelationKind`, not its discriminant

`Context` stores its graph as `UltraGraphWeighted<Contextoid<..>, u64>` and writes edges with
`self.base_context.add_edge(a, b, weight as u64)`. The public API takes a `RelationKind`; the graph
stores a number. The type is discarded at the boundary and cannot be read back.

`RelationKind` is `#[repr(u8)]` with four variants, so the cast inflates one byte into eight and
loses the meaning on the way.

`ultragraph` bounds its weight `W: Clone + Default`. A tighter `Copy + Ord + Default` appears on
three weighted algorithms, none of which the context graph calls: it uses `add_node`, `add_edge`,
`contains_node`, `contains_edge`, `get_node`, `is_empty`, `number_nodes` and `number_edges`, all
structural. So `RelationKind` qualifies once it has a `Default`.

None of the four variants is naturally the default, and `add_edge` always supplies a real one, so
the `Default` exists only to satisfy the bound. It is declared on the variant that reads most
neutrally and documented as the bound's filler rather than as a meaningful relation.

### 10. The id aliases move out of core into the context crate

`ContextId` and `ContextoidId` are declared in `deep_causality_core` today and **core never uses
them**; the only consumers are `deep_causality` and `deep_causality_context`. That is the same
pattern as `TeloidTag` and `TeloidID`, which core also declares for a concept it does not own.

They **move**, rather than being duplicated: core keeps no context identifier alias afterwards.
The reason is coupling rather than tidiness. An alias in core is core's decision, so widening
`ContextoidId` there would change every context type's width without the context crate having said
anything. A context's id width is a property of contexts, so the crate that owns `Context` and
`Contextoid` owns the alias, and a consumer who needs a different width edits one declaration in
the crate that means it.

The principle generalises, and core has one more instance of the same defect: `TeloidTag` and
`TeloidID` are declared in core with no user there, naming a concept `deep_causality_ethos` owns
and independently declares. **Core should declare only vocabulary core uses.** Moving the Teloid
pair is left alone here because nothing in this change touches that crate's vocabulary.

The move is not free. `deep_causality` imports the two from core in four source files and three
test files, and re-exports them at its root; every one of those repoints to the context crate.
Under the no-re-export contract from the preceding change, `deep_causality` does not re-export them
at all once they are context items — consumers name the context crate, which is the same rule
already applied to `Context` itself.

The crate has 111 raw `u64`: 49 `id:` struct fields, 57 function signatures and the 2 graph-weight
positions Decision 9 covers. All of them route through the crate's own aliases, so the integer axis
becomes changeable at one site exactly as the real axis does.

### 11. Precision and range are two axes, and the crate welds both

The real scalar selects **precision**: the significand, where the failure mode is rounding, bounded
and gradeable. The integer alias selects **range**: the exponent, where the failure mode is
overflow, which is not an approximation of the right answer but a wrong one.

Decision 2 unwelds the first and Decision 10 unwelds the second. Doing one without the other leaves
the crate explicitly parameterised on precision while still hardcoding how many contextoids a
context can address.

## Risks / Trade-offs

- **The blast radius is every context signature in the workspace.** `VS`/`VT` appear 648 times
  across four crates plus examples, and `deep_causality_ethos` threads them through `Teloid`,
  `TeloidStore` and `EffectEthos`. → Land four parameters first and get the workspace green before
  starting on the frame, so the two failure modes never overlap.
- **A frame is a worse fit for a context with no geometry than six parameters were.** A discrete
  simulation with ticks and no space names `NoSpaceTime` twice over. → This is the cost the usage
  counts quantify; it is accepted in exchange for the other four parameters, and `NoSpaceTime`
  makes it one word rather than two type names.
- **Generic structs change inference at call sites.** `EuclideanSpace::new(1, 1.0, 2.0, 3.0)`
  currently needs no annotation; `EuclideanSpace<R>` may. → Measure it in the examples, which are
  the most annotation-averse code in the repo, and keep the shipped aliases concrete so the common
  path stays inference-free.
- **Two new crate dependencies.** `deep_causality_algebra` for the bound, `deep_causality_metric`
  for the signature. → Both sit far below the context crate in the tier order, so neither creates a
  cycle; the tier block is re-derived as part of the change.
- **`BFloat16` has two decimal digits.** A context that stores geodetic coordinates at `BFloat16`
  is arithmetically valid and practically useless. → The change makes precision selectable, not
  advisable; the crate documents the range each shipped scalar holds rather than refusing one.

## Migration Plan

1. `Coordinate`, `Temporal` and the rest gain associated types; `Metric` becomes `Distance`;
   `MetricCoordinate` is removed. Crate green.
2. Node types gain their scalar parameter under the algebra bound. Crate green at `f64`.
3. `VS`/`VT` come off `Context`, `Contextoid`, `ContextoidType` and the three graph traits.
   Workspace green at four parameters.
4. `QuaternionSpace` loses `Spatial`; the two spacetimes merge; the F2 restriction is documented.
5. `NoSpaceTime`, then `ContextFrame` with `SIGNATURE` and `metric_family`.
6. `Contextoid<D, F>`. Workspace green at two parameters.
7. A test constructing a context at a non-`f64` scalar, which is the acceptance criterion for
   step 2.

Steps 1–3 are independently landable and leave the workspace releasable. Rollback is a branch
revert.

### 12. `RelationKind::Datial` carries the `Default`

The bound `W: Clone + Default` on the backing graph's weight requires a default. `add_edge` always
supplies a real relation, so the default is never produced through the API and exists only to
satisfy the bound.

`Datial` takes it. No variant is naturally first, and a data relation is the least specific of the
four, so it is the one whose accidental appearance claims least.

*Alternative considered: `Option<RelationKind>`, with `None` as the default.* Rejected. `None` would
assert that an edge has no relation, and an edge that exists has one. A wrong claim reads worse than
an arbitrary one, and it would add an unwrap to every read.

## Open Questions

None.
