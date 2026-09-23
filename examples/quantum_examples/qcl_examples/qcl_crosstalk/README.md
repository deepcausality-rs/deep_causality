# Crosstalk attribution: direct cause or common cause

This example decides whether two qubits' correlated errors have a direct cause or a common cause.
Three causal structures fit the passive observation exactly, because they are Markov equivalent, and
a fourth is a cycle. It is the keystone QCL example: the only one that runs `validate` and then
`control` on one problem.

```bash
cargo run -p quantum_examples --example qcl_crosstalk
```

What happens, in order:

1. **`build()` refuses the cyclic candidate** as `CyclicStructureUnsupported`, by decision, before
   any check runs. Under van der Lugt & Lorenz's Definition 3.1 the C₃ criterion does not reject a
   cycle, so the builder does, and the error names the scope limit instead of an obstruction.
2. **`validate` screens the other three.** The Markov check admits each (the factors are diagonal,
   which puts the whole weight of the discrimination on the interventions), and the
   decomposability check runs on the structure each candidate's own supports encode. All three are
   admitted; a screen results.
3. **`control` takes the screen.** Only the screen enters `control`; a plant config with structural
   candidates cannot. `fork` makes one world per admitted candidate, each with its own ledger.
4. **`design` returns a plan.** Two cheap interventions, `do(Q1=|1⟩)` and `do(Q2=|1⟩)`, at total
   cost 2, cover all three hypothesis pairs at the 5-bit floor. Process tomography alone would
   cover them at cost 200.
5. **The first planned experiment is observed** on the world where H₁ is true, drawing 1024 shots
   from the shipped Born sampler at H₁'s predicted read-out. Each world's prediction is judged
   against that observation, and `adjudicate` folds the three verdicts. They are read-outs against
   a real-valued spec, so no commutation test runs: a threshold on a real quantity is a classical
   proposition, and the guard for projection-valued verdicts does not apply. H₁ survives, about a
   hundred bits from its nearest rival.

Predictions and the plan are computed; the observation is sampled. The physics behind each
predicted read-out is a modelling assumption and is stated as one in `model.rs`.
