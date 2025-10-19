mod extensions;
mod fp;
mod kind;
mod witness;
pub mod utils_tests;

pub use fp::{Effect3, Effect4, Functor, Monad, MonadEffect3, MonadEffect4};
pub use kind::{HKT, HKT2, HKT3, HKT4, Placeholder};
pub use witness::{OptionWitness, ResultWitness};