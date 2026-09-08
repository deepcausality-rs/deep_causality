<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Review checklist: where an expected value came from

A test asserts that a computed value equals an expected one. The test is worth something only if the
expected value was obtained **independently of the code under test**. If it was not, the test asserts
that the same expression was typed twice, and it passes whether the expression is right or wrong.

This is the check that most often fails silently, because a circular test looks exactly like a good
one. It has a real input, a real call, and a real assertion.

## The allow-list

An expected value is acceptable if it comes from one of these, and from nothing else.

**1. A closed form, evaluated by hand, written as a literal.**
Entropy of a uniform distribution on 8 outcomes is `3.0` bits. Write `3.0`. Do not write
`(8f64).log2()` — that is the formula under test, retyped.

**2. A published value, cited.**
A reference implementation's output, a table in a paper, a textbook worked example. The citation goes
in a comment: which source, which table, which row.

**3. A demonstrably different algorithm.**
A naïve O(n³) oracle against an optimised path. A dense computation against a sparse one. Different
means *different*, not the same formula with the loops reordered.

**4. An algebraic invariant.**
Idempotence, a conservation law, an inverse round trip, a symmetry, a bound the result must satisfy.
`H(X|Y) = H(X)` under independence. Four quarter-turns are the identity. A norm is non-negative and
obeys the triangle inequality.

**5. A property over a generated family.**
A statement that holds for every input in a generated set: entropy is non-negative; a permutation
preserves the multiset; increasing the ridge penalty decreases the coefficient norm.

## The reject-list

Reject the test and rewrite it if the expected value is:

- **Produced by calling the function under test.** Including calling it with different arguments and
  relating the results, when the relation is the property being tested.
- **The function's own formula, retyped in the test body.** The most common form. If you can see the
  implementation in the assertion, it is circular.
- **Produced by a helper that calls the function, or that shares its expression.** Circularity hides
  one level down as readily as at the surface. Follow the helper.
- **Taken from the implementation being replaced.** This is the one that matters for C1, C2, C4 and
  C6, all of which port or absorb existing code. A test that pins a port against its origin asserts
  that the port is faithful — **including to the origin's defects**. Every defect this change fixes
  would pass such a test: the R1–R3 closure agrees with the reference, the direct-form modulus agrees
  with itself, the unconverged iterate agrees with the unconverged iterate.
- **Recorded from the new implementation's first run.** A value pasted in because "that is what it
  prints" is the implementation asserting about itself, with an extra step.

## The rule for ported code

Agreement with the origin is a **useful additional test and never the only one**.

Each ported function gets at least one assertion from the allow-list — a closed form, a published
value, or an invariant — that pins it to the mathematics rather than to its predecessor. The
agreement test then sits beside it and answers a different question: whether the port changed
anything. Both are worth having; only the first can catch an inherited defect.

Where a stage deliberately *changes* the origin's behaviour, the agreement test is the one that must
fail, and it is updated with the reason recorded. That is the case for the physics entropy kernel
moving from nats to bits, and for the MHD matvec that stops skipping columns in silence.

## What review asks

For every numeric literal in an assertion:

1. Does a comment say where it came from?
2. Is that source on the allow-list?
3. If it is a closed form — is the closed form itself written down, so the next reader can check the
   arithmetic rather than trusting the digits?
4. If the test covers a ported function — is there also an allow-list assertion, or is the origin the
   only authority?

A literal with no stated provenance is treated as circular until shown otherwise. That default is
deliberate: the cost of stating the source is one comment, and the cost of not stating it is a suite
nobody can audit later.
