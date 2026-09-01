<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## Context

`deep_causality_quantum` is 4432 lines of working substrate: the Markov commutativity check and its
depth-aware tolerance, the C₃-exclusion search, the Choi round trip and composition, the orthomodular
`Projection<R, D>` verdict carrier, the Haruna gate layer over `Gf2Chain`, and the 𝔽₂ homology and
representative machinery under it. Four consumers exercise it, one of them running.

QCL is the language over that substrate. Its design is
[`qcl-design-note.md`](../../notes/quantum/qcl-design-note.md), re-verified against the tree on
2026-08-31. Every gap in [`qcl-gaps.md`](../../notes/quantum/qcl-gaps.md) is closed, so nothing here
waits on missing mathematics.

Two constraints bound everything below. The **modality split** is enforced by the compiler: the
verifiable path is the default build and the emergent path sits behind `qpu`. And the **verdict law**
is a property of `Projection<R, D>`, which is orthomodular and fails distributivity, so where a
verdict may enter and how verdicts may combine are not free choices.

An independent five-lens review of this design ran on 2026-08-31. Of thirteen blockers raised,
eleven were refuted against the shipped code and two survived. Both survivors are recorded as
decisions below rather than left as risks.

## Goals / Non-Goals

**Goals:**

- Turn every decision into a margin with a count, so no stage returns a bare `bool` and a vacuous
  pass is visible as one.
- Give the pipeline typed carriers that seal their validated interiors, replacing hand-packed tensors
  with manual shapes.
- Make read-out decisions statistical rather than exact-real comparisons.
- Express all four consumers through one builder and one stage vocabulary, with the
  `validate` → `Screened<R>` → `control` hand-off unskippable.
- Decide `check_class_invariance` so the geometric-QEC consumer's second check is implementable.

**Non-Goals:**

- Decoding. Stim and PyMatching decode; the contribution is upstream.
- Fault-tolerance claims, or Lean coverage the tree does not have.
- Device models. The rotation-with-detuning plant belongs to an example.
- Owning graph traversal, topology, or the `Verdict` trait.
- Real-time control-loop latency. The loop is FPGA or equivalent; QCL is the design-time tool that
  decides what goes into it, so §10's tick-latency framing is retired rather than pursued.

## Decisions

### D1. `check_class_invariance` is decided over the code space, not the full Hilbert space

**Measured, not assumed.** A criterion quantifying over every basis state decides Z̄ correctly on all
36 (boundary, cohomology generator) pairs of the 3×3 simplicial torus and rejects S̄ and T̄ on the
first pair. That is not an implementation defect. Haruna's Eq. (3.21) writes
`S(∂₂f) = exp(i(π/2)(I − S_Z(f)))` and closes because `S_Z(f)` acts as the identity **on the code
space**; a full-space criterion is strictly stronger and rejects sound gates.

So the predicate takes the stabilizer generators. For a CSS code from a chain complex the
Z-stabilizers are the `∂₂` columns and the X-stabilizers the `δ₀` columns, both already reachable
through `ChainComplex`. `LogicalBasis` gains them, and the enumeration ranges over the code space.

*Alternatives rejected.* Comparing unitaries: the consumer's own subject is a 32-qubit toric code and
`SimQpu` caps at 24, so this is dead on arrival. Narrowing the check to Z̄ and X̄: decidable today
with the shipped Pauli predicate, and it discards S̄, T̄, CS† and CCZ, which is where Haruna's
contribution lives.

### D2. Gate phases are exact rationals, so the check carries no tolerance

Every diagonal Haruna gate is `exp(2πi·Q(n)/M)` for an integer polynomial `Q`, a power-of-two `M`,
and `n` the overlap `|supp(γ) ∩ x|`. `Q(n)/M` is a rational by construction and the question asked of
it is integrality, so `Rational<i64>` from `deep_causality_num_rational` carries it and the residual
is exactly zero or exactly not.

The polynomials are pinned by their single-qubit gates: at `n = 1`, `Z̄` gives a half turn, `S̄` a
quarter, `T̄` an eighth, matching `diag(1, −1)`, `diag(1, i)` and `diag(1, e^{iπ/4})`.

*Consequence for §3.1's table.* The `check_class_invariance` row's threshold is not a tolerance. It
reports an exact residual and a count, which is a stronger member of the decision form rather than an
exception to it.

