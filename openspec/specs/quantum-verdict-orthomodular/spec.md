# quantum-verdict-orthomodular Specification

## Purpose
TBD - created by archiving change add-quantum-crate. Update Purpose after archive.
## Requirements
### Requirement: An orthomodular projection-lattice Verdict carrier

The crate SHALL provide a `Verdict` carrier as a newtype over projections on a fixed
finite-dimensional Hilbert space (the full subspace/projection lattice) — the
Birkhoff–von Neumann quantum logic — with `bottom = 0`, `top = I`, `complement = I − P`
(orthocomplement), and meet/join on ranges, extending `core.verdict.carriers` (today Boolean-proved,
MV witness-only) with the orthomodular class that fails distributivity the way the MV carrier fails
excluded middle. No blanket `Verdict` impl SHALL be provided for a general tensor/operator/process-
matrix type, because general effects `0 ≤ E ≤ I` form only an effect algebra with partial meet/join.

#### Scenario: The orthomodular laws hold and distributivity fails

- **WHEN** the orthomodular carrier is exercised on projections in general position (including non-commuting projections)
- **THEN** it satisfies the bounded-lattice + orthocomplement laws and the orthomodular law, and a
  witnessing triple shows distributivity failing (as documented on the carrier)

#### Scenario: Verdicts are extracted at the measurement boundary

- **WHEN** a quantum causaloid aggregates results
- **THEN** verdicts are extracted from operators at the measurement boundary (Born rule → `Prob`, or a
  projection → the orthomodular lattice), never taken over the operators themselves, and no blanket
  operator `Verdict` instance exists

