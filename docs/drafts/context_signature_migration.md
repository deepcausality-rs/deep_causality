# Migrating to the consolidated context signature

*Draft. Publish alongside the `deep_causality_context` release that carries these changes.*

Companion to [Context moves out](context_crate_release_blogpost_draft.md), which covers the crate
extraction. That post tells you where the types now live. This one tells you what changed about the
types themselves.

Every change below is breaking. Most are mechanical.

---

## The short version

| What changed | What you do |
|---|---|
| `Context` takes four parameters, not six | Drop the two value parameters |
| Spatial, temporal and spacetime types take a scalar | Write `EuclideanSpace<f64>`, or use the aliases |
| `Metric` trait renamed `Distance` | Rename at the call site |
| `MetricCoordinate` removed | Bound on `Spatial` and `Coordinate` directly |
| `EuclideanTime`, `LorentzianTime` and `EuclideanSpacetime` renamed `NewtonianTime`, `MinkowskiTime` and `NewtonianSpacetime` | Rename; the `TimeKind` and `SpaceTimeKind` variants follow. Expect `Metric::PGA(4)` from `NewtonianSpacetime` |
| `LorentzianSpacetime` merged into `MinkowskiSpacetime` | Rename; the fields and constructor are identical. Expect a NaN interval for `NoScale`, `Steps` and `Symbolic` |
| A spacetime's coordinate 0 is `t` | Re-index `coordinate(i)` and `Adjustable` grid positions to `t, x, y, z`; constructor arguments keep `(id, x, y, z, t, ..)` |
| Every space and spacetime type reads grid position `k` at `PointIndex::new1d(k)`, `new2d(k, 0)`, `new3d(0, 0, k)` or `new4d(0, 0, k, 0)` | Fill a 4D grid for `MinkowskiSpacetime` along z, not t. A 1D or 2D grid now gives each coordinate its own cell and must hold one per coordinate |
| `GalileanSpacetime`, `NoSpace` and `NoTime` added | Add a `SpaceTimeKind::Galilean` arm to an exhaustive match |
| `update_metric_tensor` returns `Result<(), MetricTensorError>` | Handle the error for a tensor that is not finite, not symmetric, or not of signature (−, +, +, +); its `MetricTensorErrorEnum` names the rule and the entry or eigenvalue counts |
| `TangentSpacetime::euclidean_distance` and `spatial_velocity` removed | Compute them from `position()` and `velocity_vector()` |
| `CausalSetSpacetime` fields are private; `causal_depth` renamed `predecessor_count` | Read through `id()`, `label()` and `predecessors()`; extend the past with `add_predecessor`, which returns `false` for the element itself or one already recorded |
| `ConformalSpacetime` removed | Record the causal order with `CausalSetSpacetime`, which keeps each element's predecessors |
| `QuaternionSpace` removed | Carry orientation beside a position in your own type |
| `ContextId` and `ContextoidId` moved out of core | Import from `deep_causality_context` |
| Edges carry `RelationKind` | Nothing, unless you read edges back — now you can |
| `GeoSpace` carries a vertical datum and checks its coordinates | Pass a fifth argument and handle the `Result`; generic code adds `R: FromPrimitive`. `update` and `adjust` refuse the coordinates `new` refuses |
| `GeoSpace::distance` is the straight line between WGS 84 geocentric positions | Expect NaN unless both datums are `WGS84` |
| `UncertainFloat64Data` and `UncertainBooleanData` removed | Name the type and its scalar |
| Each crate declares its own `FloatType` | Import from the crate you are using |
| Time and spacetime records follow the renames; `SpaceTimeRecord` gains `Galilean` and `ProjectionErrorEnum` gains `Rejected` | In a backend, map stored `Euclidean` to `Newtonian` and `Lorentzian` to `Minkowski`, and add the new arms |

---

## Four parameters, not six

`Context` and `Contextoid` lost two parameters. The traits they are bounded on now carry their value
types as associated types.

```rust
// before
Context<D, S, T, SP, VS, VT>

// after
Context<D, S, T, ST>
```

A trait parameter earns its place when one type implements the trait at several value types. None
did: every spatial type had exactly one coordinate type, and every temporal type exactly one time
unit. The parameters were restating what the implementor already fixed.

If you wrote a bound, it loses the value parameter:

```rust
// before
fn f<S: Spatial<VS>, VS>(s: &S) { }

// after
fn f<S: Spatial>(s: &S) { }
```

And if you named the value type, project it:

```rust
fn f<S: Spatial>(s: &S) -> S::Coord { *s.coordinate(0).unwrap() }
```

The symbolic dimension went with them. `Context` had a sixth parameter for symbolic nodes that no
model outside the crate's own tests used.

## Precision is a parameter

Every spatial, spacetime and real-valued temporal type now takes the scalar it measures in.