*Trap this already caught.* A phase lives in ℝ/ℤ. Comparing two `Turns` for equality rather than
testing that their difference is integral reports `3/2` and `1/2` as different phases. The rational
carrier makes that visible; a float would have hidden it under a tolerance.

### D3. `predict` is model evaluation, and marginalisation is a separate gated operation

The note uses `predict` in two senses and the review flagged it. It is settled by the ledger's own
field: `predictions: N, // model evaluations; tracked, never billed`. `predict` evolves each forked
world under the chosen probe. Marginalising a validated factorization is a distinct operation, gated
on `partial_trace_preservation_boundary`, and it is what §8 rule 6 forbids doing unguarded.

The contraction `predict` needs ships: `FactorSupports::space_map` gives the leg-to-dimension map,
`embed_on_legs` lifts each factor onto it, and the product is the joint operator. The review built
and ran that contraction from those parts.

### D4. `fork` is built above core, because routing and forking are dual

`Either<L, R>` in `deep_causality_haft` is the coproduct, and `CausalFlow::either` consumes the flow
and runs **one** arm with the state moved into it. A counterfactual fork needs every world live with
its own ledger copy, which is a product. No eliminator of a coproduct produces one. `Either` remains
the right carrier on the way out, where `adjudicate` returns one surviving hypothesis against a
residual ambiguity.

### D5. The decision form wraps the shipped checks; three of them need a report-returning path

`CommutatorCheck` already has the target shape. Three shipped checks do not expose what §3.1 says
they measure: `check_completely_positive` and `check_trace_preserving` return `Result<(), _>` after
computing a spectrum and a defect, and `quantum_markov_check` returns `Err` on the first failing pair
and drops the whole report, so a rejected candidate reports no margins and no `tested_pairs()`.

QCL does not reimplement them (§8 rule 5). Each gains a sibling that returns its `CheckReport<R>`,
with the existing signature kept, so this is additive rather than a breaking change to a crate at
0.2.0. The failure path is where §3.2's count obligation matters most, so it is where the report must
survive.

### D6. Two working types, and the tolerance family has no integer member

`config::<FloatType, IntType>()`. Widening `FloatType` buys accuracy, bounded by `epsilon()`.
Widening `IntType` buys headroom against overflow, which nothing bounds. So the §3.3 tolerance family
covers the real side only, and the integer side gets checked arithmetic: `NaturalNumber`'s
`checked_difference` returns `None` on an overdrawn budget and `monus` clamps, which is exactly a
ledger draw-down and needs no hand-written guard.

### D7. `design` is a DP over covered-pair subsets

Minimum-cost set cover whose universe is the `C(n,2)` hypothesis pairs and whose sets are the `k`
experiments. The exact solve is `dp[S | cover(e)] = min(dp[S], dp[S] + cost(e))` at
`O(2^C(n,2) · k)`: **linear in k, exponential in n**. Enumerating experiment subsets would be `2^k`,
which is the wrong enumeration and the source of the note's retired "exponential in k". `design`
returns a `DesignPlan`, because the crosstalk consumer's answer is a pair of interventions and a
single experiment cannot express one.

### D8. The circuit data types are always compiled; the sampler seam stays gated

`GateOp` and `QuantumCircuit` left the `qpu` gate when the Haruna layer was retyped, because the
always-on gate layer emits them. `QpuSampler` and `SimQpu` remain behind `qpu`. A named shot budget
selects the emergent modality, and because the modality is a compile-time guarantee, a config naming
shots in a build without `qpu` is a build error rather than a runtime one.

## Risks / Trade-offs

**[`LogicalBasis` gains stabilizers, changing a type shipped this cycle]** → The constructor is
additive: `from_complex` keeps its meaning and a second constructor supplies the stabilizer
generators. `is_logically_trivial` over Paulis is unaffected, and its agreement with the diagonal path
on Z̄ is already a test.

**[The code-space enumeration could be expensive on a wide code]** → The atom decomposition is
bounded by the union of the supports involved, not by the register width, and the implementation
already carries an explicit cap that errors rather than hanging. Cost is reported, and a code that
exceeds it fails loudly.

**[`ShotBudget` has no sampling path for the plant carrier]** → `born_projective_probability` ships
in the default build, generic in the scalar, on the `DensityMatrix` the calibration plant uses. The
sampler that turns a probability into shots is the new part, and it must follow `FloatType` rather
than pinning `f64`, or §10.4's precision sweep breaks exactly where both control consumers decide.

