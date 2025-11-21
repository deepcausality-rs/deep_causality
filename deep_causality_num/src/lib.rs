mod alias;
mod cast;
mod complex;
pub mod float;
mod float_option;
mod identity;
pub mod num;
mod ops;
pub mod utils_tests;

pub use crate::alias::{Matrix3, Vector3};
pub use crate::cast::as_primitive::AsPrimitive;
pub use crate::cast::as_scalar::float_as_scalar_impl::FloatAsScalar;
pub use crate::cast::as_scalar::int_as_scalar_impl::IntAsScalar;
pub use crate::cast::from_primitive::FromPrimitive;
pub use crate::cast::num_cast::NumCast;
pub use crate::cast::to_float::{FloatFromInt, IntoFloat};
pub use crate::cast::to_primitive::ToPrimitive;
pub use crate::complex::complex_number::{Complex, Complex32, Complex64, ComplexNumber};
pub use crate::complex::octonion_number::{Octonion, OctonionNumber};
pub use crate::complex::quaternion_number::{Quaternion, QuaternionNumber};

pub use crate::float::Float;
pub use crate::float_option::FloatOption;
pub use crate::identity::one::{ConstOne, One};
pub use crate::identity::zero::{ConstZero, Zero};
pub use crate::num::Num;
pub use crate::ops::num_ops::*;
