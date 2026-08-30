# linear-scalar-contract Specification

## Purpose
TBD - created by archiving change add-linear-algebra-crate. Update Purpose after archive.
## Requirements
### Requirement: The crate defines no scalar trait of its own
`deep_causality_linear` SHALL bound every operation on a trait already published by `deep_causality_algebra` or `deep_causality_num`, and SHALL NOT define a scalar trait, a numeric marker, or a scalar newtype.

The tower already carries the whole scalar layer: `CommutativeSemiring`
(`algebra/semiring_commutative.rs:35`), `CommutativeRing` (`ring_commutative.rs:31`),
`IntegralDomain` (`domain_integral.rs:55`), `EuclideanDomain` (`domain_euclidean.rs:53`), `Field`
(`field.rs:38`), `RealField` (`field_real.rs:25`), `Normed` (`normed.rs:16`), `NormedScalar`
(`scalar_normed.rs:24`), `ConjugateScalar` (`scalar_conjugate.rs:33`), `Scalar` (`scalar.rs:29`) and
`NaturalNumber` (`num/integer/natural.rs:58`). A linear-algebra crate that introduced its own would
fork the hierarchy the `deep_causality_num` split built, and would reintroduce that split's
coherence traps.

The one scalar the tower did not carry is 𝔽₂, and the packed representation needs it as an element
type. That is resolved by the tower gaining it rather than by this crate defining it:
`num-finite-field` puts `Gf2` in `deep_causality_num` alongside every other primitive. The rule here
is therefore unconditional — there is no scalar this crate has to invent.

#### Scenario: The packed representation names a tower scalar
- **WHEN** the element type of the bit-packed 𝔽₂ matrix is inspected
- **THEN** it is `Gf2` from `deep_causality_num`, not a type declared in this crate

#### Scenario: No new scalar abstraction appears
- **WHEN** the crate's public surface is enumerated
- **THEN** every scalar bound names a trait from `deep_causality_algebra` or `deep_causality_num`

#### Scenario: A tower scalar works unchanged
- **WHEN** an operation is called with `i64`, `f64`, `Float106`, `Complex<f64>` or `Rational<i64>`
- **THEN** it compiles without an adapter, a newtype, or a `where` clause supplied by the caller

### Requirement: Operations are banded by the algebraic structure they need
Every operation SHALL be bounded on the weakest tower trait sufficient for it, and no operation requiring division SHALL be bounded on a structure lacking multiplicative inverses.

This is the point of having a tower rather than a `f64` library with generics bolted on. Bounding
elimination on `RealField` would exclude 𝔽₂ and ℚ, which have no ordering and need none. Bounding the
determinant on `Field` would exclude ℤ, over which it is perfectly well defined — the determinant is
a **polynomial in the entries and needs no division at all**, whereas Gaussian elimination divides by
its pivot and therefore leaves ℤ.

The bands, which are a tree rather than a chain:

```
CommutativeSemiring ............. ℕ
  └─ CommutativeRing ............ ℤ, 𝔽₂, ℚ, ℝ, ℂ
       └─ IntegralDomain ........ ℤ, 𝔽₂, ℚ, ℝ, ℂ    — not ℝ[ε]
            ├─ Field ............ 𝔽₂, ℚ, ℝ, ℂ       — not ℤ
            │    ├─ DivisibleByIntegers ... ℚ, ℝ, ℂ   — not 𝔽₂
            │    │    └─ NormedScalar / RealField ... ℝ, ℂ, `Float106`
            │    └─ FiniteField ......... 𝔽₂          — not ℚ, ℝ, ℂ
            └─ EuclideanDomain .. ℤ                  — not 𝔽₂, ℚ, ℝ, ℂ
```

| band | tower bound | what it admits | operations |
|---|---|---|---|
| semiring | `CommutativeSemiring` | ℕ | add, scale, matmul, matrix–vector, dot, transpose, trace |
| ring | `CommutativeRing` | ℤ, 𝔽₂, ℚ, ℝ, ℂ | the above, plus subtract, negate, **determinant** |
| integral domain | `IntegralDomain` | ℤ, 𝔽₂, ℚ, ℝ, ℂ | the above, plus anything resting on cancellation |
| Euclidean domain | `EuclideanDomain` | ℤ | fraction-free determinant and exact rank — see `linear-integer-algebra` |
| field | `Field` | 𝔽₂, ℚ, ℝ, ℂ | rref, rank, kernel basis, image basis, inverse, solve |
| integer-divisible field | `DivisibleByIntegers` | ℚ, ℝ, ℂ | anything dividing by an integer — see `num-finite-field` |
| normed field | `NormedScalar` | ℝ, ℂ, `Float106` | norms, and pivot selection by magnitude |
| real field | `RealField` | ℝ, `Float106` | SVD, QR, eigendecomposition, Cholesky, conjugate gradient |

`IntegralDomain` is the rung that makes the integer path's exactness a stated promise rather than an
assumption. Bareiss elimination is correct because cancellation holds, and cancellation holds because
there are no zero divisors — not because a Euclidean valuation exists. `EuclideanDomain` now sits
above it, so the bound `linear-integer-algebra` uses carries the promise its algorithm rests on,
which it did not when `EuclideanDomain: CommutativeRing`.

