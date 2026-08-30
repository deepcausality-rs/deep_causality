# num-finite-field Specification

## Purpose
TBD - created by archiving change add-linear-algebra-crate. Update Purpose after archive.
## Requirements
### Requirement: 𝔽₂ is a tower scalar owned by `deep_causality_num`
`deep_causality_num` SHALL define `Gf2`, the two-element field, and `deep_causality_algebra` SHALL implement the tower traits for it up to `Field` and `IntegralDomain`.

The bit-packed 𝔽₂ matrix needs an element type. `linear-scalar-contract` forbids
`deep_causality_linear` from defining a scalar newtype, and the tower has none to offer, so the type
has nowhere to live until the tower owns it. The prototype already shows the shape the crate needs:
`prototype/linear_a/src/gf2_scalar.rs:15` is `pub struct Gf2(pub bool)` and
`prototype/consumer_b/src/packed_gf2.rs:76` sets `type Scalar = Gf2` on the packed representation.

Packing is about **storage**, not about the element type. The measurement in
`linear-matrix-representations` rejects storing one byte per bit; it does not reject naming the
element. A packed matrix still has to answer `get` with something, and that something is 𝔽₂.

The type lives in `deep_causality_num` and its law markers in `deep_causality_algebra`, which is
where every primitive already sits — `i8` is foreign to the algebra crate too, and
`impl IntegralDomain for i8 {}` is written there without difficulty.

#### Scenario: The tower admits 𝔽₂ as a field
- **WHEN** `Gf2` is checked against `Field`
- **THEN** it satisfies it, reached through `CommutativeRing` and `InvMonoid` exactly as the other scalars are

#### Scenario: 𝔽₂ has no zero divisors
- **WHEN** `Gf2` is checked against `IntegralDomain`
- **THEN** it satisfies it, because a field has no zero divisors

#### Scenario: The linear crate defines no scalar
- **WHEN** `deep_causality_linear`'s public surface is enumerated
- **THEN** it contains no scalar newtype, and its packed representation names `Gf2` from the tower

#### Scenario: Arithmetic is mod 2
- **WHEN** `1 + 1`, `1 - 1` and `-1` are evaluated over `Gf2`
- **THEN** each is `0`, `0` and `1` respectively

### Requirement: The tower separates fields by characteristic, not by finiteness
`deep_causality_algebra` SHALL provide `DivisibleByIntegers` and `FiniteField` as separate refinements of `Field`, and SHALL NOT express the distinction as a finite-versus-infinite predicate.

Admitting 𝔽₂ to `Field` needs a guard, because `Field` is **blanket-implemented**
(`field.rs:41`): a type becomes a field the moment it satisfies `CommutativeRing + InvMonoid + Div +
DivAssign`, with no per-type opt-in. Every `T: Field` bound in the workspace therefore widens the
day `Gf2` lands, and nothing marks which of them can take it. This is the failure the tower has
already had once — a blanket over `Float` widened to `Num` and silently admitted integers to
`Field`.

What actually breaks is **characteristic**, and the workspace has the defect in it today. Sixteen
sites compute `T::one() + T::one()` as two. **Three** sit under a `Field` bound and are reachable by
𝔽₂, all in `deep_causality_multivector`: `commutator_geometric`
(`types/multifield/algebra/mod.rs:163`), whose `let half = T::one() / (T::one() + T::one());` is a
division by zero over 𝔽₂; `types/multifield/ops/conversions.rs:139`; and
`types/multivector/ops/ops_product_impl.rs:316`.

The other thirteen are already excluded, each by a bound 𝔽₂ cannot reach — `RealField` (nine),
`ConjugateScalar` (two, in the tensor-network solver) and `Real` (three, in `num_dual`). That
classification is by compile probe against each bound, not by reading the `where` clause: a
file-level reading of the same sites over-counted the exposure.

Finiteness is neither necessary nor sufficient for that. 𝔽₃ is finite and halves perfectly well;
𝔽₄ is finite and does not; the rational function field 𝔽ₚ(x) is infinite and does not. The property
the code depends on is that `n · 1 ≠ 0` for every `n > 0`, which is exactly characteristic zero.

The trait is named `DivisibleByIntegers` rather than `CharacteristicZero`. They are the same
condition, but a reader at a bound site needs the consequence rather than the property:
`fn halve<T: DivisibleByIntegers>` says why the bound is there, where the textbook name asks the
reader to already know field theory to see it. The textbook term stays reachable as the value
`Characteristic::CHARACTERISTIC`, which is `0` for exactly these types, and the trait's own
documentation gives it along with the equivalent statement that ℤ embeds in such a field.

The two refinements are disjoint **as a matter of definition** — every finite field has prime
characteristic — but they do not partition the fields: 𝔽ₚ(x) is in neither. Stating the tower's cut
as finite-against-infinite would claim a partition that is not one, and would guard the wrong
property while doing it.

