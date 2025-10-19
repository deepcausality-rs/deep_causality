mod applicative;
mod effect;
mod extensions;
mod foldable;
mod functor;
mod hkt;
mod monad;
mod monad_effect;
pub mod utils_tests;

pub use crate::applicative::Applicative;
pub use crate::effect::{Effect3, Effect4, Effect5};
pub use crate::extensions::hkt_option_ext::OptionWitness;
pub use crate::extensions::hkt_result_ext::ResultWitness;
pub use crate::extensions::hkt_vec_ext::VecWitness;
pub use crate::foldable::Foldable;
pub use crate::functor::Functor;
pub use crate::hkt::{HKT, HKT2, HKT3, HKT4, HKT5, Placeholder};
pub use crate::monad::Monad;
pub use crate::monad_effect::{MonadEffect3, MonadEffect4, MonadEffect5};

// Traversable
