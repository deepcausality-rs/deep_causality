## ADDED Requirements

### Requirement: do-operator formalized as graph surgery

The formalization SHALL model Pearl's `do(X = x)` as a total surgery on the causal hypergraph, built from the substrate primitive defined in `comonoid-graph-join` D10: delete every in-wire key of `X` and pin `X`'s mechanism to the constant `x` (the JKZ "intervention = endofunctor cutting wires" / Lorenz–Tull "opening a mechanism"). Because parent contributions are keyed, the finer single-edge cut — delete wire key `(P1, X)` while keeping `(P2, X)` — SHALL be expressible as its own operation. The surgery is one operation on the substrate, respected by both interpreters (truncated factorization under the classical fold; the Lorenz–Tull opening under the deferred QCM fold). `Core/Intervention.lean` SHALL define the surgery as a total function on the reified graph and SHALL record that, in the acyclic regime, the surgical result's acyclicity is checkable via the `ultragraph::has_cycle` freeze gate. Each theorem MUST carry a `THEOREM_MAP.md` row and a Rust witness, and the file MUST typecheck standalone with bare `lean`.

#### Scenario: Intervention cuts incoming edges and pins the output

- **WHEN** `do(X = x)` is applied to a formalized causal graph
- **THEN** the model yields a graph with `Pa(X) = ∅` and `X` fixed to `x`, and the surgery is total (defined for every node and value)

#### Scenario: Single-edge cut is expressible

- **WHEN** an intervention severs only the influence of parent `P1` on `X`, preserving `P2`
- **THEN** the model expresses it as deletion of the wire key `(P1, X)`, and the next evaluation of `X` joins over the remaining fired parents

#### Scenario: Acyclic surgery stays acyclic

- **WHEN** an acyclic graph is intervened upon in the acyclic regime
- **THEN** the result remains acyclic, corresponding to `ultragraph::has_cycle` accepting it at freeze

### Requirement: do-operator formalized as a handler over the Free program

The formalization SHALL also model interventions as an alternate handler/algebra over the `RelayTo` `Free` program (distinct interpreter over the same program-as-data), so that dynamic (adaptive) interventions are expressible without re-deriving the engine.

#### Scenario: Intervention is an alternate fold algebra

- **WHEN** the intervention handler interprets a `CausalEffect` program
- **THEN** it is a `Free::fold` with an intervention algebra, over the same program the default engine folds

### Requirement: Intervention commutes with encapsulation

The formalization SHALL prove that intervention commutes with encapsulation (nesting a subgraph then intervening equals intervening then nesting), i.e. `do` is functorial with respect to the recursive causaloid structure. This inherits from the associativity of bind-threading (`core.causal_monad.assoc`).

#### Scenario: Nest-then-intervene equals intervene-then-nest

- **WHEN** a subgraph is encapsulated and an intervention is applied on the boundary node
- **THEN** the model proves the two orders yield equal results, with a bound Rust witness
