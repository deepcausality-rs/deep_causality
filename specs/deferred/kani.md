
## Failures that Kani can spot

Kani spot two major kinds of failures: assertions and panics. If the proof harness allows some program execution that results in a panic, then Kani will report that as a failure. In addition, we saw (very briefly) a couple of other kinds of failures: null pointer dereferences and overflows. In this section, we're going to expand on these additional checks, to give you an idea of what other problems Kani will find.

Bounds checking and pointers
Rust is safe by default, and so includes dynamic (run-time) bounds checking where needed. Consider this Rust code (available here):


/// Wrap "too-large" indexes back into a valid range for the array
fn get_wrapped(i: usize, a: &[u32]) -> u32 {
if a.len() == 0 {
return 0;
}
return a[i % a.len() + 1];
}
We can again write a simple property test against this code:


    proptest! {
        #[test]
        fn doesnt_crash(i: usize, a: Vec<u32>) {
            get_wrapped(i, &a);
        }
    }
This property test will immediately find a failing case, thanks to Rust's built-in bounds checking.

But what if we change this function to use unsafe Rust?


return unsafe { *a.as_ptr().add(i % a.len() + 1) };
Now the error becomes invisible to this test:


# cargo test
[...]
test bounds_check::tests::doesnt_crash ... ok
The property test still causes an out-of-bounds access, but this undefined behavior does not necessarily cause an immediate crash. (This is part of why undefined behavior is so difficult to debug.) Through the use of unsafe code, we removed the runtime check for an out of bounds access. It just turned out that none of the randomly generated tests triggered behavior that actually crashed. But if we write a Kani proof harness:


#[cfg(kani)]
#[kani::proof]
fn bound_check() {
let size: usize = kani::any();
kani::assume(size < 4096);
let index: usize = kani::any();
let array: Vec<u32> = vec![0; size];
get_wrapped(index, &array);
}
And run this proof with:


cargo kani --harness bound_check
We still see a failure from Kani, even without Rust's runtime bounds checking.

Also, notice there were many checks in the verification output. (At time of writing, 345.) This is a result of using the standard library Vec implementation, which means our harness actually used quite a bit of code, short as it looks. Kani is inserting a lot more checks than appear as asserts in our code, so the output can be large.

We get the following summary at the end:


SUMMARY:
** 1 of 345 failed (8 unreachable)
Failed Checks: dereference failure: pointer outside object bounds
File: "./src/bounds_check.rs", line 11, in bounds_check::get_wrapped

VERIFICATION:- FAILED
Notice that, for Kani, this has gone from a simple bounds-checking problem to a pointer-checking problem. Kani will check operations on pointers to ensure they're not potentially invalid memory accesses. Any unsafe code that manipulates pointers will, as we see here, raise failures if its behavior is actually a problem.

Consider trying a few more small exercises with this example:

Exercise: Switch back to the normal/safe indexing operation and re-try Kani. How does Kani's output change, compared to the unsafe operation? (Try predicting the answer, then seeing if you got it right.)
Exercise: Try Kani's experimental concrete playback feature on this example.
Exercise: Fix the error, run Kani, and see a successful verification.
Exercise: Try switching back to the unsafe code (now with the error fixed) and re-run Kani. Does it still verify successfully?
Click to see explanation for exercise 1
Click to see explanation for exercise 2
Overflow and math errors
Consider a different variant on the function above:


fn get_wrapped(i: usize, a: &[u32]) -> u32 {
return a[i % a.len()];
}
We've corrected the out-of-bounds access, but now we've omitted the "base case": what to return on an empty list. Kani will spot this not as a bound error, but as a mathematical error: on an empty list the modulus operator (%) will cause a division by zero.

Exercise: Try to run Kani on this version of get_wrapped, to see what this kind of failure looks like.
Rust can also perform runtime safety checks for integer overflows, much like it does for bounds checks. (Though Rust disables this by default in --release mode, it can be re-enabled.) Consider this code (available here):


fn simple_addition(a: u32, b: u32) -> u32 {
return a + b;
}
A trivial function, but if we write a property test for it, we immediately find inputs where it fails, thanks to Rust's dynamic checks. Kani will find these failures as well. Here's the output from Kani:


# cargo kani --harness add_overflow
[...]
SUMMARY:
** 1 of 2 failed
Failed Checks: attempt to add with overflow
File: "./src/overflow.rs", line 7, in overflow::simple_addition

VERIFICATION:- FAILED
This issue can be fixed using Rust's alternative mathematical functions with explicit overflow behavior. For instance, if the wrapping behavior is intended, you can write a.wrapping_add(b) instead of a + b. Kani will then report no issues.

Exercise: Classic overflow failure
A classic example of a subtle bug that persisted in many implementations for a very long time is "finding the midpoint" in quick sort. This often naively looks like this (code available here):


fn find_midpoint(low: u32, high: u32) -> u32 {
return (low + high) / 2;
}

cargo kani --harness midpoint_overflow
Kani immediately spots the bug in the above code.

Exercise: Fix this function so it no longer overflows. (Hint: depending on which approach you take, you may need to add the assumption that high > low to your proof harness. Don't add that right away, see what happens if you don't. Just keep it in mind.)
Exercise: Prove your new implementation actually finds the midpoint correctly by adding an assertion to the test harness.
Click to see solutions for these exercises
A very common approach for resolving the overflow issue looks like this:


return low + (high - low) / 2;
But if you naively try this (try it!), you'll find a new underflow error: high - low might result in a negative number, but has type u32. Hence, the need to add the assumption we suggested above, to make that impossible. (Adding an assumption, though, means there's a new way to "use it wrong." Perhaps we'd like to avoid that! Can you avoid the assumption?)

After that, you might wonder how to "prove your new implementation correct." After all, what does "correct" even mean? Often we're using a good approximation of correct, such as the equivalence of two implementations (often one much "simpler" than the other somehow). Here's one possible assertion we could write in the proof harness:


assert!(result as u64 == (a as u64 + b as u64) / 2);
You might have even come up with this approach to avoiding the overflow issue in the first place! Having two different implementations, using different approaches, but proven to yield the same results, gives us greater confidence that we compute the correct result.

## Failures that Kani cannot spot

Rust’s definition of UB is so broad that Rust has the following warning:

Warning The following list is not exhaustive. There is no formal model of Rust's semantics for what is and is not allowed in unsafe code, so there may be more behavior considered unsafe. The following list is just what we know for sure is undefined behavior. Please read the Rustonomicon (https://doc.rust-lang.org/nomicon/index.html) before writing unsafe code.

Given the lack of a formal semantics for UB, and given Kani's focus on memory safety, there are classes of UB which Kani does not detect, or only makes a best-effort attempt to detect them. A non-exhaustive list of these, based on the non-exhaustive list from the Rust documentation, is:

Data races.
Kani focuses on sequential code.
Breaking the pointer aliasing rules.
Kani can detect if misuse of pointers causes memory safety or assertion violations, but does not track reference lifetimes.
Mutating immutable data.
Kani can detect if modification of immutable data causes memory safety or assertion violations, but does not track reference lifetimes.
Invoking undefined behavior via compiler intrinsics.
Kani makes a best effort attempt to check the preconditions of compiler intrinsics, but does not guarantee to do so in all cases. See also current support for compiler intrinsics.
Executing code compiled with platform features that the current platform does not support (see target_feature).
Kani relies on rustc to check for this case.
Calling a function with the wrong call ABI or unwinding from a function with the wrong unwind ABI.
Kani relies on rustc to check for this case.
Producing an invalid value, even in private fields and locals.
Kani won't create invalid values with kani::any() but it also won't complain if you transmute an invalid value to a Rust type (for example, a 0 to NonZeroU32).
Incorrect use of inline assembly.
Kani does not support inline assembly.
Using uninitialized memory.
See the corresponding section in our Rust feature support.

Rust feature support
The table below tries to summarize the current support in Kani for the Rust language features according to the Rust Reference. We use the following values to indicate the level of support:

Yes: The feature is fully supported. We are not aware of any issue with it.
Partial: The feature is at least partially supported. We are aware of some issue with with it.
No: The feature is not supported. Some support may be available but analyses should not be trusted.
As with all software, bugs may be found anywhere regardless of the level of support. In such cases, we would greatly appreciate that you filed a bug report.

Reference	Feature	Support	Notes
3.1	Macros By Example	Yes
3.2	Procedural Macros	Yes
4	Crates and source files	Yes
5	Conditional compilation	Yes
6.1	Modules	Yes
6.2	Extern crates	Yes
6.3	Use declarations	Yes
6.4	Functions	Yes
6.5	Type aliases	Yes
6.6	Structs	Yes
6.7	Enumerations	Yes
6.8	Unions	Yes
6.9	Constant items	Yes
6.10	Static items	Yes
6.11	Traits	Yes
6.12	Implementations	Yes
6.13	External blocks	Yes
6.14	Generic parameters	Yes
6.15	Associated Items	Yes
7	Attributes	Yes
8.1	Statements	Yes
8.2.1	Literal expressions	Yes
8.2.2	Path expressions	Yes
8.2.3	Block expressions	Yes
8.2.4	Operator expressions	Yes
8.2.5	Grouped expressions	Yes
8.2.6	Array and index expressions	Yes
8.2.7	Tuple and index expressions	Yes
8.2.8	Struct expressions	Yes
8.2.9	Call expressions	Yes
8.2.10	Method call expressions	Yes
8.2.11	Field access expressions	Yes
8.2.12	Closure expressions	Yes
8.2.13	Loop expressions	Yes
8.2.14	Range expressions	Yes
8.2.15	If and if let expressions	Yes
8.2.16	Match expressions	Yes
8.2.17	Return expressions	Yes
8.2.18	Await expressions	No	See Notes - Concurrency
9	Patterns	Partial	#707
10.1.1	Boolean type	Yes
10.1.2	Numeric types	Yes
10.1.3	Textual types	Yes
10.1.4	Never type	Yes
10.1.5	Tuple types	Yes
10.1.6	Array types	Yes
10.1.7	Slice types	Yes
10.1.8	Struct types	Yes
10.1.9	Enumerated types	Yes
10.1.10	Union types	Yes
10.1.11	Function item types	Yes
10.1.12	Closure types	Partial	See Notes - Advanced features
10.1.13	Pointer types	Partial	See Notes - Advanced features
10.1.14	Function pointer types	Partial	See Notes - Advanced features
10.1.15	Trait object types	Partial	See Notes - Advanced features
10.1.16	Impl trait type	Partial	See Notes - Advanced features
10.1.17	Type parameters	Partial	See Notes - Advanced features
10.1.18	Inferred type	Partial	See Notes - Advanced features
10.2	Dynamically Sized Types	Partial	See Notes - Advanced features
10.3	Type layout	Yes
10.4	Interior mutability	Yes
10.5	Subtyping and Variance	Yes
10.6	Trait and lifetime bounds	Yes
10.7	Type coercions	Partial	See Notes - Advanced features
10.8	Destructors	Partial
10.9	Lifetime elision	Yes
11	Special types and traits	Partial
Box<T>	Yes
Rc<T>	Yes
Arc<T>	Yes
Pin<T>	Yes
UnsafeCell<T>	Partial
PhantomData<T>	Partial
Operator Traits	Partial
Deref and DerefMut	Yes
Drop	Partial
Copy	Yes
Clone	Yes
14	Linkage	Yes
15.1	Unsafe functions	Yes
15.2	Unsafe blocks	Yes
15.3	Behavior considered undefined	Partial
Data races	No	See Notes - Concurrency
Dereferencing dangling raw pointers	Yes
Dereferencing unaligned raw pointers	No
Breaking pointer aliasing rules	No
Mutating immutable data	No
Invoking undefined behavior via compiler intrinsics	Partial	See Notes - Intrinsics
Executing code compiled with platform features that the current platform does not support	No
Producing an invalid value, even in private fields and locals	No
Notes on partially or unsupported features
Code generation for unsupported features
Kani aims to be an industrial verification tool. Most industrial crates may include unsupported features in parts of their code that do not need to be verified. In general, this should not prevent users using Kani to verify their code.