```rust
// before
EuclideanSpace::new(1, 1.0, 2.0, 3.0)

// after — the scalar is inferred from the arguments
EuclideanSpace::new(1, 1.0, 2.0, 3.0)

// and it can be named
let narrow: EuclideanSpace<BFloat16> = EuclideanSpace::new(1, x, y, z);
let wide:   EuclideanSpace<Float106> = EuclideanSpace::new(1, x, y, z);
```

Construction usually infers. What changes is anywhere you *name* the type — a struct field, a
function signature, a type alias:

```rust
// before
type MyContext = Context<Data<f64>, EuclideanSpace, EuclideanTime, EuclideanSpacetime>;

// after
type MyContext = Context<
    Data<FloatType>,
    EuclideanSpace<FloatType>,
    NewtonianTime<FloatType>,
    NewtonianSpacetime<FloatType>,
>;
```

**The ready-made aliases are unchanged.** `BaseContext`, `BaseContextoid`, `UniformContext` and
`UniformContextoid` stay concrete, so if you reach the context through one of those, nothing about
this section applies to you.

### What a scalar has to satisfy

One bound: `RealField`, from `deep_causality_algebra`. The crate names no concrete float outside its
own aliases, so a type that satisfies the bound works without an entry being added for it. `f32`,
`f64`, `BFloat16` and `Float106` all do.

Further bounds appear where the code needs more than the algebra gives. `Adjustable` impls require
`Default`, because `ArrayGrid` needs it to initialise its backing array. `Display` impls that render
with `{:?}` require `Debug`. Code that converts a constant or a record value into the scalar requires
`FromPrimitive`: `GeoSpace::new`, the time-scale conversions and the `Recordable` impls.

### Ticks stay integers

`DiscreteTime` and `EntropicTime` are unchanged. A tick is a count, not a precision: its failure
mode is overflow, not rounding, and widening its significand buys nothing.

One consequence to know about. `TimeKind<R>`'s tick-based variants carry no `R`, so constructing one
from them alone leaves the scalar unpinned:

```rust
// the compiler cannot infer R here
let t: TimeKind<FloatType> = TimeKind::Discrete(DiscreteTime::new(2, TimeScale::Second, 42));
```

## `Metric` is now `Distance`

The trait computes `fn distance(&self, other: &Self) -> Self::Coord`, so it is named for what it
does. The name it gave up was needed: `deep_causality_metric::Metric` is the signature enum, and two
types called `Metric` cannot both be imported into one module.

```rust
// before
use deep_causality_context::Metric;

// after
use deep_causality_context::Distance;
```

`MetricCoordinate` is gone. It existed to express "spatial and coordinate together", which is now
one inline bound.

## The two flat spacetimes are one

`MinkowskiSpacetime` and `LorentzianSpacetime` had identical fields and constructors. They differed
in `Display`, and in `Copy`, which only `MinkowskiSpacetime` derived. Both were flat. A
relativistic spacetime is a four-dimensional manifold with a metric of Lorentz signature, curved or
flat; Minkowski spacetime is the one that is flat and geodesically complete (Malament 2012, §2.1).
The merged type carries the name of what it models.

```rust
// before
LorentzianSpacetime::new(1, x, y, z, t, TimeScale::Second)

// after — same arguments, same order
MinkowskiSpacetime::new(1, x, y, z, t, TimeScale::Second)
```

`SpaceTimeKind::Lorentzian` is gone with it. If you matched on it, match on
`SpaceTimeKind::Minkowski`.

`Display` prints the time scale, which `MinkowskiSpacetime`'s did not. The type does not derive
`Copy`, so clone where you copied a `MinkowskiSpacetime`. The interval is NaN for a time scale that
names no duration (`NoScale`, `Steps`, `Symbolic`); the two old types read the raw count as seconds.

## A spacetime reports its own signature

New trait, implemented by every spacetime node type that has coordinates (`GalileanSpacetime`,
`NewtonianSpacetime`, `MinkowskiSpacetime`, `TangentSpacetime`, `SpaceTimeKind`) and by
`NoSpaceTime`. `CausalSetSpacetime` holds only an order and does not implement it:

```rust
pub trait MetricSignature {
    fn metric(&self) -> Metric;
}
```

`GalileanSpacetime` and `NewtonianSpacetime` report `Metric::PGA(4)`, the signature (0,+,+,+) of
the spatial metric of a classical spacetime, time first. `MinkowskiSpacetime` and
`TangentSpacetime` report `Metric::Lorentzian(4)`, (−,+,+,+). `SpaceTimeKind` forwards to the
variant it holds.

This is what lets one context hold nodes on different manifolds and still answer correctly for each:

```rust
let signature = match contextoid.vertex_type() {
    ContextoidType::SpaceTempoid(st) => st.metric(),
    _ => panic!("not a spacetime node"),
};

if is_lorentzian(&signature) { /* relativistic step */ }
```

It composes with `detect_convention` and `is_lorentzian` from `deep_causality_metric` directly,
because a node supplies the same `Metric` those functions already take.

