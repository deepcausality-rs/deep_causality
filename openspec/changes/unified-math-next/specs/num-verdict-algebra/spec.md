<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## MODIFIED Requirements

### Requirement: A verdict carrier with meet, join, and complement

`deep_causality_algebra` SHALL provide a `Verdict` trait — a bounded lattice with complement — supplying `bottom`, `top`, `meet`, `join`, and `complement`, so that the `Collection` aggregation output type is a stated bound rather than an ad-hoc bool/probability coercion. The complement SHALL support the `None` aggregation (`None` = `Any` post-composed with complement). The exact class (Boolean algebra vs probability MV-algebra) SHALL be pinned per design D4, with the probability carrier's `complement = 1 − p` recorded as an MV-algebra (not Boolean) complement.

The owning crate is corrected here from `deep_causality_num` to `deep_causality_algebra`. The trait
moved during the numeric crate split and lives at `deep_causality_algebra/src/algebra/verdict.rs`;
the requirement had not followed it. The obligation is otherwise unchanged.

#### Scenario: None is expressible via complement

- **WHEN** the `None` aggregation is evaluated as "no child fires"
- **THEN** it is the join-fold (`Any`) of the children post-composed with `complement`, using the `Verdict` trait's complement

#### Scenario: bool is a Boolean-algebra verdict

- **WHEN** `bool` implements `Verdict`
- **THEN** `meet = ∧`, `join = ∨`, `complement = !`, `bottom = false`, `top = true`, and the Boolean-algebra laws hold

#### Scenario: The probability carrier is an MV-algebra verdict

- **WHEN** the probability carrier implements `Verdict`
- **THEN** `complement = 1 − p` (an MV-algebra complement), and the caveat that it is not a Boolean algebra is documented (assumption #5 Q2)

#### Scenario: The trait is reachable from its owning crate

- **WHEN** a consumer imports `Verdict`
- **THEN** it resolves from `deep_causality_algebra`

## ADDED Requirements

### Requirement: The verdict carrier is instantiated at every shipped real scalar

`Verdict` SHALL be implemented for `f32` and for `Float106` alongside the existing `f64` instance, so that the aggregation output type exists at every precision the stack ships.

The trait is instantiated today at `bool`, `f64`, the probability carrier and two uncertain types.
The two missing instances are the reason the causality engine cannot reason at another precision:
its aggregation output type has nowhere to land, so the `f64` pin upstream of it cannot be lifted
however generic the surrounding code becomes.

Both new instances take the same MV-algebra reading as the existing `f64` one — `complement = 1 − p`
on the unit interval — so this adds instances, not a second interpretation.

#### Scenario: The narrow scalar is a verdict carrier

- **WHEN** `f32` is used as the aggregation output type
- **THEN** `bottom`, `top`, `meet`, `join` and `complement` are available and the lattice laws hold

#### Scenario: The wide scalar is a verdict carrier

- **WHEN** `Float106` is used as the aggregation output type
- **THEN** the same operations are available and the lattice laws hold at that precision

#### Scenario: The new instances agree with the existing one

- **WHEN** the same lattice expression is evaluated at `f32`, `f64` and `Float106` on values exactly representable in all three
- **THEN** the three results agree

#### Scenario: The complement is the MV-algebra complement at every precision

- **WHEN** `complement` is applied at `f32` or `Float106`
- **THEN** it is `1 − p`, and the documented caveat that this is not a Boolean complement applies unchanged

#### Scenario: The law tests cover the new instances

- **WHEN** the Bazel-registered verdict law tests run
- **THEN** they exercise `f32` and `Float106` as well as the existing carriers
