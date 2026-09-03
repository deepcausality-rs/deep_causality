<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Deferred: engine precision parametricity

**Status.** Deferred out of `unified-math-next` on 2026-09-04, by the maintainer's decision. The
numeric aliases in `deep_causality_core` and `deep_causality` stay exactly as they are, and
`ScalarValue` is not touched.

**Why it was deferred.** The aliases are an early expression of precision-as-a-parameter, written
before the mathematics stack settled on `Real`, `RealField` and `Scalar`. Reworking them is a design
question — what the engine's scalar contract should be now the tower exists — not a mechanical
unpinning, so it needs its own change with its own investigation.

**The one piece that shipped anyway.** The missing `Verdict` instances at `f32` and `Float106` went
into `unified-math-next` as a delta on `num-verdict-algebra`. They stand alone, close a real gap, and
are the prerequisite this change will need on day one.

**Findings already established, so the next change starts from them.**

- Every one of the eleven aliases in `deep_causality_core/src/alias/` is unused workspace-wide,
  including inside `deep_causality_core` itself. Every occurrence is the definition or its own
  docstring.
- `deep_causality` carries a complete independent duplicate of the same names in
  `src/alias/alias_primitives.rs`, and that copy is the live one. `deep_causality/src/lib.rs`
  re-exports nine named types from core, none of them an alias.
- `IntType` exists only in core and has no counterpart in the live set. Its docstring carries the
  precision-versus-range argument that `deep_causality_quantum`'s tolerance documentation cites.
- Core's `lib.rs` does `pub use crate::alias::*`, so any disposition of them is a breaking change to
  a published crate.
- `Causaloid` is already generic in four parameters and carries exactly one `f64` field. The context
  layer is already precision-parametric at the trait level; two node types ship at `u64` and one at
  `i64`, which is the existing evidence the parameter works. What remains pinned is the concrete node
  types and their implementations — 44 `Contextuable`-family impls.
- `ScalarValue` cannot become `algebra::Scalar`: that would drop `i64`, `i32`, `u64`, `u32` and
  `usize`, because integers were deliberately removed from the algebra hierarchy during the numeric
  crate split, and it would break the shipped integer time types.
- `deep_causality` does not depend on `deep_causality_num`, so a `Float106` instance cannot be
  written there without a new dependency edge.
- Routing the engine's geometry through `deep_causality_physics` for its constants is precluded:
  `deep_causality` is tier 4 and physics is tier 7.

**The open question this change must answer first.** Which of the two alias sets is the real one.

**Also deferred here, though separable.** Making the discovery loaders generic in their scalar. They
parse into a `CausalTensor<f64>` and callers widen afterwards, so higher precision is unreachable from
file input — a `Float106` load is rounded to double before it is widened. This could be pulled forward
on its own if wanted sooner; it is filed here because it is the same design question in different
clothes.

---

The requirements below were drafted for `unified-math-next` and are retained as the starting draft
for the dedicated change. They have not been reviewed against the deferral.

## ADDED Requirements

### Requirement: The engine's numeric aliases stop naming a concrete float

`deep_causality`'s `NumericalValue` and `FloatType` aliases SHALL cease to fix `f64` for code that can be generic, and the reasoning, observation and inference paths SHALL accept any scalar the algebra tower admits.

Every crate above `num` in the mathematics stack is generic over `RealField`, and the stack's stated
thesis is that precision is a parameter. The engine sitting at `f64` is the largest inconsistency
with that thesis in the workspace, and it is not a decision anyone made: the aliases predate unified
math.

The blast radius is smaller than it looks, and the reason is worth stating because it changes the
shape of the work. `Causaloid` is already generic in four parameters and carries exactly one `f64`
field. The context layer is already precision-parametric at the trait level, and two node types
already ship at `u64` and one at `i64` — which is the existing proof that the parameter works. What
remains pinned is the concrete node types and their implementations, and the missing carrier
instances beneath them.

#### Scenario: A model reasons end to end at a narrow precision
- **WHEN** a causal graph is built and reasoned over with `f32` as its scalar
- **THEN** it compiles and produces a verdict without naming `f64`

#### Scenario: A model reasons end to end at a wide precision
- **WHEN** the same graph is built at `Float106`
- **THEN** it compiles and produces a verdict carrying the wider precision

#### Scenario: The f64 path is unchanged
- **WHEN** an existing `f64` model runs after this change
- **THEN** its results are identical to its pre-change results

### Requirement: The missing verdict instances are what unblock the engine

The verdict carrier SHALL be instantiated at every shipped real scalar, and this SHALL be done before the engine's node types are unpinned.

The carrier is a bounded lattice with complement, and the aggregation output type is a stated bound
rather than an ad-hoc coercion. Today it is instantiated at `bool`, at `f64`, at the probability
carrier and at two uncertain types. `f32` and `Float106` are absent, and their absence is what
actually stops a model from reasoning at another precision — not the aliases, which are downstream of
it.

This is why the ordering matters: unpinning the node types first would produce code that is generic
in a parameter no carrier can satisfy.

#### Scenario: The carrier is instantiated at every shipped scalar
- **WHEN** the verdict instances are enumerated after this change
- **THEN** `f32` and `Float106` are present beside `f64`

#### Scenario: The new instances obey the same laws
- **WHEN** the lattice and complement law tests run against the new instances
- **THEN** they pass, exactly as they do for the existing ones

#### Scenario: The ordering is respected
- **WHEN** the first node type is unpinned
- **THEN** the carrier instances it needs already exist

### Requirement: The dead aliases in the core crate are resolved, not left in place

`deep_causality_core`'s numeric aliases SHALL be either removed or documented as deprecated with their status stated, and the decision SHALL be recorded.

