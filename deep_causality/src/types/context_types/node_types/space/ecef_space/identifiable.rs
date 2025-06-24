use crate::prelude::{EcefSpace, Identifiable};

impl Identifiable for EcefSpace {
    fn id(&self) -> u64 {
        self.id
    }
}
