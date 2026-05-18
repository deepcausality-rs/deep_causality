/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Triple HKT Composition: 3D Stress Analysis Blueprint
//!
//! A 3D linear-elastic stress analysis runs on a single hex split into six
//! tetrahedra. The cross-crate composition (topology x tensor x Clifford
//! algebra) lives in one `ManifoldWitness::extend` call. Every domain-specific
//! step inside the closure is a clearly-marked placeholder that an engineer
//! swaps for their real material model, boundary conditions, normal field, or
//! failure criterion.
//!
//! ## Pipeline
//!
//! per vertex:
//!   STEP 1  strain field           (placeholder: prescribed)
//!   STEP 2  constitutive law       (placeholder: isotropic Hooke)
//!   STEP 3  surface normal         (placeholder: radial from centroid)
//!   STEP 4  Cauchy traction        (real: tensor contraction)
//!   STEP 5  material-frame rotor   (placeholder: 10-degree Cl(3,0) rotor)
//!   STEP 6  scalar of interest     (real: von Mises stress)
//!
//! Each step is documented inside the closure with a "REPLACE WITH" comment
//! that names the production-grade alternative.

use deep_causality_haft::CoMonad;
use deep_causality_metric::Metric;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};
use deep_causality_topology::{Manifold, ManifoldWitness, Simplex, SimplicialComplex, Skeleton};

/// `f64` is the right precision here: stress magnitudes span ~10 orders of
/// magnitude and engineering accuracy lives at four to six digits. Float106
/// yields no observable gain unless you have an ill-conditioned solve.
pub type FloatType = f64;

// ============================================================================
// MESH: two tetrahedra sharing a face  (smallest non-trivial 3D manifold)
// ============================================================================
//
// Two tets that share triangle [1, 2, 3]:
//   tet0 = [0, 1, 2, 3]   "lower" tet, peak at v0
//   tet1 = [1, 2, 3, 4]   "upper" tet, peak at v4
//
// This is the minimum mesh that has an interior triangle, which is what
// makes the d3 boundary operator non-trivial. A real analysis would scale
// up by tiling the same patch over a structured or unstructured mesh; the
// pipeline below stays identical.
//
// To go larger: replace this MESH section with a mesh loader (Gmsh `.msh`,
// VTK `.vtu`, or a structured-grid generator). Everything below the
// boundary-operator builders stays the same.

const N_VERTICES: usize = 5;

/// Vertex coordinates.
const VERTICES: [[FloatType; 3]; 5] = [
    [0.0, 0.0, 0.0], // v0  - lower peak
    [1.0, 0.0, 0.0], // v1  \
    [0.0, 1.0, 0.0], // v2   } shared triangle [1,2,3]
    [0.0, 0.0, 1.0], // v3  /
    [1.0, 1.0, 1.0], // v4  - upper peak
];

/// Sorted-order tetrahedra.
const TETS: [[usize; 4]; 2] = [[0, 1, 2, 3], [1, 2, 3, 4]];

/// 9 distinct edges.
const EDGES: [[usize; 2]; 9] = [
    [0, 1],
    [0, 2],
    [0, 3],
    [1, 2],
    [1, 3],
    [1, 4],
    [2, 3],
    [2, 4],
    [3, 4],
];

/// 7 distinct triangles. The interior one is [1,2,3] (shared by both tets).
const TRIANGLES: [[usize; 3]; 7] = [
    [0, 1, 2],
    [0, 1, 3],
    [0, 2, 3],
    [1, 2, 3], // shared interior face
    [1, 2, 4],
    [1, 3, 4],
    [2, 3, 4],
];

