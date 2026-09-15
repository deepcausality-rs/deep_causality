/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Structural stress on a tetrahedral mesh
//!
//! A 3D linear-elastic stress analysis runs over a mesh of two tetrahedra sharing a face. The
//! cross-crate composition — topology × tensor × Clifford algebra — lives in one
//! `ManifoldWitness::extend` call, and the per-vertex pipeline sits inside its closure:
//!
//! ```text
//! STEP 1  strain field           blueprint: a prescribed analytic field
//! STEP 2  constitutive law       blueprint: isotropic Hooke
//! STEP 3  surface normal         blueprint: radial from the centroid
//! STEP 4  Cauchy traction        production: a tensor contraction
//! STEP 5  material-frame rotor   blueprint: a fixed 10-degree Cl(3,0) rotor
//! STEP 6  scalar of interest     production: von Mises stress
//! ```
//!
//! Four of the six steps are blueprint bodies, each carrying a "REPLACE WITH" comment naming the
//! production-grade model an engineer swaps in: a material law, boundary conditions, a normal
//! field, a failure criterion. Steps 4 and 6 are the real thing already, and the composition
//! around all six holds whichever bodies fill them.
//!
//! Two tetrahedra sharing a triangle is the smallest mesh carrying an interior face, which is
//! what gives the `d3` boundary operator something to say. Scaling up means tiling the same patch
//! over a structured or unstructured mesh, and the pipeline below carries over unchanged.

use deep_causality_algebra::Real;
use deep_causality_haft::CoMonad;
use deep_causality_linear::CsrMatrix;
use deep_causality_metric::Metric;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{Lift, lift, lower};
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};
use deep_causality_topology::{
    Manifold, ManifoldWitness, Simplex, SimplicialComplex, SimplicialManifold, Skeleton,
};

// ============================================================================
// Mesh: two tetrahedra sharing the face [1, 2, 3]
// ============================================================================
//
//   tet0 = [0, 1, 2, 3]   the lower tet, peak at v0
//   tet1 = [1, 2, 3, 4]   the upper tet, peak at v4
//
// Swapping this section for a mesh loader — Gmsh `.msh`, VTK `.vtu`, or a structured-grid
// generator — leaves everything below the boundary-operator builders as it stands.

const N_VERTICES: usize = 5;

/// Vertex coordinates. The table holds `f64`, the widest form a source file can hold, and
/// [`vertex`] lifts a row into the working type.
const VERTEX_COORDS: [[f64; 3]; N_VERTICES] = [
    [0.0, 0.0, 0.0], // v0  - lower peak
    [1.0, 0.0, 0.0], // v1  \
    [0.0, 1.0, 0.0], // v2   } shared triangle [1,2,3]
    [0.0, 0.0, 1.0], // v3  /
    [1.0, 1.0, 1.0], // v4  - upper peak
];

/// Sorted-order tetrahedra.
const TETS: [[usize; 4]; 2] = [[0, 1, 2, 3], [1, 2, 3, 4]];

/// The nine distinct edges.
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

/// The seven distinct triangles. `[1,2,3]` is the interior one, shared by both tets.
const TRIANGLES: [[usize; 3]; 7] = [
    [0, 1, 2],
    [0, 1, 3],
    [0, 2, 3],
    [1, 2, 3], // shared interior face
    [1, 2, 4],
    [1, 3, 4],
    [2, 3, 4],
];

// ============================================================================
// Material: isotropic linear-elastic steel
// ============================================================================

/// Young's modulus of steel, in Pa.
const YOUNGS_MODULUS: f64 = 200.0e9;
/// Poisson ratio of steel, dimensionless.
const POISSON_RATIO: f64 = 0.30;

/// The prescribed strain field: a uniaxial stretch along x with the Poisson contraction it
/// implies in y and z, plus a shear term that puts the off-diagonal components to work.
const STRAIN_AXIAL: f64 = 1.0e-3;
const STRAIN_LATERAL: f64 = -0.3e-3;
const STRAIN_SHEAR: f64 = 0.5e-3;