Nothing in the workspace imports `NumericalValue`, `FloatType`, `NumberType` or `IntType` from
`deep_causality_core`. The engine keeps a second, independent copy of the same aliases in its own
crate, and that copy is the one in use. So there are not one set of pins but two, and only one of
them matters.

The repository does not delete files without asking, and these are published items in a published
crate, so removal is a decision for the maintainer rather than a consequence of this stage. What this
stage requires is that the situation stop being invisible.

The `IntType` alias carries an extensive docstring distinguishing precision from range — that
widening `IntType` buys headroom rather than accuracy, and that integer code carries an overflow
discipline instead of a tolerance. That reasoning is worth keeping wherever the alias ends up.

#### Scenario: The status is explicit
- **WHEN** the core aliases are read after this change
- **THEN** each is either absent, or present and documented as unused with its intended disposition

#### Scenario: The decision is recorded
- **WHEN** the stage's notes are read
- **THEN** they record whether the aliases were removed or retained, and on whose decision

#### Scenario: No consumer is broken
- **WHEN** the workspace builds after the change
- **THEN** nothing failed to resolve an alias, because nothing imported them

### Requirement: The scalar marker trait is not replaced by the algebra tower's scalar bound

`ScalarValue` SHALL NOT be replaced by `algebra::Scalar`, and any replacement SHALL preserve its integer implementors.

The assessment proposed the substitution as a simplification. It is not one. `ScalarValue` is
`Copy + Clone + PartialOrd + Default` implemented for `f64`, `f32`, `i64`, `i32`, `u64`, `u32` and
`usize`. `Scalar` is `Real + Div + FromPrimitive`, and integers are deliberately not in the algebra
hierarchy — they were dropped from it during the numeric crate split, on the grounds that a
commutative semiring is not a ring.

Substituting would therefore drop five of the seven implementors, and it would break the shipped
`u64` time types that are the existing evidence that the engine's genericity works.

If the marker is to be narrowed at all, it is narrowed on its own terms, with the integer
implementors kept.

#### Scenario: The integer implementors survive
- **WHEN** the marker's implementors are enumerated after this change
- **THEN** all seven remain

#### Scenario: The shipped integer time types still compile
- **WHEN** the `u64` and `i64` time types are built after this change
- **THEN** they compile and their tests pass

#### Scenario: The rejected substitution is recorded
- **WHEN** the stage's notes are read
- **THEN** they record why the algebra tower's scalar bound does not serve here

### Requirement: A dependency edge is added before an instance that needs it

Where a carrier instance requires a type from a crate the implementing crate does not depend on, the dependency edge SHALL be added deliberately and its tier consequence checked.

`deep_causality` does not depend on `deep_causality_num`. Its path dependencies are `algebra`,
`uncertain`, `ast`, `core`, `data_structures`, `haft` and `ultragraph`. So a `Float106` instance
cannot simply be written in `deep_causality`: either the instance belongs in `algebra`, which already
has the type available, or the edge is added.

Placing the instance beside the trait in `algebra` is the smaller change and needs no new edge.

#### Scenario: The instance is placed where the type is reachable
- **WHEN** the wide-precision verdict instance is added
- **THEN** it compiles without adding a dependency to `deep_causality`

#### Scenario: A new edge is justified if taken
- **WHEN** any dependency edge is added by this stage
- **THEN** the stage's notes record why, and the tier tables are updated

### Requirement: This stage does not refactor the geometry it touches

The stage SHALL NOT collapse the engine's duplicated Euclidean norms, its inline speed of light, or its metric literals, and SHALL leave them for separate work.

Those duplications are real and are recorded in the assessment. They are also not this stage's
subject: collapsing them advances precision-parametricity by exactly zero, because the collapsed
helper would be as pinned as the copies. `AGENTS.md` is explicit that unrelated code is not
refactored and neighbouring issues are not fixed inside another task's scope.

There is a further reason specific to one of them. The assessment proposed routing the geometry
through `deep_causality_physics` for its constants. That is impossible: `deep_causality` is tier 4
and `deep_causality_physics` is tier 7, so the edge would invert the dependency graph.

#### Scenario: The norms are untouched
- **WHEN** the engine's space and spacetime metric implementations are read after this change
- **THEN** they are unchanged except where a scalar parameter replaced a concrete float

#### Scenario: The tier violation is recorded rather than attempted
- **WHEN** the stage's notes are read
- **THEN** they record that routing the engine's geometry through the physics crate is precluded by the tier order

### Requirement: The file-loading path becomes generic in its scalar

The discovery loaders SHALL produce a tensor in the caller's scalar rather than always in `f64`.

The loaders parse into a `f64`-element tensor and callers widen afterwards, which makes higher
precision unreachable from file input: the value has already been rounded to double before the widen
happens. For `Float106` that discards the precision the type exists to provide.

The honest constraint is that parsing text into an arbitrary scalar needs a bound that admits it, and
that a parse routed through `f64` would defeat the purpose. The requirement is therefore on the
observable outcome — that a wide-precision load is not narrowed in transit — rather than on a
particular bound.

#### Scenario: A wide-precision load keeps its digits
- **WHEN** a file containing values with more than double precision is loaded at `Float106`
- **THEN** the loaded values carry those digits, and are not equal to the same values parsed at `f64` and widened

#### Scenario: The existing f64 path is unchanged
- **WHEN** an existing load at `f64` runs after this change
- **THEN** its values are identical to before

#### Scenario: An unparseable value is refused at every precision
- **WHEN** a malformed value is loaded at any scalar
- **THEN** a typed error names the row and column, as it does today