#### Scenario: The classical scalars are integer-divisible
- **WHEN** `f32`, `f64`, `Float106`, `Complex<f64>` and `Rational<i64>` are checked against `DivisibleByIntegers`
- **THEN** each satisfies it

#### Scenario: 𝔽₂ is refused integer divisibility
- **WHEN** `Gf2` is checked against `DivisibleByIntegers`
- **THEN** it fails to compile

#### Scenario: The two refinements do not overlap
- **WHEN** a type satisfies `FiniteField`
- **THEN** it does not satisfy `DivisibleByIntegers`, and the reason is recorded at both impl sites

#### Scenario: The cut is not claimed to be a partition
- **WHEN** the documentation of `FiniteField` and `DivisibleByIntegers` is read
- **THEN** it states that a field may be in neither, and names 𝔽ₚ(x) as the case

### Requirement: An operation that divides by an integer is bounded on `DivisibleByIntegers`
Every operation forming `T::one() + T::one()` or any other integer multiple of `one` as a divisor SHALL be bounded on `DivisibleByIntegers` rather than on `Field`.

This is the admission control the separation exists for. The bound is unverifiable in the same sense
as every other law in this tower — the compiler cannot check that `n · 1 ≠ 0` — but it is checkable
where it matters, at the signature, which is what stops 𝔽₂ reaching a body that halves.

The four exposed sites are the migration, and they are the whole of it: the remaining twelve are
already excluded by `RealField`. Bounding on `DivisibleByIntegers` makes the compiler enumerate any
that were missed, the same way removing the operator default enumerated the eight law-marker
stragglers.

#### Scenario: Halving is unavailable over 𝔽₂
- **WHEN** an operation that divides by two is called with `Gf2`
- **THEN** it fails to compile, and the error names `DivisibleByIntegers`

#### Scenario: The exposed sites are found rather than assumed
- **WHEN** `DivisibleByIntegers` replaces `Field` on the operations that halve
- **THEN** the compiler reports every remaining site that needs it

#### Scenario: Halving still works where it always did
- **WHEN** the same operations are called with `f64`, `Complex<f64>` or `Rational<i64>`
- **THEN** they compile and return what they returned before

### Requirement: `FiniteField` carries its order
`FiniteField` SHALL expose the order of the field, and `Characteristic` SHALL expose the characteristic, so that generic code can read both without naming a concrete type.

A finite field has order `q = p^k` for a prime `p` and `k ≥ 1`, and the two numbers are different
questions: 𝔽₄ has order 4 and characteristic 2. Code that reduces mod p needs the characteristic;
code that enumerates the field or computes a Frobenius power needs the order. Exposing only one
would make the other unreachable, and 𝔽₄ is the case that shows they are not the same number.

#### Scenario: 𝔽₂ reports both numbers
- **WHEN** the order and characteristic of `Gf2` are read
- **THEN** both are 2

#### Scenario: Characteristic zero reports zero
- **WHEN** the characteristic of `f64` is read
- **THEN** it is 0, the conventional value for "no such `n`"

### Requirement: 𝔽₂ does not reach the ordered, normed or Euclidean rungs
`Gf2` SHALL NOT implement `RealField`, `Normed`, `NormedScalar`, `ConjugateScalar` or `EuclideanDomain`.

𝔽₂ has no order, so pivot selection by magnitude is meaningless over it; it has no modulus, so no
norm; and it has no conjugation. Elimination over 𝔽₂ needs none of these — any non-zero entry is a
pivot, and there is only one non-zero entry — which is why `linear-dense-algorithms` gives the
row-operation trait an overridable pivot rule rather than a magnitude comparison.

`EuclideanDomain` is the one exclusion that is a tower convention rather than a mathematical fact.
Every field is a Euclidean domain, 𝔽₂ included. This tower reserves the rung for the integers, as
`linear-scalar-contract` records, and admitting 𝔽₂ would blur a boundary that exists to keep the
exact-integer path separate from the field path.

#### Scenario: The ordered and normed rungs refuse 𝔽₂
- **WHEN** `Gf2` is checked against `RealField`, `Normed`, `NormedScalar` and `ConjugateScalar`
- **THEN** each fails to compile

#### Scenario: The Euclidean rung refuses 𝔽₂ by convention
- **WHEN** `Gf2` is checked against `EuclideanDomain`
- **THEN** it fails to compile
- **AND** the documentation records that this is the tower's boundary and not a claim about 𝔽₂

#### Scenario: Elimination over 𝔽₂ needs no ordering
- **WHEN** `rref` runs over the packed 𝔽₂ representation
- **THEN** it selects pivots without comparing magnitudes and without an epsilon