/// The material frame sits at this angle, in degrees, in the `e₁∧e₂` plane.
const MATERIAL_ANGLE_DEG: f64 = 10.0;

/// `Cl(3,0)` holds `2^3` coefficients, indexed by bitmask over `(e₁, e₂, e₃)`.
const COEFFICIENTS: usize = 8;
const SCALAR: usize = 0;
const E1: usize = 1;
const E2: usize = 2;
const E3: usize = 4;
const E12: usize = E1 | E2;

/// The working scalar. `f64` suits this problem: stress magnitudes span some ten orders of
/// magnitude and engineering accuracy lives at four to six digits, which `f64` covers. An
/// ill-conditioned solve is where `Float106` starts to pay.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let (lambda, mu) = lame(
        lift::<FloatType>(YOUNGS_MODULUS),
        lift::<FloatType>(POISSON_RATIO),
    );
    print_material(
        lift::<FloatType>(YOUNGS_MODULUS),
        lift::<FloatType>(POISSON_RATIO),
        lambda,
        mu,
    );

    let manifold = build_mesh_manifold()?;
    let (rotor, rotor_rev) = material_rotor()?;

    // One comonadic walk over the manifold. The closure crosses three crates at every vertex:
    // tensor for the constitutive law and the contraction, multivector for the material-frame
    // rotor, topology for the cursor walk that `extend` supplies.
    let result = ManifoldWitness::extend(&manifold, |w| {
        let i = w.cursor();
        if i >= N_VERTICES {
            return lift::<FloatType>(0.0);
        }

        // STEP 1: strain at this vertex.
        let strain = prescribed_strain(i);

        // STEP 2: the constitutive law carries strain to stress.
        let stress = hooke_isotropic(&strain, lambda, mu);

        // STEP 3: the local outward normal.
        let normal = vertex_normal(i);

        // STEP 4: the Cauchy traction t = σ·n.
        let traction = cauchy_traction(&stress, &normal);

        // STEP 5: the traction in the material frame. Von Mises is invariant under rotation, so
        // this value stands ready for the moment STEP 6 becomes a direction-sensitive criterion
        // such as Tsai-Hill on a composite.
        let _traction_local = rotate_into_frame(&traction, &rotor, &rotor_rev);

        // STEP 6: the scalar the engineering decision rests on.
        von_mises(&stress)
    });

    print_table(result.data().as_slice());
    print_footer();

    Ok(())
}

/// One vertex, lifted into the working type.
fn vertex(i: usize) -> [FloatType; 3] {
    let [x, y, z] = VERTEX_COORDS[i];
    [lift(x), lift(y), lift(z)]
}

// ============================================================================
// Boundary operators d1, d2, d3
// ============================================================================
// The discrete-exterior-calculus convention: for a sorted simplex [v₀, …, vₙ], the i-th face,
// which omits vᵢ, carries sign (-1)^i. The entries are incidence signs, so they stay integers and
// keep clear of the working scalar.

/// The position of a simplex in its table.
fn find_index<T: PartialEq + core::fmt::Debug>(
    haystack: &[T],
    needle: &T,
) -> Result<usize, String> {
    haystack
        .iter()
        .position(|x| x == needle)
        .ok_or_else(|| format!("the mesh tables list {needle:?}"))
}

fn build_d1() -> Result<CsrMatrix<i8>, Box<dyn std::error::Error>> {
    let mut triplets = Vec::with_capacity(2 * EDGES.len());
    for (edge_idx, &[a, b]) in EDGES.iter().enumerate() {
        // boundary [a, b] = [b] - [a]
        triplets.push((a, edge_idx, -1i8));
        triplets.push((b, edge_idx, 1i8));
    }

    Ok(CsrMatrix::from_triplets(
        N_VERTICES,
        EDGES.len(),
        &triplets,
    )?)
}

