## ADDED Requirements

### Requirement: Tensor-train-native observable extraction

The `solvers/qtt` module SHALL provide observable-extraction functions that compute diagnostics of a 2-D
incompressible velocity train pair `(u, v)` **directly on the trains** where possible. It SHALL provide:
kinetic energy `½(‖u‖² + ‖v‖²)` from the train norm; the divergence residual `‖∇·u‖` from
`QttProjector2d::divergence` then the train norm; the maximum bond dimension across the `(u, v)` cores
(the compression / rank metric); and the maximum speed `max √(u² + v²)` (dequantizing the trains). The
functions SHALL be free functions usable without the CfdFlow DSL.

#### Scenario: Kinetic energy and divergence are computed without dequantizing
- **WHEN** kinetic energy and the divergence residual are requested for a velocity train pair
- **THEN** they are computed from the train `norm` (and the projector's `divergence`), agreeing with the
  values obtained by dequantizing and integrating on the dense field within rounding tolerance

#### Scenario: Bond dimension reports the rank
- **WHEN** the maximum bond dimension is requested
- **THEN** it equals the largest bond across the cores of `u` and `v`

#### Scenario: Max speed reflects the dense field
- **WHEN** the maximum speed is requested
- **THEN** it equals `max √(u² + v²)` over the dequantized grid
