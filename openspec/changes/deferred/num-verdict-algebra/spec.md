<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Deferred: the verdict carrier at f32 and Float106

**Status.** Cut from `unified-math-next` on 2026-09-04, by the maintainer's decision. It follows
[`engine-precision-parametric`](../engine-precision-parametric/spec.md), whose deferral removed its
only consumer.

**Why it was cut.** Three reasons, the second of which makes the requirement below wrong as written.

*No consumer.* With the engine's aliases untouched, nothing in the workspace wants an `f32` or
`Float106` causal model. The change's own D6 rule — only functions with a caller get built — forbids it.

*The stated pin was not the pin.* `design.md` D10 called the missing `Verdict` instances "the binding
pin on the engine: the aggregation output type has nowhere to land". Reasoning actually goes through
`Aggregatable: Verdict`, which lives in `deep_causality`, not `deep_causality_algebra`, and carries
its own required `aggregate` method. The two traits are instantiated differently today:

| Trait | Implemented for |
|---|---|
| `Verdict` (algebra) | `bool`, `f64`, `Prob`, `Uncertain<bool>`, `Uncertain<f64>` |
| `Aggregatable` (deep_causality) | `bool`, `f64`, `UncertainBool`, `UncertainF64` |

`Prob` has `Verdict` and no `Aggregatable`. That asymmetry is already in the tree, and it is the proof
that a `Verdict` instance alone does not make a carrier usable for reasoning. Adding the two instances
as specified would have produced two more carriers in exactly `Prob`'s position.

*It would have added duplication.* A usable `f32` carrier needs an `Aggregatable` impl in
`deep_causality` as well, and `f64`'s is roughly thirty lines of probability semantics — `product()`
for `All`, inclusion-exclusion for `Any`/`None` — that hardcodes `0.5` in its `Some(k)` branch while
ignoring the `threshold` parameter it is passed. An `f32` version is a near-verbatim copy; a blanket
impl cannot subsume them because `bool`'s is structurally different. A change whose purpose is
removing duplication would have created a three-way copy.

`Float106` needs more still: `deep_causality` does not depend on `deep_causality_num`, so its
`Aggregatable` impl requires a new dependency edge. Placing the `Verdict` instances in `algebra` was
meant to avoid that edge; it does not, because the second impl has to live in `deep_causality`
regardless.

**Also recorded here, unfixed.** The published `num-verdict-algebra` spec still says
`deep_causality_num` provides the `Verdict` trait. It moved to `deep_causality_algebra` during the
numeric crate split (`algebra/src/algebra/verdict.rs:20`). Correcting a stale spec was not worth a
capability of its own in this change; it belongs with this work.

**What a future change needs to decide.** Whether `Aggregatable` should stay a hand-written impl per
carrier, or become a blanket over a bound that `bool` can also satisfy — and what the ignored
`threshold` parameter and the hardcoded `0.5` are supposed to mean.

---

The requirements below were drafted for `unified-math-next` and are retained as a starting draft.
They specify only the `Verdict` half and are therefore incomplete; see above.

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
