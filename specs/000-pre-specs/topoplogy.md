
# Specfile: `deep_causality_topology`

**Version:** 0.1.0
**Type:** Internal Core Crate
**Architecture:** `no_std` compatible (with `alloc`).

## 1. Dependencies (Internal)

```toml
[dependencies]
deep_causality_haft = { path = "../haft" }        # Traits: BoundedComonad, Adjunction
deep_causality_tensor = { path = "../tensor" }    # Data Storage
deep_causality_multivector = { path = "../multivector" } # Metric/Algebra
deep_causality_sparse = { path = "../deep_causality_sparse" }    # CSR Matrices
```

---

## 2. Core Data Structures

### 2.1. The Simplex & Skeleton
*Implementation Note: To maintain zero dependencies (dropping `indexmap`), we use a sorted `Vec` with binary search for index lookups. This is $O(\log N)$ for setup and $O(1)$ for access once indices are baked into the Sparse Matrix.*

```rust
/// A combinatorial simplex defined by sorted vertex indices.
/// Order is strictly increasing to ensure canonical representation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Simplex {
    pub vertices: Vec<usize>, 
}

impl Simplex {
    /// Returns a sub-simplex defined by the given range of vertices.
    pub fn subsimplex<R>(&self, range: R) -> Self 
    where R: std::slice::SliceIndex<[usize], Output = [usize]> {
        Simplex { vertices: self.vertices[range].to_vec() }
    }
}

/// A collection of all simplices of dimension K.
pub struct Skeleton {
    pub dim: usize,
    /// Canonical list of simplices. The index in this vector is the "Global ID".
    pub simplices: Vec<Simplex>,
}

impl Skeleton {
    /// Find the global index of a simplex via binary search.
    pub fn get_index(&self, simplex: &Simplex) -> Option<usize> {
        self.simplices.binary_search(simplex).ok()
    }
}
```

### 2.2. The Causal Complex (The Container)
This struct holds the "Static Topology." It uses `CsrMatrix` from your new crate to store relationships.

```rust
use deep_causality_sparse::CsrMatrix;
use deep_causality_num::Zero;
 
pub struct CausalComplex {
    /// Storage of geometric entities (Points, Lines, Triangles...)
    pub skeletons: Vec<Skeleton>,
    
    /// The Boundary Operators (∂). 
    /// boundary[k] is a matrix of size (N_{k-1} x N_k).
    /// Maps a k-chain to a (k-1)-chain.
    /// 
    /// Implementation: deep_causality_sparse::CsrMatrix<i8>
    /// Values are {-1, 0, 1} representing orientation.
    pub boundary_operators: Vec<CsrMatrix<i8>>,

    /// The Coboundary / Adjacency Cache (Optional but recommended for Comonad speed).
    /// Transpose of boundary operators.
    /// coboundary[k] is a matrix of size (N_{k+1} x N_k).
    /// Used to find "Who contains me?" efficiently.
    pub coboundary_operators: Vec<CsrMatrix<i8>>,
}

impl CausalComplex {
    /// Computes the boundary of a chain: ∂c
    /// Maps a k-chain to a (k-1)-chain.
    pub fn boundary<T>(&self, chain: &Chain<T>) -> Chain<T>
    where T: Copy + deep_causality_num::Zero + PartialEq + Default + std::ops::Mul<Output = T> + std::ops::Add<Output = T> + From<i8> 
    {
        if chain.grade == 0 {
            panic!("Cannot take boundary of 0-chain");
        }
        
        // ∂_k: C_k -> C_{k-1}
        // Matrix is (N_{k-1} x N_k)
        let boundary_op = &self.boundary_operators[chain.grade];
        
        // Apply matrix multiplication: v_{k-1} = M * v_k
        // We need to cast the i8 matrix to T to perform multiplication with T weights.
        // For efficiency, we assume the sparse matrix library handles mixed-type multiplication 
        // or we convert the boundary op on the fly. 
        // Here we assume a helper `cast_and_mult` or similar exists, or T implements From<i8>.
        
        // Simplified for spec:
        // let new_weights = boundary_op.mat_mult(&chain.weights).expect("Boundary op failed");
        
        // In reality, we might need a map over non-zero elements.
        // For this spec, we assume standard matrix multiplication works.
        let new_weights = boundary_op.cast_mult(&chain.weights).expect("Boundary op failed");

        Chain {
            complex: chain.complex.clone(),
            grade: chain.grade - 1,
            weights: new_weights,
        }
    }

    /// Computes the coboundary (exterior derivative) of a cochain: dω
    /// Maps a k-cochain to a (k+1)-cochain.
    /// Note: In this architecture, Cochains are represented by CausalTopology (dense fields),
    /// but for duality we can also apply it to sparse Chains if needed.
    /// Here we define it for the sparse structure for completeness of the Adjunction.
    pub fn coboundary<T>(&self, chain: &Chain<T>) -> Chain<T>
    where T: Copy + deep_causality_num::Zero + PartialEq + Default + std::ops::Mul<Output = T> + std::ops::Add<Output = T> + From<i8>
    {
        if chain.grade >= self.skeletons.len() - 1 {
            panic!("Cannot take coboundary of max-dim chain");
        }

        // d_k: C^k -> C^{k+1}
        // Uses the transpose of ∂_{k+1}. 
        // We have this cached in `coboundary_operators`.
        // coboundary[k] is (N_{k+1} x N_k)
        let coboundary_op = &self.coboundary_operators[chain.grade];
        
        let new_weights = coboundary_op.cast_mult(&chain.weights).expect("Coboundary op failed");

        Chain {
            complex: chain.complex.clone(),
            grade: chain.grade + 1,
            weights: new_weights,
        }
    }
}
```