Because of that, the general rule is that Kani generates an assert(false) statement followed by an assume(false) statement when compiling any unsupported feature. assert(false) will cause verification to fail if the statement is reachable during the verification stage, while assume(false) will block any further exploration of the path. However, the analysis will not be affected if the statement is not reachable from the code under verification, so users can still verify components of their code that do not use unsupported features.

In a few cases, Kani aborts execution if the analysis could be affected in some way because of an unsupported feature (e.g., global ASM).

Assembly
Kani does not support assembly code for now. We may add it in the future but at present there are no plans to do so.

Check out the tracking issues for inline assembly (asm! macro) and global assembly (asm_global! macro) to know more about the current status.

Concurrency
Concurrent features are currently out of scope for Kani. In general, the verification of concurrent programs continues to be an open research problem where most tools that analyze concurrent code lack support for other features. Because of this, Kani emits a warning whenever it encounters concurrent code and compiles as if it was sequential code.

Standard library functions
Kani overrides a few common functions (e.g., print macros) to provide a more verification friendly implementation.

Advanced features
The semantics around some advanced features (traits, types, etc.) from Rust are not formally defined which makes it harder to ensure that we can properly model all their use cases.

We are aware of a lack of sanity checking the Variant type in projections #448. If you become aware of other issues concerning these features, please file a bug report.

Panic strategies
Rust has two different strategies when a panic occurs:

Stack unwinding (default): Walks back the stack cleaning up the data from each function it encounters.
Abortion: Immediately ends the program without cleaning up.
Currently, Kani does not support stack unwinding. This has some implications regarding memory safety since programs sometimes rely on the unwinding logic to ensure there is no resource leak or persistent data inconsistency. Check out this issue for updates on stack unwinding support.

Uninitialized memory
Reading uninitialized memory is considered undefined behavior in Rust. Kani has partial, experimental support for detecting access to uninitialized memory with the -Z uninit-checks option. See this issue for more details.

Destructors
At present, we are aware of some issues with destructors, in particular those related to advanced features.

Intrinsics
Please refer to Intrinsics for information on the current support in Kani for Rust compiler intrinsics.

Floating point operations
Kani supports floating point numbers, but some supported operations on floats are "over-approximated." These are the trigonometric functions like sin and cos and the sqrt function as well. This means the verifier can raise errors that cannot actually happen when the code is run normally. For instance, (#1342) the sin/cos functions basically return a nondeterministic value between -1 and 1. In other words, they largely ignore their input and give very conservative answers. This range certainly includes the "real" value, so proof soundness is still preserved, but it means Kani could raise spurious errors that cannot actually happen. This makes Kani unsuitable for verifying some kinds of properties (e.g. precision) about numerical algorithms. Proofs that fail because of this problem can sometimes be repaired by introducing "stubs" for these functions that return a more acceptable approximation. However, note that the actual behavior of these functions can vary by platform/os/architecture/compiler, so introducing an "overly precise" approximation may introduce unsoundness: actual system behavior may produce different values from the stub's approximation.

Unstable features
In general, unstable Rust features are out of scope and any support for them available in Kani should be considered unstable as well.

The following are examples of unstable features that are not supported in Kani:

Generators
C-variadics

As explained in Comparison with other tools, Kani is based on a technique called model checking, which verifies a program without actually executing it. It does so through encoding the program and analyzing the encoded version. The encoding process often requires "modeling" some of the library functions to make them suitable for analysis. Typical examples of functionality that requires modeling are system calls and I/O operations. In some cases, Kani performs such encoding through overriding some of the definitions in the Rust standard library.

The following table lists some of the symbols that Kani overrides and a description of their behavior compared to the std versions:

Name	Description
assert, assert_eq, and assert_ne macros	Skips string formatting code, generates a more informative message and performs some instrumentation
debug_assert, debug_assert_eq, and debug_assert_ne macros	Rewrites as equivalent assert* macro
print, eprint, println, and eprintln macros	Skips string formatting and I/O operations
unreachable macro	Skips string formatting and invokes panic!()
std::process::{abort, exit} functions	Invokes panic!() to abort the execution