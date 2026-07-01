# Hosting Quantum Causal Models in the EPP

**Causaloid hypergraph + causal monad**

Status: well-posed. Structural soundness closed by existing monad law; one genuinely-quantum proposition open. Not a subsumption claim — a reconstruction claim.

---

## Result

The EPP does not subsume the quantum causal model (QCM) framework (Allen et al. 2017; Barrett, Lorenz & Oreshkov 2019, 2021) by axiom. It **hosts** it by construction: the causaloid hypergraph, with the PropagatingProcess monad carrying evaluation state, is a substrate on which a QCM is reconstructible, with the quantum Markov condition recovered as a commutativity check over operator-valued monad-state.

This is narrower than "Inherently Subsumptive" (§3.6.2) and stronger as a result: it states exactly what was added and where the boundary sits.

---

## Why the hypergraph, not the monad, is the carrier of structure

The monad's bind is irreversible by construction: `m >>= f >>= g` admits no inverse (§3.6.2, consequence 5). Quantum causal structure is symmetric under inversion: a unitary `U` and `U†` carry the same structure with all arrows reversed (Lorenz §6). The bind ordering therefore cannot be where quantum causal structure lives.

The hypergraph is the right structural carrier:

- A hyperedge `{Pa(A_i)} -> A_i` names the parent set directly — a faithful carrier for the QCM factor `rho_{A_i | Pa(A_i)}`, more so than a binary DAG where parent sets are implicit.
- Native non-DAG structure aligns with the QCM frontier: cyclic quantum causal models (Barrett, Lorenz & Oreshkov 2021) and indefinite causal order / the quantum switch (Ormrod, Vanrietvelde & Barrett 2022). The interesting quantum phenomena are non-DAG.
- Lineage: the causaloid roots in Hardy's quantum-gravity work. The EPP dropped Hardy's process matrix for an unconstrained function (implementability + nesting); the Barrett–Lorenz–Oreshkov line kept it (and proved the central theorem). Same ancestor, opposite engineering bet.

---

## What must be reconstructed

The substance of QCM is the theorem (Allen et al. 2017; generalised Barrett et al. 2019):

> Factorisation `rho_{BC|A} = rho_{B|A} . rho_{C|A}` is equivalent to `A` being a complete common cause, where the causal relation is defined via influence through unitary evolution.

