[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# ArrayGrid - A Faster Tensor For Low Dimensional Data

ArrayGrid stores scalars, vectors, and low-dimensional matrices, similar in idea to a tensor but
limited to 1 to 4 dimensions. Every ArrayGrid is a static, fixed-size const generic array. Because
all structural parameters are known at compile time, the compiler can lay the data out
cache-aligned, which makes ArrayGrid faster than a tensor on low-dimensional data.


## Usage

Important details:

* All const generic parameters are required, whichever ArrayType you use.
* To change the ArrayGrid type, change the enum.
* Indexing is bounds-checked at run time: a PointIndex outside the array boundaries panics, so keep it within the const dimensions.

```rust
use deep_causality_data_structures::{ArrayGrid, ArrayType, PointIndex};

// Consts dimensions requires for const generic paramaters
// Use these to check whether your PointIndex stays within the Array boundaries.
const WIDTH: usize = 5;
const HEIGHT: usize = 5;
const DEPTH: usize = 5;
const TIME: usize = 5;

pub fn main(){
    // Make a simple 1D Array of type usize
    let array_type = ArrayType::Array1D;
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    // Create a 1D index
    let p = PointIndex::new1d(1);

    // Store a usize with the point index
    ag.set(p, 42);

    // Get the usize for the point index
    let res = ag.get(p);
    assert_eq!(res, 42);
    
    // Make a custom struct 
    // ArrayGrid requires Copy + Default to store MyStuct
    #[derive(Debug, Default, Copy, Clone)]
    struct MyStruct{
        number: usize,
        mod_five: bool,
    }
    
    // Make a 4D array aka matrix over x,y,z that stores My struct
    // Notice, only the ArrayType changes to do that. 
    let array_type = ArrayType::Array4D;
    let ag: ArrayGrid<MyStruct, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    // Create a new 4D point index where only time varies
    let idx_t0 = PointIndex::new4d(1, 1, 1, 0);
    let idx_t1 = PointIndex::new4d(1, 1, 1, 1);
    let idx_t2 = PointIndex::new4d(1, 1, 1, 2);

    // Create some data for each index 
    let my_struct_t0 = MyStruct{ number: 23, mod_five: false };
    let my_struct_t1 = MyStruct{ number: 24, mod_five: false };
    let my_struct_t2 = MyStruct{ number: 25, mod_five: true };

    // Store data
    ag.set(idx_t0, my_struct_t0);
    ag.set(idx_t1, my_struct_t1);
    ag.set(idx_t2, my_struct_t2);

    // Get data at t2
    let res = ag.get(idx_t2);
    
    // Verify results
    let exp_number = 25;
    assert_eq!(res.number, exp_number);
    let exp_mod = true;
    assert_eq!(res.mod_five, exp_mod);
}
```

## Performance

The benchmark measures set operations on a 1D, 2D, 3D, and 4D ArrayGrid.
The get operation runs in near-constant time O(1) thanks to the const generics, so the
benchmark skips it.


### Performance Summary
- **1D Grid**: 604.71 ps (±5.01 ps) 
- **2D Grid**: 581.33 ps (±2.81 ps) 
- **3D Grid**: 862.16 ps (±13.60 ps) 
- **4D Grid**: 1.137 ns (±0.029 ns) 


### Key Observations
1. The 1D and 2D grid operations are fastest.
2. Cost rises with dimension, most visibly in 3D and 4D.
3. 4D operations take nearly twice as long as the lower-dimensional ones.

## Technical Details
- Sample size: 100 measurements per benchmark
- Outliers detected and handled (2-8% outliers per benchmark)
- All benchmarks use random access patterns to simulate real-world usage

## Hardware & OS
- Architecture: ARM64 (Apple Silicon, M3 Max)
- OS: macOS Darwin 24.1.0 (Sequoia 15.1)
- Kernel: XNU 11215.41.3~2
- Machine: MacBook Pro (T6031)

## Problem

DeepCausality adjusts the values stored in a context hypergraph, often through an adjustment matrix.
The matrix can be attached to each element of the context graph and may need periodic updates.

The adjustment for temporal-spatial data depends on the structure of the data. A tensor would
represent a multi-dimensional adjustment in a single structure, but tensors carry overhead that
costs performance, especially on low (<5) dimensional data. Adjusting values in a context graph
needs at most a 4D matrix, so a tensor is unnecessary. The tensor overhead comes from its complex
object model, whose non-aligned data layout increases CPU cache misses.

## Solution

ArrayGrid is a custom data structure indexed by a PointIndex struct. A tensor stays parametric over
N dimensions and needs a complex object representation; a Grid is limited to 1 to 4 dimensions and
stores every variant as a static fixed-size array, which the compiler can lay out cache-aligned.
Performance matters because a context hypergraph may grow to millions of nodes, and a global
adjustment must then run as fast as possible.

## Index

Rust has no variadic arguments, so indexing a grid of variable dimension needs another approach.
Passing a vector risks null index errors. Because the grid is limited to 4D, the index is a struct
with four `usize` fields; the constructors set unused fields to zero, so every index has the same
signature. X, Y, Z are the 3D coordinates and T is time, the fourth dimension:

```rust
/// A point used to index a GridArray up to four dimensions.
#[derive(Debug, Clone, Copy)]
pub struct PointIndex {
    pub x: usize, // Height 
    pub y: usize, // Width
    pub z: usize, // Depth
    pub t: usize, // Time
}

impl PointIndex{
    pub fn new1d(x: usize) -> Self {Self { x, y: 0, z: 0, t: 0 }}
    pub fn new2d(x: usize, y: usize) -> Self { Self { x, y, z: 0, t: 0 }}
    pub fn new3d(x: usize, y: usize, z: usize) -> Self { Self { x, y, z, t: 0 } }
    pub fn new4d(x: usize, y: usize, z: usize, t: usize) -> Self {Self { x, y, z, t }}
}
```

## Storage API

Each of the four dimensions needs its own storage implementation. The storage trait abstracts over
them while keeping the const generic array sizes. It follows
the [graph storage API in Petgraph](https://github.com/petgraph/petgraph/issues/563).
Every implementation defines `height`; the other dimension getters return `None` by default unless
the implementing type overrides them.

```rust
use crate::PointIndex;

pub trait Storage<T>where  T: Copy {
    fn get(&self, p: PointIndex) -> &T;
    fn set(&mut self, p: PointIndex, elem: T);
    fn height(&self) -> Option<&usize>;
    fn depth(&self) -> Option<&usize> { None }
    fn time(&self) -> Option<&usize> { None }
    fn width(&self) -> Option<&usize> { None }
}
```

The dimension getters (`height`, `depth`, `time`, `width`) return an option of a reference
(`Option<&usize>`) rather than a reference to an option (`&Option<usize>`). An `Option<&usize>`
does not require the storage to hold an `Option`: an implementation without a dimension returns
`None` and stores nothing. Neither form lets a caller change the stored value, since replacing
`Some` with `None` in place takes a `&mut Option<usize>`.

## Storage Implementation

The storage trait could be implemented for any heap-allocated type, such as a vector. The
PointIndex also permits a fixed-size array through const generics, which is faster. The 2D matrix
type, for example, is implemented over a 2D static array. The const generic array needs an extra
`Sized` bound to compile.

```rust
impl<T, const W: usize, const H: usize> Storage<T> for [[T; W]; H]
    where
        T: Copy,
        [[T; W]; H]: Sized,
{
    fn get(&self, p: PointIndex) -> &T { &self[p.y][p.x] }
    fn set(&mut self, p: PointIndex, elem: T) { self[p.y][p.x] = elem }
    fn height(&self) -> Option<&usize> { Some(&H) }
    fn width(&self) -> Option<&usize> { Some(&W) }
}
```

Besides `get` and `set`, the 2D array implements `height` and overrides `width` to expose the
array boundaries. Indexing stays bounds-checked, so an out-of-range index panics; each type returns
all its applicable bounds so a caller can check an index first. The 3D and 4D types follow the same
pattern.

## Grid Type

The grid type abstracts over the specific storage through the storage trait. Three details:

* Grid abstracts over `Storage<T>` without otherwise referencing `T`, so it needs a `PhantomData` for `T`.
* Grid is a container, so it provides interior mutability through `RefCell`.
* Each storage returns its bounds as an `Option` of a reference; Grid dereferences it and returns a
  value, because it cannot return a reference into the `RefCell`.

```rust
#[derive(Debug, Clone)]
pub struct Grid<S, T>
    where
        T: Copy,
        S: Storage<T>,
{
    inner: RefCell<S>,  // Requiered for interior mutability
    ty: PhantomData<T>, // Required due to missing binding to type T
}

```

The constructor takes the storage and stores it in a `RefCell`, so `get` and `set` access it
through `borrow` and `borrow_mut`. The impl requires `T: Copy + Default`. The listing below shows
the important parts of the Grid implementation.

```rust

impl<S, T> Grid<S, T>
    where
        T: Copy + Default,
        S: Storage<T>,
{
    pub fn new(storage: S) -> Self {
        Self {
            inner: RefCell::new(storage),
            ty: Default::default(),
        }
    }

    pub fn get(&self, p: PointIndex) -> T { self.inner.borrow().get(p).to_owned() }

    pub fn set(&self, p: PointIndex, value: T) { self.inner.borrow_mut().set(p, value); }

    pub fn depth(&self) -> Option<usize> { ...}
    pub fn height(&self) -> Option<usize> {...} 
```

The grid type is not meant for direct use, because it needs an instance of the storage type before
it can be constructed. ArrayGrid instead abstracts over all four storage implementations through an
enum.

## ArrayGrid

Each of the four storage implementations has a different type signature, but interfaces and
function signatures stay stable only with a single type. The const generic parameters also differ
per implementation, so a shared type needs as many generic parameters as the largest
implementation, the 4D array. Type aliases keep the const generic array signatures readable:

```rust
// Fixed sized static ArrayGrid
pub type ArrayGrid1DType<T, const H: usize> = Grid<[T; H], T>;
pub type ArrayGrid2DType<T, const W: usize, const H: usize> = Grid<[[T; W]; H], T>;
pub type ArrayGrid3DType<T, const W: usize, const H: usize, const D: usize> = Grid<[[[T; W]; H]; D], T>;
pub type ArrayGrid4DType<T, const W: usize, const H: usize, const D: usize, const C: usize> = Grid<[[[[T; W]; H]; D]; C], T>;
```

A plain enum identifies each of the four storage implementations:

```rust
pub enum ArrayType {
    Array1D,
    Array2D,
    Array3D,
    Array4D,
}
```

ArrayGrid itself is an enum whose variants each hold an instance of the corresponding storage.
Because of the const generic requirement, the enum is generic over all four dimensions plus the
stored type `T`: five generic parameters. Each additional dimension would add another, which is
why ArrayGrid stops at 4D.

```rust
// T Type
// W Width
// H Height
// D Depth
// C Chronos (Time) since T was already taken for Type T
pub enum ArrayGrid<T, const W: usize, const H: usize, const D: usize, const C: usize>
    where
        T: Copy,
{
    ArrayGrid1D(ArrayGrid1DType<T, H>),
    ArrayGrid2D(ArrayGrid2DType<T, W, H>),
    ArrayGrid3D(ArrayGrid3DType<T, W, H, D>),
    ArrayGrid4D(ArrayGrid4DType<T, W, H, D, C>),
}
```

The type aliases make the enum signatures readable and help verify the type embedding. The
ArrayGrid implementation has three parts:

1) Constructor
2) API
3) Getters

