[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# sliding_window

The sliding window over-allocates memory to delay array copies, trading space for time.
A window of size N holds about C-1 elements without an array copy, where the capacity C is N x M
for a multiple M.

For example, a window size N of 7 and a multiple M of 7 give a capacity C of 49 (7*7),
so the window holds up to 48 elements before a rewind copies the window to the start of the array.

Two implementations exist:

1) Vector backed
2) Array backed

## Vector backed implementation

Takes the window size N and the multiple M as runtime arguments.

See:

* [Benchmark](benches/benchmarks)
* [Code](src/window_type/)
* [Test](tests/window_type/)

## Array backed implementation

Takes the window size SIZE and the CAPACITY as const generic parameters, because a static array
needs its length at compile time.

See:

* [Benchmark](benches/benchmarks/)
* [Code](src/window_type)
* [Test](tests/window_type)

## Configuration

When N is small (1 ... 50), M alone determines performance. A multiple of 100 to 1000
runs 30% to 50% faster than a small multiple (2 to 10). Above a threshold total capacity,
performance drops sharply because CPU cache misses increase. The threshold depends on the CPU and its cache size.

Benchmark several configurations to find the best total capacity for your N and M. When N is fixed,
run an optimizer to find the M that maximizes write throughput.

## Usage

* ArrayStorage and VectorStorage have different signatures because only ArrayStorage uses const generics.
* Size is the maximum number of elements the sliding window holds.
* Capacity is the maximum number of elements stored before a rewind.

```rust
use dcl_data_structures::prelude::{ArrayStorage, SlidingWindow,sliding_window};

// Size refers to the maximum number of elements the sliding window can store.
const SIZE: usize = 4;
// Capacity refers to the maximum number of elements before a rewind occurs.
// Note, CAPACITY > SIZE and capacity should be a multiple of size.
// For example, size 4 should be stored 300 times before rewind;
// 4 * 300 = 1200
const CAPACITY: usize = 1200;

// Util function that helps with type inference.
fn get_sliding_window() -> SlidingWindow<ArrayStorage<Data, SIZE, CAPACITY>, Data> {
    sliding_window::new_with_array_storage()
}

pub fn main(){
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);

    // Filled means, the window holds 4 elements. 
    assert!(!window.filled());

    // If you try to access an element before the window id filled, you get an error.
    let res = window.first();
    assert!(res.is_err());

    let res = window.last();
    assert!(res.is_err());

    // Add some data
    window.push(Data { dats: 3 });
    window.push(Data { dats: 2 });
    window.push(Data { dats: 1 });
    window.push(Data { dats: 0 });
    assert!(window.filled());

    // Now we can access elements of the window
    // Last element added was 0
    let res = window.last();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 0);

    // First (oldest) element added was 3
    let res = window.first();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 3);

    // When we add more data after the window filled,
    // the "last" element refers to the last added
    // and the oldest element will be dropped.
    let d = Data { dats: 42 };
    window.push(d);

    let res = window.last();
    assert!(res.is_ok());

    let data = res.unwrap();
    assert_eq!(data.dats, 42);

    // Because 42 was added at the front,
    // 3 was dropped at the end therefore
    // the oldest element is now 2
    let res = window.first();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 2);
}
```

## Performance

### Single Push Operations

| Implementation      	| Single Push Time 	| Notes                                                	|
|----------------|----------------|--------------------------| 
| ArrayStorage | ~2.08ns | Good balance | 
| VectorStorage | ~2.5ns | Most flexible |

Batch Operations (100 elements)

| Implementation      	| Batch Push Time 	 | Notes                                                	|
|----------------|----------------|--------------------------| 
| ArrayStorage | ~1.95ns           | Consistent performance |
| VectorStorage | ~2.3ns            | Predictable scaling |

Sequential Operations

| Implementation | Operation Time | Notes                    | 
|----------------|----------------|--------------------------| 
| ArrayStorage | ~605ps | Excellent cache locality | 
| VectorStorage | ~850ps | Most predictable         |


## Technical Details
- Sample size: 100 measurements per benchmark
- Outliers detected and handled (2-8% outliers per benchmark)
- All benchmarks use random access patterns


## Hardware & OS
- Architecture: ARM64 (Apple Silicon, M3 Max)
- OS: macOS Darwin 24.1.0
- Kernel: XNU 11215.41.3~2
- Machine: MacBook Pro (T6031)

# Implementation Details

**Memory Layout**
* ArrayStorage stores the window in an inline `[T; CAPACITY]` array; it allocates nothing on the heap.
* VectorStorage pre-allocates a `Vec` of size x multiple and aligns the struct to a 64-byte cache line.

**Head Management**
* `push` writes at the tail and moves the head to `tail - size` once the window is filled.
* When the tail reaches the capacity, a rewind moves the last `size` elements to the start with `copy_within`.

**Safe Rust**
* Both storages use bounds-checked indexing and no `unsafe`.

## When to Use Each Implementation

### ArrayStorage

✅ Use When:
* Window size and capacity are known at compile time
* Heap allocation is unwanted
* Throughput matters (faster in all three benchmarks above)

❌ Avoid When:
* Window size is only known at runtime

### VectorStorage
✅ Use When:
* Window size or multiple is chosen at runtime

❌ Avoid When:
* Size and capacity are known at compile time
* Every nanosecond per push counts


### Recommendation
Start with ArrayStorage. Use VectorStorage when the window size is only known at runtime.

### Remember to benchmark with your specific use case:

Performance depends on
* Window size
* Data types
* Access patterns
* System architecture
* CPU type