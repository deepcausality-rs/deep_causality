<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Context

`unified_math_next.md` assessed the mathematics stack against its four consumers and produced a
ranked list of nine work items. A sixteen-agent sweep then re-read every claim against the tree.
The sweep's value was not confirmation — it was refutation. Roughly a third of the ranking's
premises did not survive, and the corrections changed what the work is:

- The Meek R4 omission is defended on a precondition BRCD's own paper falsifies.
- `deep_causality_core`'s numeric aliases are dead code; the engine carries a second, live copy.
- `Causaloid` is already generic in four parameters and holds one `f64` field, so the feared viral
  type parameter does not exist.
- A blanket `Distribution` over `RealField` cannot compile, and the generic sampling it was meant
  to provide already ships behind `RealRng`.
- A statistics crate over `linear` is tier 4; tier 3 is arithmetically impossible.
- `hypot`, `mul_add`, `max` and `min` have no caller, no meaning on a dual, or both.

Three findings are defects rather than duplication: the Meek closure, `vector_norm_l2`'s overflow,
and the quantum modulus that shares its shape. The rest is the ordinary consequence of a stack built
bottom-up — `linear` landed after its consumers, and the engine's aliases predate unified math.

The constraints are the repository's own: no `unsafe`, no macros in `src`, one type per module, a
`tests/` tree mirroring `src/`, full coverage on added code, both build systems green, no commits and
no deletions without asking.

## Goals / Non-Goals

**Goals:**

- Close the three defects, each with a test that fails before the fix.
- Retire duplication that has a consumer, and only where retiring it is an improvement.
- Add the verdict-carrier instances the engine will need before it can be made precision-parametric.
- Run one test-first cycle over every stage, ports included.

**Non-Goals:**

- Building mathematics with no consumer in this repository. The excluded set is enumerated in the
  proposal and is large: graph algorithms, the retrieval family, splines, matrix functions, samplers,
  big integers.
- Unifying the interpolation out-of-range policies. The divergence is contractual.
- Refactoring the engine's geometry. It is real duplication and it is another change's subject.
- Changing what BRCD computes, beyond the orientation correction and its recorded consequences.
- Unpinning the engine's numeric aliases, or touching `ScalarValue`. Both are deferred to a change of
  their own; see D13.

## Decisions

### D1. Meek belongs in topology as a graph operation, not in algorithms as a helper

Meek's rules are graph mathematics over a partially directed graph, and `MixedGraph` is the
workspace's carrier for one. The neighbouring closure already lives there: `topological_sort`,
`has_cycle` and `find_cycle` are inherent methods in `mixed_graph/acyclicity/`. The existing
implementation already imports `MixedGraph` and `EdgeKind` from topology, so the code is reaching
across a boundary it should be living behind.

*Alternative considered:* leaving it in `algorithms` and exporting it. Rejected — it puts one subject
in two crates and leaves the next consumer to rediscover it.

### D2. R1–R4 is the default; R1–R3 is a named opt-in

The complete closure is what a caller should get without asking. Reference parity is a real need —
for a differential test, or for reproducing a published number — so the restricted closure stays, but
as something a caller requests with the reason at the call site.

This inverts the present arrangement, where the restricted behaviour is the default and its
limitation is recorded in a doc comment whose stated precondition the caller violates.

*Alternative considered:* a boolean parameter. Rejected — a boolean at a call site says nothing about
why, and the two closures have different completeness hypotheses, which is a distinction worth two
names.

### D3. The severity of the R4 omission is established by search before the code moves

Background knowledge being present is necessary for R4 to matter, not sufficient. Whether the rule
fires on the shapes BRCD builds decides whether this stage repairs a live defect or closes a latent
one. Both are worth doing; conflating them would put an unverified severity claim into the record.

The search enumerates graphs to a stated vertex bound, augments each as Algorithm 1 does, and
compares the two closures. A counterexample becomes a regression test. No counterexample is itself a
result — recorded with its bound, and not mistaken for a proof.

*Alternative considered:* proving it analytically. Better if it succeeds, but it is not a
prerequisite for the fix, and an exhaustive small-graph search is decisive within its bound and
cheap.

**Result, recorded 2026-09-04.** No difference, over 3.8 million augmented graphs to five vertices.
R4 never fires on this family; nor does R3. The four controls that make that negative result
meaningful — including a positive control proving R4 *can* fire on its own configuration — are in
`notes/c1-meek.md`. C1's justification is therefore latency and generality, not a live defect.