fn main() {
    println!("=== Triple HKT: 3D Stress Analysis Blueprint ===");
    println!("Precision:   {}", core::any::type_name::<FloatType>());
    println!(
        "Mesh:        2 tetrahedra sharing a face = {} vertices, {} edges, {} triangles, {} tetrahedra",
        N_VERTICES,
        EDGES.len(),
        TRIANGLES.len(),
        TETS.len()
    );

    let (lambda, mu) = lame(YOUNGS_MODULUS_STEEL, POISSON_STEEL);
    println!(
        "Material:    steel  E = {:.2e} Pa, nu = {}",
        YOUNGS_MODULUS_STEEL, POISSON_STEEL
    );
    println!(
        "Lame:        lambda = {:.3e} Pa, mu = {:.3e} Pa\n",
        lambda, mu
    );

    let manifold = build_cube_manifold();
    let (rotor, rotor_rev) = material_rotor();

    // One comonadic walk over the manifold. The closure crosses three crates
    // per vertex: tensor (constitutive law + contraction), multivector
    // (material-frame rotor), topology (cursor walk supplied by `extend`).
    let result = ManifoldWitness::extend(&manifold, |w| {
        let i = w.cursor();
        if i >= N_VERTICES {
            return 0.0;
        }

        // STEP 1: strain at this vertex
        let strain = prescribed_strain(i);

        // STEP 2: constitutive law -> stress
        let stress = hooke_isotropic(&strain, lambda, mu);

        // STEP 3: local outward normal
        let normal = vertex_normal(i);

        // STEP 4: Cauchy traction t = sigma . n
        let traction = cauchy_traction(&stress, &normal);

        // STEP 5: rotate traction into the material frame. Unused
        // because von Mises is rotation-invariant; the rotation matters
        // when the engineer swaps STEP 6 for a direction-sensitive
        // criterion (Tsai-Hill on composites, for example).
        let _traction_local = rotate_into_frame(&traction, &rotor, &rotor_rev);

        // STEP 6: scalar of interest
        von_mises(&stress)
    });

    let out = result.data().as_slice();
    println!("Vertex  Position      von Mises (Pa)");
    println!("------- ------------- ---------------");
    for i in 0..N_VERTICES {
        let [x, y, z] = VERTICES[i];
        println!(
            "v{:<2}     ({:.0},{:.0},{:.0})       {:.3e}",
            i, x, y, z, out[i]
        );
    }

    println!("\nOne `extend` call. Three crates participated:");
    println!("  topology    supplied the 3D mesh and the per-vertex walk");
    println!("  tensor      ran the constitutive law and the Cauchy contraction");
    println!("  multivector applied the material-frame rotation in Cl(3,0)");
}

// ============================================================================
// BOUNDARY OPERATORS  d1, d2, d3
// ============================================================================
// Standard discrete-exterior-calculus convention: for a sorted simplex
// [v_0, ..., v_n], the i-th face (omit v_i) contributes sign (-1)^i.

fn find_index<T: PartialEq>(haystack: &[T], needle: &T) -> usize {
    haystack
        .iter()
        .position(|x| x == needle)
        .expect("simplex not found")
}

fn build_d1() -> CsrMatrix<i8> {
    let mut triplets = Vec::with_capacity(2 * EDGES.len());
    for (edge_idx, &[a, b]) in EDGES.iter().enumerate() {
        // boundary [a, b] = [b] - [a]
        triplets.push((a, edge_idx, -1i8));
        triplets.push((b, edge_idx, 1i8));
    }
    CsrMatrix::from_triplets(N_VERTICES, EDGES.len(), &triplets).unwrap()
}

fn build_d2() -> CsrMatrix<i8> {
    let mut triplets = Vec::with_capacity(3 * TRIANGLES.len());
    for (tri_idx, &[a, b, c]) in TRIANGLES.iter().enumerate() {
        // boundary [a, b, c] = [b, c] - [a, c] + [a, b]
        let bc = find_index(&EDGES, &[b, c]);
        let ac = find_index(&EDGES, &[a, c]);
        let ab = find_index(&EDGES, &[a, b]);
        triplets.push((bc, tri_idx, 1i8));
        triplets.push((ac, tri_idx, -1i8));
        triplets.push((ab, tri_idx, 1i8));
    }
    CsrMatrix::from_triplets(EDGES.len(), TRIANGLES.len(), &triplets).unwrap()
}

fn build_d3() -> CsrMatrix<i8> {
    let mut triplets = Vec::with_capacity(4 * TETS.len());
    for (tet_idx, &[a, b, c, d]) in TETS.iter().enumerate() {
        // boundary [a, b, c, d] = [b, c, d] - [a, c, d] + [a, b, d] - [a, b, c]
        let bcd = find_index(&TRIANGLES, &[b, c, d]);
        let acd = find_index(&TRIANGLES, &[a, c, d]);
        let abd = find_index(&TRIANGLES, &[a, b, d]);
        let abc = find_index(&TRIANGLES, &[a, b, c]);
        triplets.push((bcd, tet_idx, 1i8));
        triplets.push((acd, tet_idx, -1i8));
        triplets.push((abd, tet_idx, 1i8));
        triplets.push((abc, tet_idx, -1i8));
    }
    CsrMatrix::from_triplets(TRIANGLES.len(), TETS.len(), &triplets).unwrap()
}

