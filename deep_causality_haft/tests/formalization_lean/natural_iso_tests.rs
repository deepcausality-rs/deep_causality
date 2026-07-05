/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/NaturalIso.lean` (Mac Lane, CWM §I.4).

use deep_causality_haft::{
    Either, Functor, HKT, NaturalIso, NoConstraint, OptionWitness, Satisfies,
};

// Option A ≅ Either<(), A> — two genuinely different constructors.
struct UnitSumWitness;
impl HKT for UnitSumWitness {
    type Constraint = NoConstraint;
    type Type<T> = Either<(), T>;
}

struct OptSumIso;
impl NaturalIso<OptionWitness, UnitSumWitness> for OptSumIso {
    fn to_target<T>(fa: Option<T>) -> Either<(), T>
    where
        T: Satisfies<NoConstraint>,
    {
        match fa {
            Some(a) => Either::Right(a),
            None => Either::Left(()),
        }
    }

    fn to_source<T>(ga: Either<(), T>) -> Option<T>
    where
        T: Satisfies<NoConstraint>,
    {
        match ga {
            Either::Right(a) => Some(a),
            Either::Left(()) => None,
        }
    }
}

/// THEOREM_MAP: haft.natural_iso.laws
#[test]
fn test_natural_iso_laws() {
    // Round trips
    for x in [Some(5), None::<i32>] {
        assert_eq!(OptSumIso::to_source(OptSumIso::to_target(x)), x);
    }
    for y in [Either::<(), i32>::Left(()), Either::Right(9)] {
        assert_eq!(OptSumIso::to_target(OptSumIso::to_source(y)), y);
    }

    // Naturality: to_target ∘ fmap_F h = fmap_G h ∘ to_target
    let h = |a: i32| a * 5;
    let sum_fmap = |e: Either<(), i32>| -> Either<(), i32> {
        match e {
            Either::Left(()) => Either::Left(()),
            Either::Right(a) => Either::Right(h(a)),
        }
    };
    for x in [Some(4), None::<i32>] {
        assert_eq!(
            OptSumIso::to_target(OptionWitness::fmap(x, h)),
            sum_fmap(OptSumIso::to_target(x))
        );
    }
}
