## ADDED Requirements

### Requirement: Surface-force diagnostic on an immersed cut body
The solver SHALL provide a read-only diagnostic that integrates the **pressure** force on an
immersed cut body — `F_p = −∮_S p n dA` — over the body's `CutFaceFragment`s (each carrying area and
outward normal), given a per-cell pressure, and a `∮ n dA` closure check on the fragment normals.
A helper SHALL convert a force component to a nondimensional coefficient `C = F / (½ρU²A)`. The
diagnostic SHALL be precision- and dimension-generic (`R: RealField`, any `D`), operate on a field
snapshot, and not be on the per-step path. The **viscous (friction) traction**
(`μ(∇u+∇uᵀ)·n`, from a `sharp`+gradient reconstruction) extends it and is added/validated with the
isolated-cylinder drag (it is verified against the reference drag, not a fast analytic gate).

#### Scenario: No net force in a uniform pressure field
- **WHEN** the surface force is integrated over a closed immersed body in a uniform-pressure, zero-velocity field
- **THEN** the net force is zero to rounding

#### Scenario: Force in a linear pressure gradient
- **WHEN** the pressure force is integrated over an immersed body in a field with a known linear pressure gradient
- **THEN** the computed force matches the analytic `−∇p · V_solid` within the cell-center (O(h)) approximation

#### Scenario: Coefficients from a force
- **WHEN** a force vector is converted with a reference velocity and length
- **THEN** the drag and lift coefficients are `F·x̂ / (½ρU²L)` and `F·ŷ / (½ρU²L)`