### D4. `Real` gains `cbrt` and nothing else

Four of the assessment's five methods fail, each on its own ground, and the grounds are different
enough to be worth separating.

`hypot` fails on demand: no caller outside the crate that defines it. `mul_add` is not mathematics —
its purpose is a single rounding, which a dual number cannot deliver, so the trait would promise
implementors something not all of them provide. `max` and `min` add no algebra over the `PartialOrd`
the trait already requires, and on `Dual` the derived lexicographic order makes them silently wrong
at a real-part tie. The crate's own `Real` impl already avoids that order in `clamp` and `abs`.

`cbrt` is kept, but on narrower grounds than an earlier draft claimed. It has **no caller**: the site
it replaces is `signed_cbrt`, a private seven-line helper with two call sites in one Cardano cubic
solve, which computes `sign(x)·|x|^(1/3)` through `powf`. Calling that "a real consumer" applied a
looser test than the one that killed `hypot`.

What survives is measured. `cbrt` is exact on 200 of the first 200 perfect cubes where `powf(1/3)`
manages 3, and differs by one to ten ULP elsewhere. But both call sites pass a computed discriminant
that is never an exact cube, so the exactness result does not apply there and the realistic gain is
about one ULP, inside a solve whose dominant error is cancellation in that same discriminant.

The decision stands because the cost is near zero — two impl sites, no breakage — so even a thin
benefit clears the bar, and a correct `cbrt` is what a future caller should find. The justification,
not the decision, was what needed fixing.

### D5. Entropy takes base and zero policy as parameters

The three implementations disagree on base (bits versus nats — a factor of `ln 2`, so they are not
the same quantity), on normalisation, and on zero policy (skip at exactly zero versus below epsilon).
Only one validates its input.

Convergence would change two callers' results. Picking one silently would be worse. Parameters let
one implementation serve all three with each keeping its semantics, and move the difference from
three crates apart to the call site.

This makes the absorption net-neutral in lines, which is the honest accounting: **the value here is
resolving a three-way semantic divergence, not deleting code.** The line savings in this stage come
from log-sum-exp and the Gaussian log-density, which genuinely agree.

*Alternative considered:* one function per caller. Rejected — that is what exists now.

### D6. Only functions with a caller are built

Nine of the roughly thirty functions in the assessment's list have no library consumer. A new crate
is the easiest place in a repository to add speculative surface at scale, and `AGENTS.md` forbids
generalisation that was not requested. Each excluded function is straightforward to add when
something calls it.

Mutual information is the closest call, because SURD computes information quantities. It stays out
because what SURD computes is specific mutual information over marginalised tensor axes, not the
slice-shaped general function — building the general one would leave SURD's beside it.

### D7. `tensor` does not depend on the statistics crate

`tensor`'s statistics extension holds log-sum-exp and a Gaussian log-density that the new crate also
provides. Delegating would remove the last duplicate pair, at the cost of moving `tensor` to tier 5,
`multivector` to 6 and `topology` to 7, invalidating the tier tables and the dependency figure.

The cost is disproportionate to two functions. `tensor` keeps its tensor-shaped wrappers; the new
crate sits below it so a later change can delegate without a cycle if the balance shifts.

*Decided by the maintainer: no. The tier tables and the dependency figure are unaffected by this
change, and the two remaining duplicate functions are accepted as the price.*

### D8. Interpolation policies stay divergent

The assessment called the divergence the risk. It is a contract. The `clamped` marker is a
requirement of `weather-table-consumption` and is pinned by six assertions; SRP's rejection gates a
shipped coupled-march step through `?`, on a leg that flies a thrust coefficient beyond the table's
domain. Three policies for three physical contracts is correct.

Only the corner-folding kernel is duplicated, and it is character-for-character identical across
three multilinear samplers — but its floor-finding is a workaround for the missing `ToPrimitive`
bound, so it should not be baked into a shared primitive. It is folded into C3's retirement instead.

### D9. `ScalarValue` keeps its integer implementors

Substituting `algebra::Scalar` would drop `i64`, `i32`, `u64`, `u32` and `usize`, because integers
were deliberately removed from the algebra hierarchy during the numeric crate split. It would also
break the shipped `u64` and `i64` time types — which are the existing evidence that the engine's
genericity works at all.