fn build_d2() -> Result<CsrMatrix<i8>, Box<dyn std::error::Error>> {
    let mut triplets = Vec::with_capacity(3 * TRIANGLES.len());
    for (tri_idx, &[a, b, c]) in TRIANGLES.iter().enumerate() {
        // boundary [a, b, c] = [b, c] - [a, c] + [a, b]
        triplets.push((find_index(&EDGES, &[b, c])?, tri_idx, 1i8));
        triplets.push((find_index(&EDGES, &[a, c])?, tri_idx, -1i8));
        triplets.push((find_index(&EDGES, &[a, b])?, tri_idx, 1i8));
    }

    Ok(CsrMatrix::from_triplets(
        EDGES.len(),
        TRIANGLES.len(),
        &triplets,
    )?)
}

fn build_d3() -> Result<CsrMatrix<i8>, Box<dyn std::error::Error>> {
    let mut triplets = Vec::with_capacity(4 * TETS.len());
    for (tet_idx, &[a, b, c, d]) in TETS.iter().enumerate() {
        // boundary [a, b, c, d] = [b, c, d] - [a, c, d] + [a, b, d] - [a, b, c]
        triplets.push((find_index(&TRIANGLES, &[b, c, d])?, tet_idx, 1i8));
        triplets.push((find_index(&TRIANGLES, &[a, c, d])?, tet_idx, -1i8));
        triplets.push((find_index(&TRIANGLES, &[a, b, d])?, tet_idx, 1i8));
        triplets.push((find_index(&TRIANGLES, &[a, b, c])?, tet_idx, -1i8));
    }

    Ok(CsrMatrix::from_triplets(
        TRIANGLES.len(),
        TETS.len(),
        &triplets,
    )?)
}

fn build_mesh_manifold()
-> Result<SimplicialManifold<FloatType, FloatType>, Box<dyn std::error::Error>> {
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
    let boundaries = vec![build_d1()?, build_d2()?, build_d3()?];
    let complex = SimplicialComplex::new(skeletons, boundaries, vec![], vec![]);

    let total = N_VERTICES + EDGES.len() + TRIANGLES.len() + TETS.len();
    let data = CausalTensor::new(vec![lift::<FloatType>(0.0); total], vec![total])?;

    Ok(Manifold::new(complex, data, 0)?)
}

/// Lame parameters `lambda, mu` from Young's modulus `E` and Poisson ratio `nu`.
fn lame(e: FloatType, nu: FloatType) -> (FloatType, FloatType) {
    let one = lift::<FloatType>(1.0);
    let two = lift::<FloatType>(2.0);
    let mu = e / (two * (one + nu));
    let lambda = e * nu / ((one + nu) * (one - two * nu));

    (lambda, mu)
}

// ============================================================================
// STEP 1: strain field  (blueprint)
// ============================================================================
// REPLACE WITH: strain taken from a displacement field through the symmetric gradient
// `ε = (∇u + ∇uᵀ)/2`. The discrete gradient is `d1` applied to vertex displacements.
//
// The body here is a closed-form analytic field, which drives the pipeline with inputs that vary
// from vertex to vertex.
fn prescribed_strain(vertex_idx: usize) -> Sym3 {
    let [x, _y, _z] = vertex(vertex_idx);
    let zero = lift::<FloatType>(0.0);

    [
        lift::<FloatType>(STRAIN_AXIAL) * x,
        lift::<FloatType>(STRAIN_LATERAL) * x,
        lift::<FloatType>(STRAIN_LATERAL) * x,
        lift::<FloatType>(STRAIN_SHEAR) * x,
        zero,
        zero,
    ]
}

// ============================================================================
// STEP 2: constitutive law  (blueprint: isotropic Hooke)
// ============================================================================
// REPLACE WITH: the material model the problem calls for.
//   - J2 plasticity with a yield surface and return mapping (metals past yield)
//   - Mooney-Rivlin / Neo-Hookean (rubber, soft tissue)
//   - Drucker-Prager (soils, concrete)
//   - Anisotropic C_ijkl (composites, single crystals) — a rank-4 CausalTensor
//   - Viscoelastic Maxwell / Kelvin-Voigt (polymers)
fn hooke_isotropic(strain: &Sym3, lambda: FloatType, mu: FloatType) -> Sym3 {
    let trace = strain[0] + strain[1] + strain[2];
    let lt = lambda * trace;
    let two = lift::<FloatType>(2.0);

    [
        lt + two * mu * strain[0],
        lt + two * mu * strain[1],
        lt + two * mu * strain[2],
        two * mu * strain[3],
        two * mu * strain[4],
        two * mu * strain[5],
    ]
}

