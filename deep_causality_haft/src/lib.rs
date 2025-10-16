mod extensions;
mod fp;
mod kind;

pub use fp::{Effect3, Functor, Monad, MonadEffect3};
pub use kind::{HKT, HKT2, HKT3, OptionWitness, Placeholder, ResultWitness};
