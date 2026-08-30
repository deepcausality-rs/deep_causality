## MODIFIED Requirements

### Requirement: Jacobi-preconditioned CG fallback
`deep_causality_linear` SHALL provide a preconditioned variant of the
matrix-free CG (additive API), and the wall-aware grade-0 solve SHALL use
it with the diagonal of the boundary-corrected `Δ₀` wherever the direct
solve does not apply (per-edge metrics, non-uniform geometry). Plain
`cg_solve` semantics are unchanged.

The requirement previously named `deep_causality_sparse`. The solver moves with the rest of that
crate's contents into `deep_causality_linear`; its signature, convergence behaviour and iteration
counts are unchanged. During the deprecation window the retired crate re-exports it, so callers that
have not migrated reach the same function.

#### Scenario: Preconditioned CG converges faster on walled lattices
- **WHEN** the same walled-lattice Poisson problem is solved by plain and Jacobi-preconditioned CG at the same tolerance
- **THEN** both converge to agreeing solutions and the preconditioned solve uses no more iterations (strictly fewer on the recorded benchmark case)

#### Scenario: Existing CG callers unaffected
- **WHEN** the existing `cg_solve` test suite runs
- **THEN** all results are unchanged

#### Scenario: The solver is reachable through both paths during the window
- **WHEN** a caller imports the preconditioned solver from either the retired crate or `deep_causality_linear`
- **THEN** both resolve to the same function
