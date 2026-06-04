## ADDED Requirements

### Requirement: Dual number type for forward-mode automatic differentiation

`deep_causality_num` SHALL provide a `Dual<T>` type representing a dual number `a + b·ε` with `ε² = 0`, over `T: Real` (the analytic real-scalar trait from the `real-scalar` capability; this capability depends on `num-real-trait`), mirroring the module layout of `Complex<T>` (a folder module with the type and constructors in `mod.rs` and per-trait implementation files). The bound SHALL be `Real`, not `RealField`: a dual's component needs the analytic operations but never a field inverse. It SHALL expose:

- constructors `Dual::new(re, du)`, `Dual::constant(re)` (with `du = 0`), and `Dual::variable(re)` (with `du = 1`, the differentiation seed);
- accessors `value()` returning the real part `a` and `derivative()` returning the infinitesimal coefficient `b`.

The type SHALL be generic over the precision parameter `T` and SHALL NOT provide concrete precision aliases (`Dual32`/`Dual64`): such aliases would defeat the precision-as-a-parameter design that `Float`/`Real`/`RealField` exist to express. It SHALL be re-exported from the crate root as `Dual`.

#### Scenario: Seed and accessors

- **WHEN** `Dual::variable(x0)` is constructed
- **THEN** `value()` returns `x0` and `derivative()` returns `1`; and `Dual::constant(c)` has `value() == c` and `derivative() == 0`

### Requirement: Arithmetic carries the derivative in the epsilon channel

`Dual<T>` SHALL implement `Add`, `Sub`, `Mul`, and `Neg` (and `Div` where the divisor's real part is invertible) so that the `ε` channel carries the exact derivative by the standard rules: `(a+bε)+(c+dε) = (a+c)+(b+d)ε`; `(a+bε)−(c+dε) = (a−c)+(b−d)ε`; `(a+bε)(c+dε) = ac+(ad+bc)ε`; `−(a+bε) = −a+(−b)ε`; and `(a+bε)/(c+dε) = a/c + ((bc−ad)/c²)ε` for invertible `c`. Consequently, evaluating any function composed from these operations on `Dual::variable(x0)` SHALL yield `f(x0)` in the real part and `f'(x0)` in the `ε` part, exact to machine precision (no finite-difference step, no symbolic expansion).

#### Scenario: Polynomial derivative is exact

- **WHEN** `f(x) = x·x·x + x + x` (i.e. `x³ + 2x`) is evaluated on `Dual::variable(x0)`
- **THEN** the result's `value()` equals `x0³ + 2·x0` and its `derivative()` equals `3·x0² + 2`, to floating-point tolerance

#### Scenario: Product rule falls out of multiplication

- **WHEN** `f(x) = u(x)·v(x)` is evaluated on `Dual::variable(x0)` for differentiable `u`, `v`
- **THEN** the result's `derivative()` equals `u'(x0)·v(x0) + u(x0)·v'(x0)`

### Requirement: Dual implements the Real trait (analytic, not a field)

`Dual<T>` SHALL implement the `Real` trait (from the `real-scalar` capability), so a dual number is a first-class analytic real scalar. Implementing `Real` requires the **complete** `Real` analytic surface, so `Dual<T>` SHALL provide every `Real` method — constants (`pi`, `e`, `epsilon`), elementary functions (`sqrt`, `exp`, `ln`, `log`, `log2`, `log10`, `powf`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`), sign/rounding/shape (`abs`, `floor`, `ceil`, `round`, `clamp`), and exceptional-value predicates/constructors (`nan`, `is_nan`, `is_infinite`, `is_finite`) — each propagating its closed-form derivative through the `ε` channel (for example `exp(a+bε) = e^a + e^a·b·ε`, `sin(a+bε) = sin a + cos a·b·ε`, `sqrt(a+bε) = √a + (b/(2√a))·ε`; non-smooth ops such as `floor` propagate a zero `ε`-channel). `Dual<T>` SHALL NOT implement `Field` or `RealField` (it is not invertible). Because `Dual<T>: Real`, a dual SHALL be usable wherever a `Real` bound is required, and duals SHALL nest (`Dual<Dual<T>>: Real`) for higher-order derivatives.

#### Scenario: Dual is a Real but not a Field

- **WHEN** a `Dual<f64>` value is used in a context bounded on `Real`, and in a context bounded on `Field`/`RealField`
- **THEN** the `Real` context accepts it and the `Field`/`RealField` context rejects it at compile time

#### Scenario: Chain rule through a transcendental composition

- **WHEN** `f(x) = sin(x)·exp(x)` is evaluated on `Dual::variable(x0)`
- **THEN** the result's `derivative()` equals `cos(x0)·exp(x0) + sin(x0)·exp(x0)`, to floating-point tolerance

#### Scenario: Nested duals give second derivatives

- **WHEN** a function is evaluated on `Dual::variable(Dual::variable(x0))` (a `Dual<Dual<f64>>`)
- **THEN** the nested `ε` channels carry the second derivative `f''(x0)`, to floating-point tolerance

### Requirement: Dual numbers form a commutative ring, not a field

`Dual<T>` SHALL implement the property markers `Associative`, `Commutative`, and `Distributive` (all three hold, since `T[ε]/(ε²)` is a quotient of the commutative ring `T[x]`), and the algebra-tower traits it satisfies — `Zero`, `One`, `AddMonoid`, `MulMonoid`, `Ring`, `AssociativeRing`, `CommutativeRing`, and `Module<T>`. It SHALL NOT implement `Field`: even though all three markers hold, `ε` is a zero divisor (`ε·ε = 0`) so a general multiplicative inverse does not exist. Division SHALL be defined only when the real part of the divisor is invertible; the partiality SHALL be documented on the type.

#### Scenario: Ring identities hold

- **WHEN** the additive identity `Dual::zero()` and multiplicative identity `Dual::one()` are used
- **THEN** `d + zero == d`, `d · one == d`, and `d · zero == zero` for any `Dual<T>` value `d`

#### Scenario: Epsilon is nilpotent

- **WHEN** the pure-infinitesimal `Dual::new(0, 1)` is squared
- **THEN** the result equals `Dual::zero()` (`ε² = 0`), witnessing that `Dual<T>` is not a field
