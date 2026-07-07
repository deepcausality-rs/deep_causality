## ADDED Requirements

### Requirement: Every in-wire resolves; a node fires on resolved wires, never on arrival order

The engine SHALL associate one slot per in-wire (parent → node edge) and resolve every slot to either `Fired(effect)` or `Inactive`. A node SHALL fire only when all of its in-wire slots are resolved and at least one is `Fired`; a node whose slots are all `Inactive` SHALL resolve `Inactive` without firing. The engine SHALL resolve wires from non-descendants of the start node as `Inactive` at initialization (reachability pre-pass), and SHALL resolve the abandoned cone as `Inactive` when a `RelayTo` command redirects evaluation. Ready nodes SHALL be processed in ascending node index (canonical schedule). The classical run REQUIRES a frozen acyclic graph (`ultragraph::has_cycle`); a cyclic graph is rejected.

#### Scenario: Reconvergence node waits for both parents

- **WHEN** a node has parents `P1` and `P2`, both on active paths, and `P1` fires first
- **THEN** the node is not evaluated until `P2`'s wire is also resolved, and the node then fires exactly once

#### Scenario: Mid-graph start does not deadlock

- **WHEN** evaluation starts at a node `S` and a downstream reconvergence node has a parent that is not a descendant of `S`
- **THEN** that parent's wire resolves `Inactive` at initialization and the node fires from its fired parents alone

#### Scenario: RelayTo abandons a branch without deadlock

- **WHEN** a `RelayTo` command redirects evaluation away from a pending branch feeding a reconvergence node
- **THEN** the abandoned wires resolve `Inactive`, the current round ends, and evaluation continues at the relay target with the command's sub-program (sequential composition of rounds)

#### Scenario: Dead paths propagate

- **WHEN** all parents of a node resolve `Inactive`
- **THEN** the node resolves `Inactive` without evaluating, and its own out-wires resolve `Inactive`

#### Scenario: Linear and tree graphs are unaffected

- **WHEN** every node has at most one parent
- **THEN** evaluation results are bit-identical to the previous engine

### Requirement: Fan-in delivers labeled parent effects to the node's join mechanism

At a node with two or more `Fired` parents, the engine SHALL deliver the fired parent effects keyed by parent node index (`ParentEffects`, canonical ascending-key iteration) to the node's declared join mechanism, which produces the single effect the node consumes. Any join function SHALL be permitted (asymmetric mechanisms included); determinism derives from the keys, not from algebraic properties of the function. With exactly one `Fired` parent the node SHALL receive that parent's effect unchanged (join of one is identity; no declaration required). A node with two or more `Fired` parents and no declared join mechanism SHALL yield a descriptive `CausalityError` naming the node and its fired parents. Engine-owned channel rules apply before the join: error = short-circuit on the first error in ascending parent-index order (left-zero); log = ascending parent-index concatenation. The `Causaloid` API change SHALL be additive only (`join_fn` field + `new_join*` constructors following the existing `Option<CausalFn>` static-dispatch pattern), with a `ContextualJoinFn` variant receiving `(&ParentEffects<I>, Option<&CTX>)` so kernel configuration rides the context channel as `ContextualCausalFn` config does.

#### Scenario: Asymmetric join mechanism

- **WHEN** a node declares a join computing `w1·x1 − w2·x2` over parents `1` and `2`
- **THEN** the engine delivers `{1 ↦ x1, 2 ↦ x2}` and the result distinguishes the parents regardless of which fired first

#### Scenario: Join of one fired parent is identity

- **WHEN** a structurally reconvergent node receives exactly one `Fired` parent at run time
- **THEN** the node evaluates that effect as-is, with no join declaration required

#### Scenario: Undeclared multi-fired reconvergence errors loudly

- **WHEN** two or more parents fire into a node that has no declared join mechanism
- **THEN** evaluation yields a `CausalityError` identifying the node and the fired parent indices

#### Scenario: A parent error short-circuits the join

- **WHEN** any fired parent effect carries an error
- **THEN** the combined result is that error (first in ascending parent-index order), the join mechanism is not invoked, and logs are preserved in canonical order

### Requirement: The LinearJoin multi-parent kernel ships with the change

