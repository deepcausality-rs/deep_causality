# A Quantum Causal Model, and the Check That Decides Whether It Is One

Not every graph carrying quantum operators is a quantum causal model. This example shows the
condition that separates the two, and where the library enforces it.

```bash
cargo run -p quantum_examples --example qcm_freeze_check
```

## The problem

A quantum causal model describes a process as a graph whose nodes carry Choi–Jamiołkowski factors.
The factors that share a Hilbert leg have to **pairwise commute**.

That is not a modelling preference or a convenience. The model's Markov condition is a statement
about those factors being simultaneously assignable, and non-commuting operators have no joint
assignment to make. A graph that fails the condition is not a quantum causal model that happens to
be awkward — it is not a quantum causal model.

Which makes the condition worth checking, and worth checking somewhere specific.

## Freezing is where the check belongs

A causal graph in this library is **dynamic** while it is being built and **frozen** once it is ready
to run. Freezing is the one moment at which the structure is complete and nothing has yet depended
on it. That makes it the only place a structural condition can be enforced without either rejecting
a half-built graph or discovering the problem after a result has already been used.

`freeze_quantum` runs the pairwise commutator checks at that boundary:

| Outcome | What happens |
|---|---|
| every shared-leg pair commutes | the graph freezes; the report names the pairs tested and the worst margin |
| some pair does not | the freeze **aborts**, the error names the pair, and the graph **rolls back to dynamic** |

The rollback is the part worth noticing. A model that cannot be frozen is never left half-frozen, so
a failure costs the caller a rebuild and never a result computed on a structure that does not hold.

## What the run does

Two models, differing in exactly one operator:

```text
sigma_z and diag(3, -1) on leg 0   commute        freezes, reports the margin
sigma_x and sigma_z     on leg 0   anticommute    aborts, names the pair, rolls back
```

Pauli `X` and `Z` anticommute, so `[σx, σz] = −2i σy` and its norm is as far from zero as a
single-qubit commutator gets. That is deliberate: a pair that *nearly* commuted would test the
tolerance rather than the condition.

The commuting pair is `σz` with `diag(3, −1)`. Both are diagonal, and any two diagonal operators
commute exactly, so the margin is zero rather than merely small. `diag(3, −1)` is used instead of a
multiple of the identity, which would commute with everything and demonstrate nothing about
diagonality.

## Output

```text
[1] sigma_z and diag(3, -1) on leg 0
    expected: they commute
    froze cleanly
      pairs tested       1
      worst margin       0.000e0
      is_frozen()        true

[2] sigma_x and sigma_z on leg 0
    expected: they anticommute
    freeze aborted
      offending pair     nodes 0 and 1
      reason             ‖[ρ_0, ρ_1]‖_F exceeds the Q-TOL threshold (margin > 1)
      is_frozen()        false
```

The error names the pair. A check that reported only that something failed would leave a real model
of any size with nowhere to start looking.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!` and
`const_scalar_from_float!`. It sits at `Float106` rather than `f64` on purpose: a hard-coded `f64` is
invisible while the alias *is* `f64`, and a compile error the moment the two differ.

All four scalars run and all four agree. The margin here is either exactly zero or as large as a
single-qubit commutator can be, and neither is a quantity a shorter mantissa can shave.

The nodes' own predicate is written out in `model.rs` rather than taken from the library's test
helpers. Those are fixed at `f64` and would pin the graph to one precision while the factors
followed the alias — which is how a precision parameter quietly stops being one.

## What this example covers

The goal is to reformulate the essence of the validity condition as a step in the library's own
lifecycle, and to get precision as a parameter for free once it is in that form. The essence is that
the condition is structural, that the freeze boundary is where structure is settled, and that a
failed check leaves nothing behind. The model keeps that and holds everything else simple: two nodes,
one shared leg, single-qubit factors given directly rather than derived from a channel, and one
pairwise check.

A model of a real process adds what this leaves out: factors obtained from actual channels through
the Choi–Jamiołkowski isomorphism, several legs per node with overlapping supports, a tolerance
chosen from the numerical conditioning of those factors rather than left at its default, and the C₃
structural check that runs alongside this one.

## How to grow the example toward a real model

Each step keeps the structure already here.

- **Factors from channels.** Build the Choi matrix of an actual quantum channel instead of writing
  the operator down; the check and the freeze are unchanged.
- **More legs and more nodes.** `factors_on_shared_leg` declares one support per node. Declaring
  several turns one pairwise check into the full set the condition asks for.
- **A chosen tolerance.** `CommutatorTolerance::default()` is a starting point. Deriving it from the
  factors' norms is what a model with numerically-obtained factors needs.
- **The `C₃` check alongside.** `freeze_quantum` already takes the structural argument this example
  passes empty, so adding a `C₃` relation exercises the other half of the freeze.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias and the two freeze scenarios |
| `model.rs` | the operators, the two-node graph, and the factor store |
| `constants.rs` | the numbers the operators are written with |
| `utils_print.rs` | the presentation, and the only `lower` calls |

## Key APIs used

- `freeze_quantum` — the freeze-boundary Markov commutativity check
- `ProcessFactors` / `FactorSupports` — the node-keyed factor store and its Hilbert-leg registry
- `CommutatorTolerance` — how far from commuting a pair may sit
- `QuantumErrorEnum::CommutatorNonZero` — the abort, naming the offending pair
