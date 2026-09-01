<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: design returns a plan of experiments

`design` SHALL return a `DesignPlan` rather than one experiment, carrying an ordered list of the
chosen experiments, the total cost of that list, the hypothesis pair each chosen experiment resolves,
and every pair left uncovered. The crosstalk consumer's answer is a pair of interventions, and a
single experiment cannot express one.

The plan ranges over the hypotheses that survived `validate`, so a candidate rejected by
`check_faithfulness` contributes no pair to the universe and costs no device time.

#### Scenario: A plan names the interventions and what each one resolves

- **WHEN** `crosstalk_attribution` declares four candidates, `check_faithfulness` rejects the cyclic
  one as a `C₃`, and `.design(MinCostCover { floor_bits: ft(5.0) })` runs over `do_q1`, `do_q2`,
  `echo_both` and `process_tomography`
- **THEN** the returned `DesignPlan` lists `do_q1` and `do_q2` in order, reports the total cost of
  that list, and names for each entry the pair of the three surviving hypotheses it resolves

#### Scenario: The same instance yields the same plan

- **WHEN** two covers of equal total cost exist for one instance
- **THEN** `design` returns the same ordered plan on every run, breaking the tie over the declared
  experiment order, so a plan is reproducible from the configuration alone

### Requirement: The plan is the exact minimum-cost cover over the hypothesis pairs

`design` SHALL solve minimum-cost set cover whose universe is the `C(n, 2)` pairs over the `n`
surviving hypotheses and whose sets are the `k` offered experiments, as a dynamic program over
subsets of covered pairs that relaxes `dp[S | cover(e)]` against `dp[S] + cost(e)`. The solve is
`O(2^C(n,2) · k)`, linear in `k` and exponential in `n`. Enumerating subsets of experiments at `2^k`
is the wrong enumeration and SHALL NOT be used.

An experiment `e` covers the pair `(h_i, h_j)` when the predicted separation between the two
hypotheses' read-outs at `e` reaches `MinCostCover`'s `floor_bits`.

#### Scenario: The cover is optimal rather than greedy

- **WHEN** an instance admits a greedy cover at cost 3 and an optimal cover at cost 2, with
  `process_tomography` covering every pair alone at cost 200
- **THEN** `design` returns the cost-2 plan, because the dynamic program is exact over the pair
  universe and a per-experiment score is not consulted

#### Scenario: Offering more experiments costs linearly

- **WHEN** `n` stays at 3 and the offered experiments grow from 4 to 40
- **THEN** the table keeps its `2^3` entries over the three pairs, the work grows by the factor `k`
  grew by, and the returned plan is still the exact optimum

#### Scenario: Too many hypotheses fails loudly

- **WHEN** a configuration screens more hypotheses than the declared cap on `n`
- **THEN** `design` returns a structured error naming `n` and `C(n, 2)` before allocating the table,
  rather than starting a solve it cannot finish

### Requirement: Pairs no experiment resolves are reported

`DesignPlan` SHALL list every hypothesis pair that no offered experiment resolves at `floor_bits`,
together with the number of pairs examined, so a plan that discriminates nothing is visible as one.
A partial cover is a plan with an uncovered list rather than a failure.

#### Scenario: A plan that discriminates nothing

- **WHEN** every offered probe's predicted separation on every pair falls below `floor_bits`
- **THEN** `design` returns a plan with an empty experiment list, a total cost of zero, all
  `C(n, 2)` pairs listed as uncovered, and an examined-pair count equal to `C(n, 2)`

#### Scenario: A partial cover is returned with its gap

- **WHEN** four of five pairs are covered at total cost 4 and the fifth is separated by no offered
  experiment
- **THEN** `design` returns the cost-4 plan, lists the fifth pair as uncovered, and returns `Ok`

### Requirement: design and adjudicate report a measured quantity, a threshold and a count

`design` and `adjudicate` SHALL each return their decision in the crate's shared decision form, a
measured quantity against a threshold with a margin and a count of what was examined, rather than a
bare `bool` or a bare choice. `design` measures worst-pair separation against `floor_bits` over the
pairs it examined. `adjudicate` measures separation at the taken shots against `floor_bits` over the
worlds it folded. This is the shape `CommutatorCheck` records and `QuantumMarkovReport::tested_pairs`
and `worst_margin` expose, generalised by `Check<R>` and `CheckReport<R>`.