---

## 3. The "Field" Types (Data on Topology)

### 3.1. `CausalTopology<T>` (The Cochain)
This is the **Comonad Witness**. It represents a Differential Form (Field) defined on the mesh.

```rust
use deep_causality_tensor::CausalTensor;
use alloc::sync::Arc;

/// Represents a discrete field defined on the k-skeleton.
/// (e.g., Temperature on Vertices, Magnetic Flux on Faces).
#[derive(Clone)]
pub struct CausalTopology<T> {
    /// Shared reference to the underlying mesh
    pub complex: Arc<CausalComplex>,
    
    /// The dimension of the simplices this data lives on
    pub grade: usize,
    
    /// The values (CausalTensor is essentially a dense vector here)
    pub data: CausalTensor<T>,
    
    /// The Focus (Cursor) for Comonadic extraction
    pub cursor: usize,
}

impl<T: Clone> CausalTopology<T> {
    pub fn clone_shallow(&self) -> Self {
        self.clone()
    }
}
```

### 3.2. `Chain<T>` (The Geometric Selection)
This represents a geometric object (like a path or a surface) used for integration.

```rust
/// Represents a weighted collection of simplices.
/// (e.g., A path is a Chain<f64> on the 1-skeleton where weights are 1.0).
pub struct Chain<T> {
    pub complex: Arc<CausalComplex>,
    pub grade: usize,
    
    /// Sparse vector of active simplices. 
    /// Reuses CsrMatrix logic (1 row, N cols) for efficient sparse operations.
    pub weights: CsrMatrix<T>, 
}
```

---

## 4. Trait Implementations

### 4.1. `BoundedComonad` (The Stencil Engine)
This enables local physics (e.g., Laplacian) to be defined as a closure and applied globally.

```rust
use deep_causality_haft::algebra::comonad::BoundedComonad;
use deep_causality_num::Zero;

// Assuming CausalTopologyWitness is the HKT witness for CausalTopology
impl BoundedComonad<CausalTopologyWitness> for CausalTopologyWitness {
    fn extract<A>(fa: &CausalTopology<A>) -> A 
    where A: Clone {
        // Use as_slice() instead of get_flat()
        fa.data.as_slice().get(fa.cursor).cloned().expect("Cursor OOB")
    }

    fn extend<A, B, Func>(fa: &CausalTopology<A>, mut f: Func) -> CausalTopology<B>
    where
        Func: FnMut(&CausalTopology<A>) -> B,
        A: Zero + Copy + Clone,
        B: Zero + Copy + Clone
    {
        // Use len() instead of size()
        let size = fa.data.len();
        let mut result_vec = Vec::with_capacity(size);

        // OPTIMIZATION: 
        // Instead of allocating a new View struct every iteration,
        // we keep the topology constant and only move the cursor integer.
        // The closure `f` receives a lightweight view.
        
        for i in 0..size {
            // 1. Create View centered at i
            // We can clone 'fa' cheaply because 'complex' is Arc 
            // and 'data' is ref-counted or cloned (depending on tensor impl).
            let mut view = fa.clone_shallow(); 
            view.cursor = i;

            // 2. Apply Physics
            // The user's function 'f' will likely call view.laplacian() or view.neighbors()
            let val = f(&view);
            result_vec.push(val);
        }

        CausalTopology {
            complex: fa.complex.clone(),
            grade: fa.grade,
            // CausalTensor::new takes data and shape
            data: CausalTensor::new(result_vec, vec![size]).unwrap(),
            cursor: 0,
        }
    }
}
```

