# DeepCausality, distilled: the effect ethos

Source: every file in `deep_causality_ethos/` (0.4.1), read in full on 2026-09-30: `src/`, `tests/`,
`Cargo.toml`, `BUILD.bazel`, `README.md`, `CHANGELOG.md` and `examples.txt`. Checked against the
one consumer, `examples/csm_examples/csm_effect_ethos/`, the CSM in `deep_causality/`, and the DDIC
paper in `papers/ddic.pdf`. Behavioural claims marked *(probe)* were confirmed by a scratch
program against the crate; the program is not part of the repository.

## The finding

The effect ethos is a norm checker. It takes a proposed action, a context and a set of tags, and
returns a verdict, `Impermissible`, `Obligatory` or `Optional(cost)`, with the IDs of the norms
that produced it. A norm (`Teloid`) is a predicate over `(Context, ProposedAction)` plus a
modality and three numbers for conflict resolution. Norms sit in a graph whose edges are either
`Inherits` or `Defeats`.

The verdict is not a function of its inputs. Conflict resolution walks the norms in `HashSet`
iteration order and decides defeat against whatever has not yet been removed. With a chain where
norm 1 defeats norm 2 and norm 2 defeats norm 3, the same ethos, action and context returned
`Impermissible [1, 3]` on 1126 of 2000 calls and `Optional(1) [1]` on 874 *(probe)*. The README
calls resolution "deterministic and reproducible".

Three further properties decide what a verdict means. Norms are selected by tag, and the norm's
`action_identifier` is never compared with the action: a norm written for `takeoff` rules on
`land` *(probe)*. A norm reached through an `Inherits` edge joins the result without its predicate
being evaluated: an inherited prohibition whose predicate is false still makes the verdict
`Impermissible` *(probe)*. And a defeat edge fires when the defeater exceeds the defeated norm on
specificity, timestamp *or* priority, so a newer norm defeats an older one that is a hundred times
more specific and a hundred times higher in priority *(probe)*.

Nothing in the causal engine calls the ethos. The CSM method `fire_action_with_ethos_check` fires
the action unconditionally; its body carries the comment "Ethos checking has been moved to
deep_causality_ethos crate". The single example runs the CSM and the ethos side by side, prints the
alert, and then prints that the ethos forbids it.

## 1. The pieces

```rust
pub struct Teloid<D, S, T, ST> {                  // one norm
    id: TeloidID,                                 // u64
    action_identifier: String,                    // stored, displayed, never matched
    activation_predicate: Option<fn(&Context<D,S,T,ST>, &ProposedAction) -> bool>,
    uncertain_activation_predicate: Option<UncertainActivationPredicate<D,S,T,ST>>,
    uncertain_parameter: Option<UncertainParameter>,
    modality: TeloidModal,                        // Obligatory | Impermissible | Optional(i64)
    timestamp: u64, specificity: u32, priority: u32,
    tags: Vec<TeloidTag>,                         // &'static str
    metadata: Option<HashMap<String, String>>,
}

pub struct EffectEthos<D, S, T, ST> {
    teloid_store: TeloidStore<D,S,T,ST>,          // HashMap<TeloidID, Teloid>
    tag_index: TagIndex,                          // HashMap<TeloidTag, HashSet<TeloidID>>
    teloid_graph: TeloidGraph,                    // UltraGraphWeighted<TeloidID, TeloidRelation>
    id_to_index_map: HashMap<TeloidID, usize>,
    is_verified: bool,
}
```

A teloid has exactly one predicate, deterministic or uncertain; the two constructors enforce it.
`Teloid` equality compares IDs only. `UncertainActivationPredicate` is
`fn(&Context, &ProposedAction) -> Result<UncertainBool, CausalityError>`, and `UncertainParameter`
defaults to threshold 0.8, confidence 0.95, epsilon 0.05, 1000 samples. `EffectEthos` is generic
over the four context parameters, so the crate depends on `deep_causality` (for `ProposedAction`
and the uncertain types), `deep_causality_context` and `ultragraph`, and nothing else.

## 2. Building an ethos