`DivisibleByIntegers` is the rung that keeps 𝔽₂ out of a body that halves. It is not a convenience:
`Field` is blanket-implemented, so admitting 𝔽₂ to the tower widens every `T: Field` bound in the
workspace at once, and three of them divide by `one + one`. `num-finite-field` carries the reasoning
and the measurement.

**`Field` and `EuclideanDomain` are disjoint sets of concrete types in this tower**, even though
mathematically every field is a Euclidean domain. `EuclideanDomain` is implemented for the six signed
integer types and nothing else (`domain_euclidean.rs`), so `f64: EuclideanDomain` does not hold. A
`Field`-bounded operation therefore rejects `i64`, and an `EuclideanDomain`-bounded one rejects
`f64` — neither band covers the other, and an operation wanted for both is written at
`CommutativeRing` or provided twice.

`CommutativeRing` is the load-bearing band. It is the lowest bound at which the shared container
operations are correct, and ℤ, 𝔽₂, ℚ, ℝ and ℂ all sit in it. Integer support is therefore not a
feature added alongside the field work — it is what bounding each operation at its lowest correct
level yields, and over-bounding to `Field` is what would have excluded ℤ.

ℕ admits the least: `CommutativeSemiring` has no additive inverse, so subtraction and the determinant
are both unavailable over it. That is not a limitation to work around; `3u64 - 5u64` has no value.

#### Scenario: The determinant works over the integers
- **WHEN** the determinant of an `i64` matrix is taken
- **THEN** it compiles and returns an `i64`
- **AND** no division by a pivot occurs

#### Scenario: Elimination does not admit the integers
- **WHEN** `rref` is attempted on an `i64` matrix
- **THEN** it fails to compile, because `i64` is not a `Field`

#### Scenario: The integer path does not admit the floats
- **WHEN** the fraction-free determinant is attempted on an `f64` matrix
- **THEN** it fails to compile, because `f64` is not an `EuclideanDomain` in this tower

#### Scenario: A ring operation admits both
- **WHEN** matrix multiplication is called on `i64` and on `f64` matrices
- **THEN** both compile, because both are `CommutativeRing`

#### Scenario: Elimination admits an unordered field
- **WHEN** `rref` is called over 𝔽₂ or over `Rational<i64>`
- **THEN** it compiles and runs, requiring no ordering and no epsilon

#### Scenario: Pivoted solve admits complex
- **WHEN** `solve` is called on a `Complex<f64>` matrix
- **THEN** it compiles and pivots on `modulus_squared`
- **AND** it does not require the scalar to be ordered

#### Scenario: Subtraction does not admit the naturals
- **WHEN** matrix subtraction is attempted over `u64`
- **THEN** it fails to compile

#### Scenario: The integer path's exactness is a stated promise
- **WHEN** the bound on the fraction-free determinant is traced upward
- **THEN** it reaches `IntegralDomain`, and the absence of zero divisors is what the exact divisions rest on

#### Scenario: An operation that halves does not admit 𝔽₂
- **WHEN** an operation that divides by two is attempted over `Gf2`
- **THEN** it fails to compile, because `Gf2` is not `DivisibleByIntegers`

#### Scenario: Elimination admits 𝔽₂ because it never halves
- **WHEN** `rref` is called over `Gf2`
- **THEN** it compiles, because elimination divides only by pivots and every non-zero element of 𝔽₂ is its own inverse

#### Scenario: An over-bound is a defect
- **WHEN** an operation's bound is stronger than its body needs
- **THEN** it is loosened rather than documented as a limitation

### Requirement: Each bound is documented with the property it needs
Every public operation SHALL state, in its documentation, which algebraic property its bound supplies and what would break without it.

A bound with no stated reason is indistinguishable from an accident, and the next person to touch it
cannot tell whether loosening it is safe. The tower work already produced one instance of this: a
blanket over `Float` was widened to `Num` and silently admitted integers to `Field`, because nothing
recorded why the narrower bound was there.

#### Scenario: A bound explains itself
- **WHEN** a public operation's documentation is read
- **THEN** it names the property the bound supplies — inverses, ordering, an epsilon, conjugation, exact divisibility

### Requirement: Exact and approximate scalars are distinguished at the API
An operation over an exact structure SHALL NOT accept, expose or apply a tolerance, and an operation whose correctness depends on a tolerance SHALL take it as an argument or derive it from the scalar's `epsilon`.

Rank over 𝔽₂, rank over ℤ and rank over ℝ are three different questions, and the workspace has
already been bitten by conflating them: `chain_complex_impl.rs:94` computes homology by thresholding
f64 singular values at `1e-5` on a matrix whose entries are `{-1, 0, 1}`. Making the distinction
visible in the signature is what stops that recurring.

#### Scenario: An exact path has no tolerance in its signature
- **WHEN** the operations over 𝔽₂, ℤ and `Rational` are enumerated
- **THEN** none takes a tolerance parameter

#### Scenario: An approximate path names its tolerance
- **WHEN** an operation's result depends on a threshold
- **THEN** the threshold is an argument or is derived from `epsilon`, and never a literal in the body

#### Scenario: The ranks are separately reachable
- **WHEN** a caller wants the rank of an integer matrix
- **THEN** the exact rank and the numerical rank are different calls, and the caller chooses