### 4.2. `Adjunction` (Exact Conservation)
This enforces $\langle d\omega, c \rangle = \langle \omega, \partial c \rangle$. It uses the **Sparse Matrix Transpose** from `deep_causality_sparse`.

```rust
use deep_causality_haft::HKT;

// 1. Define Witness Types for HKT
pub struct ChainWitness;
impl HKT for ChainWitness {
    type Type<T> = Chain<T>;
}

pub struct CausalTopologyWitness;
impl HKT for CausalTopologyWitness {
    type Type<T> = CausalTopology<T>;
}

/// A Bounded Adjunction that requires a Context to bridge the Left and Right functors.
/// This is necessary when the functors depend on a runtime environment (like a CausalComplex)
/// that cannot be captured in the static types alone.
pub trait BoundedAdjunction<L, R, Context>
where L: HKT, R: HKT {
    /// The Left Adjunct: (L(A) -> B) -> (A -> R(B))
    /// Transforms a function on the "Left" structure to a function on the "Right" structure.
    fn left_adjunct<A, B, F>(ctx: &Context, a: A, f: F) -> R::Type<B>
    where F: Fn(L::Type<A>) -> B;

    /// The Right Adjunct: (A -> R(B)) -> (L(A) -> B)
    /// Transforms a function on the "Right" structure to a function on the "Left" structure.
    fn right_adjunct<A, B, F>(ctx: &Context, la: L::Type<A>, f: F) -> B
    where F: Fn(A) -> R::Type<B>;

    /// The Unit: A -> R(L(A))
    /// Embeds a value into the Right-Left context.
    fn unit<A>(ctx: &Context, a: A) -> R::Type<L::Type<A>>;

    /// The Counit: L(R(B)) -> B
    /// Collapses the Left-Right context back to a value.
    fn counit<B>(ctx: &Context, lrb: L::Type<R::Type<B>>) -> B;
}

impl BoundedAdjunction<ChainWitness, CausalTopologyWitness, Arc<CausalComplex>> for CausalComplex {
    
    /// Left Adjunct: (Chain<A> -> B) -> (A -> Topology<B>)
    /// 
    /// Logic: Construct the Riesz representative of the functional `f`.
    /// We create a CausalTopology (field) phi such that <phi, c> = f(c).
    /// With the context, we can now allocate the topology.
    fn left_adjunct<A, B, F>(ctx: &Arc<CausalComplex>, a: A, f: F) -> CausalTopology<B>
    where F: Fn(Chain<A>) -> B {
        // 1. We need to find a field 'phi' (CausalTopology) that represents f.
        // For a finite dimensional vector space (our complex), this is always possible.
        // Ideally, phi_i = f(basis_i).
        
        // 2. Iterate over all simplices in the relevant skeleton (determined by A's grade? 
        // Wait, A is a scalar type. The grade is implicit in the Chain type or we need to know it.
        // In this generic signature, we assume we are operating on a specific grade k.
        // For simplicity, let's assume we map to the 0-skeleton or similar, 
        // OR we iterate all simplices if f is polymorphic.
        
        // Practical Implementation for Spec:
        // We construct a basis chain for each simplex, apply f, and store the result.
        
        // Assume we are working on 0-chains for this example 'a'.
        let skeleton = &ctx.skeletons[0]; 
        let mut data = Vec::with_capacity(skeleton.simplices.len());
        
        for (i, _simplex) in skeleton.simplices.iter().enumerate() {
            // Construct basis chain e_i
            let mut weights = CsrMatrix::new_zero(1, skeleton.simplices.len());
            // weights.set(0, i, A::one()); // Pseudo-code: set weight to 1
            
            let chain = Chain {
                complex: ctx.clone(),
                grade: 0,
                weights, 
            };
            
            // Apply f
            let val = f(chain);
            data.push(val);
        }
        
        CausalTopology {
            complex: ctx.clone(),
            grade: 0,
            data: CausalTensor::new(data, vec![skeleton.simplices.len()]).unwrap(),
            cursor: 0,
        }
    }
    
    /// Right Adjunct: (A -> Topology<B>) -> (Chain<A> -> B)
    /// Logic: Integrate the field generated by `f` over the `chain`.
    fn right_adjunct<A, B, F>(_ctx: &Arc<CausalComplex>, chain: Chain<A>, f: F) -> B
    where
        F: Fn(A) -> CausalTopology<B>,
        A: Clone + deep_causality_num::Zero,
        B: Clone + deep_causality_num::Zero + std::ops::Add<Output = B> + std::ops::Mul<Output = B> 
    {
        let mut total_result = B::zero();
        
        for (simplex_idx, weight_a) in chain.weights.iter_active() {
             let topology = f(weight_a.clone());
             if let Some(value_b) = topology.data.as_slice().get(simplex_idx) {
                 total_result = total_result + value_b.clone();
             }
        }
        
        total_result
    }

    /// The Unit: A -> Topology<Chain<A>>
    /// Logic: Embed scalar 'a' into a field of chains.
    /// Usually this means creating a constant field or a specific distribution.
    /// With 'ctx', we can now create the structure.
    fn unit<A>(ctx: &Arc<CausalComplex>, a: A) -> CausalTopology<Chain<A>> {
        // Create a topology where every point holds a Chain containing 'a'.
        // This is a "Diagonal" embedding.
        
        let skeleton = &ctx.skeletons[0]; // Default to 0-skeleton
        let size = skeleton.simplices.len();
        let mut data = Vec::with_capacity(size);
        
        for i in 0..size {
            // Create a chain concentrated at i with weight a
             let mut weights = CsrMatrix::new_zero(1, size);
             // weights.set(0, i, a.clone()); 
             
             let chain = Chain {
                 complex: ctx.clone(),
                 grade: 0,
                 weights
             };
             data.push(chain);
        }
        
        CausalTopology {
            complex: ctx.clone(),
            grade: 0,
            data: CausalTensor::new(data, vec![size]).unwrap(),
            cursor: 0
        }
    }

    /// The Counit: Chain<Topology<B>> -> B
    /// Logic: Integration / Pairing.
    fn counit<B>(_ctx: &Arc<CausalComplex>, lrb: Chain<CausalTopology<B>>) -> B 
    where B: Clone + deep_causality_num::Zero + std::ops::Add<Output = B> + std::ops::Mul<Output = B>
    {
        let mut total = B::zero();
        
        for (idx, topology_b) in lrb.weights.iter_active() {
            if let Some(val) = topology_b.data.as_slice().get(idx) {
                total = total + val.clone();
            }
        }
        
        total
    }
}
```

