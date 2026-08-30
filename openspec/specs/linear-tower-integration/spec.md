# linear-tower-integration Specification

## Purpose
TBD - created by archiving change add-linear-algebra-crate. Update Purpose after archive.
## Requirements
### Requirement: Every container implements the tower traits its structure supports
Each of the crate's four containers SHALL implement every `deep_causality_algebra` trait its algebraic structure satisfies, and SHALL document at the impl site any trait it stops short of and why.

An HKT witness makes a container composable through `deep_causality_haft`. It does not make it
composable through the tower: a function bounded on `Ring` or `Module` cannot take a container that
never declares them, however well it behaves. Both surfaces have to be present, or generic code has
to pick one and lose the other.

The crate inherits an unfinished instance of exactly this. `CsrMatrix<f64>` reaches `AbelianGroup`
and stops — verified by compile probe — because `Distributive` and `Annihilating` are not
implemented for it. Everything else `Ring` needs is already there: `One`
(`identity/mod.rs:40`), `Mul` (`arithmetic/mod.rs:213`) and `Associative<Multiplicative>`
(`algebra/group.rs:23`). A matrix ring over a ring **is** a ring, and the tower is two marker impls
away from saying so.

`Module<S>` is *not* part of that gap, and the distinction is worth recording because the obvious
inference is wrong. `Module<R>` is blanket-implemented over
`AbelianGroup + Mul<R, Output = Self> + MulAssign<R>` (`algebra/module.rs:65`), and `CsrMatrix`
satisfies all three already — the additive markers carry it to `AbelianGroup`, and the scaling is
implemented at `arithmetic/mod.rs:283,321`. It is a module today, and an impl written by hand is
E0119. `Ring` is the only rung actually missing.

The move is when this gets finished, not carried across.

#### Scenario: The sparse matrix reaches the ring rung
- **WHEN** `CsrMatrix<f64>` is checked against `Ring`
- **THEN** it satisfies it

#### Scenario: Each container declares what it is
- **WHEN** the dense matrix, the vector, the packed 𝔽₂ matrix and the sparse matrix are enumerated
- **THEN** each names the highest tower trait it satisfies, and the impls for every trait below it are present

#### Scenario: A container that stops short says why
- **WHEN** a container does not implement a trait its shape might suggest
- **THEN** the reason is documented at the impl site, not left to inference

#### Scenario: Generic code takes any of them
- **WHEN** a function bounded on `Ring` is called with each of the matrix containers
- **THEN** each is admitted without an adapter

### Requirement: The vector is a module over its scalar ring
The dense vector SHALL satisfy `Module<R>` for its scalar ring `R`, and every matrix container SHALL satisfy it for the same `R`.

`Module<R: Ring>` (`algebra/module.rs:33`) is the tower's name for a vector space, and a
linear-algebra crate whose vector type does not implement it has skipped the integration the crate
exists for. The trait requires `AbelianGroup + Mul<R, Output = Self> + MulAssign<R>` — addition,
negation and scaling by a ring element, which is every operation a vector has before an inner
product is chosen.

Stating it as `Module<R>` rather than `Field` is what admits ℤ. A module over a ring is the general
notion; a vector space is the special case where the ring is a field. The census found 60 rank-1
constructions against 46 rank-2, so the vector is the larger half of the crate, and bounding it at
the general notion is what lets `deep_causality_topology`'s integer chains stay integer.

#### Scenario: The vector is a module over the integers
- **WHEN** a vector of `i64` is scaled by an `i64`
- **THEN** it compiles through `Module<i64>` without a field bound

#### Scenario: The vector is a module over the reals
- **WHEN** a vector of `f64` is scaled by an `f64`
- **THEN** it compiles through `Module<f64>`

#### Scenario: Scaling is visible to the tower
- **WHEN** a function bounded on `Module<R>` is called with each container
- **THEN** each is admitted, and no container relies on an inherent `scale` method to be scalable

### Requirement: Law markers name the operation they hold for
Every law marker a container implements SHALL name its operator, and no container SHALL claim `Commutative<Multiplicative>` for a multiplication that is matrix multiplication.

