## ADDED Requirements

### Requirement: The causaloid is the well-founded fixpoint of its signature functor

`Core/Causaloid.lean` SHALL define the signature functor
`F(X) = Atom(I →ᴹ O) + Coll(Bag X, AggLogic) + Graph(Hyper X, Λ-edges)` and prove
`Causaloid ≅ μX.F(X)` — the initial algebra (μX, not the coalgebra νX), well-founded via a nesting
guard or structural fuel so that every causaloid tree is finite (`core.causaloid.fixpoint`, closing
assumption #9). The three summands SHALL correspond one-to-one to the three sealed `CausaloidType`
forms (Singleton/Collection/Graph; assumption #11a), and the model SHALL be self-contained
(bare-`lean`, zero `sorry`), citing standard initial-algebra semantics with deviation notes.

#### Scenario: The fixpoint theorem is closed and well-founded

- **WHEN** `lean lean/DeepCausalityFormal/Core/Causaloid.lean` is run
- **THEN** it typechecks standalone with `core.causaloid.fixpoint` closed, and the recursion is
  accepted by structural well-foundedness (no infinite causaloid tree inhabits the model)

#### Scenario: The three summands mirror the sealed forms

- **WHEN** the Lean `F` and the Rust `CausaloidType` are compared
- **THEN** `Atom`/`Coll`/`Graph` correspond exactly to `Singleton`/`Collection`/`Graph`, with the
  correspondence stated in the proof header and checked by the Rust witness

### Requirement: Evaluate factors through the Hardy inversion

The formalization SHALL prove that the element carries no ordering asymmetry: `evaluate` factors as
(symmetric local data) ∘ (asymmetric wiring) — the causal function sees values with intrinsic
identity and never a spacetime position or a before/after, while sequencing (`bind`) and the
order-free merge (`∇`) live entirely in the composition layer (`core.causaloid.inversion`). The
proof header SHALL cite Hardy (arXiv:gr-qc/0509120, Eq. 2 p. 4) and state the factorization as the
formal content of the inversion thesis in
`openspec/notes/causal-algebra/Causaloid-structure.md`. The Hardy correspondence target
(`{bind, ∇ ∘ (Λ ⊗ Λ)}` reconstructs the ⊗^Λ regimes) SHALL remain out of scope, recorded as an open
target.

#### Scenario: The factorization theorem is closed

- **WHEN** `Core/Causaloid.lean` is checked
- **THEN** `core.causaloid.inversion` is closed: evaluation equals the composite of a
  wiring-independent element map and an element-independent wiring map, and no element operation
  takes an order or position argument

### Requirement: Hyperedges carry identity-keyed per-edge Λ decoration slots

The main crate's hypergraph SHALL support an optional per-edge decoration Λ — an arrow transform
attached to an incoming connection, keyed by intrinsic edge identity and never by order — so that a
reconvergent join is expressible as `join = ∇ ∘ (Λ₁ ⊗ Λ₂)`: connection asymmetry lives on the edge,
the fuse `∇` stays commutative. The undecorated edge SHALL behave as the identity Λ, and existing
graphs (no decorations) SHALL evaluate exactly as before.

#### Scenario: The undecorated graph is unchanged

- **WHEN** a graph with no Λ decorations is evaluated
- **THEN** the result is identical to the pre-change behavior (identity Λ on every edge)

#### Scenario: Decorations are identity-keyed and order-free

- **WHEN** two incoming edges of a join carry decorations Λ₁ and Λ₂
- **THEN** each Λᵢ is looked up by its edge's intrinsic identity, and swapping the enumeration
  order of the edges does not change which Λ applies to which input

### Requirement: The fixpoint layer is tested and proved in Lean

`core.causaloid.fixpoint` and `core.causaloid.inversion` SHALL each have a `THEOREM_MAP.md` row and
a Rust witness under `deep_causality/tests/formalization_lean/` (Bazel-registered), and the Λ-slot
behavior SHALL be covered by main-crate tests including a reconvergent-join case with distinct
decorations.

#### Scenario: Both bridge sides exist for the fixpoint layer

- **WHEN** `THEOREM_MAP.md` and the traceability gate are checked
- **THEN** each `core.causaloid.{fixpoint,inversion}` id has a proved Lean location and a passing
  Rust witness, and `Core/Causaloid.lean` typechecks standalone with bare `lean`
