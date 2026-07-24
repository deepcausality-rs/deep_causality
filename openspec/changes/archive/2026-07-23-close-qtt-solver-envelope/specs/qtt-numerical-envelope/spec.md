## ADDED Requirements

### Requirement: The QTT solvers refuse configurations outside their numerical envelope

The QTT solver constructors SHALL reject a configuration they cannot integrate, returning
`PhysicsError::PhysicalInvariantBroken` naming the violated limit and the values that violated it —
the contract the DEC family already honours in `dec_ns_solver::cfl_check`.

At minimum the envelope SHALL cover the penalization parameter `η > 0` (finite), the time step against
the explicit-stability limit for the penalization term (`dt ≤ 2η` for forward Euler on
`du/dt = −u/η`) and against the diffusive limit (`dt ≤ dx²/(4ν)` for 2-D FTCS), and the viscosity
`ν ≥ 0` (finite). The grid spacing SHALL be positive and finite.

`QttImmersed2d::new` and `QttIncompressible2d::new` currently validate nothing — they destructure
their arguments straight into `Ok(Self { .. })` — and no layer above them fills the gap:
`QttMarchConfigBuilder::build` checks only that the grid is `2^L` and that seed shapes match. With
`η = 0`, `−1/η` evaluates to `−inf` and the march proceeds. One solver family in this crate refuses an
out-of-envelope configuration; its sibling returns numbers.

#### Scenario: A non-positive penalization parameter is refused

- **WHEN** a QTT immersed solver is constructed with `η = 0` or `η < 0`
- **THEN** construction returns an error naming the parameter, rather than producing a solver whose
  forcing term is infinite

#### Scenario: A time step outside the stability limit is refused

- **WHEN** `dt` exceeds the explicit-stability limit for the penalization term, or the diffusive limit
  for the given `ν` and `dx`
- **THEN** construction returns an error stating which limit was exceeded, the configured value, and
  the limit's value

#### Scenario: The diagnostic names the limit and the values

- **WHEN** any envelope check fails
- **THEN** the message identifies the violated limit and reports both the configured and the limiting
  value, so the caller can act without reading the solver source

#### Scenario: An in-envelope configuration is unaffected

- **WHEN** a configuration inside the envelope is constructed
- **THEN** it succeeds and marches exactly as before, with no change in results

### Requirement: Positivity guards behave identically at every supported precision

A numerical guard SHALL behave the same at every precision the crate documents as supported (`f32`,
`f64`, `Float106`). A guard whose threshold is lifted from an `f64` literal SHALL NOT silently vanish
when the working scalar cannot represent it.

The four compressible marchers share `let tiny = R::from_f64(1e-300).unwrap_or_else(R::zero)`. At
`R = f32` the `num-traits` `f64 → f32` conversion is an infallible cast, so `from_f64` returns
`Some(0.0)` rather than `None`, the `unwrap_or_else` fallback never fires, and `tiny` is exactly zero —
the floor disappears in a supported precision with no diagnostic.

#### Scenario: A guard threshold is representable in the working precision

- **WHEN** a guard threshold is established for a given scalar type
- **THEN** it is representable and non-degenerate in that type, or its unrepresentability is an error
  rather than a silent zero

#### Scenario: The same input is judged the same way at f32 and f64

- **WHEN** a state that should trip a positivity guard is presented at `f32` and at `f64`
- **THEN** both trip it
