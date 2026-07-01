# qtt-shock-fitting Specification

## Purpose
TBD - created by archiving change add-cfd-compressible-qtt-marcher. Update Purpose after archive.
## Requirements
### Requirement: Fitted shock interface with exact Rankine–Hugoniot jump

`deep_causality_cfd` SHALL represent the bow shock as a **tracked moving interface** (a low-dimensional
surface unknown), not as a captured gradient, and SHALL apply the **exact Rankine–Hugoniot jump** across it to
connect the free-stream state to the post-shock state. Each side of the interface SHALL be solved as a smooth
(low-rank) field by the `compressible-qtt-flux` marcher.

#### Scenario: 1-D fitted normal shock (the buildable milestone)
- **WHEN** a 1-D fitted normal shock is set up at a flight Mach number with the free-stream state ahead
- **THEN** the post-shock state matches the exact Rankine–Hugoniot values for that Mach number, and the field
  on each side is smooth (bond dimension `O(1)`)

#### Scenario: Each side stays low-rank
- **WHEN** the fitted-interface solution is QTT-encoded
- **THEN** the upstream and downstream fields are each low-rank (no captured discontinuity inflating the bond)

### Requirement: Interface motion and coupling

The interface location SHALL evolve from the local jump/flow state (dynamic-by-construction), and the bulk
solve SHALL couple to it through the jump condition each step. The coupling SHALL be stable for a single
topologically-stable bow shock (the reentry regime).

#### Scenario: Steady bow-shock standoff
- **WHEN** a blunt-body free-stream is marched to quasi-steady state
- **THEN** the fitted shock settles to a stable standoff distance consistent with the blunt-body relation,
  without rank blow-up

### Requirement: No wrong-shock-speed coupling

Because the shock is fitted (not smeared), reacting/relaxation sources applied in the post-shock region SHALL
not suffer the captured-shock wrong-propagation-speed pathology (LeVeque–Yee). The post-shock chemistry SHALL
start from the exact RH state.

#### Scenario: Reacting source starts from the exact post-shock state
- **WHEN** a Tier-A reacting/ionization source is applied behind the fitted shock
- **THEN** it is driven by the exact RH post-shock temperature/density (no dependence on a numerically smeared
  shock thickness)