Matrix addition commutes; matrix multiplication does not. A marker on the type alone cannot say
which is meant, which is why the operator parameter exists. `CsrMatrix` already gets this right —
`algebra/group.rs:19-23` claims `Associative<Additive>`, `Commutative<Additive>` and
`Associative<Multiplicative>`, and deliberately not `Commutative<Multiplicative>` — and its comment
records that the flat marker made the true claim unstatable, because promising associativity also
promised commutativity.

The dense and packed matrices carry the same three and the same omission. The vector has no
multiplication at all, so it carries the additive pair only.

#### Scenario: Matrix multiplication is associative and not commutative
- **WHEN** each matrix container's markers are enumerated
- **THEN** `Associative<Multiplicative>` is present and `Commutative<Multiplicative>` is absent

#### Scenario: A commutative-ring bound rejects the matrices
- **WHEN** a function bounded on `CommutativeRing` is called with a matrix container
- **THEN** it fails to compile

#### Scenario: The vector claims only additive laws
- **WHEN** the vector's markers are enumerated
- **THEN** they are additive, and no multiplicative law is claimed

### Requirement: Bounds name algebraic structure rather than operator bundles
Every public operation SHALL be bounded on a tower trait, and SHALL NOT be bounded on an ad-hoc collection of `core::ops` traits standing in for one.

The code being moved bounds this way throughout: `mat_mult_impl` takes
`T: Copy + Clone + Mul<Output = T> + Zero + PartialEq + Default`, `transpose_impl` takes
`T: Copy + Zero`, and `vec_mult_impl` takes `T: Copy + Zero + Add<Output = T> + Mul<Output = T>`.
Each of those is a semiring spelled out longhand, and spelling it longhand costs three things: the
bound states no algebraic claim, so nothing records why those operators and not others; a reader
cannot tell whether ℕ is admitted deliberately or by accident; and the operations cannot be
composed with anything bounded on the tower.

Rewriting them as `CommutativeSemiring` is not a widening or a narrowing. It says what was already
meant, in the vocabulary the rest of the workspace uses.

#### Scenario: No operation is bounded on a bare operator bundle
- **WHEN** the crate's public bounds are enumerated
- **THEN** each names a tower trait, with `Copy`, `Clone` and `Default` permitted alongside as representation requirements

#### Scenario: The rewritten bound admits what the old one admitted
- **WHEN** an operation whose bound was an operator bundle is called with the scalars it accepted before
- **THEN** each is still accepted

### Requirement: The lowering sweep is measured, and each lowered bound names what it admits
Every bound loosened from `Field` or `RealField` SHALL name the number set it newly admits, and a test SHALL instantiate the operation at that set.

Integer admission is not a feature bolted alongside the field work — it is what bounding each
operation at its lowest correct level yields. But a loosened bound that nothing instantiates is
indistinguishable from an untested one, and the loosening is the moment a body can quietly stop
being correct for the types it now takes.

The operations that divide keep their field bound; the operations that do not are the sweep.
Addition, subtraction, negation, scaling, matrix multiplication, matrix–vector multiplication,
transpose, trace and the dot product need no inverses. The determinant needs no division either,
which is the case that matters most: it is a polynomial in the entries, and bounding it on `Field`
is what would exclude ℤ from an operation over which it is perfectly well defined.

The sweep does not stop at this crate's boundary. `CausalTensor::matmul` is bounded
`T: Ring + Copy + Default + PartialOrd` (`tensor_product/mod.rs:13`), and matrix multiplication
needs no ordering; each such bound outside the crate is recorded rather than silently left, because
loosening it is a change to a published surface.

#### Scenario: A lowered bound is exercised at its new admission
- **WHEN** an operation is loosened from `Field` to `CommutativeRing`
- **THEN** a test calls it with `i64`

#### Scenario: An operation that divides keeps its bound
- **WHEN** the operations bounded on `Field` are enumerated
- **THEN** each divides, and none is loosened

#### Scenario: An over-bound outside the crate is recorded
- **WHEN** a bound in `deep_causality_tensor` or `deep_causality_sparse` is found stronger than its body needs
- **THEN** it is recorded with the number set it excludes, and loosening it is scheduled rather than done silently

#### Scenario: The naturals reach what they support
- **WHEN** matrix multiplication, transpose, trace and the dot product are called over `u64`
- **THEN** each compiles
- **AND** subtraction, negation and the determinant do not

