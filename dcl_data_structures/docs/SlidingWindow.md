[//]: # (---)
[//]: # (SPDX-License-Identifier: MIT)
[//]: # (---)

# sliding_window

This sliding window implementation over-allocates to trade space (memory) for time complexity by delaying expensive array copy operations.
Specifically, for a sliding window of size N, the number of elements that can be hold without any array copy
is approx C-1, where C is the total capacity defined as NxM with M as a multiple.   

For example, if the window size N is 7, and the multiple M is 7, then the max capacity C is 49 (7*7), 
means the sliding window can hold up to 48 elements before a rewind occurs.   

Two different implementations are available:
1) Vector backed
2) Array backed
 
## Vector backed implementation

Take window size N and multiple M as arguments 

See:
* [Benchmark](../benches/benchmarks/bench_window_vec.rs)
* [Code](../src/window_type/storage_vec.rs)
* [Test](../tests/window_vector_backed_tests.rs)

## Array backed implementation

Takes window size SIZE and a CAPACITY as generic parameters.
This is because static arrays requiring const generics parameter.

See:
* [Benchmark](../benches/benchmarks/bench_window_arr.rs)
* [Code](../src/window_type/storage_array.rs)
* [Test](../tests/window_array_backed_tests.rs)

## Configuration  

When N is reasonable small (1 ... 50), then only M determines the performance. In this case, a multiple of 100 to 1000, 
gives an additional 30% to 50% performance boost over a comparable small multiplier (2 to 10). However, 
when the total capacity exceeds a certain threshold, performance deteriorates significantly because of increased CPU cache misses.
This threshold depends on the actual CPU cache size and total system load.

Therefore, it is generally recommended to run benchmarks with various configurations
to determine the best total capacity based on N and M. When the window size N is known to be fixed, 
then it's best to run an optimizer to find the best value for M that maximizes total write throughput. 

## Performance

Both implementations perform well on inserts with the array backed implementation 
being about 1/3 faster than the vector backed implementation. Read operations are basically free O(1) since 
the sliding window is just a slice over the backing data structure.
