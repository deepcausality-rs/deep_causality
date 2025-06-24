use crate::prelude::{AdjustableNedSpace, Identifiable};

impl Identifiable for AdjustableNedSpace {
    fn id(&self) -> u64 {
        self.id
    }
}
