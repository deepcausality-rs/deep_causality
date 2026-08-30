//! Two independent implementations of the Design-B traits, in a crate that is
//! NOT the crate that defined the traits — the orphan-rule situation
//! `deep_causality_tensor` and `deep_causality_topology` would be in.
//!
//! `Dense<F>` stands in for `CausalTensor`. `PackedGf2<W>` is the bit-packed
//! 𝔽₂ matrix that qcl-gaps.md G-01 asks for.

mod dense;
mod packed_gf2;

pub use dense::{Dense, DensePivoted};
pub use packed_gf2::PackedGf2;

#[cfg(test)]
mod tests;
