# real-scalar Specification

## Purpose
TBD - created by archiving change num-real-trait. Update Purpose after archive.
## Requirements
### Requirement: Real trait for analytic real scalars without field invertibility

`deep_causality_num` SHALL provide a `Real` trait modelling an analytic real scalar decoupled from field invertibility. Its supertraits SHALL be `CommutativeRing + PartialOrd + Neg<Output = Self> + Copy + Clone + AddAssign + SubAssign + MulAssign`, and SHALL NOT include `Div`, `DivAssign`, `InvMonoid`, or `Field`. It SHALL declare the analytic surface currently on `RealField` that does not depend on division: the constants (`pi`, `e`, `epsilon`); the elementary functions (`sqrt`, `exp`, `ln`, `log`, `log2`, `log10`, `powf`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`); sign/rounding/shape (`abs`, `floor`, `ceil`, `round`, `clamp`); and exceptional-value predicates/constructors (`nan`, `is_nan`, `is_infinite`, `is_finite`). The trait SHALL be re-exported from the crate root.

#### Scenario: Real is implementable without division

- **WHEN** a type provides commutative-ring arithmetic (`+ − ×`) and the analytic surface but has no total multiplicative inverse
- **THEN** it can implement `Real` without implementing `Field`, `Div`, or `DivAssign`

#### Scenario: f32, f64, and Float106 are Real

- **WHEN** the elementary functions and constants are invoked through a `Real` bound on `f32`, `f64`, or `Float106`
- **THEN** they produce results identical to the prior `RealField` implementation for those types

### Requirement: RealField extends Real and Field, preserving its surface

`RealField` SHALL be refactored to `RealField: Real + Field`, with the analytic method declarations relocated from `RealField` into `Real` and only the field-specific surface (division-based operations such as `inverse`) remaining on `RealField`. The refactor SHALL be behavior-preserving: any existing `T: RealField` bound SHALL continue to resolve the identical method set (the analytic surface now inherited via the `Real` supertrait, the field surface via `Field`), so no existing `RealField`-generic code requires modification. The concrete `RealField` implementations for `f32`, `f64`, and `Float106` SHALL relocate their analytic method bodies into `impl Real` blocks verbatim, yielding bit-identical numeric results.

#### Scenario: Existing RealField-generic code is unchanged

- **WHEN** a function bounded on `T: RealField` calls analytic operations (for example `x.sin()`, `x.exp()`, `T::pi()`) and field operations (`x / y`)
- **THEN** it compiles and runs unchanged after the refactor, resolving the analytic operations through the `Real` supertrait and the field operations through `Field`

#### Scenario: RealField implies Real

- **WHEN** a generic context holds `T: RealField`
- **THEN** `T: Real` holds automatically (a `T: RealField` value is accepted wherever `Real` is required), and every `RealField` value is a `Real` value

#### Scenario: A non-field analytic type is Real but not RealField

- **WHEN** a type implements `Real` but not `Field` (it lacks a total inverse)
- **THEN** it is accepted by `Real` bounds and rejected by `RealField` bounds at compile time

### Requirement: A Float type inherits the algebra tower through blanket impls

The crate SHALL provide the real-scalar algebra for floating-point types via **blanket implementations over the `Float` trait**, not per-type implementations. `Float` SHALL declare the constants `pi()` and `e()` and SHALL require the in-place arithmetic operators (`AddAssign`, `SubAssign`, `MulAssign`, `DivAssign`) in addition to the by-value ones. The crate SHALL provide `impl<T: Float>` for the markers (`Associative`, `Commutative`, `Distributive`), `AbelianGroup`, `DivisionAlgebra<T>`, `Real` (delegating each method to the corresponding `Float` operation), and `RealField`; the intermediate traits (`Ring`, `CommutativeRing`, `Field`, etc.) SHALL remain blanket-derived. Consequently, a type that implements `Float` SHALL automatically satisfy `Real`, `RealField`, `DivisionAlgebra`, and the full derived tower **without any per-type algebra implementation**. Only `Zero`, `One`, and `Num` (which sit below `Float`) SHALL be implemented per float type. These blankets SHALL NOT conflict with the existing non-`Float` impls for `Complex`, `Quaternion`, `Octonion`, or the integer types (guaranteed because `Float` is a crate-local trait).

#### Scenario: f32, f64, and Float106 are real scalars via the Float blankets

- **WHEN** `f32`, `f64`, or `Float106` is used where `Real`/`RealField`/`DivisionAlgebra` is required
- **THEN** it satisfies the bound through the `impl<T: Float>` blankets, with no per-type `Real`/`RealField`/marker/`AbelianGroup`/`DivisionAlgebra` implementation present, and produces results identical to the prior per-type implementations

#### Scenario: A new float type needs only `impl Float`

- **WHEN** a new floating-point type implements `Float` (and the below-`Float` traits `Zero`/`One`/`Num`)
- **THEN** it automatically gains `Real`, `RealField`, `DivisionAlgebra`, and the derived algebra tower with no further edits to the algebra trait files

#### Scenario: The Float blankets do not collide with hypercomplex or integer types

- **WHEN** the `impl<T: Float>` blankets coexist with the explicit impls for `Complex`/`Quaternion`/`Octonion` and the integer types
- **THEN** the crate compiles with no conflicting-implementation (coherence) error, because none of those types implements the crate-local `Float` trait

