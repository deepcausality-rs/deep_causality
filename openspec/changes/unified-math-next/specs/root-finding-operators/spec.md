<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Every root finder reports non-convergence instead of returning its last iterate

A root finder SHALL return a typed error carrying its iteration count and achieved residual when it reaches its iteration cap without meeting its tolerance, and SHALL NOT return its last iterate as though it had converged.

This is the defect the stage exists to remove. The electroweak radiative-correction solver iterates
twenty times against a hard-coded tolerance and returns whatever it last computed, converged or not.
A caller cannot distinguish a solved fixpoint from a failed one, and the value it gets back is a
plausible number in both cases.

Six further solvers share the shape without the same silence — bisections with a fixed iteration
count, Newton iterations at a hundred steps — and their failure behaviour differs from one another
because each was written where it was needed.

#### Scenario: Non-convergence is a typed error
- **WHEN** an iteration reaches its cap without meeting its tolerance
- **THEN** a typed error carrying the iteration count and the achieved residual is returned

#### Scenario: A converged result reports its cost
- **WHEN** an iteration converges
- **THEN** the result carries the iteration count and the achieved residual alongside the root

#### Scenario: The failure is reachable in a test
- **WHEN** a function with no root in the supplied bracket, or one that cannot converge in the cap, is offered
- **THEN** the error path is taken, and a test asserts the specific variant

### Requirement: Bisection requires and checks a bracketing interval

Bisection SHALL return a typed error when the supplied interval does not bracket a sign change, and SHALL NOT proceed on an unbracketed interval.

A bisection on an interval whose endpoints share a sign converges to an endpoint and reports it as a
root. The three shipped bisections each construct their bracket by doubling out to a cap, so the
unbracketed case is reachable whenever that search fails.

#### Scenario: A valid bracket converges
- **WHEN** an interval whose endpoints have opposite signs is supplied for a continuous function
- **THEN** the root is found within the tolerance, and matches a value known in closed form

#### Scenario: An unbracketed interval is refused
- **WHEN** the endpoints share a sign
- **THEN** a typed error is returned before any iteration

#### Scenario: A root at an endpoint is found
- **WHEN** the root lies exactly on one of the endpoints
- **THEN** it is returned rather than missed by the interior search

#### Scenario: A degenerate interval is refused
- **WHEN** the two endpoints are equal
- **THEN** a typed error is returned

### Requirement: Newton accepts either a supplied derivative or a dual-number function

Newton's method SHALL be available both with an explicitly supplied derivative and with a function evaluated over dual numbers, and the two forms SHALL agree.

The crate already owns forward-mode automatic differentiation through its dual-number dependency, so
a caller that has a closed-form function should not have to write its derivative by hand. A caller
that has a cheaper analytic derivative, or a function it cannot express over duals, keeps the
explicit form.

The two agreeing is the test that the dual path is wired correctly, and it is not circular: each is
independently checked against a root known in closed form first.

#### Scenario: Both forms find a known root
- **WHEN** each form is applied to a function whose root is known in closed form
- **THEN** both converge to that root within tolerance

#### Scenario: The two forms agree
- **WHEN** both are applied to the same function from the same start
- **THEN** their iterates agree to the precision in use

#### Scenario: A zero derivative is refused
- **WHEN** the derivative at an iterate is zero
- **THEN** a typed error is returned rather than a division producing an infinity

#### Scenario: Divergence is caught by the cap
- **WHEN** Newton is started where it diverges
- **THEN** the iteration cap is reached and the non-convergence error is returned

### Requirement: The operators compile without an allocator

Every root finder SHALL compile under the crate's no-allocator configuration, and SHALL NOT allocate on its working path.

`deep_causality_calculus` is `no_std`-capable and refuses to build without one of its float-backend
features. A root finder that allocates a history vector, or that returns a boxed error, would build
under the default feature set and fail on bare metal — a failure the default test run would not
catch.

#### Scenario: The crate builds without default features
- **WHEN** the crate is built with default features disabled and the no-std feature selected
- **THEN** compilation succeeds with every root finder present

#### Scenario: The working path does not allocate
- **WHEN** a root finder runs
- **THEN** it performs no heap allocation

### Requirement: The operators match the crate's existing level of abstraction

The root finders SHALL be introduced at the abstraction level the crate already uses for its integrators, rather than as a parallel style.

The crate expresses its numerical operators as types with an arrow-based interface beside plain
entry points — the differentiation and integration operators each appear in both forms. A new
family introduced only as free functions would sit at a different level from its neighbours, and
`AGENTS.md` requires new code to land at the right level of the existing hierarchy rather than
inventing one.

#### Scenario: The surface matches its neighbours
- **WHEN** the root-finding surface is compared with the existing differentiation and integration operators
- **THEN** it follows the same construction and invocation pattern

#### Scenario: A composed pipeline typechecks
- **WHEN** a root finder is composed with an existing operator through the crate's arrow interface
- **THEN** it compiles and runs

### Requirement: Migrating a shipped solver to the shared operator is a stated behaviour change

Replacing a hand-rolled solver in a shipped kernel SHALL be recorded as a behaviour change wherever the kernel's failure behaviour differs from the shared operator's, and each kernel's caller SHALL be updated deliberately.

The physics and algorithms kernels that would adopt these operators today return a value on
non-convergence. Adopting an operator that returns an error changes what those kernels do when they
fail, and therefore what their callers must handle. That is the improvement, and it is still a change
to shipped behaviour that belongs in the record rather than in a diff.

Migration is a separate task from introducing the operators, so the operators can land without
touching a solver.

#### Scenario: The operators land before any kernel is migrated
- **WHEN** the operators are introduced
- **THEN** no shipped kernel has changed, and the crate's own suite is green

#### Scenario: Each migration is recorded
- **WHEN** a kernel is migrated
- **THEN** the stage's notes name the kernel, its old failure behaviour and its new one

#### Scenario: The caller handles the new failure
- **WHEN** a migrated kernel fails to converge
- **THEN** its caller propagates or handles the typed error, and a test covers that path
