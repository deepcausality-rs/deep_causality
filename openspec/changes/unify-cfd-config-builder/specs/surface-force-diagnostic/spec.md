## ADDED Requirements

### Requirement: The drag observable reports the pressure and friction contributions

The marching pipeline's drag observable SHALL offer an opt-in that reports the **pressure** and
**viscous (friction)** contributions to the drag coefficient as separate series, alongside the
combined series it already emits.

The split SHALL come from the same `pressure_surface_force` and `viscous_surface_force` integrations
the combined coefficient is summed from, so the two contributions add to the combined value by
construction and no second force path exists. The opt-in SHALL require an immersed body, as the
combined drag observable already does, and SHALL be off by default so existing reports are unchanged.

This makes the pressure/friction ratio a reportable quantity of the configured march. It is the split
the isolated-cylinder rung compares against Dröge & Verstappen (2005), Table II, and it was
previously reachable only by calling the two surface-force functions directly.

#### Scenario: The split sums to the combined coefficient

- **WHEN** a march is configured with the drag split opt-in
- **THEN** the report carries a pressure series and a friction series in addition to the combined
  drag series, and at every step the two contributions sum to the combined value to rounding

#### Scenario: The split is off by default

- **WHEN** a march is configured with the drag observable and no split opt-in
- **THEN** the report carries the combined drag and lift series only, unchanged from before

#### Scenario: The split needs a body

- **WHEN** the split is requested on a mesh carrying no immersed body
- **THEN** the run returns the same error the combined drag observable returns for a missing body
