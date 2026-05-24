/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod surd_algo_cdl_info_leak_tests;
// `surd_algo_cdl_max_order_tests` and `surd_algo_cdl_tests` are skipped under
// Miri: the CDL (`Option<f64>`) path branches on tight (`< 1e-14`) thresholds
// and uses float-keyed sort tie-breaking. Miri's soft-float emulation drifts
// intermediate results by ~1 ULP, which crosses those threshold/tie edges and
// causes `ShapeMismatch` errors or wrong state-bucket classification. The
// non-CDL SURD coverage above still runs under Miri. Tests are correct and
// pass under normal CI.
#[cfg(not(miri))]
mod surd_algo_cdl_max_order_tests;
mod surd_algo_cdl_none_tests;
#[cfg(not(miri))]
mod surd_algo_cdl_tests;
#[cfg(test)]
mod surd_algo_tests;
#[cfg(test)]
mod surd_consistency_tests;
#[cfg(test)]
mod surd_max_order_tests;
#[cfg(test)]
mod surd_result_tests;
