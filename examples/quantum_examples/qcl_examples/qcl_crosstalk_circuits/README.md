# QCL-2: crosstalk attribution over circuit-derived candidates

This example reproduces the crosstalk decision of `qcl_crosstalk` with `H₁`, `H₂` and the cyclic
`H₄` written as circuits, so the wiring carries the structure; `qcl_crosstalk` declares its four
structures as factorizations. The decision is the same: the cycle refused at `build()`, three
candidates admitted, the plan `{do(Q1), do(Q2)}` at cost 2 against tomography at 200, and
`H1 Q1->Q2` the survivor.

```bash
cargo run -p quantum_examples --example qcl_crosstalk_circuits
```

| Candidate | Form | How the structure is carried |
|---|---|---|
| `H₁ Q1 → Q2` | one wire, `Q1`'s box then `Q2`'s | the wire leaves node 0 and enters node 1 |
| `H₂ Q2 → Q1` | the same boxes, `Q2`'s first, grouped so node 0 stays `Q1` | the wire leaves node 1 and enters node 0 |
| `H₃ Q1 ← B → Q2` | the `qcl_crosstalk` factorization | declared supports |
| `H₄ Q1 → Q2 → B → Q1` | four boxes on one wire grouped into a cycle | refused at `build()` as `CyclicStructureUnsupported` |

Each circuit goes through `.over_circuit`, is screened by Markov and C₃ on its dilation, and the
dilation's normalised factors become the candidate the plant pipeline forks. `H₃` is not a circuit
here: a common bath driving two qubits needs a bath node with two output wires, and under the
dilation's leg convention, a node dimension `d = d_in · d_out` and a leg of dimension `d²`, that
node has `d = 16` and a leg of 256, each single-wire child a leg of 16, a child's conditional factor
`4096 × 4096` with `2^24` entries at the cap, and the Markov union over the three legs
`65536 × 65536` with `2^32` entries. The `qcl_crosstalk` factorization is a legal QCM by
construction and stands in its place, as the spec records.
