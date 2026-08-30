/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Chain complexes, boundary operators, and homology over a chosen coefficient field.
//!
//! # What this crate is for
//!
//! A chain complex is a sequence of modules `C_k` with maps `∂ₖ : C_k → C_{k−1}` satisfying
//! `∂ₖ ∘ ∂ₖ₊₁ = 0`. That definition mentions no space, no metric and no cell. Homology —
//! `H_k = ker ∂ₖ / im ∂ₖ₊₁` — is defined the moment the composite vanishes, and everything needed to
//! compute it is linear algebra over the boundary matrices.
//!
//! Geometry supplies chain complexes, and is not the only thing that does. A quantum
//! error-correcting code is a chain complex with no cells: `H_X` and `H_Z` are parity-check matrices
//! whose product vanishes over 𝔽₂, and their homology is the code's logical space. A crate that
//! wanted homology should not have to carry a Hodge star to get it.
//!
//! So the chain-complex layer lives here, and `deep_causality_topology` supplies the geometric half
//! on top of it.
//!
//! # The law
//!
//! Every implementor of [`ChainComplex`] owes `∂ₖ ∘ ∂ₖ₊₁ = 0`. It is not checkable by the trait, and
//! every Betti number this crate computes is wrong without it.
//!
//! # Coefficients
//!
//! Boundary matrices carry `i8`, because their entries are incidence numbers lying in `{−1, 0, 1}`
//! by construction. The coefficient *field* is a property of the computation rather than of the
//! complex, and [`HomologyField`] carries it at the call site. The two answers differ: real
//! projective space has `β₁ = 0` over ℚ and `β₁ = 1` over 𝔽₂.

//! # Why the dependency set is two crates
//!
//! Every other math crate here depends on `deep_causality_algebra`, and most provide
//! `deep_causality_haft` HKT witnesses. This one does neither, for the same reason in both cases:
//! it has no generic parameter for either to attach to.
//!
//! **Algebra bounds coefficient types, and this crate has no coefficient type.** Across
//! `deep_causality_topology` and `deep_causality_linear` the algebra tower appears as a bound on a
//! container's element parameter — `RealField` on 77 sites, `Field` on 24, `Ring` on 7 — as in
//! `Chain<T>` where `T: AbelianGroup` or `SimplicialComplex<T>` where `T: RealField`. No container
//! in the workspace *implements* an algebra trait; the only `impl AbelianGroup for` is the tower's
//! own blanket over scalars.
//!
//! Here the coefficients are fixed. [`ChainComplex::boundary_matrix`] returns `CsrMatrix<i8>`
//! because incidence numbers lie in `{−1, 0, 1}` by construction, [`Gf2Chain`]'s entries are always
//! `Gf2`, and [`HomologyField`] is an enum naming a field rather than a parameter ranging over one.
//! There is no `T` to constrain, so a dependency on the tower would be an unused one — and
//! `cargo-machete` in `.github/workflows/rust_deps.yml` would say so.
//!
//! **`Gf2Chain<W>` is not a functor in `W`.** Every HKT witness in the workspace binds
//! `type Type<T>` to a container generic in its *element*: `CsrMatrix<T>`, `DenseVector<T>`,
//! `Chain<T>`. `W` is the storage word — `u8` through `u64` — and the elements are single bits.
//! `fmap: Gf2Chain<A> -> Gf2Chain<B>` would mean re-packing into a different word width, which maps
//! no content and satisfies no functor law about it.
//!
//! Both crates remain reachable transitively through `deep_causality_linear`. If a coefficient
//! parameter is ever added — a `ChainComplex<R>` over a general ring, which
//! `openspec/changes/extract-homology-crate/design.md` rejects on the merits — the algebra bound
//! arrives with it.

#![cfg_attr(not(feature = "std"), no_std)]

// Unconditional, matching `deep_causality_linear`. Bazel does not read Cargo features, so a
// feature-gated `extern crate alloc` is absent under `bazel test` and every `alloc::` path in the
// crate fails to resolve.
extern crate alloc;
extern crate core;

pub mod errors;
pub mod traits;
pub mod types;
pub mod utils_tests;

pub use crate::errors::homology_error::{HomologyError, HomologyErrorEnum};
pub use crate::traits::chain_complex::ChainComplex;
pub use crate::types::gf2_chain::Gf2Chain;
pub use crate::types::homology_field::HomologyField;