// ============================================================================
// STEP 3: surface normal  (blueprint)
// ============================================================================
// REPLACE WITH: the outward unit normal taken from the incident boundary triangles. A corner
// vertex averages its adjacent face normals weighted by face area. The normal is a property of
// the boundary, so gate the call by boundary membership.
//
// The body here returns a radial direction from the mesh centroid, which every vertex of this
// two-tet mesh has and which keeps the rest of the pipeline driven.
fn vertex_normal(vertex_idx: usize) -> [FloatType; 3] {
    let centroid = mesh_centroid();
    let [x, y, z] = vertex(vertex_idx);
    let dx = x - centroid[0];
    let dy = y - centroid[1];
    let dz = z - centroid[2];
    let zero = lift::<FloatType>(0.0);
    let r = Real::sqrt(dx * dx + dy * dy + dz * dz);

    if r > zero {
        [dx / r, dy / r, dz / r]
    } else {
        [lift::<FloatType>(1.0), zero, zero]
    }
}

fn mesh_centroid() -> [FloatType; 3] {
    let mut c = [lift::<FloatType>(0.0); 3];
    for i in 0..N_VERTICES {
        let v = vertex(i);
        c[0] += v[0];
        c[1] += v[1];
        c[2] += v[2];
    }
    let n = N_VERTICES.lift::<FloatType>();

    [c[0] / n, c[1] / n, c[2] / n]
}

// ============================================================================
// STEP 4: Cauchy traction  (production: a tensor contraction)
// ============================================================================
// `tᵢ = σᵢⱼ nⱼ`, written as an einsum contraction, so the kernel is the same whether it is handed
// a 3×3 isotropic stress or the rank-4 response of an anisotropic stiffness.
//
// The shapes are fixed at compile time: nine components make a `[3, 3]`, three make a `[3]`, and
// contracting the second axis against the first leaves a `[3]`. The `extend` closure this runs
// inside returns the working scalar, so these calls state their invariant through `expect`.
fn cauchy_traction(stress: &Sym3, normal: &[FloatType; 3]) -> [FloatType; 3] {
    let sigma_full = vec![
        stress[0], stress[3], stress[4], stress[3], stress[1], stress[5], stress[4], stress[5],
        stress[2],
    ];
    let sigma_tensor =
        CausalTensor::new(sigma_full, vec![3, 3]).expect("nine components shape a 3x3 tensor");
    let normal_tensor =
        CausalTensor::new(normal.to_vec(), vec![3]).expect("three components shape a 3-vector");

    let ast = EinSumOp::contraction(sigma_tensor, normal_tensor, vec![1], vec![0]);
    let result = CausalTensor::ein_sum(&ast).expect("a 3x3 contracts with a 3-vector");
    let s = result.as_slice();

    [s[0], s[1], s[2]]
}

// ============================================================================
// STEP 5: material-frame rotor  (blueprint)
// ============================================================================
// REPLACE WITH: a rotor taken from the local material-orientation field.
//   - metals: a rotor built from grain-direction Euler angles
//   - composites: a rotor aligned with the fibre tangent
//   - finite-strain plasticity: the rotation from the polar decomposition F = R U
//
// The body here returns a fixed rotor in the `e₁∧e₂` plane.
fn material_rotor()
-> Result<(CausalMultiVector<FloatType>, CausalMultiVector<FloatType>), Box<dyn std::error::Error>>
{
    let metric = Metric::Euclidean(3);
    let half_theta = lift::<FloatType>(MATERIAL_ANGLE_DEG.to_radians() / 2.0);
    let c = Real::cos(half_theta);
    let s = Real::sin(half_theta);

    let mut r = vec![lift::<FloatType>(0.0); COEFFICIENTS];
    r[SCALAR] = c;
    r[E12] = -s;

    // Reversion flips the sign of the grade-2 part.
    let mut r_rev = vec![lift::<FloatType>(0.0); COEFFICIENTS];
    r_rev[SCALAR] = c;
    r_rev[E12] = s;

    Ok((
        CausalMultiVector::new(r, metric)?,
        CausalMultiVector::new(r_rev, metric)?,
    ))
}

