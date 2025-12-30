use criterion::{Criterion, criterion_group, criterion_main};
use deep_causality_metric::Metric;
use deep_causality_multivector::{MultiField, StandardMultiVector};
use std::hint::black_box;

fn bench_geometric_product_size(c: &mut Criterion, size: usize) {
    let metric = Metric::from_signature(3, 0, 0);
    // Create two fields
    let shape = [size, size, size];
    let dx = [1.0, 1.0, 1.0];
    let num_cells = size * size * size;

    // StandardMultiVector uses the default float type
    let mv = StandardMultiVector::unchecked(vec![0.5; 8], metric);

    let field_a = MultiField::from_coefficients(&vec![mv.clone(); num_cells], shape, dx);
    let field_b = field_a.clone();

    let backend_suffix = if cfg!(feature = "mlx") { "mlx" } else { "cpu" };
    let bench_name = format!("geometric_product_{}_3d_{}^3", backend_suffix, size);

    c.bench_function(&bench_name, |b| b.iter(|| black_box(&field_a * &field_b)));
}

fn bench_geometric_product(c: &mut Criterion) {
    // 64 cells (Current)
    bench_geometric_product_size(c, 4);
    // 512 cells
    bench_geometric_product_size(c, 8);
    // 4096 cells
    bench_geometric_product_size(c, 16);
}

fn bench_gradient(c: &mut Criterion) {
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [4, 4, 4];
    let dx = [1.0, 1.0, 1.0];

    let num_cells = 64;
    let mv = StandardMultiVector::unchecked(vec![0.5; 8], metric);

    let field = MultiField::from_coefficients(&vec![mv; num_cells], shape, dx);

    let bench_name = if cfg!(feature = "mlx") {
        "multifield_gradient_mlx_3d"
    } else {
        "multifield_gradient_cpu_3d"
    };

    c.bench_function(bench_name, |b| b.iter(|| field.gradient()));
}

fn bench_conversion(c: &mut Criterion) {
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [4, 4, 4];
    let dx = [1.0, 1.0, 1.0];

    let num_cells = 64;
    let mv = StandardMultiVector::unchecked(vec![0.5; 8], metric);

    let data_vec = vec![mv; num_cells];

    let bench_name_from = if cfg!(feature = "mlx") {
        "multifield_from_coefficients_mlx"
    } else {
        "multifield_from_coefficients_cpu"
    };

    c.bench_function(bench_name_from, |b| {
        b.iter(|| MultiField::from_coefficients(black_box(&data_vec), shape, dx))
    });

    let field = MultiField::from_coefficients(&data_vec, shape, dx);

    let bench_name_to = if cfg!(feature = "mlx") {
        "multifield_to_coefficients_mlx"
    } else {
        "multifield_to_coefficients_cpu"
    };

    c.bench_function(bench_name_to, |b| b.iter(|| field.to_coefficients()));
}

criterion_group!(
    benches,
    bench_geometric_product,
    bench_gradient,
    bench_conversion
);
criterion_main!(benches);
