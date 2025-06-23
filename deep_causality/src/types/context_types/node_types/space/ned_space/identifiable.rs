use crate::prelude::{Identifiable, NedSpace};

impl Identifiable for NedSpace {
    fn id(&self) -> u64 {
        self.id
    }
}