fn build_cube_manifold() -> Manifold<f64, FloatType> {
    let vertices: Vec<Simplex> = (0..N_VERTICES).map(|i| Simplex::new(vec![i])).collect();
    let edges: Vec<Simplex> = EDGES.iter().map(|e| Simplex::new(e.to_vec())).collect();
    let triangles: Vec<Simplex> = TRIANGLES.iter().map(|t| Simplex::new(t.to_vec())).collect();
    let tets: Vec<Simplex> = TETS.iter().map(|t| Simplex::new(t.to_vec())).collect();

    let skeletons = vec![
        Skeleton::new(0, vertices),
        Skeleton::new(1, edges),
        Skeleton::new(2, triangles),
        Skeleton::new(3, tets),
    ];
    let boundaries = vec![build_d1(), build_d2(), build_d3()];
    let complex = SimplicialComplex::new(skeletons, boundaries, vec![], vec![]);

    let total = N_VERTICES + EDGES.len() + TRIANGLES.len() + TETS.len();
    let data = CausalTensor::new(vec![0.0f64; total], vec![total]).unwrap();
    Manifold::new(complex, data, 0).expect("manifold construction")
}

// ============================================================================
// MATERIAL: isotropic linear-elastic steel
// ============================================================================

const YOUNGS_MODULUS_STEEL: FloatType = 200.0e9; // Pa
const POISSON_STEEL: FloatType = 0.30; // dimensionless

/// Lame parameters `lambda, mu` from Young's modulus `E` and Poisson ratio `nu`.
fn lame(e: FloatType, nu: FloatType) -> (FloatType, FloatType) {
    let mu = e / (2.0 * (1.0 + nu));
    let lambda = e * nu / ((1.0 + nu) * (1.0 - 2.0 * nu));
    (lambda, mu)
}

/// Symmetric 3x3 tensor packed as `[xx, yy, zz, xy, xz, yz]`.
type Sym3 = [FloatType; 6];

// ============================================================================
// STEP 1: STRAIN FIELD  (PLACEHOLDER)
// ============================================================================
// REPLACE WITH: strain derived from a displacement field via the symmetric
// gradient `epsilon = (grad u + grad u^T) / 2`. The discrete gradient is
// `d1` applied to vertex displacements.
//
// The current body is a closed-form analytic field used only to drive the
// pipeline with non-trivial inputs.
fn prescribed_strain(vertex_idx: usize) -> Sym3 {
    let [x, _y, _z] = VERTICES[vertex_idx];
    // Uniaxial stretch in x with the corresponding Poisson contraction in
    // y and z, plus a shear term to exercise the off-diagonal components.
    [1.0e-3 * x, -0.3e-3 * x, -0.3e-3 * x, 0.5e-3 * x, 0.0, 0.0]
}

// ============================================================================
// STEP 2: CONSTITUTIVE LAW  (PLACEHOLDER: ISOTROPIC HOOKE)
// ============================================================================
// REPLACE WITH: the material model your problem requires.
//   - J2 plasticity with yield surface and return mapping (metals past yield)
//   - Mooney-Rivlin / Neo-Hookean (rubber, soft tissue)
//   - Drucker-Prager (soils, concrete)
//   - Anisotropic C_ijkl (composites, single crystals)  - a rank-4 CausalTensor
//   - Viscoelastic Maxwell / Kelvin-Voigt (polymers)
fn hooke_isotropic(strain: &Sym3, lambda: FloatType, mu: FloatType) -> Sym3 {
    let trace = strain[0] + strain[1] + strain[2];
    let lt = lambda * trace;
    [
        lt + 2.0 * mu * strain[0],
        lt + 2.0 * mu * strain[1],
        lt + 2.0 * mu * strain[2],
        2.0 * mu * strain[3],
        2.0 * mu * strain[4],
        2.0 * mu * strain[5],
    ]
}

// ============================================================================
// STEP 3: SURFACE NORMAL  (PLACEHOLDER)
// ============================================================================
// REPLACE WITH: outward unit normal computed from the incident boundary
// triangles. For corner vertices, average the adjacent face normals weighted
// by face area. For interior vertices, this concept is undefined; gate by
// boundary membership before calling.
//
// The current body returns a radial direction from the cube centroid, which
// is well-defined for all 8 cube vertices and good enough to keep the rest
// of the pipeline driven.
fn vertex_normal(vertex_idx: usize) -> [FloatType; 3] {
    // Radial direction from the mesh centroid. Well-defined for every
    // vertex of the two-tet mesh; serves as a stand-in for the real
    // boundary-normal calculation an engineer plugs in.
    let centroid = mesh_centroid();
    let [x, y, z] = VERTICES[vertex_idx];
    let dx = x - centroid[0];
    let dy = y - centroid[1];
    let dz = z - centroid[2];
    let r = (dx * dx + dy * dy + dz * dz).sqrt();
    if r > 0.0 {
        [dx / r, dy / r, dz / r]
    } else {
        [1.0, 0.0, 0.0]
    }
}