**Constructor**

The constructor matches on the ArrayType and creates a Grid with the matching dimensions and
storage. `T: Default` fills the new array with `T::default()`.

```rust
impl<T, const W: usize, const H: usize, const D: usize, const C: usize> ArrayGrid<T, W, H, D, C>
    where
        T: Copy + Default,
{
    pub fn new(array_type: ArrayType) -> ArrayGrid<T, W, H, D, C> {
        match array_type {
            ArrayType::Array1D => ArrayGrid::ArrayGrid1D(Grid::new([T::default(); H])),
            ArrayType::Array2D => ArrayGrid::ArrayGrid2D(Grid::new([[T::default(); W]; H])),
            ArrayType::Array3D => ArrayGrid::ArrayGrid3D(Grid::new([[[T::default(); W]; H]; D])),
            ArrayType::Array4D => ArrayGrid::ArrayGrid4D(Grid::new([[[[T::default(); W]; H]; D]; C])),
        }
    }
}
```

**API**

The API sets or gets a value of type `T`, the most common operations for an adjustment matrix.
It does not expose array dimensions, which would require matching over every variant; the getters
below give low-level access to the underlying grid instead.

```rust
impl<T, const W: usize, const H: usize, const D: usize, const C: usize> ArrayGrid<T, W, H, D, C>
    where
        T: Copy + Default,
{
    pub fn get(&self, p: PointIndex) -> T {
        match self {
            ArrayGrid::ArrayGrid1D(grid) => { grid.get(p) }
            ArrayGrid::ArrayGrid2D(grid) => { grid.get(p) }
            ArrayGrid::ArrayGrid3D(grid) => { grid.get(p) }
            ArrayGrid::ArrayGrid4D(grid) => { grid.get(p) }
        }
    }

    pub fn set(&self, p: PointIndex, value: T) {
        match self {
            ArrayGrid::ArrayGrid1D(grid) => { grid.set(p, value) }
            ArrayGrid::ArrayGrid2D(grid) => { grid.set(p, value) }
            ArrayGrid::ArrayGrid3D(grid) => { grid.set(p, value) }
            ArrayGrid::ArrayGrid4D(grid) => { grid.set(p, value) }
        }
    }
}
```