**If you implement `SpaceTemporal` on your own type, you must now also implement
`MetricSignature`.** It is a supertrait.

`TangentSpacetime` keeps `Lorentzian(4)` because `update_metric_tensor` accepts only a symmetric
tensor of that signature and refuses any other with an `UpdateError`.

## `QuaternionSpace` is gone

A quaternion is a rotation operator acting on a space, not a space of its own. An orientation
belongs beside a position in a pose; a context locates things, and a rotation locates none.

`SpaceKind::Quaternion` went with the type. If you were storing orientation in a context, put it in
your own type alongside the position it belongs to.

## A context with no spatial extent says so

`NoSpace<R>`, `NoTime` and `NoSpaceTime<R>` are zero-sized and fill the spatial, temporal and
spacetime slots of a context whose graph holds no node of that kind. A context that holds a root,
some data and a clock names what it lacks instead of naming types its graph never holds:

```rust
type ClockContext = Context<
    Data<FloatType>,
    NoSpace<FloatType>,
    NewtonianTime<FloatType>,
    NoSpaceTime<FloatType>,
>;
```

This is the common shape. Across this workspace the contextoid variants are used Root 45, Datoid 47,
Tempoid 15, Spaceoid 5, SpaceTempoid 5.

## Identifiers moved out of core

`ContextId` and `ContextoidId` are context vocabulary, so they live in the context crate:

```rust
// before
use deep_causality_core::{ContextId, ContextoidId};
// or
use deep_causality::{ContextId, ContextoidId};

// after
use deep_causality_context::{ContextId, ContextoidId};
```

Both are `IdentificationValue`, whose width is core's — `Identifiable::id` returns it, and every
context type implements `Identifiable`, so the three cannot disagree.

## Edges carry their relation

`Context`'s backing graph stored edge weights as `u64`, casting `RelationKind` on the way in. Nothing
read them back, because the only edge accessor returned a bool. A context therefore lost an edge's
relation on a round trip through its own graph.

The graph now carries `RelationKind` itself, and two new accessors read it:

```rust
fn get_edge(&self, a: usize, b: usize) -> Option<&RelationKind>;
fn extra_ctx_get_edge(&self, a: usize, b: usize) -> Option<&RelationKind>;
```

Writing edges is unchanged. `RelationKind` gained a `Default` on `Datial` to satisfy the graph's
weight bound; edge construction always supplies a relation, so the default never reaches an edge
through this crate's API.

**If you implement `ContextuableGraph` or `ExtendableContextuableGraph` yourself, each gained one
required method.**

## `GeoSpace` carries its vertical datum

An altitude is a number and a reference, and the number alone locates nothing. `GeoSpace` documented
one reference for the whole type, so a pressure altitude or a height above terrain had nowhere to say
so.

```rust
// before
GeoSpace::new(1, 52.52, 13.40, 34.0)

// after: the datum is named, and a latitude outside [-90, 90] is refused
GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::WGS84)?
```

`VerticalDatum` has five members, grouped by the height type each belongs under. `WGS84` is an
ellipsoidal height. `EGM96` and `EGM2008` are gravity-related heights. `ISA` and `Terrain` are
neither — a pressure altitude is a pressure in length units, and a terrain height is measured against
a surface that varies with position and with the model that produced it.

`Display` prints the datum, so two positions with equal altitudes against different datums no longer
render identically.

## The fixed uncertain aliases are gone

`UncertainData` and `UncertainBoolData` are generic in their scalar. Two aliases welded them back:

```rust
// before
UncertainFloat64Data::new(id, data)
UncertainBooleanData::new(id, data)

// after
UncertainData::<FloatType>::new(id, data)
UncertainBoolData::<FloatType>::new(id, data)
```

Their modules were renamed to match the types they hold: `data_uncertain_f64` is now
`uncertain_data`, and `data_uncertain_bool` is now `uncertain_bool_data`. The old names described a
scalar the types no longer fix.

## `FloatType` is declared per crate

`deep_causality`, `deep_causality_context` and `deep_causality_ethos` each declare their own, instead
of re-exporting `deep_causality_core`'s.

```rust
// before — one alias, silently shared
use deep_causality::FloatType;

// after — import from the crate you are working in
use deep_causality_context::FloatType;
```

Changing one crate's working scalar no longer silently reconfigures another. A mismatch between two
crates' choices is a type error at the boundary, which is the point.

Examples already follow this rule and declare their own before `main`.

---

## What did not change

`TimeScale`'s variants keep their spellings, including the inconsistent `Nanoseconds` and
`Microseconds` beside `Millisecond` and `Second`. Those names are a contract that projections match
on, so tidying one is a data migration elsewhere rather than a rename here.

`Context` keeps four explicit type parameters. Bundling them behind a single frame type was built and
then removed: it collapsed 24 generic declarations and changed nothing else, and an opaque bundle can
encode assumptions a reader cannot see. A library handling four generic parameters should show them.
