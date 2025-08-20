mod identifiable;

use crate::{TeloidID, TeloidModal, TeloidTag};
use std::collections::HashMap;

pub type TeloidMetaData = HashMap<String, String>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Teloid {
    id: TeloidID,
    tags: Vec<TeloidTag>,
    modality: TeloidModal,
    metadata: Option<TeloidMetaData>,
}

impl Teloid {
    pub fn new(
        id: TeloidID,
        tags: Vec<TeloidTag>,
        modality: TeloidModal,
        metadata: Option<TeloidMetaData>,
    ) -> Self {
        Self {
            id,
            tags,
            modality,
            metadata,
        }
    }
}