### D10. The verdict carrier is instantiated now, ahead of the work that needs it

The missing `f32` and `Float106` verdict instances are the binding pin on the engine: the aggregation
output type has nowhere to land, so no amount of genericity upstream would let a model reason at
another precision. Unpinning anything first would produce code generic in a parameter no carrier
satisfies.

The unpinning itself is deferred (D13), and the instances are still worth adding here. They are two
implementations beside the trait, they close a real gap in `num-verdict-algebra` on their own terms,
and they remove the first obstacle from the deferred change's path.

The instances go in `deep_causality_algebra`, beside the trait, because `deep_causality` does not
depend on `deep_causality_num` and placing them there would need a new dependency edge for no gain.

### D11. The blanket `Distribution` is withdrawn

It cannot compile: a blanket over `RealField` collides with the concrete `Distribution<u64>`,
`<u32>` and `<bool>` on the same type, because `RealField` is upstream and the compiler cannot rule
out future implementations for those types.

It would also be wrong if it compiled. A sampler generic over `FromPrimitive` builds its value from a
primitive, which for `Float106` means 53 bits of entropy widened into a 106-bit type — the opposite
of what the crate deliberately does.

The need is already met by `RealRng`. The two physics sites that sample at `f64` and lift carry
module comments claiming the crate supports only `f32` and `f64`; that is false today, and fixing
those sites needs no change to `rand` at all.

### D12. The test-first cycle binds ports as well as new code

The five phases — unimplemented API, failing suite, defect audit, implementation, mutation — are the
generalisation of `linear-test-first-development` from one crate to five stages. The part that matters
most is that it binds the stages that move existing code, where the temptation to paste the
implementation and write tests after is strongest, and where a test written after a port encodes the
port's fidelity to its origin including that origin's defects.

Hence the anti-circularity rule: every expectation comes from a closed form, a citation, a
demonstrably different algorithm, an invariant, or a generated property — never from the code under
test, its formula retyped, or the implementation being replaced. Agreement with the origin is an
additional test, never the only one.

### D13. The engine's numeric aliases are deferred to a change of their own

`deep_causality_core` and `deep_causality` each carry a full set of numeric aliases, and both sets
stay exactly as they are. This change does not unpin them, does not remove them, and does not touch
`ScalarValue`.

The aliases are an early expression of precision-as-a-parameter, written before the mathematics stack
had settled on `Real`, `RealField` and `Scalar`. Reworking them is therefore not a mechanical
unpinning — it is a design question about what the engine's scalar contract should be now that the
tower exists, and about which of the two alias sets is the real one. That question deserves its own
change with its own investigation, not a task group inside a stage whose subject is something else.

What the sweep established stays on the record for that change to start from: every one of the eleven
aliases in `deep_causality_core/src/alias/` is unused workspace-wide, including inside core itself;
`deep_causality` carries a complete independent duplicate in `src/alias/alias_primitives.rs` which is
the live one; core's `lib.rs` re-exports the aliases publicly, so any disposition of them is a
breaking change to a published crate; and `IntType` exists only in core, so it has no counterpart in
the live set. Those findings are recorded in the deferred note rather than acted on here.

What survives into this change is the one piece that stands alone and does not depend on the design
question: the missing `Verdict` instances at `f32` and `Float106`. They are two implementations in
`deep_causality_algebra` beside the trait, they close a real gap in `num-verdict-algebra`, and they
are the prerequisite the deferred change will need on day one.

*Alternative considered:* keeping the loader-genericity work here, since the discovery loaders parse
into an `f64` tensor and could serve the algorithms crates at higher precision without any engine
change. Deferred with the rest because it is the same design question wearing different clothes — but
it is the one item that could be pulled forward on its own if it is wanted sooner.

### D14. The physics entropy kernel moves to bits, and its name states its base

`shannon_entropy_kernel` computes in nats while both SURD implementations compute in bits, a factor
of `ln 2`. It moves to bits, which is the conventional base for a quantity named for Shannon and
makes all three call sites agree on what they compute.

This changes a published kernel's returned value. It is not a refactor and is not presented as one:
the kernel has one internal caller — its own causal wrapper — and its existing tests pin the nats
result, so those expectations change deliberately alongside the implementation.

The name is made unambiguous at the same time, so that the base is readable at the call site rather
than inferable only from the source. The suite pins the base directly, with a uniform distribution on
`n` outcomes asserting `log2 n`.

