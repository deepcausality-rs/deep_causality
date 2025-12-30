/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for GammaProvider trait implementations.

use deep_causality_metric::Metric;
use deep_causality_multivector::{BackendGamma, CpuGammaLoader, GammaProvider};
use deep_causality_tensor::{CpuBackend, TensorBackend};

// =============================================================================
// CpuBackend GammaProvider tests
// =============================================================================

#[test]
fn test_cpu_backend_implements_gamma_provider() {
    // Verify CpuBackend implements GammaProvider<f32>
    fn assert_gamma_provider<B: GammaProvider<f32>>() {}
    assert_gamma_provider::<CpuBackend>();
}

#[test]
fn test_cpu_backend_implements_gamma_provider_f64() {
    // Verify CpuBackend implements GammaProvider<f64>
    fn assert_gamma_provider<B: GammaProvider<f64>>() {}
    assert_gamma_provider::<CpuBackend>();
}

#[test]
fn test_cpu_gamma_loader_accessible_via_provider() {
    let metric = Metric::from_signature(3, 0, 0);

    // Access gamma matrices through BackendGamma with explicit type params
    let gammas = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_gammas(&metric);
    let shape = CpuBackend::shape(&gammas);

    assert_eq!(shape, vec![3, 4, 4]);
}

#[test]
fn test_cpu_basis_gammas_via_provider() {
    let metric = Metric::from_signature(2, 0, 0);

    let basis = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_basis_gammas(&metric);
    let shape = CpuBackend::shape(&basis);

    assert_eq!(shape, vec![4, 2, 2]);
}

#[test]
fn test_cpu_dual_basis_via_provider() {
    let metric = Metric::from_signature(2, 0, 0);

    let dual = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_dual_basis_gammas(&metric);
    let shape = CpuBackend::shape(&dual);

    assert_eq!(shape, vec![4, 2, 2]);
}

// =============================================================================
// MlxBackend GammaProvider tests
// =============================================================================

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod mlx_provider_tests {
    use super::*;
    use deep_causality_multivector::MlxGammaLoader;
    use deep_causality_tensor::MlxBackend;

    #[test]
    fn test_mlx_backend_implements_gamma_provider() {
        fn assert_gamma_provider<B: GammaProvider<f32>>() {}
        assert_gamma_provider::<MlxBackend>();
    }

    #[test]
    fn test_mlx_gamma_loader_accessible_via_provider() {
        let metric = Metric::from_signature(3, 0, 0);

        let gammas = <MlxGammaLoader as BackendGamma<MlxBackend, f32>>::get_gammas(&metric);
        let shape = MlxBackend::shape(&gammas);

        assert_eq!(shape, vec![3, 4, 4]);
    }
}
