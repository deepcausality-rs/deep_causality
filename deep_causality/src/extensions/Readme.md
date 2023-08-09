# Type Extensions

A type extensions implements a local trait for an external type.

Specifically, here we implement the following traits for standard collections in Rust:
* AssumableReasoning
* CausableReasoning
* InferableReasoning
* ObservableReasoning

Each trait has a significant default implementation that the Rust compiler
inserts into the type extension whenever the trait with the default implementation is imported. 
Traits and default implementations are defined in the protocols folder.

Because Rust does not have unified collections, it is therefore necessary
to write one type extension for each collection type.

Type extensions are implemented for the following Rust standard collections:
* Array: https://doc.rust-lang.org/std/primitive.array.html
* HashMap: https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html
* BTreeMap: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
* Vector: https://doc.rust-lang.org/std/vec/struct.Vec.html
* VecDeque: https://doc.rust-lang.org/std/collections/struct.VecDeque.html

Set, HashSet, and LinkedList are not implemented because these would require significant more trait constraints such
as Eq, Hash etc. and its unclear where these collections would be used especially when Vector is already quite useful.

Because Rust default implementation can only rely on methods defined in the trait,
a certain number of methods needs to be implemented in the extension. Most of them
are fairly simple (i,e, len, empty, get_all_items) and therefore generated with compiler macros.
All compiler macros are defined in the deep_causality_macros crate.

Extension traits in Rust
http://xion.io/post/code/rust-extension-traits.html