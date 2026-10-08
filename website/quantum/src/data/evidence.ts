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
 *   proved     the quantum section of lean/THEOREM_MAP.md, rows marked `proved`
 *   deferred   the CJ reconstruction isomorphism plus the six QCM targets the
 *              same section names as deferred (see `deferred` in ./formalization.ts)
 */
export const TESTS = {
  proved: 14,
  deferred: 7,
} as const;
