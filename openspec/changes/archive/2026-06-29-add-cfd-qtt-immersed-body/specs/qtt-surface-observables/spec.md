## ADDED Requirements

### Requirement: Drag and lift from the penalization-force contraction

The `solvers/qtt` module SHALL compute the force the immersed body exerts on the fluid as a tensor-train
contraction of the body mask with the velocity deficit: `F = (1/η) ∫ χ_body ⊙ (u_body − u) dV` per
component (via the train `inner` product and the cell volume — no surface reconstruction). It SHALL
expose the nondimensional drag and lift coefficients `C_d = F_x / (½ ρ U² D)`, `C_l = F_y / (…)`, through
the QTT observe set and `Report`.

#### Scenario: Drag is a single contraction of the mask and the field
- **WHEN** drag/lift are requested for a penalized flow state
- **THEN** they are computed from the mask–velocity-deficit contraction, with no cut-cell surface or
  boundary-fiber reconstruction

#### Scenario: Drag series is collected by the pipeline
- **WHEN** a QTT march is composed with the drag observable and an immersed body, then run
- **THEN** the report carries a drag (and lift) series, one sample per step

### Requirement: Neutral wall heat flux from a penalized passive scalar

The `solvers/qtt` module SHALL optionally advect–diffuse a passive scalar `T` on the same rollout, with
the body penalized to a wall temperature `T_w`, and SHALL expose the wall heat flux as the penalization
heat integral `Q = (1/η) ∫ χ_body ⊙ (T_w − T) dV` (the same contraction shape as drag). This SHALL be
**neutral** (no chemistry) — the seam the Gap-2 reacting energy equation replaces.

#### Scenario: Wall heat flux responds to the thermal field
- **WHEN** the passive scalar is advected past a body held at a wall temperature different from the flow
- **THEN** a non-zero wall heat-flux observable is produced, computed as the mask–temperature-deficit
  contraction

### Requirement: Self-verifying immersed validation (no-slip, accuracy-vs-bond, DEC cross-reference)

The immersed QTT solver SHALL be validated as a self-verifying example that gates the method's
correctness invariants and **exits nonzero** on a break: (a) **no-slip** — the velocity inside the body
falls to the penalization floor; (b) **accuracy-vs-bond** — the drag coefficient **converges** as the
round bond cap is raised (the headline QTT-CFD metric); and (c) **physical drag** — the streamwise drag
on a body in a free-stream is positive and finite. The committed DEC cylinder `C_d` SHALL be **reported as
a cross-reference**, disclaimed for the periodic-blockage difference (the periodic penalized box is not
the DEC solver's inflow/outflow/far-field configuration, and the penalization-integral force is inflated
by the smoothing skirt and blockage, so an absolute match is not claimed — the convergence *trend* is the
verification result, not the absolute number).

#### Scenario: No-slip holds and drag is physical
- **WHEN** a cylinder in a periodic free-stream is marched to a quasi-steady state
- **THEN** the velocity inside the body is at the penalization floor, and the streamwise drag coefficient
  is positive and finite

#### Scenario: Drag converges as the bond cap rises
- **WHEN** the run is repeated at increasing bond caps
- **THEN** the change in the drag coefficient between successive caps shrinks — the drag converges as the
  tensor-train is allowed more rank (the accuracy-vs-bond trade-off)
