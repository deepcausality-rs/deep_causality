## ADDED Requirements

### Requirement: Small-cut-cell stabilization
The solver SHALL apply a small-cut-cell stabilization that restores a usable CFL bound for
arbitrarily small cut volumes, so that the presence of a vanishing cut does not force the
global time step toward zero or destabilize the march. The chosen mechanism (cell-merging
or flux-redistribution) SHALL be selected on measured cylinder-case accuracy versus
complexity and recorded in the change's design before the validation gate closes.

#### Scenario: A tiny cut cell marches without CFL collapse
- **WHEN** a flow is marched on a lattice containing a deliberately tiny cut volume with stabilization enabled
- **THEN** the march completes at the unstabilized full-cell time step without a CFL abort, while an unstabilized control run on the same configuration aborts

### Requirement: Stabilization does not break conservation
Stabilization SHALL preserve discrete conservation and the divergence-free property of the
projected velocity, since the combinatorial exterior derivative is independent of the cut
geometry. Any redistribution SHALL be conservative (it moves a conserved update between
cells without creating or destroying it).

#### Scenario: Redistribution conserves the global update
- **WHEN** a conservative update is redistributed from a small cut cell over its neighbors
- **THEN** the global sum of the conserved quantity is unchanged to rounding and the projected field remains divergence-free at the solve's exactness