The change SHALL implement `LinearJoin<R: Scalar>` — config `{ weights: BTreeMap<usize, R>, bias: R }` carried on the join causaloid's context channel, evaluated by a shipped fn-pointer `linear_join` computing `bias + Σ_{p ∈ fired} weights[p] · v_p` in ascending key order. Per-parent value policy: a command on the value channel yields a `CausalityError`; `Pure(None)` contributes nothing; a fired parent without a weight entry contributes nothing; missing config yields a `CausalityError`. The kernel SHALL be generic over `Scalar` (precision-free; `Dual` passes through), SHALL be surgery-local (removing a wire key or a parent resolving `Inactive` changes the result by exactly that parent's term, with no kernel redefinition), and its docstring SHALL cite Pearl (2009) and Lorenz (2022). The kernel is the designated lift target for the QCM change (complex tier via the num crate's complex field; operator-valued entries deferred) — its shape SHALL NOT assume `R` is real.

#### Scenario: Weighted asymmetric join

- **WHEN** a diamond node declares `LinearJoin { weights: {1 ↦ w1, 4 ↦ w2}, bias: b }` and parents `1` and `4` fire with values `x1`, `x4`
- **THEN** the node's input value is `b + w1·x1 + w2·x4`, independent of arrival order

#### Scenario: Surgery locality

- **WHEN** the wire from parent `1` is cut (or parent `1` resolves `Inactive`) and the node re-evaluates
- **THEN** the result is `b + w2·x4` — the same kernel, no redefinition — and the per-kernel Lean lemma states this equality with a Rust witness

#### Scenario: Dual-number sensitivity

- **WHEN** the kernel runs at `R = Dual<S>` with parent `1` seeded as the variable
- **THEN** the `ε` channel of the output equals `w1` (intervention sensitivity), witnessed in a test

#### Scenario: Defined behavior on degenerate parents

- **WHEN** a fired parent carries a command on its value channel, or `Pure(None)`, or has no weight entry
- **THEN** the kernel yields a `CausalityError` for the command, and a no-contribution term for `Pure(None)` and for the missing weight

### Requirement: Determinism is proved as unique valuation plus schedule invariance

The formalization SHALL prove in `Core/GraphJoin.lean` (bare-`lean`, self-contained): (a) `unique_valuation` — the acyclic labeled equation system `σ(n) = f_n(σ|Pa(n)^Fired)` has exactly one total resolution, by well-founded induction, with no algebraic hypotheses on mechanisms; (b) `schedule_invariance` — every admissible schedule computes that valuation, for command-free runs; (c) disjoint-key map-union lemmas (commutativity/associativity/idempotence by construction); (d) `inactive_discard`. Relay composition SHALL be definitional (`eval = round; on command, recurse at target`), with its determinism stated as a separate result scoped to the canonical schedule. Each theorem MUST carry a `THEOREM_MAP.md` row (the command-free scope of `schedule_invariance` stated in the row) and a Rust witness.

#### Scenario: Two layouts agree

- **WHEN** the same diamond model is built under two different node-index assignments (producing two distinct canonical schedules)
- **THEN** the Lean model proves the valuations equal and the Rust witness confirms the real engine agrees modulo relabeling

#### Scenario: No commutativity obligation on mechanisms

- **WHEN** a join mechanism is an arbitrary (non-commutative, non-associative) function of its labeled parents
- **THEN** `unique_valuation` and `schedule_invariance` hold unchanged, because determinism derives from the keyed structure

### Requirement: The substrate serves both classical do-surgery and QCM factorization

The labeled structure SHALL be interpreter-neutral: fan-out is out-wire structure only, with the copy law (`every out-wire of n carries σ(n)`) stated in Lean as a law of the classical interpreter, not of the substrate (no-cloning compatibility). `Pa(n)` SHALL be exposed keyed by parent index so that (a) do-surgery is expressible as wire-key deletion plus mechanism pinning, and (b) the QCM factorization `σ = ∏ ρ_{A|Pa(A)}` and its `PairwiseCommute`/`ValidProcess` predicates (formalize-main-crate) attach to the same labeled parent sets, including overlapping ones. Acyclicity SHALL be enforced as classical-run admissibility only, leaving the substrate digraph-agnostic for the deferred cyclic-QCM relaxation.

#### Scenario: Copy law is scoped to the classical interpreter

- **WHEN** the Lean model states the fan-out law
- **THEN** it is a theorem about the classical evaluation algebra, and no substrate-level duplication law exists that a quantum instantiation would violate

#### Scenario: Edge surgery has a labeled target

- **WHEN** an intervention severs the influence of parent `P1` on node `X` while preserving parent `P2`
- **THEN** the surgery is the deletion of wire key `(P1, X)`, expressible because parent contributions are keyed, and the next evaluation of `X` joins over the remaining fired parents

### Requirement: Reconvergent tests updated to declared-join semantics

Existing tests that build reconvergent (multi-parent) graphs SHALL be updated: graphs whose reconvergence fires multiple parents declare a join mechanism and assert the result derived from that declared mechanism (not from arrival order, and not from current engine output); graphs where only one branch activates at run time are confirmed unchanged. The change SHALL first scan tests and examples to enumerate all reconvergent graphs and classify them (multi-fired vs single-fired; stateless vs stateful).

#### Scenario: Diamond test asserts the declared join

- **WHEN** the `logic_graph` diamond (node `idx2`, parents `idx1`/`idx4`) fires both parents
- **THEN** its test declares a join mechanism and asserts the mechanism-derived value

#### Scenario: Golden tests pin the unaffected surface

- **WHEN** the linear and tree graph test suites run against the wire-slot engine
- **THEN** all results are bit-identical to the previous engine's recorded outputs
