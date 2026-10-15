


cargo bench -p ultragraph

    Running benches/bench_main.rs (target/release/deps/bench_main-39a334d054d21e3e)
small_add_node          time:   [17.742 ns 17.845 ns 17.971 ns]
Found 14 outliers among 100 measurements (14.00%)
3 (3.00%) low severe
8 (8.00%) low mild
1 (1.00%) high mild
2 (2.00%) high severe

medium_add_node         time:   [22.155 ns 22.202 ns 22.248 ns]
Found 3 outliers among 100 measurements (3.00%)
3 (3.00%) low mild

large_add_node          time:   [30.520 ns 30.694 ns 30.896 ns]
Found 8 outliers among 100 measurements (8.00%)
3 (3.00%) low severe
3 (3.00%) low mild
1 (1.00%) high mild
1 (1.00%) high severe

small_get_node          time:   [7.2241 ns 7.2923 ns 7.3692 ns]

medium_get_node         time:   [4.7962 ns 4.8176 ns 4.8391 ns]
Found 3 outliers among 100 measurements (3.00%)
1 (1.00%) low mild
2 (2.00%) high mild

large_get_node          time:   [2.7991 ns 2.8057 ns 2.8134 ns]
Found 7 outliers among 100 measurements (7.00%)
1 (1.00%) low mild
6 (6.00%) high mild

small_articulation_points
time:   [1.2890 µs 1.2971 µs 1.3065 µs]
Found 3 outliers among 20 measurements (15.00%)
1 (5.00%) high mild
2 (10.00%) high severe

medium_articulation_points
time:   [10.953 µs 10.992 µs 11.039 µs]
Found 2 outliers among 20 measurements (10.00%)
2 (10.00%) high mild

large_articulation_points
time:   [136.04 µs 137.61 µs 139.40 µs]
Found 1 outliers among 20 measurements (5.00%)
1 (5.00%) high mild

xlarge_articulation_points_5k_triangles
time:   [934.69 µs 944.05 µs 953.89 µs]

small_bridges           time:   [1.1826 µs 1.1893 µs 1.1959 µs]

medium_bridges          time:   [10.396 µs 10.449 µs 10.505 µs]

large_bridges           time:   [131.69 µs 133.64 µs 135.14 µs]

xlarge_bridges_5k_triangles
time:   [907.01 µs 915.75 µs 923.44 µs]

small_biconnected_components
time:   [2.0177 µs 2.0285 µs 2.0397 µs]
Found 2 outliers among 20 measurements (10.00%)
2 (10.00%) high mild

medium_biconnected_components
time:   [17.559 µs 17.654 µs 17.788 µs]
Found 3 outliers among 20 measurements (15.00%)
1 (5.00%) high mild
2 (10.00%) high severe

large_biconnected_components
time:   [194.36 µs 197.55 µs 201.69 µs]

xlarge_biconnected_components_5k_triangles
time:   [1.2299 ms 1.2410 ms 1.2527 ms]
Found 1 outliers among 20 measurements (5.00%)
1 (5.00%) high mild