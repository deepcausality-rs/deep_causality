use crate::prelude::{Identifiable, AdjustableNedSpace};

impl Identifiable for AdjustableNedSpace {
    fn id(&self) -> u64 {
        self.id
    }
}
