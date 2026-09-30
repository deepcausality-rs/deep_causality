/** A count as an English word, for the start of a sentence. Covers 0 to 20. */
export const numberWord = (n: number): string => {
  const w = [
    'zero', 'one', 'two', 'three', 'four', 'five', 'six', 'seven', 'eight', 'nine', 'ten',
    'eleven', 'twelve', 'thirteen', 'fourteen', 'fifteen', 'sixteen', 'seventeen', 'eighteen',
    'nineteen', 'twenty',
  ];
  return w[n] ?? String(n);
};

/**
 * Counts quoted across the site. Each has one source and one command.
 *
 *   count      cargo test -p deep_causality_quantum --all-features
 *              772 unit and integration tests plus 1 doc test, all passing
 *              (a `#[test]` grep finds a different number: it counts attributes,
 *              not the tests the harness runs)
 *   proved     the quantum section of lean/THEOREM_MAP.md, rows marked `proved`
 *   deferred   the CJ reconstruction isomorphism plus the six QCM targets the
 *              same section names as deferred (see `deferred` in ./formalization.ts)
 *   papers     files under deep_causality_quantum/papers/
 */
export const TESTS = {
  count: 773,
  proved: 14,
  deferred: 7,
  papers: 7,
} as const;
