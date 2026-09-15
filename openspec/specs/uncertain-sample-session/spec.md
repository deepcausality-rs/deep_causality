<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# uncertain-sample-session Specification

## Purpose
A draw is a function of three numbers — a session's seed, the sample index, and the drawing leaf's
ordinal — and of nothing else. Nothing is stored between calls, no global or thread-local is
consulted, and a seed replays its values in a later process.

This replaces a root-only global sample cache and a thread-local seed slot. The cache bought one
property, that a root re-sampled at an index returns the same value, and it bought it with a static
that leaked by design, a `cfg(test)` branch that meant the shipping code was the one branch the
suite never ran, and a lock. Addressing obtains the property by construction and obtains a stronger
one besides: two graphs agree about a shared leaf at a given index whenever the leaf holds the same
ordinal in both.

The ordinal, rather than the node's heap address, is what makes this replayable. An `Arc` pointer
differs between two runs of one program and between two structurally identical trees, so a draw
derived from it could not be reproduced from a recorded seed.

## Requirements

### Requirement: The sampler's state is a value the caller owns

`deep_causality_uncertain` SHALL own no global mutable state, and every input a draw depends on SHALL be reachable from a `SampleSession` the caller constructs, holds and drops.

Five globals exist today: `NEXT_UNCERTAIN_ID`, `GLOBAL_SAMPLE_CACHE`, the thread-local
`SAMPLER_SEED` slot, the sample-index source, and `deep_causality_rand`'s thread RNG. After this
change the crate owns none of them. The thread RNG remains in `rand`, as it does for the standard
library, and is reached only by the zero-argument convenience draw.

`SampleSession::seeded(seed)` and `SampleSession::qmc(seed)` replace `seed_sampler` and
`clear_sampler_seed`. A session is Monte-Carlo or Quasi-Monte-Carlo by construction, so there is no
sampler discriminant to carry anywhere.

#### Scenario: The crate declares no mutable global

- **WHEN** `deep_causality_uncertain/src` is searched for `static`, `thread_local!` and `OnceLock`
- **THEN** no mutable global declaration remains

#### Scenario: Two sessions do not interfere

- **WHEN** two `SampleSession` values with different seeds draw from the same `Uncertain` value on one thread
- **THEN** each produces the stream its own seed determines, and neither observes the other's state

#### Scenario: Tests need no process isolation

- **WHEN** the crate's test suite is read after this change
- **THEN** it contains no `rusty_fork_test!` invocation
- **AND** `rusty-fork` is absent from the crate's dev-dependencies

### Requirement: A draw is addressed by seed, index and leaf ordinal

Every leaf draw SHALL be a pure function of the session seed, the sample index, and a leaf ordinal assigned by a deterministic pre-pass, and SHALL NOT depend on any heap address, allocation order, or previously drawn value.

The ordinal is assigned by one traversal of the tree that dedupes by node identity, so a leaf
reached twice receives one ordinal and `x + x` draws `x` once. Node identity is a pointer and is
used **only** as a key within that traversal; it is never mixed into a generator seed, because a
heap address differs between runs and between two structurally identical trees, and a draw derived
from one could not be reproduced.

This is the scheme `QmcSampler::new` already uses to assign Sobol dimensions. The Monte-Carlo path
joins it.

#### Scenario: A seeded run reproduces across processes

- **WHEN** a seeded session draws a recorded sequence from a given tree, and the same program is run again in a fresh process
- **THEN** every drawn value equals the recorded one exactly

#### Scenario: Two structurally identical trees agree

- **WHEN** two `Uncertain` values are built separately from identical constructor calls and each is drawn at the same index under equally seeded sessions
- **THEN** the two draws are equal

#### Scenario: A shared leaf is drawn once per sample

- **WHEN** an expression uses the same `Uncertain` leaf twice, such as `x + x`, and is sampled at one index
- **THEN** the leaf contributes one draw, and the result is twice that draw

#### Scenario: The same index means the same draw for every tree sharing a leaf

- **WHEN** two different expressions over a shared leaf are each sampled at index `i` under one session
- **THEN** the shared leaf yields the same value in both

### Requirement: The root sample cache is removed

The crate SHALL NOT retain sampled values between calls, and no draw SHALL be served from storage.

The cache being removed is consulted only at the root — `types/sampler/` contains no reference to
it — so it never made two different roots over a shared leaf agree. It also grew without bound:
`sample()` drew a random `u64` index, `sample_with_index` inserted one entry under it, and nothing
in `src/` ever called `clear`, so a ten-thousand-sample estimate left ten thousand entries for the
life of the process. Its shipped branch was the untested one, because `cfg(test)` replaced the
`OnceLock` static with a `thread_local!`.

The one property it provided is delivered by the addressing scheme above, without storage.

#### Scenario: Repeated draws allocate no growing store

- **WHEN** an expectation over ten thousand samples is computed and the process's resident set is compared before and after
- **THEN** no per-draw entry is retained

#### Scenario: The per-draw memo is unaffected

- **WHEN** one sample is evaluated over a tree with shared sub-expressions
- **THEN** each distinct node is evaluated once within that sample, by the sampler's per-call memo

### Requirement: A statistical gate in the suite is seeded

Every test asserting on the outcome of a sequential probability ratio test or any other sampled decision SHALL run under a seeded session, and SHALL NOT be made to pass by widening its assertion.

Measured before this change: `uncertain_maybe_f106_tests::test_lift_to_uncertain_success` failed 2
of 12 consecutive runs. It gates an unseeded SPRT at a true presence probability of 0.9 against a
0.8 threshold with a 100-sample budget, and the default entropy source is the operating system, so
failing to accept within budget is an ordinary outcome rather than a defect in the gate.

#### Scenario: The previously flaky gate is deterministic

- **WHEN** the presence-gate test suite is run one hundred times
- **THEN** every run produces the same result

#### Scenario: The assertion is unchanged in strength

- **WHEN** the repaired test is compared with its predecessor
- **THEN** it asserts the same exact outcome, and the repair is the seeded session rather than a relaxed threshold, a wider tolerance or a larger budget