```rust
let mut ethos = EffectEthos::new()
    .add_deterministic_norm(1, "takeoff", &["flight_safety"], battery_low,
                            TeloidModal::Impermissible, /*timestamp*/ 0, /*specificity*/ 10, /*priority*/ 100)?
    .add_uncertain_norm(2, "takeoff", &["flight_safety"], wind_gusty, UncertainParameter::default(),
                        TeloidModal::Impermissible, 0, 5, 50)?
    .link_defeasance(1, 2)?;                      // or link_inheritance(parent, child)
ethos.verify_graph()?;                            // freezes, rejects a cycle
```

Adding a norm inserts it into the store, the tag index and the graph and clears `is_verified`. A
duplicate ID is `FailedToAddTeloid`. Linking looks up both IDs (`TeloidNotFound`) and refuses a
frozen graph (`GraphIsFrozen`). `verify_graph` freezes the graph and checks for a cycle; on a cycle
it unfreezes the graph and returns `GraphIsCyclic`. `unfreeze` clears `is_verified`.

`EffectEthos::from(store, index, graph)` assembles an ethos from parts and derives the ID-to-index
map from the graph's node order. It checks nothing; a norm in the store but not the graph fails at
evaluation with `TeloidNotFound`.

## 3. Evaluating an action

`evaluate_action(&action, &context, &tags) -> Result<Verdict, DeonticError>`
(`types/effect_ethos/deontic_inference.rs`):

1. **Gate.** An unfrozen graph is `GraphNotFrozen`. A frozen but unverified graph is
   `GraphIsCyclic`, whether or not it has a cycle.
2. **Select.** Collect the IDs under every requested tag into a `HashSet`. None is
   `NoRelevantNormsFound`.
3. **Activate.** Run each candidate's predicate. An uncertain predicate is active when
   `probability_exceeds_from_entropy(threshold, confidence, epsilon, max_samples)` returns true. A
   predicate that returns an error, or a test that returns an error, counts as inactive. No active
   norm is `InconclusiveVerdict`.
4. **Resolve** (`resolve_conflicts.rs`). Seed a breadth-first queue with the active norms, in the
   `HashSet`'s order, and put them all in a belief map. For each dequeued norm, look at its
   incoming `Defeats` edges. If a defeater is still in the belief map and has higher specificity,
   later timestamp or higher priority, remove the norm and do not expand it. Otherwise add every
   `Inherits` child to the belief map and the queue, without evaluating the child's predicate.
5. **Verdict** (`derive_verdict.rs`). Any `Impermissible` survivor makes the verdict
   `Impermissible`; else any `Obligatory` makes it `Obligatory`; else it is `Optional` with the sum
   of the survivors' costs. The justification lists every survivor, whatever its modality, in
   `HashMap` order.

`explain_verdict` renders the verdict, one line per justifying norm, and a fixed sentence per
outcome. It explains the result, not which norms were defeated; a comment in the code says so.

## 4. Relation to DDIC and to the causal engine

The README cites the Defeasible Deontic Inheritance Calculus (Olson, Salas-Damian, Forbus;
arXiv:2407.04869) as the formalism behind conflict resolution. In DDIC, inheritance is
modality-specific (axioms R1–R4: obligations and prohibitions inherit in different directions
along the behaviour hierarchy), each norm carries a context and a time, and Lex Posterior and Lex
Specialis are derived from those axioms rather than stated. The crate keeps the vocabulary and the
three heuristics, and replaces the calculus with two untyped edge kinds and a numeric comparison.
There is no behaviour hierarchy: `action_identifier` is a free string that nothing reads.

The ethos reads the same `Context` a causaloid reads, which is the design's point: a norm and a
cause can depend on the same data, space and time. In the code, the two meet only in user code.
`deep_causality` cannot depend on the ethos (the ethos depends on it), the CSM's ethos hook is a
no-op, and the one example gates nothing. A caller who wants a governed action must evaluate the
ethos and decide what to do with each verdict and each error, including `InconclusiveVerdict` when
no norm applies.

## 5. Where the code and the claims part

