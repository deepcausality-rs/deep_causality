## ADDED Requirements

### Requirement: The blended coordinate map is assembled through a fluent builder

The blended coordinate-map configuration SHALL be assembled through a fluent builder whose `build()`
validates, rather than through a positional constructor taking the lattice extents, the fan
geometry, and the blend parameter as seven unnamed arguments.

The builder SHALL name each parameter at its call site (the `2^Lx × 2^Ly` lattice, the radial range,
the angular range, and the blend parameter `λ`), and SHALL reject a configuration the map cannot be
built from before the metric assembly runs. The positional constructor SHALL be crate-private.

The map configuration remains a **configuration input**, not a `CfdConfigBuilder` entry: it is read
by a compressible march rather than run by `CfdFlow`.

#### Scenario: A blended map is configured by name

- **WHEN** a caller configures a body-fitted fan or a Cartesian-capture rectangle
- **THEN** each parameter is named at the call site, and the blend parameter `λ` is distinguishable
  from the angular range without consulting the constructor signature

#### Scenario: An invalid blend configuration is refused at build

- **WHEN** the configuration carries a non-positive radial or angular range, or a blend parameter
  outside `[0, 1]`
- **THEN** `build()` returns `PhysicsError` naming the violated condition, before any metric field
  is assembled

#### Scenario: The map stays outside the config-builder entry set

- **WHEN** a caller looks for a `CfdConfigBuilder` entry for the coordinate map
- **THEN** none exists, and the map is constructed through its own builder as a configuration input
