<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Real gains cbrt and nothing else

`Real` SHALL gain `cbrt` and SHALL NOT gain `hypot`, `mul_add`, `max` or `min`.

The assessment listed five methods. Four fail on inspection, each for its own reason, and adding them
would grow a foundational trait's obligations for no caller.

`hypot` has no caller: a search for `.hypot(` across the workspace returns hits only inside
`deep_causality_num` itself. Its mathematics is unobjectionable — smooth away from the origin, with
an ordinary chain rule — but nothing outside the crate that defines it wants it.

`cbrt` has no caller either, and it is kept anyway, so the difference has to be stated rather than
implied. What `cbrt` has that `hypot` does not is a site doing its job badly: `signed_cbrt`, a private
helper computing `sign(x)·|x|^(1/3)` through `powf`, which returns `NaN` for a negative base and is
therefore already broken under duals. `hypot` has no such site. The test is not "is there a caller"
but "is there code doing this worse", and only `cbrt` passes it.

`mul_add` is not an algebraic operation. Its entire purpose on a primitive float is a single rounding
in place of two. A dual number's derivative component is `x.re·y.du + x.du·y.re + z.du`, a sum of two
products, which no fused instruction computes; `Dual` can only ever implement it as `x*y + z` and
deliver none of the property the name promises. Putting it on `Real` makes generic code believe it is
getting a fused rounding that, for some implementors, it is not.

`max` and `min` add no algebra — `Real` already requires `PartialOrd`, so both are derivable at the
call site — and on `Dual` they are actively wrong. `Dual` derives `PartialOrd`, which orders
lexicographically on `(re, du)`, so a `max` built on that comparison returns, when two duals share a
real part, whichever carries the larger derivative. That is a choice with no analytic meaning which
silently varies with the seed direction. The crate's own `Real` impl already avoids the derived order
for exactly this reason: `clamp` and `abs` both branch on the real part.

#### Scenario: cbrt is reachable through the bound
- **WHEN** generic code bounded on `Real` calls `cbrt`
- **THEN** it compiles and returns the real cube root, including for a negative argument

#### Scenario: The four rejected methods are absent
- **WHEN** `Real`'s method set is enumerated
- **THEN** it contains no `hypot`, `mul_add`, `max` or `min`

#### Scenario: The blanket implementor forwards to Float
- **WHEN** a type reaching `Real` through the blanket implementation calls `cbrt`
- **THEN** the call forwards to the existing `Float::cbrt` with no reimplementation

### Requirement: cbrt on a dual number follows the chain rule and does not special-case its singularity

`Dual<T>` SHALL implement `cbrt` as `cbrt(a) + b/(3·cbrt(a)²)·ε` and SHALL NOT guard the point where that derivative is undefined.

The cube root is real-analytic away from zero, so its dual extension is the ordinary chain rule and
is defined for negative arguments — which is exactly what the callers being retired need, since
`powf(1/3)` returns `NaN` for a negative base.

At `a = 0` the derivative is infinite. The crate's established convention is to let the arithmetic
produce the infinity rather than to intercept it: `sqrt` computes `self.du / (s * 2)` with no guard,
so `sqrt(0 + 1ε)` already yields an infinite `ε` component. `cbrt` follows that convention. A guard
here would be a silent substitution of a finite value for one that is genuinely unbounded, which is
the failure mode this change is elsewhere removing.

#### Scenario: The derivative is the chain rule
- **WHEN** `cbrt` is applied to a dual with a non-zero real part
- **THEN** the real component is the cube root of the real part and the dual component is `b / (3·cbrt(a)²)`

#### Scenario: A negative argument is handled
- **WHEN** `cbrt` is applied to a dual with a negative real part
- **THEN** the real component is the negative real cube root, not `NaN`

#### Scenario: The singularity is not concealed
- **WHEN** `cbrt` is applied to a dual whose real part is zero and whose dual part is non-zero
- **THEN** the dual component is infinite, matching `sqrt`'s existing behaviour at the same point

### Requirement: RealField carries ToPrimitive

`RealField` SHALL require `ToPrimitive` as a supertrait, so that code bounded on `RealField` can convert a scalar back to a primitive without restating the bound.

