/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[cfg(test)]
mod adjunction_tests;
#[cfg(test)]
mod applicative_tests;
#[cfg(test)]
mod arrow_tests;
#[cfg(test)]
mod bifunctor_tests;
#[cfg(test)]
mod comonad_tests;
#[cfg(test)]
mod cybernetic_loop_tests;
#[cfg(test)]
mod endomorphism_tests;
// `Free`/`Cofree` are alloc-only; gate like `formalization_lean/free_monad_tests`.
#[cfg(all(test, feature = "alloc"))]
mod eq_debug_functor_tests;
#[cfg(test)]
mod foldable_tests;
#[cfg(test)]
mod functor_tests;
#[cfg(test)]
mod io_tests;
#[cfg(test)]
mod monad_tests;
#[cfg(test)]
mod monoidal_merge_tests;
#[cfg(test)]
mod morphism_tests;
#[cfg(test)]
mod parametric_monad_tests;
mod profunctor_tests;
#[cfg(test)]
mod pure_tests;
#[cfg(test)]
mod riemann_map_tests;
#[cfg(test)]
mod traversable_tests;