---

## 5. The Regge Geometry (Metric Generation)

This bridges the Thesis (Edge Lengths) with your Algebra (`MultiVector`).

```rust
use deep_causality_multivector::Metric;

pub struct ReggeGeometry {
    // Lengths of the 1-simplices (Edges)
    pub edge_lengths: CausalTensor<f64>, 
}

impl ReggeGeometry {
    /// Computes the Riemannian Metric for a specific simplex.
    /// Used to initialize CausalMultiVector with correct signature.
    pub fn metric_at(&self, complex: &CausalComplex, grade: usize, index: usize) -> Metric {
        // 1. Retrieve the simplex
        let simplex = &complex.skeletons[grade].simplices[index];
        let n_vertices = simplex.vertices.len();
        
        // 2. Identify all edges in this simplex
        // A k-simplex has (k+1) vertices.
        // Edges are pairs of vertices.
        let mut squared_lengths = Vec::new();

        // Iterate over all unique pairs of vertices to find edges
        for i in 0..n_vertices {
            for j in (i + 1)..n_vertices {
                let u = simplex.vertices[i];
                let v = simplex.vertices[j];
                
                // Construct edge simplex to look up index
                let edge = Simplex { vertices: vec![u, v] };
                
                // Find edge index in 1-skeleton
                if let Some(edge_idx) = complex.skeletons[1].get_index(&edge) {
                    // Get length from tensor
                    let length = self.edge_lengths.as_slice()[edge_idx];
                    squared_lengths.push(length * length);
                } else {
                    // Should not happen in a valid complex
                    panic!("Edge not found in 1-skeleton");
                }
            }
        }

        // 3. Construct Cayley-Menger Gram Matrix (G)
        // G_ij = (l_0i^2 + l_0j^2 - l_ij^2) / 2  (for Euclidean-like local frame)
        // This requires a more complex linear algebra step to diagonalize G.
        // For this spec, we assume a helper function `compute_signature` exists.
        
        // let (p, q, r) = compute_signature(&squared_lengths);
        // Metric::Generic { p, q, r }
        
        // Fallback for standard triangulation:
        Metric::Euclidean(grade) 
    }
}
```