#### Scenario: design reports the pair that is closest to the floor

- **WHEN** a plan covers all three pairs and the tightest of them separates at 5.2 bits against
  `floor_bits = ft(5.0)`
- **THEN** the returned report carries the measured 5.2, the threshold 5.0, the resulting margin, and
  an examined count of 3 pairs

#### Scenario: A fold over one world is visible as vacuous

- **WHEN** `adjudicate` runs on a fork that left one live world
- **THEN** it reports a folded-world count of 1 alongside its separation, so the reader sees that
  nothing was discriminated

### Requirement: A projection-valued fold checks commutation

`adjudicate` SHALL test `Projection::commutes_with` on every pair of projection-valued verdicts
before combining them, because `Projection<R, D>` is orthomodular and fails distributivity outside
the commuting family. A fold whose projections do not all pairwise commute SHALL return `Ambiguous`,
naming the offending pair and the number of pairs tested.

#### Scenario: Non-commuting verdicts fold to Ambiguous

- **WHEN** two forked worlds carry the rank-1 projections onto `|0⟩` and `|+⟩`, whose commutator
  defect exceeds `Projection::default_tolerance`
- **THEN** `adjudicate` returns `Ambiguous` naming that pair and the pairs-tested count, and declares
  no surviving hypothesis

#### Scenario: A commuting fold answers

- **WHEN** every pair of projection-valued verdicts commutes within the carrier's tolerance
- **THEN** the verdicts combine through `Verdict::meet` and `Verdict::join`, and `adjudicate` returns
  the surviving hypothesis on one side of `deep_causality_haft`'s `Either`, with the residual
  ambiguity on the other

### Requirement: The commutation guard applies to projection-valued verdicts only

`adjudicate` SHALL select the fold by the kind of verdict a world carries: projection-valued verdicts
take the commutation test, and read-outs judged against a real-valued spec do not. A threshold on a
real quantity is a classical proposition, those form a Boolean algebra where distributivity holds
unconditionally, and applying the guard there would reject sound folds.

#### Scenario: A real-valued spec fold runs no commutation test

- **WHEN** the calibration pipeline forks on `Spec::at_least(ft(0.999))` after `observe` and
  `adjudicate` folds the three resulting worlds
- **THEN** no commutation test runs, and the commutation guard produces no `Ambiguous` on that path

#### Scenario: Verdicts reach adjudicate from the measurement boundary

- **WHEN** a world reaches `adjudicate` carrying an operator value that no `observe` turned into a
  verdict
- **THEN** `adjudicate` constructs no verdict from it, because verdicts are extracted at the
  measurement boundary and no blanket `Verdict` impl exists over a general operator type

### Requirement: The design stage carries its precision on the axis each quantity lives on

Every quantity the design stage carries SHALL be written against an algebraic bound and SHALL take
its width from the run's parameters rather than from a literal type. A separation in bits and a cost
are real, so they are `R` and follow `FloatType`; an experiment count, a shot count and the covered
subsets of the dynamic program are ℕ, so they are bounded on `NaturalNumber` and follow `IntType`.

The two axes are not interchangeable. `floor_bits` is compared against a measured separation and its
comparison SHALL derive from the `Tolerance<R>` family, because both sides are real and the failure
mode is rounding. A count SHALL NOT be given a tolerance, because its failure mode is overflow and
overflow has no `epsilon()` to bound it.

#### Scenario: The floor and the separation move together with the scalar

- **WHEN** `FloatType` is changed and the same design problem is solved again
- **THEN** `floor_bits`, the measured separation and the cover's total cost all re-type with it, and
  the comparison between separation and floor stays inside the tolerance family

#### Scenario: The cover's bookkeeping is exact

- **WHEN** the dynamic program accumulates a cost over a subset of covered pairs
- **THEN** the subset index and the experiment count are integer quantities carried on
  `NaturalNumber`, and no tolerance is applied to either

#### Scenario: An exhausted experiment budget is reported, not rounded

- **WHEN** a plan's cost is drawn against a budget that cannot cover it
- **THEN** the shortfall is reported through `checked_difference` returning `None` rather than
  through a negative number or a clamped float