1. **Order-dependent resolution** (the finding). Defeat is decided against the belief map as it
   stands when a norm is dequeued, the queue is seeded in `HashSet` order, and a defeated defeater
   may already have removed its target. The test suite has no chain of two defeats, so nothing
   catches it.
2. **Defeat is a disjunction.** `specificity > || timestamp > || priority >` makes any one
   advantage decisive and ignores the other two. The doc comment on `resolve_conflicts` names Lex
   Specialis and Lex Posterior; the README says the three rules "check the survivors for
   consistency". Neither describes the code. Defeat also applies only along explicit `Defeats`
   edges; two active norms with opposite modalities and no edge both survive, and the modality
   precedence in step 5 settles them.
3. **Inheritance bypasses activation.** An `Inherits` child joins the belief set whether or not its
   predicate holds, and the chain continues through it. `test_evaluate_action_deep_inheritance`
   pins this with always-true predicates, so it does not distinguish the two readings.
4. **Action identity is ignored.** Selection is by tag alone. A caller who tags two actions'
   norms alike gets each action judged by the other's norms.
5. **Failures are silent and fail open.** An erroring or unsampleable uncertain predicate makes its
   norm inactive, so a prohibition that cannot be evaluated does not prohibit. No test evaluates an
   uncertain norm at all; the uncertain branch of `evaluate_action` is unexercised.
6. **Panic on a frozen graph.** `add_deterministic_norm` and `add_uncertain_norm` call
   `add_node(..).expect("Failed to add node")`. After `verify_graph`, adding a norm panics with
   `GraphIsFrozen` *(probe)* instead of returning the `DeonticError` the signature promises.
7. **Misleading or dead errors.** An unverified graph reports `GraphIsCyclic`.
   `derive_verdict` returns `NoRelevantNormsFound` for an empty set while its doc says
   `InconclusiveVerdict`, and its mixed-modality branch cannot be reached with three modalities.
   `MissingContext` is never constructed and its message names "the CausalState". The local
   `teloid_cache` in `evaluate_action` is written and never read.
8. **Documentation drift.** The README's usage block pins `deep_causality_ethos = "0.3"`; the
   crate is 0.4.1. `BaseTeloidStore`'s doc calls teloids "temporal causal units". The crates.io
   badge and `Cargo.toml` documentation link point at `deep_causality`. `explain_verdict` omits the
   closing parenthesis and the newline after each norm, so several norms run together on one line.
9. **Public test helpers.** `lib.rs` declares `pub mod utils_test`, so `TestEthos` and the dummy
   predicates are part of the published API.

## 6. File map

| Concern | Path |
|---|---|
| Public surface | `deep_causality_ethos/src/lib.rs`, `src/alias/mod.rs` |
| Norm | `src/types/teloid/` (`mod.rs`, `getters.rs`, `part_eq.rs`) |
| Modality, relation, verdict | `src/types/teloid_modal/`, `teloid_relation/`, `teloid_verdict/` |
| Store, tag index, graph | `src/types/teloid_store/`, `tag_index/`, `teloid_graph/` |
| Builder API and linking | `src/types/effect_ethos/api.rs` |
| Evaluation pipeline | `src/types/effect_ethos/deontic_inference.rs` |
| Defeat and inheritance walk | `src/types/effect_ethos/resolve_conflicts.rs` |
| Modality precedence | `src/types/effect_ethos/derive_verdict.rs` |
| Freeze, verify, explain | `src/types/effect_ethos/freeze.rs`, `verify.rs`, `deontic_explainable.rs` |
| Traits | `src/traits/` (`DeonticInferable`, `DeonticExplainable`, `TeloidStorable`, `Teloidable`) |
| Errors | `src/errors/deontic_error.rs` |
| Test helpers (public) | `src/utils_test/test_utils_effect_ethos.rs` |
| Engine hook (no-op) | `deep_causality/src/types/csm_types/csm/eval.rs` (`fire_action_with_ethos_check`) |
| Example | `examples/csm_examples/csm_effect_ethos/` |
| Paper | `deep_causality_ethos/papers/ddic.pdf` |
