use criterion::{Criterion, criterion_group, criterion_main};
use deep_causality_multivector::{Metric, MultiVector, PGA3DMultiVector, StandardMultiVector};
use std::hint::black_box;

fn bench_geometric_product_euclidean_2d(c: &mut Criterion) {
    let m = Metric::Euclidean(2);
    let a = StandardMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    let b = StandardMultiVector::new(vec![4.0, 3.0, 2.0, 1.0], m).unwrap();

    c.bench_function("geometric_product_euclidean_2d", |bencher| {
        bencher.iter(|| black_box(a.clone()) * black_box(b.clone()))
    });
}

fn bench_geometric_product_pga_3d(c: &mut Criterion) {
    // PGA 3D is 4D algebra (16 elements)
    let p = PGA3DMultiVector::new_point(1.0, 2.0, 3.0);
    let t = PGA3DMultiVector::translator(2.0, 0.0, 0.0);

    c.bench_function("geometric_product_pga_3d", |bencher| {
        bencher.iter(|| black_box(t.clone()) * black_box(p.clone()))
    });
}

fn bench_addition_euclidean_3d(c: &mut Criterion) {
    let m = Metric::Euclidean(3);
    let data = vec![1.0; 8];
    let a = StandardMultiVector::new(data.clone(), m).unwrap();
    let b = StandardMultiVector::new(data, m).unwrap();

    c.bench_function("addition_euclidean_3d", |bencher| {
        bencher.iter(|| black_box(a.clone()) + black_box(b.clone()))
    });
}

fn bench_reversion_pga_3d(c: &mut Criterion) {
    let t = PGA3DMultiVector::translator(2.0, 0.0, 0.0);

    c.bench_function("reversion_pga_3d", |bencher| {
        bencher.iter(|| black_box(t.clone()).reversion())
    });
}

// === Dixon Algebra / Large Dimension Benchmarks ===

fn bench_dixon_cpu(c: &mut Criterion) {
    // Dixon Algebra: Cl(0, 6) -> 64 dimensions
    // Used in particle physics (GUTs).
    let m = Metric::from_signature(0, 6, 0);
    // 2^6 = 64 elements
    let data = vec![0.5; 64];

    // CPU Algebraic Implementation (CausalMultiVector)
    let a = StandardMultiVector::unchecked(data.clone(), m);
    let b = StandardMultiVector::unchecked(data, m);

    c.bench_function("geometric_product_dixon_cpu_algebraic", |bencher| {
        bencher.iter(|| black_box(a.clone()) * black_box(b.clone()))
    });
}

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
fn bench_dixon_mlx(c: &mut Criterion) {
    use deep_causality_multivector::CausalMultiField;
    use deep_causality_tensor::MlxBackend;

    // Dixon Algebra: Cl(0, 6) -> 64 dimensions
    let m = Metric::from_signature(0, 6, 0);
    let data = vec![0.5f32; 64];

    // CPU Algebraic Vector (for initial data)
    let mv = StandardMultiVector::unchecked(data, m);

    // MLX Matrix Implementation (via MultiField 1x1x1)
    // This forces the use of the Matrix Isomorphism Bridge on the GPU/NPU
    let field_a = CausalMultiField::<MlxBackend, f32>::from_coefficients(
        &vec![mv.clone()],
        [1, 1, 1],
        [1.0, 1.0, 1.0],
    );
    let field_b = field_a.clone();

    c.bench_function("geometric_product_dixon_mlx_matrix", |bencher| {
        bencher.iter(|| {
            let result = &field_a * &field_b;
            // Force MLX evaluation
            use deep_causality_tensor::TensorBackend;
            let _ = MlxBackend::to_vec(result.data());
            black_box(result)
        })
    });
}

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
fn bench_dixon_mlx(_c: &mut Criterion) {}

// Algebra Cl(0, 9) -> 512 dimensions (Experimental)
fn bench_cl09_cpu(c: &mut Criterion) {
    let m = Metric::from_signature(0, 9, 0);
    let data = vec![0.5; 512];
    let a = StandardMultiVector::unchecked(data.clone(), m);
    let b = StandardMultiVector::unchecked(data, m);

    c.bench_function("geometric_product_cl09_cpu_algebraic", |bencher| {
        bencher.iter(|| black_box(a.clone()) * black_box(b.clone()))
    });
}

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
fn bench_cl09_mlx(c: &mut Criterion) {
    use deep_causality_multivector::CausalMultiField;
    use deep_causality_tensor::MlxBackend;
    use deep_causality_tensor::TensorBackend;

    let m = Metric::from_signature(0, 9, 0);
    let data = vec![0.5f32; 512];
    let mv = StandardMultiVector::unchecked(data, m);

    let field_a = CausalMultiField::<MlxBackend, f32>::from_coefficients(
        &vec![mv.clone()],
        [1, 1, 1],
        [1.0, 1.0, 1.0],
    );
    let field_b = field_a.clone();

    c.bench_function("geometric_product_cl09_mlx_matrix", |bencher| {
        bencher.iter(|| {
            let result = &field_a * &field_b;
            // Force MLX evaluation
            let _ = MlxBackend::to_vec(result.data());
            black_box(result)
        })
    });
}

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
fn bench_cl09_mlx(_c: &mut Criterion) {}

criterion_group!(
    benches,
    bench_geometric_product_euclidean_2d,
    bench_geometric_product_pga_3d,
    bench_addition_euclidean_3d,
    bench_reversion_pga_3d,
    bench_dixon_cpu,
    bench_dixon_mlx,
    bench_cl09_cpu,
    bench_cl09_mlx
);
criterion_main!(benches);
