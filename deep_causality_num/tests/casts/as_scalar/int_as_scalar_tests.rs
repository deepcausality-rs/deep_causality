/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::IntAsScalar;

macro_rules! test_splat {
    ($name:ident, $type:ty, $value:expr) => {
        #[test]
        fn $name() {
            let value: $type = $value;
            assert_eq!(<$type as IntAsScalar>::splat(value), value);
        }
    };
}

mod u32_as_scalar_tests {
    use super::*;

    test_splat!(splat_u32_pos, u32, 1);
    test_splat!(splat_u32_zero, u32, 0);
    test_splat!(splat_u32_max, u32, u32::MAX);
}

mod u64_as_scalar_tests {
    use super::*;

    test_splat!(splat_u64_pos, u64, 1);
    test_splat!(splat_u64_zero, u64, 0);
    test_splat!(splat_u64_max, u64, u64::MAX);
}