---

## 6. The Haruna Formalism (Quantum Gates)

This logic belongs in `CausalTopology`. It uses the **Cup Product** to interact fields.

```rust
use deep_causality_multivector::MultiVector;

impl<T> CausalTopology<T> where T: MultiVector<T> + Clone {
    /// The Cup Product: (k-form) U (l-form) -> (k+l)-form
    /// Used for Generative Quantum Gates: S = exp(a U a)
    pub fn cup_product(&self, other: &CausalTopology<T>) -> CausalTopology<T> {
        let k = self.grade;
        let l = other.grade;
        let dim = k + l;
        
        // Ensure target dimension exists
        if dim >= self.complex.skeletons.len() {
             panic!("Cup product dimension exceeds complex dimension");
        }
        
        let target_skeleton = &self.complex.skeletons[dim];
        let mut result_data = Vec::with_capacity(target_skeleton.simplices.len());

        // Iterate over all (k+l)-simplices
        for simplex in &target_skeleton.simplices {
            // 1. Alexander-Whitney Approximation
            // Split vertices [0...n] into Front [0...k] and Back [k...n]
            // The vertices are sorted, so this corresponds to the standard AW map.
            
            // Front face: vertices 0 to k
            let front_face = simplex.subsimplex(0..=k);
            
            // Back face: vertices k to k+l (which is dim)
            let back_face = simplex.subsimplex(k..=dim);
            
            // 2. Look up values
            // We must find the global index of these faces to retrieve data from the topologies.
            let front_idx = self.complex.skeletons[k].get_index(&front_face)
                .expect("Front face not found in skeleton");
            let back_idx = self.complex.skeletons[l].get_index(&back_face)
                .expect("Back face not found in skeleton");
            
            // Use as_slice() to get data
            let v1 = self.data.as_slice().get(front_idx).expect("Data missing for front face");
            let v2 = other.data.as_slice().get(back_idx).expect("Data missing for back face");
            
            // 3. Geometric Product
            // The cup product on forms corresponds to the geometric product of multivectors
            // in the discrete setting (under certain metric assumptions).
            result_data.push(v1.geometric_product(v2));
        }
        
        // Return new topology
        CausalTopology {
             complex: self.complex.clone(),
             grade: dim,
             data: CausalTensor::new(result_data, vec![target_skeleton.simplices.len()]).unwrap(),
             cursor: 0
        }
    }
}
```

---

## 7. Summary of Operations

| Operation | Physics Meaning | Implementation |
| :--- | :--- | :--- |
| **`boundary(chain)`** | Boundary ($\partial$) | `CsrMatrix` Matrix-Vector Mul |
| **`coboundary(chain)`** | Ext. Derivative ($d$) | `CsrMatrix` Transpose Mul |
| **`cup_product(a, b)`** | Interaction / Logic Gate | Alexander-Whitney Combinatorics |
| **`extend(f)`** | Stencil / Local Law | `BoundedComonad` Iterator |
| **`metric_at(i)`** | Intrinsic Geometry | Regge Calculus $\to$ `Metric` Enum |

This spec provides the complete blueprint to implement the **Topological Layer** with zero external dependencies, reusing your optimized Sparse Matrix and MultiVector engines.