fn mesh_centroid() -> [FloatType; 3] {
    let mut c = [0.0; 3];
    for v in VERTICES.iter() {
        c[0] += v[0];
        c[1] += v[1];
        c[2] += v[2];
    }
    let n = N_VERTICES as FloatType;
    [c[0] / n, c[1] / n, c[2] / n]
}

// ============================================================================
// STEP 4: CAUCHY TRACTION  (REAL: tensor contraction)
// ============================================================================
// `t_i = sigma_ij * n_j`. Implemented as an einsum contraction so the kernel
// is identical whether you pass a 3x3 isotropic stress or a 6x6x6x6 rank-4
// anisotropic stiffness response.
fn cauchy_traction(stress: &Sym3, normal: &[FloatType; 3]) -> [FloatType; 3] {
    let sigma_full = vec![
        stress[0], stress[3], stress[4], stress[3], stress[1], stress[5], stress[4], stress[5],
        stress[2],
    ];
    let sigma_tensor = CausalTensor::new(sigma_full, vec![3, 3]).unwrap();
    let normal_tensor = CausalTensor::new(normal.to_vec(), vec![3]).unwrap();
    let ast = EinSumOp::contraction(sigma_tensor, normal_tensor, vec![1], vec![0]);
    let result = CausalTensor::ein_sum(&ast).unwrap();
    let s = result.as_slice();
    [s[0], s[1], s[2]]
}

// ============================================================================
// STEP 5: MATERIAL-FRAME ROTOR  (PLACEHOLDER)
// ============================================================================
// REPLACE WITH: rotor from the local material orientation field.
//   - metals: rotor built from grain-direction Euler angles
//   - composites: rotor that aligns with the fiber tangent
//   - finite-strain plasticity: rotation pulled from polar decomposition F = R U
//
// The current body returns a fixed 10-degree rotor in the e1^e2 plane.
fn material_rotor() -> (CausalMultiVector<FloatType>, CausalMultiVector<FloatType>) {
    let metric = Metric::Euclidean(3);
    let theta: FloatType = 10.0_f64.to_radians();
    let c = (theta / 2.0).cos();
    let s = (theta / 2.0).sin();
    // Cl(3,0) bit-string basis: indices are bitmasks over (e1, e2, e3).
    //   0 = 1, 1 = e1, 2 = e2, 3 = e1^e2, 4 = e3, 5 = e1^e3, 6 = e2^e3, 7 = e1^e2^e3
    let mut r = vec![0.0; 8];
    r[0] = c;
    r[3] = -s; // -sin(theta/2) * e1^e2
    let rotor = CausalMultiVector::new(r, metric).unwrap();

    let mut r_rev = vec![0.0; 8];
    r_rev[0] = c;
    r_rev[3] = s;
    let rotor_rev = CausalMultiVector::new(r_rev, metric).unwrap();

    (rotor, rotor_rev)
}

/// Apply `R v R~` to rotate a 3-vector into the material frame.
fn rotate_into_frame(
    v: &[FloatType; 3],
    rotor: &CausalMultiVector<FloatType>,
    rotor_rev: &CausalMultiVector<FloatType>,
) -> [FloatType; 3] {
    let metric = Metric::Euclidean(3);
    // Lift the vector as pure grade-1 multivector. Indices 1, 2, 4 are e1, e2, e3.
    let coeffs = vec![0.0, v[0], v[1], 0.0, v[2], 0.0, 0.0, 0.0];
    let v_mv = CausalMultiVector::new(coeffs, metric).unwrap();
    let rotated = rotor.geometric_product(&v_mv).geometric_product(rotor_rev);
    let d = rotated.data();
    [d[1], d[2], d[4]]
}

// ============================================================================
// STEP 6: SCALAR OF INTEREST  (REAL: von Mises stress)
// ============================================================================
// REPLACE WITH: the failure criterion your engineering decision requires.
//   - Tresca: max shear stress
//   - Mohr-Coulomb: cohesion-friction (soils, concrete)
//   - Maximum principal stress: brittle materials
//   - Tsai-Hill, Hashin: laminated composites
//   - Hill48: anisotropic plasticity
fn von_mises(sigma: &Sym3) -> FloatType {
    let s11 = sigma[0];
    let s22 = sigma[1];
    let s33 = sigma[2];
    let s12 = sigma[3];
    let s13 = sigma[4];
    let s23 = sigma[5];
    let dev_sq = 0.5 * ((s11 - s22).powi(2) + (s22 - s33).powi(2) + (s33 - s11).powi(2))
        + 3.0 * (s12 * s12 + s13 * s13 + s23 * s23);
    dev_sq.sqrt()
}
