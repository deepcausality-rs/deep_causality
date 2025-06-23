use crate::prelude::{AdjustableEcefSpace, Identifiable};

impl Identifiable for AdjustableEcefSpace {
    fn id(&self) -> u64 {
        self.id 
    }
}