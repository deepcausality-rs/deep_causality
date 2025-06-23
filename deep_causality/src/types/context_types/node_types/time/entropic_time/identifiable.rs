use crate::prelude::{EntropicTime, Identifiable};

impl Identifiable for EntropicTime {
    fn id(&self) -> u64 {
        self.id
    }
}
