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
2. **`validate` screens the other three.** Each candidate's factors are conditional tables, so the
   normalisation check admits each as a process; the Markov check admits each (the factors are
   diagonal, which puts the whole weight of the discrimination on the interventions); and the
   decomposability check runs on the structure each candidate's own supports encode. All three are
   admitted; a screen results.
3. **`control` takes the screen.** Only the screen enters `control`; a plant config with structural
   candidates cannot.
4. **`design_with` computes the predictions and returns a plan.** A response model says what each
   experiment does in each candidate's world, and every predicted read-out follows from the
   factors: holding Q1 excited reads Q2 at 0.40 under H₁ and 0.10 under H₂ and H₃; holding Q2 is
   the mirror image; the passive read reads 0.04 under all three; the echo reads 0.01, 0.01 and
   0.04. The two holds, `do(Q1=|1⟩)` and `do(Q2=|1⟩)`, cover all three hypothesis pairs at the
   5-bit floor for a total cost of 2, of the 5 that all four experiments cost.
5. **A campaign runs the plan** on observations drawn from H₁, through `CausalFlow::iterate_until`:
   it runs the cheapest planned experiment, judges each world's prediction against the 1024-shot
   observation, and stops as soon as one candidate holds and separates from the others. The
   verdicts are read-outs against a real-valued spec, so no commutation test runs: a threshold on a
   real quantity is a classical proposition. H₁ survives after `do(Q1=|1⟩)` alone, about a hundred
   bits from its nearest rival, having spent 1 of the plan's cost of 2.

Predictions and the plan are computed; the observations are sampled. The conditional tables and
the response of each candidate to each experiment are modelling assumptions, stated as such in
`model.rs`.
