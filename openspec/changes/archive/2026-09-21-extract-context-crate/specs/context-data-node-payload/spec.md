## ADDED Requirements

### Requirement: `Data<T>` requires `Clone` of its payload, not `Copy`

`Data<T>` SHALL bound its payload parameter as `T: Default + Clone + PartialEq` and SHALL NOT
require `T: Copy`, so that any cloneable value can be carried as a data node.

Nothing above `Data<T>` needs `Copy`. `Datable` states no bound on `Self::Data`; `Context` requires
`D: Datable + Clone`; and the `Copy`, `Hash` and `Eq` derives on `Contextoid` are conditional and
already inapplicable to `BaseContext`, whose `EuclideanSpace` parameter derives only `Debug`,
`Clone` and `PartialEq`. `Data<T>` is the only context node type that constrains a payload
parameter at all — `EcefSpace`, `GeoSpace`, `EuclideanSpace`, `SpaceKind`, `SymbolKind`,
`CausalSetSpacetime` and `ConformalSpacetime` are `Clone`-only.

`Datable::get_data` SHALL return a clone of the payload rather than a copy of it.

#### Scenario: A sequence payload is expressible

- **WHEN** a `Data<Vec<f64>>` is constructed with a time series and read back through `Datable`
- **THEN** it compiles and `get_data` returns the series

#### Scenario: Existing payloads are unaffected

- **WHEN** every `Data<T>` instantiation in the workspace is built after the relaxation
- **THEN** all of them still compile, because the bound was widened and every `T` in use satisfies
  `Default + Clone + PartialEq`

#### Scenario: A sequence-valued data node lives in a context

- **WHEN** a `Contextoid` holding a `Data<Vec<f64>>` is added to a `Context` and retrieved
- **THEN** the context accepts it, because `Context` requires only `D: Datable + Clone`

### Requirement: `Copy` is required by the `Adjustable` impl, not by the type

The `Adjustable<T> for Data<T>` impl SHALL continue to require `T: Copy`, because it takes an
`ArrayGrid<T, WIDTH, HEIGHT, DEPTH, TIME>` and `ArrayGrid` is backed by fixed-size arrays and bounds
`T: Copy + Default`.

That is the only genuine origin of the requirement in this layer: of the 28 `Adjustable` impls
across the context node types, 22 take `ArrayGrid<f64, …>` and 4 take `ArrayGrid<u64, …>`; only the
two in `Data` take `ArrayGrid<T, …>`, so `Data<T>` is the sole path by which `ArrayGrid`'s bound
reaches a type parameter.

#### Scenario: An adjustable data node still adjusts

- **WHEN** a `Data<f64>` is updated and adjusted through an `ArrayGrid<f64, …>`
- **THEN** both operations behave exactly as they did before the relaxation, including the
  zero-value rejection in `update`

#### Scenario: A non-`Copy` payload is not adjustable

- **WHEN** a `Data<Vec<f64>>` is used
- **THEN** it satisfies `Datable` and can be carried in a `Context`, and it does not satisfy
  `Adjustable`, which is correct because there is no array-backed grid of `Vec<f64>`

### Requirement: `Data<T>` is no longer `Copy`

`Data<T>` SHALL NOT derive `Copy`, and every call site that relied on an implicit copy SHALL be
migrated to an explicit `clone()`.

Every consumer that required `Copy` of a `Datable` only incidentally SHALL drop it too. Three such
sites were found during implementation, each of which already required `Clone` beside it: the
`Display` impl for `Data<T>`, the `Model` type, and the generative `Interpreter`. None of them
copies a data node; they carry or format it.

Nothing above `Data` changes shape as a result: `BaseContextoid` is already not `Copy` because
`EuclideanSpace` is `Clone`-only.

#### Scenario: The workspace builds after the derive is removed

- **WHEN** `bazel test //...` runs after the `Copy` derive is removed from `Data<T>`
- **THEN** every target builds and passes, with each former implicit copy replaced by an explicit
  `clone()`

#### Scenario: A `Model` over a non-`Copy` data node still evolves

- **WHEN** a `Model` whose context uses a non-`Copy` `Data<T>` is passed to `evolve`
- **THEN** it compiles and runs, because `Model` and the generative interpreter require `Clone` of
  the data node rather than `Copy`

#### Scenario: `Contextoid` shape is unchanged for the canonical context

- **WHEN** `BaseContextoid` is checked for `Copy` before and after the change
- **THEN** it is not `Copy` in either case, so no consumer of `BaseContext` observes a difference
