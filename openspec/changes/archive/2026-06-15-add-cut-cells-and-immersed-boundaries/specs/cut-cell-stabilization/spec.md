## ADDED Requirements

### Requirement: Small-cut-cell stabilization
The solver SHALL provide a small-cut-cell stabilization mechanism (a volume-fraction floor on the
cut star, `CutCellRegistry::with_cell_merging` — the Berger–Helzel cell-merging family). The
mechanism selected, and the **finding** that justified it, SHALL be recorded in the change's design
before the validation gate closes.

**Finding (recorded in design D4):** the classic small-cut-cell CFL instability does **not** arise in
this DEC formulation. The consistent metric clip cancels in the codifferential `δ = M⁻¹ ∂ M`, so an
arbitrarily small cut volume does not stiffen the explicit operator, and the march is inherently
stable at the full-cell time step even with no stabilizer. Cell-merging is therefore selected not for
explicit CFL stability but to improve the masked-CG **projection conditioning** (a vanishing dual
mass otherwise degrades the constrained Leray solve). Flux-redistribution (Colella–Graves–Modiano) is
assessed and rejected: it needs a per-cell conservative update the projected-rate RK4 formulation does
not expose.

#### Scenario: A tiny cut cell marches without CFL collapse
- **WHEN** a flow is marched on a lattice containing deliberately tiny cut volumes at the full-cell
  time step
- **THEN** the march is finite and non-amplifying with no CFL abort, **with or without** the
  stabilizer (the inherent-stability finding), and enabling cell-merging preserves that march while
  improving the projection's divergence residual

### Requirement: Stabilization does not break conservation
Stabilization SHALL preserve discrete conservation and the divergence-free property of the
projected velocity, since the combinatorial exterior derivative is independent of the cut
geometry. The volume-fraction floor adjusts only the metric (Hodge star) masses; the exterior
derivative and the Leray projection's exact divergence-freeness are untouched.

#### Scenario: Cell-merging preserves divergence-freeness
- **WHEN** the cell-merging floor inflates a sliver cut cell's dual mass on a marched flow
- **THEN** the projected field remains divergence-free at the solve's exactness and discrete
  conservation is preserved (only an `O(min_fraction)` geometric error is introduced, localised to
  the floored cells)
