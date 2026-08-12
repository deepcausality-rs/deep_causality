# Runtimes

# Start files (CRTstuff)

Location: %sysroot/usr/lib/\<triple\>/

In this order:

* crt1.o: startup sequence
  * crt1.o: `-no-pie`
  * Scrt1.o: `-pie`, `-shared`
  * rcrt1.o: `-static-pie`
  * gcrt1.o: for `-pg`
* crti.o: _init/_fini prolog as first object in .init section
* crtbegin.o (compiler-rt): .init_array,.fini_array entries for stuff like EH frame registration and cleanup
  * [NOT SUPPORTED HERE] In compat mode, these also walk the .ctors, .dtors arrays.
  * crtbegin.o: `-no-pie`
  * crtbeginS.o: `-pie`, `-shared`
  * crtbeginT.o: `-static-pie`
* crtend.o (compiler-rt): sentinel value for end of .eh_frame array.
  * [NOT SUPPORTED HERE] In compat mode, these also add sentinel end values for .ctors, .ctors arrays.
  * crtend.o: `-no-pie`
  * crtendS.o: `-pie`, `-shared`
  * crtendT.o: `-static-pie`
* crtn.o: _init/_fini epilog as last object in .init section

# Sanitizers

## Compatibility Matrix
Legend:
✓ = supported in practice
✗ = not compatible

|               | ASan | MSan | TSan | HWASan | UBSan | LSan |
|---------------|------|------|------|--------|-------|------|
| **ASan**      |  —   | ✗    | ✗    | ✗      | ✓     | ✓*   |
| **MSan**      | ✗    |  —   | ✗    | ✗      | ✓     | ✗    |
| **TSan**      | ✗    | ✗    |  —   | ✗      | ✓     | ✗    |
| **HWASan**    | ✗    | ✗    | ✗    |  —     | ✓     | ✗    |
| **UBSan**     | ✓    | ✓    | ✓    | ✓      |  —    | ✓    |
| **LSan**      | ✓*   | ✗    | ✗    | ✗      | ✓     |  —   |

*LSan is typically embedded inside ASan on most platforms (e.g. Linux).
It is not a meaningful independent pairing with the others.*

## References
- https://gist.github.com/cerisier/aadefc9d3d015799540308eb4c33db2e
- https://maskray.me/blog/2021-11-07-init-ctors-init-array
- https://gist.github.com/MaskRay/6c3910c7ab1df208c36cfedf965f6b51

# TODO
* TODO(cerisier): Write about everything else
