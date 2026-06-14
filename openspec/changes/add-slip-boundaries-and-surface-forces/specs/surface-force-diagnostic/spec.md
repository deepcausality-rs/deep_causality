## ADDED Requirements

### Requirement: Surface-force diagnostic on an immersed cut body
The solver SHALL provide a read-only diagnostic that integrates the hydrodynamic force on an
immersed cut body — `F = ∮_S (−p n + μ (∇u + ∇uᵀ)·n) dA` — over the body's `CutFaceFragment`s
(each carrying area and outward normal), using the projection's recovered pressure potential (minus
the dynamic head) and the edge-cochain velocity gradient. It SHALL return the force vector and the
pressure/viscous split, and a helper SHALL convert a force to nondimensional `C_d`/`C_l` given a
reference velocity and length. Precision-generic over `R: RealField`; it operates on a
divergence-free field snapshot and is not on the per-step path.

#### Scenario: No net force in a uniform pressure field
- **WHEN** the surface force is integrated over a closed immersed body in a uniform-pressure, zero-velocity field
- **THEN** the net force is zero to rounding

#### Scenario: Exact force in a linear pressure gradient
- **WHEN** the surface force is integrated over an immersed body in a field with a known linear pressure gradient
- **THEN** the computed force matches the analytic surface integral to rounding, with the pressure and viscous contributions reported separately

#### Scenario: Coefficients from a force
- **WHEN** a force vector is converted with a reference velocity and length
- **THEN** the drag and lift coefficients are `F·x̂ / (½ρU²L)` and `F·ŷ / (½ρU²L)`
