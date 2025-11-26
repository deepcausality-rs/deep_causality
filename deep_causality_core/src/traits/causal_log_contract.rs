use core::fmt::{Debug, Display};
use deep_causality_haft::LogAppend;

pub trait CausalLogContract: LogAppend + Default + Debug + Display {
    fn is_empty(&self) -> bool;
}
