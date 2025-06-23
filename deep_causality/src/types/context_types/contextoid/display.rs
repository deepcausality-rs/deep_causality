use std::fmt::{Display, Formatter};

use crate::prelude::{
    Contextoid, Datable, Symbolic,
};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;

impl<D, S, T, ST, SYM, V> Display for Contextoid<D, S, T, ST, SYM, V>
where
    D: Datable + Display,
    S: Spatial<V> + Display,
    T: Temporal<V> + Display,
    ST: SpaceTemporal<V> + Display,
    SYM: Symbolic + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Contextoid ID: {} Type: {}", self.id, self.vertex_type)
    }
}
