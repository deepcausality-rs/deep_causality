# A Quantum Counterfactual: What the Decoder Was Reading

Quantum error correction recovers a state without ever looking at it. This example performs the
three-qubit repetition code, then asks the counterfactual question: the decoder acted on a syndrome,
so what would it have done had the syndrome said something else?

```bash
cargo run -p quantum_examples --example quantum_counterfactual
```

## The problem

A qubit on its own cannot be error-checked. Every measurement that would reveal whether it flipped
also reveals `α` and `β`, and collapses the superposition being protected. Checking it destroys it.

The way out is to spread one logical qubit across three physical ones:

```text
α|0⟩ + β|1⟩   →   α|000⟩ + β|111⟩
```

and then ask only about the **relationship** between them. `⟨Z₀Z₁⟩` answers whether two qubits
disagree, and its value is exactly `+1` or `−1` whatever `α` and `β` are. It reports an error without
reporting the state.

Two such checks distinguish all four cases:

| `⟨Z₀Z₁⟩` | `⟨Z₁Z₂⟩` | Diagnosis |
|---|---|---|
| agree | agree | no error |
| differ | agree | qubit 0 flipped |
| differ | differ | qubit 1 flipped |
| agree | differ | qubit 2 flipped |

Applying `X` to the named qubit undoes the flip exactly.

## The counterfactual

The recovery is *caused* by the syndrome. Nothing else reaches the decoder, so if the syndrome had
said something else the decoder would have acted on that instead.

`CausalFlow::alternate_value_if` is Pearl's do-operator. It substitutes the value the measurement
produced with the value an intervention forces, and records the substitution. The two pipelines are
the same steps on the same corrupted register, and differ by one line:

```text
observed        measure →                      recover
counterfactual  measure → do(syndrome := q0) → recover
```

Both recoveries run. One restores the state, one destroys it, and the difference is attributable to
the substituted value because everything else about the two runs is identical.

## What the code demonstrates

| Operation | Carries |
|---|---|
| `bind` | each stage, threading the register through the state channel |
| `fold` | amplitudes → a parity, and amplitudes → a fidelity |
| `alternate_value_if` | the intervention, as the do-operator |

The register rides the **state** channel and the syndrome rides the **value** channel. That split is
the code's own structure: the decoder sees the value channel and never the state, which is exactly
the property that lets error correction work without measuring the protected qubit.

## Output

```text
Encoding
  logical      0.600|0> + 0.800|1>
  encoded      0.600|000> + 0.800|111>
  norm^2       1.000000000

The error: an X gate on qubit 1
  before       0.600|000> + 0.800|111>
  after        0.600|010> + 0.800|101>
  norm^2       1.000000000

The syndrome: two parity measurements
  check        clean      corrupted
  <Z0 Z1>     +1.000       -1.000
  <Z1 Z2>     +1.000       -1.000

Recovery from observed
  syndrome     (differ, differ)  ->  qubit 1
  state        0.600|000> + 0.800|111>
  fidelity     1.000000000

Recovery from do(syndrome := qubit 0)
  syndrome     (differ, agree )  ->  qubit 0
  state        0.600|110> + 0.800|001>
  fidelity     0.000000000

Outcome
  fidelity to the protected state
    observed syndrome        1.000000000
    intervened syndrome      0.000000000
    no correction at all     0.000000000
```

Three things in that output are worth reading carefully.

**The parities are `±1`, never `0.6` or `0.8`.** That is not a coincidence of the numbers chosen; it
is the property the code is built on. Every basis state in the support of a code word agrees on the
parity of any two qubits, so the weights sum to one and the amplitudes cancel out of the answer.

**The error is a permutation.** The same eight numbers come out in a different order, because `X` on
qubit `k` moves the amplitude at index `i` to index `i XOR (1 << k)`. Nothing is substituted and no
amplitude is invented.

**The miscorrected state has norm 1.** Fidelity is the only honest verdict on a recovery. A check
that some amplitude came out large would pass `0.600|110⟩ + 0.800|001⟩`, which is a perfectly good
normalised state, orthogonal to the one being protected and of no use whatsoever.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!`, and `3/5` and `4/5` are
derived from integers at the working precision. It sits at `Float106` rather than `f64` on purpose:
a hard-coded `f64` is invisible while the alias *is* `f64`, and a compile error the moment the two
differ. All four scalars run and all four agree exactly, because the gates are permutations and the
two fidelities are `1` and `0` rather than numbers a mantissa can shave.

## What this example covers

The goal is to reformulate the essence of a correction as a causal process over the library's types,
and to get precision as a parameter and categorical composition for free once it is in that form.
The essence is that the decoder acts on the syndrome and on nothing else, which is what makes both
the correction and the counterfactual possible. The model keeps that and holds everything else
simple: one bit-flip error on one known qubit, no phase errors, measurement treated as reading an
expectation rather than sampling and collapsing, and a decoder that is a four-row lookup.

A fault-tolerant treatment adds what this leaves out: the repetition code protects against bit flips
only, so a real code such as Shor's or a surface code is needed to catch phase errors too; syndrome
extraction needs its own ancilla qubits and is itself noisy; and errors arrive continuously rather
than once, so the syndrome is measured repeatedly and the decoder works over a history of them.

## How to grow the example toward a fault-tolerant simulation

Each step keeps the structure already here.

- **Phase errors.** Add `Z` alongside `X` and move to the nine-qubit Shor code. `N_QUBITS` is a
  constant and `bit_flip` already generalises to any single-qubit Pauli.
- **Ancilla-based syndrome extraction.** Measure the parity with a helper qubit and a pair of
  `CNOT`s instead of reading the expectation directly, which is how a device does it.
- **Repeated rounds.** Wrap the measure-and-recover pair in `CausalFlow::iterate_n`, so the decoder
  works over a sequence of syndromes and can spot a syndrome measurement that was itself wrong.
- **Noisy syndromes.** The intervention already shows what a misread syndrome does. Replace the
  forced value with a sampled one from `deep_causality_uncertain` and the same pipeline becomes a
  study of decoder failure rates.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias, the two pipelines, and the one line that separates them |
| `model.rs` | the encoding, the `X` gate, the parity measurements, the decoder, the fidelity |
| `utils_print.rs` | the presentation, and the only `lower` calls |

## Key APIs used

- `CausalFlow::bind` — monadic composition of the stages
- `CausalFlow::alternate_value_if` — the do-operator that forces the syndrome
- `CausalEffectPropagationProcess::with_state` — threading the register through the state channel
- `HilbertState` — the three-qubit register, as eight complex amplitudes
