/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for MlxGammaLoader implementing BackendGamma trait.
//! Feature-gated for MLX backend on Apple Silicon.

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod mlx_gamma_tests {
    use deep_causality_metric::Metric;
    use deep_causality_multivector::BackendGamma;
    use deep_causality_tensor::{MlxBackend, TensorBackend};

    type MlxGammaLoader = deep_causality_multivector::MlxGammaLoader;

    // =========================================================================
    // get_gammas() tests
    // =========================================================================

    #[test]
    fn test_get_gammas_shape_cl2() {
        let metric = Metric::from_signature(2, 0, 0);
        let gammas = <MlxGammaLoader as BackendGamma<MlxBackend, f32>>::get_gammas(&metric);
        let shape = MlxBackend::shape(&gammas);

        assert_eq!(shape, vec![2, 2, 2]);
    }

    #[test]
    fn test_get_gammas_shape_cl3() {
        let metric = Metric::from_signature(3, 0, 0);
        let gammas = <MlxGammaLoader as BackendGamma<MlxBackend, f32>>::get_gammas(&metric);
        let shape = MlxBackend::shape(&gammas);

        assert_eq!(shape, vec![3, 4, 4]);
    }

    #[test]
    fn test_get_gammas_clifford_identity_cl2() {
        let metric = Metric::from_signature(2, 0, 0);
        let gammas = <MlxGammaLoader as BackendGamma<MlxBackend, f32>>::get_gammas(&metric);
        let data: Vec<f32> = MlxBackend::to_vec(&gammas);

        let dim = 2;

        for gamma_idx in 0..2 {
            let mut sq = [[0.0f32; 2]; 2];
            for r in 0..dim {
                for c in 0..dim {
                    let mut sum = 0.0f32;
                    for k in 0..dim {
                        let idx_rk = gamma_idx * dim * dim + r * dim + k;
                        let idx_kc = gamma_idx * dim * dim + k * dim + c;
                        sum += data[idx_rk] * data[idx_kc];
                    }
                    sq[r][c] = sum;
                }
            }

            assert!((sq[0][0] - 1.0).abs() < 1e-5);
            assert!((sq[1][1] - 1.0).abs() < 1e-5);
            assert!(sq[0][1].abs() < 1e-5);
            assert!(sq[1][0].abs() < 1e-5);
        }
    }

    // =========================================================================
    // get_basis_gammas() tests
    // =========================================================================

    #[test]
    fn test_get_basis_gammas_shape_cl2() {
        let metric = Metric::from_signature(2, 0, 0);
        let basis = <MlxGammaLoader as BackendGamma<MlxBackend, f32>>::get_basis_gammas(&metric);
        let shape = MlxBackend::shape(&basis);

        assert_eq!(shape, vec![4, 2, 2]);
    }

    #[test]
    fn test_get_basis_gammas_identity_blade() {
        let metric = Metric::from_signature(2, 0, 0);
        let basis = <MlxGammaLoader as BackendGamma<MlxBackend, f32>>::get_basis_gammas(&metric);
        let data: Vec<f32> = MlxBackend::to_vec(&basis);

        let dim = 2;

        assert!((data[0 * dim * dim + 0 * dim + 0] - 1.0).abs() < 1e-5);
        assert!((data[0 * dim * dim + 1 * dim + 1] - 1.0).abs() < 1e-5);
        assert!(data[0 * dim * dim + 0 * dim + 1].abs() < 1e-5);
        assert!(data[0 * dim * dim + 1 * dim + 0].abs() < 1e-5);
    }

    // =========================================================================
    // get_dual_basis_gammas() tests
    // =========================================================================

    #[test]
    fn test_get_dual_basis_gammas_shape_cl2() {
        let metric = Metric::from_signature(2, 0, 0);
        let dual =
            <MlxGammaLoader as BackendGamma<MlxBackend, f32>>::get_dual_basis_gammas(&metric);
        let shape = MlxBackend::shape(&dual);

        assert_eq!(shape, vec![4, 2, 2]);
    }

    #[test]
    fn test_get_dual_basis_gammas_identity_blade_dual() {
        let metric = Metric::from_signature(2, 0, 0);
        let dual =
            <MlxGammaLoader as BackendGamma<MlxBackend, f32>>::get_dual_basis_gammas(&metric);
        let data: Vec<f32> = MlxBackend::to_vec(&dual);

        let dim = 2;

        assert!((data[0 * dim * dim + 0 * dim + 0] - 1.0).abs() < 1e-5);
        assert!((data[0 * dim * dim + 1 * dim + 1] - 1.0).abs() < 1e-5);
        assert!(data[0 * dim * dim + 0 * dim + 1].abs() < 1e-5);
        assert!(data[0 * dim * dim + 1 * dim + 0].abs() < 1e-5);
    }

    // =========================================================================
    // CPU/MLX consistency tests
    // =========================================================================

    #[test]
    fn test_mlx_cpu_gamma_consistency() {
        use deep_causality_multivector::CpuGammaLoader;
        use deep_causality_tensor::CpuBackend;

        let metric = Metric::from_signature(2, 0, 0);

        let cpu_gammas = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_gammas(&metric);
        let mlx_gammas = <MlxGammaLoader as BackendGamma<MlxBackend, f32>>::get_gammas(&metric);

        let cpu_data: Vec<f32> = CpuBackend::to_vec(&cpu_gammas);
        let mlx_data: Vec<f32> = MlxBackend::to_vec(&mlx_gammas);

        assert_eq!(cpu_data.len(), mlx_data.len());
        for (cpu, mlx) in cpu_data.iter().zip(mlx_data.iter()) {
            assert!(
                (cpu - mlx).abs() < 1e-5,
                "CPU/MLX mismatch: {} vs {}",
                cpu,
                mlx
            );
        }
    }
}
