use std::fmt::{Display, Formatter};

use crate::prelude::{Contextoid, Datable, Symbolic};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;

impl<D, S, T, ST, SYM, VS, VT> Display for Contextoid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone + Display,
    S: Spatial<VS> + Clone + Display,
    T: Temporal<VT> + Clone + Display,
    ST: SpaceTemporal<VS, VT> + Clone + Display,
    SYM: Symbolic + Clone + Display,
    VS: Clone + Display,
    VT: Clone + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Contextoid ID: {} Type: {}", self.id, self.vertex_type)
    }
}
