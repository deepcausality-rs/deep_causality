# cayley-dickson-monoidal-applicative Specification

## Purpose
TBD - created by archiving change add-lax-monoidal-applicative. Update Purpose after archive.
## Requirements
### Requirement: The Cayley-Dickson witnesses gain the monoidal applicative stack
`ComplexWitness`, `QuaternionWitness` and `OctonionWitness` in `deep_causality_num_complex` SHALL implement `Semigroupal`, `LaxMonoidal`, `Convolutional` and `MonoidalApplicative`, closing `unified_math_gaps.md` §4.1 item E1. `zip_with` SHALL pair components positionally, slot by slot, and `unit` SHALL return the all-`()` value. Neither SHALL require `Clone` on the payload, and neither SHALL name `RealField` or any arithmetic bound, since these operations move components without computing on them.

#### Scenario: Componentwise pairing over two, four and eight slots

- **WHEN** `zip_with` is called on two `Complex`, two `Quaternion` or two `Octonion` values
- **THEN** each output slot is built from the same-indexed slot of each input, and every input component is moved exactly once

#### Scenario: The witnesses stay unconstrained

- **WHEN** the impls are inspected
- **THEN** the HKT `Constraint` remains `NoConstraint` and no arithmetic bound appears, while the arithmetic impls keep naming `RealField` themselves

#### Scenario: Applicative is reachable where `Pure` is not

- **WHEN** `MonoidalApplicative::apply` is used on a Cayley-Dickson witness
- **THEN** it works, while `Pure` remains unimplemented for it because filling n slots from one moved value is impossible

### Requirement: The componentwise choice is documented as forced, not preferred
The module documentation SHALL record why componentwise pairing is the unique lawful φ for this family rather than one option among several. For `F(A) = A^S` over a finite index set `S`, `F(())` is a singleton so η is forced; by Yoneda every natural φ has the form `φ(fa, fb)_s = (fa_{u(s)}, fb_{v(s)})` for fixed endofunctions `u, v` of `S`, and the two unit laws force `u = v = id`. The documentation SHALL also record that the resulting applicative is Reader on a finite index set, whose `pure` is the constant map and therefore the diagonal.

#### Scenario: The absence of `Pure` is explained rather than asserted

- **WHEN** a reader asks why these witnesses have `MonoidalApplicative` but not `Pure`
- **THEN** the module documentation gives the diagonal argument, not a bare statement that it is unimplemented

### Requirement: Law tests pin the structure on every arity
Each of the three witnesses SHALL carry law tests for φ naturality, φ associativity modulo the associator, and both unit laws modulo the unitors. The tests SHALL use varied and non-cherry-picked inputs rather than a single fixture, and SHALL include at least one payload type that is not a float, exercising `fmap` and `zip_with` across unrelated payload types.

#### Scenario: Naturality holds on all three arities

- **WHEN** `zip_with(fmap(fa, f), fmap(fb, g), pair)` is compared against `fmap(zip(fa, fb), f × g)` for `Complex`, `Quaternion` and `Octonion`
- **THEN** the two agree for every generated input

#### Scenario: The swap variant is rejected

- **WHEN** a pairing that crosses slot indices is substituted for the componentwise one
- **THEN** the unit law test fails, demonstrating the tests discriminate the lawful φ from a plausible alternative