**[Generalising `Check` first means every later stage is written against it]** → That is the point of
its position in the sequence, and the cost of getting it wrong is a rewrite of every stage. It is
sequenced first for that reason and its shape is taken from a shipped struct rather than invented.

**[`Uncertain`'s SPRT holds a process-global lock and never evicts its cache]** → Out of scope for
this change and recorded here because a design-time sweep runs more of these than a control loop
would. Flag it rather than build on it.

## Migration Plan

Additive throughout. No existing public function changes signature; the report-returning siblings of
D5 are new names beside the existing ones. `deep_causality_quantum` is at 0.2.0 after the Haruna
retyping, and this change is a minor bump rather than a breaking one.

### D9. A Markov certificate is inherited at two factors and nowhere else

The question was whether the Choi contraction preserves the commutation structure `check_markov`
certifies. **The literature leaves it open and says so**, which settles what QCL may claim.

Def 3.3 requires the factors of `σ` to commute pairwise, and that condition is substantial rather
than incidental because the parental sets generally overlap. Lorenz (2022) records at footnote 11
that while pairwise commutation follows from hermiticity in the two-factor case (QCCP),
*commutativity of the factors does not follow from the hermiticity of `σ` when there are more than
two factors*. Lorenz & Barrett (arXiv:2001.07774) leave the general case to future work, naming the
missing ingredient as results in operator algebra "pertaining to sets of three or more pairwise
commuting algebras" — precisely the theory an inheritance proof would need. §8 rule 7 forbids
claiming a theorem the tree does not have, and this is one the framework's own authors do not have.

So the default is **do not inherit; re-run `check_markov` on the composite**, with a sound fast path
at exactly two factors, where hermiticity does give commutation. An inherited certificate records
that it was inherited, so a reader cannot mistake it for pairs tested on the composite.

The trade is verification cost against soundness. Re-checking costs `O(n²)` in the factors with
`embed_on_legs` dominating, paid at freeze time rather than per tick, which is the cheap axis for a
design-time tool. The failure it prevents is silent: a composite violating pairwise commutation still
produces numbers and raises nothing.

Composition itself is not the obstacle. `choi_compose` is a plain double contraction over the shared
wire with no partial trace and no partial transpose, verified to a maximum relative Frobenius
residual of 3.198e-16 over 500 random CPTP pairs. F9 does not bite here. What stays open is the
narrow question of whether that contraction preserves commutation, and v1 does not need it, because
reuse is served by expansion rather than by nesting.

### D10. `check_class_invariance` reports a witness and two counts, never a margin

**A phase has no order.** Under D2 the residual is a phase difference in ℝ/ℤ carried as an exact
`Rational`. A residual of a half turn is not worse than one of an eighth turn; they are different
failures rather than graded ones. A margin presupposes a metric on failure, and this quantity has
none, so reporting a magnitude would invent an ordering with no physical content that a caller would
eventually sort by.

The crate has already answered this for its other exact check. `check_c3_exclusion` reports no
margin: it returns `NotFaithfullyRepresentable` naming the inputs and outputs that witness the C₃
obstruction. **A check whose quantity is a norm reports a margin, because norms are ordered; a check
whose quantity is exact reports the obstruction.** Q-TOL is the first kind. C₃ and class invariance
are the second.

So the report carries the failing shift and the three block occupancies of the basis state that
witnessed it, plus two counts: how many shifts were examined, which discharges §3.2's vacuous-pass
obligation, and how many states were visited, which is what makes an expensive code visible before it
is a hang. The occupancies determine both overlap counts, so they determine the phase, so they are
the counterexample and it can be recomputed by hand without the checker. Stopping at the first
failure is then a cost decision rather than a semantic one, and both counts describe what was
actually visited either way.

*Implementation note.* The witness carried no cohomology generator in the end. Deciding triviality on
the code space is a direct statement about the ratio's phase, not a commutation statement, so `H¹`
never enters the check — see the `qcl-code-checks` requirement for the derivation that removed it.

*Alternative rejected.* Worst-margin over all pairs. It is not merely degenerate under exact
arithmetic; it is meaningless, and it would read as informative.

## Open Questions

None blocking. Two are recorded elsewhere and deliberately out of scope: whether the Choi
contraction preserves commutation in the narrow sense D9 leaves open, which needs upstream operator
algebra; and `Uncertain`'s SPRT holding a process-global lock with a cache that never evicts, which
a design-time sweep exercises harder than a control loop would.