/// `R v ~R`, which carries a 3-vector into the material frame.
///
/// The coefficient vector is `COEFFICIENTS` long by construction, so the call states that
/// invariant through `expect`.
fn rotate_into_frame(
    v: &[FloatType; 3],
    rotor: &CausalMultiVector<FloatType>,
    rotor_rev: &CausalMultiVector<FloatType>,
) -> [FloatType; 3] {
    let metric = Metric::Euclidean(3);
    let mut coeffs = vec![lift::<FloatType>(0.0); COEFFICIENTS];
    coeffs[E1] = v[0];
    coeffs[E2] = v[1];
    coeffs[E3] = v[2];

    let v_mv =
        CausalMultiVector::new(coeffs, metric).expect("2^3 coefficients shape a Cl(3,0) element");
    let rotated = rotor.geometric_product(&v_mv).geometric_product(rotor_rev);
    let d = rotated.data();

    [d[E1], d[E2], d[E3]]
}

// ============================================================================
// STEP 6: scalar of interest  (production: von Mises stress)
// ============================================================================
// REPLACE WITH: the failure criterion the engineering decision rests on.
//   - Tresca: maximum shear stress
//   - Mohr-Coulomb: cohesion and friction (soils, concrete)
//   - Maximum principal stress: brittle materials
//   - Tsai-Hill, Hashin: laminated composites
//   - Hill48: anisotropic plasticity
fn von_mises(sigma: &Sym3) -> FloatType {
    let [s11, s22, s33, s12, s13, s23] = *sigma;
    let d12 = s11 - s22;
    let d23 = s22 - s33;
    let d31 = s33 - s11;
    let dev_sq = lift::<FloatType>(0.5) * (d12 * d12 + d23 * d23 + d31 * d31)
        + lift::<FloatType>(3.0) * (s12 * s12 + s13 * s13 + s23 * s23);

    Real::sqrt(dev_sq)
}

// ============================================================================
// Data structures
// ============================================================================

/// A symmetric 3×3 tensor packed as `[xx, yy, zz, xy, xz, yz]`.
type Sym3 = [FloatType; 6];

// ============================================================================
// Printing
// ============================================================================

fn print_header() {
    println!("=== Triple HKT: 3D Stress Analysis Blueprint ===");
    println!("Precision:   {}", core::any::type_name::<FloatType>());
    println!(
        "Mesh:        2 tetrahedra sharing a face = {} vertices, {} edges, {} triangles, {} tetrahedra",
        N_VERTICES,
        EDGES.len(),
        TRIANGLES.len(),
        TETS.len()
    );
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_material(e: FloatType, nu: FloatType, lambda: FloatType, mu: FloatType) {
    println!(
        "Material:    steel  E = {:.2e} Pa, nu = {}",
        lower(e),
        lower(nu)
    );
    println!(
        "Lame:        lambda = {:.3e} Pa, mu = {:.3e} Pa\n",
        lower(lambda),
        lower(mu)
    );
}

fn print_table(mises_per_vertex: &[FloatType]) {
    println!("Vertex  Position      von Mises (Pa)");
    println!("------- ------------- ---------------");
    for (i, &mises) in mises_per_vertex.iter().enumerate().take(N_VERTICES) {
        let [x, y, z] = vertex(i);
        println!(
            "v{:<2}     ({:.0},{:.0},{:.0})       {:.3e}",
            i,
            lower(x),
            lower(y),
            lower(z),
            lower(mises)
        );
    }
}

fn print_footer() {
    println!("\nOne `extend` call. Three crates participated:");
    println!("  topology    supplied the 3D mesh and the per-vertex walk");
    println!("  tensor      ran the constitutive law and the Cauchy contraction");
    println!("  multivector applied the material-frame rotation in Cl(3,0)");
}