**Getters**

For low-level access, each getter returns the underlying grid as an option of a reference. The
option covers the variant mismatch: an ArrayGrid holding a 2D grid returns `None` from the
1D, 3D and 4D getters, so calling the wrong getter shows up at the call site.

```rust
impl<T, const W: usize, const H: usize, const D: usize, const C: usize> ArrayGrid<T, W, H, D, C>
    where
        T: Copy + Default,
{
    pub fn array_grid_1d(&self) -> Option<&ArrayGrid1DType<T, H>>
    {
        if let ArrayGrid::ArrayGrid1D(array_grid) = self {
            Some(array_grid)
        } else {
            None
        }
    }

    pub fn array_grid_2d(&self) -> Option<&ArrayGrid2DType<T, W, H>> { ... }
    pub fn array_grid_3d(&self) -> Option<&ArrayGrid3DType<T, W, H, D>> { ... }
    pub fn array_grid_4d(&self) -> Option<&ArrayGrid4DType<T, W, H, D, C>> { ... }
```

## Usage

Building an ArrayGrid takes two steps:

1) Define constant array boundaries.
2) Construct an ArrayGrid with a chosen array type and value type.

```rust
use deep_causality_data_structures::{Array2D, Array3D, ArrayGrid, PointIndex};

// 1) Define constant array boundaries.
const WIDTH: usize = 5;
const HEIGHT: usize = 5;
const DEPTH: usize = 5;
const TIME: usize = 5;

fn main() {
    // 2) Construct an ArrayGrid; the first type parameter is the stored value type (usize)
    let array_type = Array2D;
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    // Create an index 
    let p = PointIndex::new2d(1, 2);
    
    // set a value 
    ag.set(p, 2);
    
    // get a value 
    let res = ag.get(p);
    
    // Make it a 3D Matrix
    let array_type = Array3D;
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);
    
    // set and get values in a 3D Matrix
    let p = PointIndex::new3d(1, 2, 3);
    ag.set(p, 3);
    let res = ag.get(p);
    
    // Low level access to the 3D grid
    let g = ag.array_grid_3d()
        .expect("failed to create array grid");

    assert_eq!(g.height(), Some(HEIGHT));
    assert_eq!(g.width(), Some(WIDTH));
    assert_eq!(g.depth(), Some(DEPTH));
}
```

The ArrayGrid constructor requires all generic parameters, whichever storage it instantiates. A
library that uses at most a 2D matrix should set the remaining const generic values (Depth, Time)
to one. Once created, an ArrayGrid behaves like any other API, with interior mutability.
