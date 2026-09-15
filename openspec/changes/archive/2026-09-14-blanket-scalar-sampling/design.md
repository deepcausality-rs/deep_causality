<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Design

## The kind marker, and why the obstacle was self-imposed

Two blanket implementations of one trait over two towers is `error[E0119]`, measured:

```text
error[E0119]: conflicting implementations of trait `SampleUniform` for type `u8`
```

Coherence cannot prove a real field will never also be a natural number, so it refuses. The crate
had reasoned from this to per-type bindings, and the reasoning was circular — the float file cited
the integers as the immovable obstacle and the integer file cited the floats.

A type parameter breaks it. `SampleUniform<FloatKind>` and `SampleUniform<UnsignedKind>` are
different items, so both blankets stand. `FloatKind` and `UnsignedKind` are never constructed and
never named at a call site; `Rng::random_range` infers the kind from the range and keeps one
signature, which is what the choice between this and two methods turned on.

`Uniform<X>` becomes `Uniform<X, K = FloatKind>`. The default is not a convenience: `Uniform` is
used only for floats in this workspace — `uncertain`'s distribution enum and this crate's own
tests — while `random_range` is used only for integers, so every existing use compiles unchanged.

## Deriving the width instead of declaring it

The rejected alternative was a `MANTISSA_DIGITS` constant on `Float`, forwarded through `Real`.
It compiles, and it fails the test that matters: `Real` is a single blanket implementation over
`Float` with 34 forwarding methods, so the declaration lands on `Float` and each of the four types
hand-writes it. The table moves address and survives.

It would also be a third statement of one fact. `Float` already reports its precision through
`epsilon()`, and two numbers that must agree with nothing enforcing it is the same hazard the
`WORDS` constant already had.

## What BFloat16 found

The scalar compiled into the generic path immediately and then failed a range assertion, twice, for
two different reasons — both pre-existing, both unreachable while the crate served `f32` and wider:

| Fault | Rate at `BFloat16` | Rate at `f32` and wider |
|---|---|---|
| unit draw returns exactly `1.0` | 6 / 2 000 | `2^-25` or below |
| affine map rounds up onto `high` | 14 / 2 000 | 0 / 2 000 |

The second is why the rejection tests the produced value rather than the draw. Whatever the
arithmetic does on the way, a result that reaches the bound is discarded, which makes the guarantee
hold for a scalar nobody has written yet.

`Uniform::new` previously documented the weaker contract — *"samples may equal `high` due to loss
of precision"* — and the code now keeps the stronger one, so the caveat is gone rather than
restated.

## Cost

Rejection replaces `% span` for integers, so the integer draw gains a loop. The mask is at most
twice the span, so under half the draws are rejected and the loop is expected to run under twice.
A constant generator cannot drive it — if its one value is out of band there is nothing else to
draw — which is a property of rejection sampling rather than a defect, and is why the test for the
out-of-band case scripts two words instead of pinning one.
