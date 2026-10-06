# deep_causality_context

The context layer of [DeepCausality](https://deepcausality.com/): the `Context` hypergraph, the
contextoids that populate it, the context node types for data, space, time and spacetime, and the
traits that describe them.

## What this crate is for

The context holds what a causal model reasons about: a typed environment that a causaloid or a
causal-monad chain reads as it runs.

A consumer of `deep_causality_core` alone pays nothing for context; a model that needs context
declares this dependency and imports from it. `deep_causality` depends on this crate
unconditionally, so a causaloid model gets context either way. The crates that depend on this one
directly are the crates that model a context.

```toml
[dependencies]
deep_causality_context = "0.1"
```

## Using context with the causal monad

With `deep_causality_core` and this crate, a monad chain carries a typed context without pulling
in the causaloid stack:

```rust,ignore
use deep_causality_context::BaseContext;
use deep_causality_core::{PropagatingEffect, PropagatingProcess};

fn start(ctx: BaseContext) -> PropagatingProcess<f64, (), BaseContext> {
    PropagatingProcess::with_state(PropagatingEffect::pure(0.0), (), Some(ctx))
}
```

## Contents

* `Context` — the context hypergraph, with extra contexts and data/time indices.
* `Contextoid` / `ContextoidType` — the nodes it holds.
* Context node types — `Data`, `Root`, and the space, time and spacetime families:
  `EuclideanSpace`, `EcefSpace`, `NedSpace` and `GeoSpace`; `NewtonianTime`, `MinkowskiTime`,
  `DiscreteTime`, `EntropicTime` and `SymbolicTime`; `GalileanSpacetime`, `NewtonianSpacetime`,
  `MinkowskiSpacetime` and `TangentSpacetime`; `SpaceKind`, `TimeKind` and `SpaceTimeKind`, enums
  over the family's types (`TimeKind` leaves out `SymbolicTime`); and `NoSpace`, `NoTime` and
  `NoSpaceTime` for a slot the context does not use. Each type in the geometry table below
  documents the definition it follows and its source, with copies of the open-access sources in
  `papers/`.
* `CausalSetSpacetime`, an element of a causal set with the elements that precede it. It holds the
  order alone and implements none of the coordinate or time traits, so it is not a context node
  type.
* `Contextuable`, `Datable`, `Spatial`, `Temporal`, `SpaceTemporal`, `Coordinate`, `Distance`,
  `MetricSignature`, `MetricTensor4D` and the indexable traits.
* `Adjustable<T>` for nodes that update from an `ArrayGrid<T, ..>`, and `UncertainAdjustable` for
  nodes that update from their own associated `Self::Data`.
* `Storable`, `ContextStore`, `StoreError`, `Context::snapshot`, `Context::restore` and
  `Context::apply` for persistence, with the `SubstrateContext` and `SubstrateContextoid` aliases.
* `RelationKind`, `TimeScale`, `VerticalDatum` and `SubstrateRef`, re-exported from
  `deep_causality_context_store`, so a model names them from this crate alone.

## Geometry

Each type below models one geometry, named as the cited source names it.

| Type | Geometry | Distance or interval | Source |
|---|---|---|---|
| `EuclideanSpace` | Three-dimensional Euclidean space | `√(Δx² + Δy² + Δz²)` | Weatherall 2021, §2 |
| `EcefSpace` | WGS 84 Earth-centred, Earth-fixed Cartesian coordinates, metres | Euclidean norm | NGA 2014, §2.1; IOGP 2012, §2.2.1 |
| `NedSpace` | Local topocentric frame, north-east-down, metres | Euclidean norm | IOGP 2012, §2.2.2 |
| `GeoSpace` | WGS 84 geodetic latitude, longitude, altitude against a `VerticalDatum` | Straight line through the geocentric conversion; NaN unless both altitudes are WGS 84 ellipsoidal heights | NGA 2014, Tables 3.1 and 3.5; IOGP 2012, §2.2.1 |
| `NewtonianTime` | Absolute time of a classical spacetime | — | Weatherall 2021, §3; Malament 2012, §4.1 |
| `MinkowskiTime` | Coordinate time of an inertial frame of Minkowski spacetime | — | Carroll 1997, ch. 1 |
| `GalileanSpacetime` | Galilean spacetime: affine, absolute time, Euclidean space at each instant, no rest frame | Spatial distance between simultaneous events; NaN between events at different times | Weatherall 2021, §3; Malament 2012, §4.1 |
| `NewtonianSpacetime` | Newtonian spacetime: Galilean spacetime with absolute space, in coordinates at rest in it | Spatial distance between any two events, a pseudometric | Weatherall 2021, §4; Stein 1967; Malament 2012, Prop. 4.1.2 |
| `MinkowskiSpacetime` | Flat Minkowski spacetime, east-coast signature (−, +, +, +) | `s² = −(cΔt)² + Δx² + Δy² + Δz²`; NaN for a time scale that names no duration | Malament 2012, §2.1; Carroll 1997, eqs. (1.3), (1.8) |
| `TangentSpacetime` | An event of a relativistic spacetime with a tangent vector and the metric tensor there | `g_ab Δxᵃ Δxᵇ` under the event's own tensor | Malament 2012, §2.1; Poisson, Pound & Vega 2011, §3.1 |
| `CausalSetSpacetime` | An element of a causal set and its past | — | Sorkin 2003, p. 5 |

`GalileanSpacetime`, `NewtonianSpacetime`, `MinkowskiSpacetime` and `TangentSpacetime` index their
coordinates `0 => t, 1 => x, 2 => y, 3 => z`, the order of Carroll 1997, eq. (1.5), with `t` at
index 0 where Carroll places `x⁰ = ct`. Each reports through `MetricSignature` the signature of the
metric it measures with, generator `i` its coordinate `i`: `Metric::Lorentzian(4)` for the
relativistic types, and `Metric::PGA(4)`, the spatial metric's (0, +, +, +), for the classical ones.
`CausalSetSpacetime` has no coordinates, no time and no signature: it holds the order alone.
`TangentSpacetime::update_metric_tensor` accepts only a finite, symmetric tensor of signature
(−, +, +, +). It counts the signs of the eigenvalues by symmetric block elimination, which keeps
them by Sylvester's law of inertia (Horn & Johnson 2013, Theorem 4.5.8).

`NoSpace`, `NoTime` and `NoSpaceTime` fill the spatial, temporal and spacetime slots of a context
whose graph holds no node of that kind.

### References

Open-access sources are copied into `papers/`.

- Carroll, S. M. (1997). *Lecture Notes on General Relativity*. arXiv:gr-qc/9712019.
  `papers/carroll_1997_lecture_notes_on_general_relativity_arXiv_gr-qc_9712019.pdf`
- Horn, R. A., & Johnson, C. R. (2013). *Matrix Analysis*, 2nd ed. Cambridge University Press.
- IOGP (2012). *Geomatics Guidance Note 7, part 2: Coordinate Conversions and Transformations
  including Formulas*, OGP Publication 373-7-2.
  `papers/iogp_2012_373-7-2_coordinate_conversions_and_transformations.pdf`
- Malament, D. B. (2012). *Topics in the Foundations of General Relativity and Newtonian
  Gravitation Theory*. University of Chicago Press.
- National Geospatial-Intelligence Agency (2014). *World Geodetic System 1984: Its Definition and
  Relationships with Local Geodetic Systems*, NGA.STND.0036_1.0.0_WGS84.
  `papers/NGA.STND.0036_1.0.0_WGS84.pdf`
- Poisson, E., Pound, A., & Vega, I. (2011). The Motion of Point Particles in Curved Spacetime.
  *Living Reviews in Relativity*, article 7. doi:10.12942/lrr-2011-7. arXiv:1102.0529.
  `papers/poisson_pound_vega_2011_motion_of_point_particles_in_curved_spacetime_arXiv_1102.0529.pdf`
- Sorkin, R. D. (2003). Causal Sets: Discrete Gravity. arXiv:gr-qc/0309009.
  `papers/sorkin_2003_causal_sets_discrete_gravity_arXiv_gr-qc_0309009.pdf`
- Stein, H. (1967). Newtonian space-time. *The Texas Quarterly* 10, 174–200.
- Weatherall, J. O. (2021). Classical Spacetime Structure. In E. Knox & A. Wilson (Eds.), *The
  Routledge Companion to Philosophy of Physics*, pp. 33–45. Routledge. arXiv:1707.05887.
  `papers/weatherall_2017_classical_spacetime_structure_arXiv_1707.05887.pdf`

## Build configurations

| Configuration | Cargo features | Supported |
|---|---|:-:|
| `std`, hosted | default | ✓ |
| `no-std` with a heap | `default-features = false, features = ["no-std"]` | ✓ |
| `no-std` on `core` alone, no heap | – | – |

Without `std` the crate needs a heap: the program must define a `#[global_allocator]`. `os-random` is host-only. Without `std` the id index is a `BTreeMap`.

CI builds every supported bare-metal configuration for `thumbv7em-none-eabihf`.
[README_NO_STD.md](https://github.com/deepcausality-rs/deep_causality/blob/main/README_NO_STD.md) describes the build configuration of every crate in the workspace.

## Configuration

The context builds without the standard library.

* `default`: Enables `std`.
* `std`: Uses the standard library (includes `alloc`).
* `no-std`: Builds without the standard library (includes `alloc`); floating-point math comes from
  `libm`. Select with `default-features = false, features = ["no-std"]`.
* `alloc`: Allocation (Vec, String). Enabled by both `std` and `no-std`; on its own it selects no
  floating-point backend and does not build.
* `os-random`: Seeds `deep_causality_rand` from the operating system. Host-only.

The context finds a node by its id through an index map. Under `std` that map is a `HashMap`.
Without `std` there is no entropy to seed the default hasher, so it is a `BTreeMap`, and a lookup
costs O(log n): at 10 000 nodes about five times a hashed lookup (measured on an M3 Max).

## The scalar is a parameter

Spatial, spacetime and real-valued temporal types carry the scalar they measure in, bounded once on
`deep_causality_algebra::RealField`. The crate names no concrete float outside its ready-made
aliases, so a type satisfying that bound works without an entry being added for it:

```rust,ignore
let narrow: EuclideanSpace<BFloat16> = EuclideanSpace::new(1, x, y, z);
let wide: EuclideanSpace<Float106> = EuclideanSpace::new(1, x, y, z);
```

`BaseContext`, `BaseContextoid`, `UniformContext` and `UniformContextoid` stay concrete, so reaching
the context through one of those needs no annotation.

Tick-based times keep their integers. A tick counts, and its failure mode is overflow rather than
rounding, so widening its significand buys nothing.

## Persistence

A context lives in memory. `deep_causality_context_store` describes what a store keeps of one;
this crate projects in both directions.

`Context::snapshot` writes the base graph and every extra context into a `ContextSnapshot`, in
canonical order, and `Context::restore` builds a context from one. A `ContextStore` puts the
projection and a backend together:

```rust,ignore
use deep_causality_context::{ContextStore, UniformContext};

let store = ContextStore::new(backend);
let ctx: UniformContext = store.hydrate(&container).await?;
// explore a branch in memory, then keep it:
let branch_id = store.store_branch("branch", &branch).await?;
```

`store_branch` links every node the store already holds under the same record, creates every node
it does not hold, and creates under a fresh identifier every node the store holds under a different
record, all in one `commit`, so a refused store leaves nothing behind. A stored branch is a record
of a world, not a continuation of one; to keep exploring it, hydrate it. With a backend that
streams, `subscribe` returns a context and the stream of changes to it, and `Context::apply` applies
each event idempotently.

### Payloads

A data payload persists once its type implements `Storable`, which maps it onto a `DataRecord`
value tree. A struct maps to `Fields`, one entry per field:

```rust
use deep_causality_context::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

#[derive(Debug, Clone, Default, PartialEq)]
struct Reading {
    temperature: f64,
    samples: u64,
}

impl Storable for Reading {
    fn to_record(&self) -> DataRecord {
        DataRecord::Fields(vec![
            ("temperature".to_string(), self.temperature.to_record()),
            ("samples".to_string(), self.samples.to_record()),
        ])
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        let DataRecord::Fields(entries) = record else {
            return Err(ProjectionError::WrongPayload(id, "Fields", record.kind_name()));
        };
        let field = |name: &'static str| {
            entries
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.clone())
                .ok_or(ProjectionError::MissingField(id, name))
        };
        Ok(Self {
            temperature: f64::from_record(id, field("temperature")?)?,
            samples: u64::from_record(id, field("samples")?)?,
        })
    }
}
```

`Data<Reading>` then snapshots, restores, stores and streams like `Data<f64>`. The crate implements
`Storable` for `f32`, `f64`, `u64`, `i64`, `bool`, `String`, `SubstrateRef`, `Vec<T>`,
`Option<T>`, `Float106` and `BFloat16`.

A store that holds structure and not values takes a data node as a `SubstrateRef`. `SubstrateContext`
names that shape, and `ContextStore::create_node_via` and `hydrate_via` move the values through a
`Substrate` on the way in and out, so a value-typed context reaches such a store unchanged.

### Extra contexts

An extra context has a name and an identifier. `extra_ctx_add_new(name, capacity, default)`
allocates the identifier as one past the highest ever held; `extra_ctx_add_new_with_id` takes one, and
refuses 0. In a store an extra is a separate container the base references, so a hydrated context
holds each referenced container as an extra under that container's identifier and name, and a
stored branch's extras become containers of their own. The store assigns container identifiers:
an extra added locally is not a container of the store, so a store event naming its identifier
as a container never reaches it, and `Context::apply` refuses an attachment under that identifier with
`ProjectionError::Identity` rather than hold two containers under one identifier.

### Precision in the store

Every scalar field of a record is `f64`. A node type over a wider or narrower scalar narrows to
`f64` on the way out and lifts on the way in, so `EuclideanSpace<Float106>` coordinates restore at
double precision and `EuclideanSpace<BFloat16>` coordinates restore rounded as that type rounds.
Ticks stay `u64`. The exception is a `Float106` data payload: `Storable` stores its two halves as
`Fields [("hi", Number), ("lo", Number)]` and restores every bit.
An `f32` payload widens exactly and reads back exactly; a `Number` it did not write rounds to
the nearest `f32`, a magnitude too small for the smallest subnormal becomes a zero of the same sign, and a
finite value beyond the `f32` range is `ProjectionError::Scalar`.

## Licence

MIT. See [LICENSE](LICENSE).