The quantum Markov condition (Lorenz Def. 3.3): a process `sigma` is Markov for a graph iff it factorises into CJ operators of CPTP maps **that commute pairwise**. The commutativity clause is the load-bearing content (Lorenz footnote 11): free at two factors (hermiticity), independent and substantial at three or more, and it bites precisely where parental sets overlap (two factors acting on the same node's Hilbert space). Nothing in the classical EPP currently expresses this; aggregate logic (§4.6) combines *outcomes* via a Boolean monoid, whereas commutativity constrains the *operators* before any outcome exists.

---

## The carrier: PropagatingProcess monad-state

CJ operators are carried in the **PropagatingProcess** (arity-5 causal monad), whose state field threads forward during evaluation via bind. Confirmed: hypergraph edge-threading *is* bind — the PropagatingProcess calls bind during evaluation.

Three consequences:

1. **The operator is threaded state, not stored data.** `rho_{A_i|Pa}` travels with the propagation as part of the value-in-context that bind threads node to node. This matches Lorenz: the process operator `sigma` is the object that evolves through the structure, not an environment that is read.

2. **No shared-mutable-state hazard.** Order-sensitivity requires *shared mutable* state (one branch's write changes another's read). Monad-state is threaded, not shared — each propagation carries its own state forward. The hazard does not arise.

3. **Encapsulation-equals-flat is inherited, not owed.** "Nested evaluation equals flat" is **associativity** of composition, not commutativity. Encapsulating a subgraph re-parenthesizes the composition; `(f . g) . h = f . (g . h)` is precisely monad law 3, already asserted and property-tested. Since edge-threading is bind, the law transfers to the hypergraph skeleton directly. Encapsulation soundness for the structural skeleton is a consequence of a law already proven.

---

## Layer stack (proven / open / forbidden)

Keep these separate; do not overload one `Commutative` notion across them.

| Layer | Content | Status |
|---|---|---|
| **A. Collection order-invariance** | Aggregation is a commutative-associative monoid over pure functions | **Enforceable now** via type bound |
| **B. Encapsulation = flat (skeleton)** | Associativity of bind-threading = monad law 3 | **Proven (inherited)** |
| **C. Shared-environmental-context boundary** | Encapsulation cuts must not reorder a context write vs dependent read | **Decidable at freeze**; does not arise for the monad-state quantum case |
| **D. Operator commutativity** | `[rho_j, rho_k] = 0` on shared Hilbert space | **Open** — the only genuinely-quantum check |

**Layer A.** Implement the marker relative to the (container, aggregation) pair, not the container alone:
`impl Commutative for Collection<C, Agg> where Agg: CommutativeMonoid`. Require `Commutative` as a type bound; non-commuting configurations become unrepresentable. HashMap-backed collections are then legal exactly under a commutative-associative aggregate (where iteration order is harmless) and rejected otherwise.

**Layer D is independent of B.** Monad-law associativity makes the encapsulated operator *well-defined*; it does not make it *commute* with neighbours. The monad proves you can compose sound subgraphs soundly; it does not prove that collapsing a subgraph via partial trace yields an effective operator that still commutes with the rest. That remains the quantum fact.

---

## Freeze-time enforcement

The dual-state hypergraph (dynamic for mutation; frozen for analytics) gives the enforcement point, modelled on the existing `freeze_dag` acyclicity gate.

- **Layer A:** type bounds reject non-commutative collections before freeze.
- **Layer C:** boundary-cut check over context-dependency edges (decidable, `freeze_dag`-shaped).
- **Layer D:** recursive walk to the leaves computing pairwise commutators on shared supports; hard pass / abort.

Freeze **enforces** Layer D but does not **establish** it. `freeze_dag` aborts on cyclicity because a cyclic model is malformed; a non-commuting *encapsulated* factor may instead mean the nesting is lossy while the flat model is valid. Freeze-abort is therefore sound but possibly incomplete (never accepts an invalid model; may reject valid ones where nesting and commutation conflict). The cost of that incompleteness is the Layer D proof obligation below.

Use freeze as a **discovery instrument**: record when the encapsulation branch fails across the test battery. Never fails -> strong signal for unconditional preservation. Fails on structural patterns -> those patterns are the conditions of a conditional theorem; then refuse to encapsulate there (keep the subgraph flat) rather than aborting the whole model.

---

## Context

QCM does not need mutable environmental context, and excluding it is sound — a QCM is a closed static specification. With operators carried as monad-state, the quantum data is not in context at all. The one exception is genuinely environmental fixed data, e.g. the Bell-state preparation `rho_A` in the common past (Lorenz §3), read by several causaloids and never written; that belongs in **immutable** context (written once at construction, frozen, read-only). The type boundary must make write methods *unreachable* on the immutable form, not merely unused. This places the quantum model in the framework's verifiable region, consistent with §4.9 (static/immutable context preserves determinism; dynamic context breaks it).

---

## What is left to do

1. **Property-test law 3 with operator-valued state.** The monad laws were verified over the value parameter; confirm the property tests exercise the arity-5 state field with *matrix* payloads, not only scalar state. "Inherited for free" rests on this. Cheap.

2. **Resolve the partial-trace preservation proposition (Layer D).** State and settle: *if a subgraph's factors pairwise-commute and the surrounding factors pairwise-commute, does the effective factor after marginalisation (partial trace) commute with its new neighbours?* Partial trace does not preserve commutation with operators outside the traced subsystem in general. Either prove it under identifiable conditions (candidate: subgraph interface is a single node, or shared supports lie entirely on the encapsulation boundary) or produce the counterexample. This is the single remaining hard, genuinely-quantum proposition. Discover its conditions empirically first via the instrumented freeze.

3. **Implement the Layer D operator-commutativity check at freeze.** Operator-valued PropagatingProcess state (complex matrices); recursive pairwise-commutator walk over hyperedges sharing a source node; hard pass / abort; instrumented for failure-pattern capture (feeds item 2).

4. **Depth-aware numerical tolerance.** The commutator test couples precision to the causal verdict (commuting vs not = valid vs invalid model), so Float106 is necessary, not gold-plating. The test is recursive; error accumulates through repeated partial traces down the encapsulation depth. The near-zero tolerance must scale with accumulated conditioning through nesting depth, or deeply nested valid models fail for numerical reasons (spurious aborts). This is error-propagation analysis through iterated partial trace.

5. **Add the immutable-context constructor (if absent).** A causaloid form bound to a read-only context handle, with write methods unreachable, for residual environmental quantum data (`rho_A`).

---

## Why it matters

1. **Substantiates the EPP's generalization along the one axis the axiom does not automatically cover** — the theory of the relata. The EPP generalizes along *substrate* (drop spacetime); QCM generalizes along the *relata* (classical variables -> quantum systems under unitary evolution). Orthogonal axes. Closing the relata axis by construction (operator-valued monad-state + commutativity check), not by blanket axiom, is falsifiable and specific. Rewrite §3.6.2 around this.

2. **Provably-sound hierarchical composition of quantum causal structure.** The causaloid supplies the unconstrained function (compute a CJ operator, sample a QPU, run an SCM); the monad supplies law-3-associative threading, so quantum subgraphs encapsulate arbitrarily deep with flat ≡ nested guaranteed by monad law rather than per-model proof. Neither Hardy's process matrix (no nesting story) nor the DAG-QCM (no general recursion) has this.

3. **Unified quantum/classical coupling through one type system.** A causaloid can sample a physical QPU, emit the result as a PropagatingEffect, and feed a classical dynamic causal process downstream — within the same causal type system and the same hyperedges. Causal structure that crosses the quantum/classical divide becomes a single graph, which neither QCM (purely quantum relata) nor Pearl (purely classical relata) can state. **Caveat:** a *physical* QPU call couples to an open world and sits in the emergent (unverifiable) modality (§4.9.4); the *simulated* quantum process (deterministic CJ operators) stays verifiable. State this boundary, since verifiability is claimed as the EPP's distinguishing virtue.

---

## Summary

Carry the CJ operators as PropagatingProcess monad-state. Because edge-threading is bind, encapsulation-equals-flat for the structural skeleton is inherited from monad law 3 — proven, not owed — and the shared-mutable-state hazard does not arise. The collection layer is closed by a type bound. What remains is exactly one proposition: that partial-trace marginalisation preserves operator commutativity under encapsulation, tested at freeze with depth-aware Float106 tolerance and discoverable empirically via instrumented freeze. Everything structural is closed by existing law; the one open item is the only fact that is actually about quantum mechanics.
