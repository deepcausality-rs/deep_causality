# Foundation: the algebra tower

The tower under everything else. A bound in a signature decides what a function accepts, and
these three examples show what each bound buys and what it costs.

```text
Real         analytic: sqrt, exp, ln, sin, ordering, rounding, constants
Field        invertible: every non-zero element has an inverse
RealField    Real + Field + ToPrimitive          f32, f64, BFloat16, Float106
ComplexField a field with conjugation and re/im  Complex<T>
Normed       a real modulus, with `Real` as an *associated* type
```

Run from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

| Example | What it shows | Command |
|---|---|---|
| [precision_as_a_parameter.rs](precision_as_a_parameter.rs) | One `RealField`-bounded series run at `BFloat16`, `f32`, `f64` and `Float106`, and the same body one bound weaker so a `Dual` carries a derivative through it | `cargo run -p mathematics_examples --example precision_as_a_parameter_examples` |
| [real_and_complex_uniformly.rs](real_and_complex_uniformly.rs) | `Normed` and `ConjugateScalar` giving one norm kernel for reals, complex numbers and duals, and why `modulus` is a trait member with its own scaled implementation | `cargo run -p mathematics_examples --example real_and_complex_uniformly_examples` |
| [verdict_logic.rs](verdict_logic.rs) | `all` / `any` / `none` written once, running over `bool` as a Boolean algebra and over `Prob` as an MV-algebra | `cargo run -p mathematics_examples --example verdict_logic_examples` |

## What each one is really about

**Precision as a parameter** is the headline, but the interesting half is the `Real` / `Field`
split. A dual number `a + b·ε` has every elementary function by the chain rule, so it is
analytic. `Field` promises an inverse for every non-zero element, and `ε` is a zero divisor, so
`Dual` lands under `Real`. Bound a function on `Real` and a derivative flows through it, while the
four concrete scalars keep working exactly as they did. That choice is made once, in a signature,
and it decides whether a model is differentiable.

**Real and complex uniformly** starts from the two families sitting in separate parts of the
tower: reals are ordered and live under `Real`, complex numbers live under `ComplexField`.
`Normed` spans them by making the real type *associated*, so `f64::Real` and `Complex<f64>::Real`
are both `f64` and one signature covers both. The payoff is concrete: `Normed::modulus` uses the
scaled form, so `|1e308 + 0i|` returns `1e308` and `|1e-200 + 0i|` returns `1e-200`, at both ends
of the range.

**Verdict logic** names the structure that pass/fail checks and graded confidences share — a
bounded lattice with an involution — so one implementation of `all`, `any` and `none` serves both.
The example also shows where the two algebras separate: `bool` has excluded middle, `Prob` keeps
the lattice and the involution, and `Verdict` promises exactly the part both hold.