`RealField` currently offers a one-way crossing: `FromPrimitive` reaches the working type through
`Scalar`, and nothing comes back. Callers work around it in two ways, and both are worse than the
bound. `deep_causality_discovery` restates `RealField + ToPrimitive` and documents why; six further
sites across physics and `num_complex` restate it without documenting. Where restating was not
convenient, the workaround is arithmetic: `surface_force.rs` finds an integer floor by stepping one
integer at a time, in both directions, with no bound on the number of steps — the distance from its
starting guess to the answer. It also carries three `unwrap_or_else(R::one)` / `(R::zero)` silent
substitutions in six lines, because the conversion it needs does not exist on the bound.

The obligation is already satisfied. The blanket implementation requires `T: Float`, and
`Float: NumCast: ToPrimitive`, so the supertrait needs no change to the blanket's where-clause and
no new implementation for `f32`, `f64` or `Float106`.

#### Scenario: A RealField consumer converts without restating the bound
- **WHEN** generic code bounded on `R: RealField` converts a value to `f64` or to an integer
- **THEN** it compiles without an added `ToPrimitive` bound

#### Scenario: The blanket implementation is unchanged
- **WHEN** the supertrait is added
- **THEN** the blanket `RealField` implementation compiles with no change to its where-clause

#### Scenario: The restatements are removed
- **WHEN** the seven `RealField + ToPrimitive` restatements are enumerated after this change
- **THEN** each has dropped the redundant bound

#### Scenario: Dual does not gain RealField
- **WHEN** `Dual<T>` is checked against `RealField`
- **THEN** it still does not implement it, because it is not a field

### Requirement: The workarounds the additions replace are retired in this stage

Every workaround that exists because `Real` lacked `cbrt` or `RealField` lacked `ToPrimitive` SHALL be removed in this stage, and the stage SHALL NOT be considered complete while any remains.

Adding two trait obligations removes nothing by itself. A stage that adds the surface and leaves the
workarounds in place has grown the API and improved nothing, and the assessment's stated benefit for
this work — that it lifts the boilerplate downstream — is delivered only by the removal.

The retirements are: `signed_cbrt` in the coherent-structures kernel, a sign branch over
`powf(1/3)`; the bounded integer-floor scan in the DEC surface-force sampler; and the seven
`RealField + ToPrimitive` bound restatements. Each is replaced by a direct call.

#### Scenario: The sign-branch cube root is gone
- **WHEN** the coherent-structures kernel is read after this change
- **THEN** it calls `cbrt` directly and carries no sign branch over `powf`

#### Scenario: The floor scan is gone
- **WHEN** the surface-force sampler is read after this change
- **THEN** it converts through `ToPrimitive` in constant time, carries no stepping loop, and carries none of the three silent substitutions the loop needed

#### Scenario: The retirements preserve behaviour
- **WHEN** each retired site's existing tests run after replacement
- **THEN** they pass unchanged, except where a test pinned the workaround's own defect, which is recorded

### Requirement: The blast radius of the two trait additions is established by inspection

The stage SHALL record what the additions to `Real` and `RealField` break, established by enumerating implementors rather than by counting dependents.

Twelve manifests declare `deep_causality_algebra` as a dependency, seven crates and five examples.
That number is the release blast radius and is not the compatibility blast radius, which is smaller:

- `Real` is implemented twice, by a blanket over `Float` and by `Dual<T>`. Both are in this
  workspace, so the new method is supplied by the same change that declares it.
- `RealField` is implemented once, by a blanket requiring `Float`. `Float: NumCast` and
  `NumCast: ToPrimitive`, so the added supertrait is already satisfied and no implementor changes.

An external crate implementing `Real` directly would have to supply `cbrt`; none is known. Everything
that reaches either trait through `Float` is unaffected.

An earlier draft of this change called the stage breaking across nineteen manifests. Both figures
were wrong and neither had been checked; the requirement is written this way so the next reader gets
the enumeration rather than a count.

#### Scenario: The implementors are enumerated, not the dependents
- **WHEN** the stage's notes are read
- **THEN** they list every implementor of `Real` and of `RealField` in the workspace
- **AND** they state which of them the additions oblige to change

#### Scenario: The already-satisfied supertrait is confirmed
- **WHEN** `ToPrimitive` is added to `RealField`
- **THEN** the blanket implementation compiles with no change to its where-clause

#### Scenario: The workspace builds after the additions
- **WHEN** `bazel test //...` runs after the trait change
- **THEN** every dependent compiles and its tests pass