*Alternative considered:* keeping nats and renaming to say so. Defensible for a thermodynamic
quantity, and rejected because it would leave the workspace computing entropy in two bases for no
reason a caller could see.

### D15. Stages are independently shippable, and three are mutually independent

C1, C3 and C5 touch disjoint code and can land in any order — C5 having shrunk to two trait
implementations in `algebra`. C2 and C4 want C3 first, because `ToPrimitive` removes workarounds
inside their scope. C6 is last because it is the largest and least urgent, with its one real defect —
the silent non-convergence — extractable as a standalone fix.

## Risks / Trade-offs

**The R4 search finds nothing within its bound** → The rule is still added, because a bound is not a
proof, and the negative result is recorded with its bound so the next reader does not repeat it. The
stage's justification shifts from repairing a live defect to closing a latent one, and the notes say
so plainly rather than retaining the stronger claim.

**Shipped numerical kernels change behaviour** → Three places: the CSR matvec that stops silently
skipping out-of-range columns, root finders that stop returning unconverged iterates, and any entropy
caller whose base changes. Each is stated in its spec, gets its own test, and is recorded in the
stage notes as a decision rather than absorbed into a refactor.

**Breaking changes to published crates** → `dag_sampling::mec_size` changes its return type;
`LinearErrorEnum` and `ultragraph`'s pathfinding trait are public and neither is
`#[non_exhaustive]`. The internal constraints are two-digit, so dependents inherit the bump without
edits — a property to confirm on the first release rather than assume.

`deep_causality_algebra`'s additions are not breaking in this workspace: twelve manifests declare it,
`Real` has two impls that we own, and `RealField`'s single blanket reaches it through `Float`, which
already implies `ToPrimitive`. An earlier draft called this stage BREAKING across nineteen manifests;
both the count and the characterisation were wrong, taken from a scoping report rather than checked.

**The test burden is the larger half** → Measured src-to-test ratios in this workspace run 0.56 to
1.42, median near 1.0. C2 is roughly 1500 source lines and 2700 test lines across 109 files. An
estimate that counts only source is wrong by a factor of two, and the phase-2 gate makes the test
cost visible before implementation rather than after.

**Mutation testing is expensive** → Every mutant costs a build plus a test run, and one crate's
algorithms directory alone yields over a thousand. It is scoped per stage to the files that stage
touched, never over the workspace. Equivalence entries are regex against regex-laden text and fail
silently both ways, so each is escaped and confirmed with the `comm` check.

**A new crate's first release** → A crate that depends on an unpublished sibling version cannot
publish. The release ordering follows the tier order, and the new crate is released after the
`algebra` bump it depends on.

**Five stages is still a large change** → The repository's archive holds 212 changes, most of them
small. This one is a programme, and the mitigation is that each stage is independently shippable and
independently archivable in its task group, rather than one commit at the end. The engine-alias work
was the sixth and is deferred, which removes the largest design unknown from the programme.

## Migration Plan

1. **C1** — Meek. Search first, then move, then R4, then chordality, then the three call sites.
   Independent of everything else.
2. **C3** — `cbrt` and `ToPrimitive`, then the workaround retirements in the same stage. Breaking;
   goes early so the dependent stages build against the final bound.
3. **C2** — the statistics crate, then consumer migration gated on its suite being green.
4. **C4** — `linear` adoption, classified per site; the two `linear` defects first, since consumers
   will start depending on the fixed behaviour.
5. **C5** — the two verdict-carrier instances in `algebra`. Independent of everything else.
6. **C6** — root finders, landing without touching a kernel; kernel migration is a separate task.

Rollback is per stage. Each is a distinct task group whose tests pass on their own, so an abandoned
stage leaves the others standing.

## Open Questions

All four questions this change opened have been answered by the maintainer and are recorded as
decisions: `tensor` does not depend on the statistics crate (D7); the engine's numeric aliases stay
as they are and their rework is deferred (D13); the physics entropy kernel moves to bits and its name
states its base (D14); `ScalarValue` is not narrowed (D13).

One question is left open deliberately, for the deferred change rather than for this one: **which of
the two alias sets is the real one**, given that `deep_causality_core`'s eleven aliases are unused
workspace-wide while `deep_causality` carries a live duplicate of the same names. Answering it is a
prerequisite for unpinning either, and it is the reason the rework is a change of its own.
