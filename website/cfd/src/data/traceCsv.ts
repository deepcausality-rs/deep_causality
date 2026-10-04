/**
 * Readers for the files an example commits with its run: the CSV tables and the gate lines of
 * `output.txt`. Shared by the three walk modules (`corridorTrace.ts`, `weatherTrace.ts`,
 * `retroTrace.ts`). Both throw on input they cannot read whole, so a malformed file fails the build.
 */

export type Row = Record<string, number>;

/**
 * Parse the two-row-header CSV `write_rows` emits: names, then `#units`, then data. Every data row
 * has one cell per name, and every cell is a number or the literal `NaN` some columns carry.
 * `name` is the file name the errors report.
 */
export function parseRows(csv: string, name: string): Row[] {
  const lines = csv.trim().split('\n');
  const names = lines[0].split(',');
  return lines
    .slice(1)
    .filter((l) => !l.startsWith('#units'))
    .map((l) => {
      const raw = l.split(',');
      if (raw.length !== names.length) {
        throw new Error(`${name}: row has ${raw.length} cells for ${names.length} columns (${names}): ${l}`);
      }
      const cells = raw.map((c) => {
        const v = Number(c);
        if (c.trim() === '' || (Number.isNaN(v) && c.trim() !== 'NaN')) {
          throw new Error(`${name}: cell '${c}' is not a number: ${l}`);
        }
        return v;
      });
      return Object.fromEntries(names.map((n, i) => [n, cells[i]]));
    });
}

/**
 * The gate lines of a run's `output.txt`, as printed: label and pass flag. A gate line is any line
 * that opens with two bracketed tags, as in `  [PASS] [tripwire] (4c) label: detail`. Each one
 * must read `[PASS]` or `[FAIL]`, then one space, a `[kind]` tag, one space and an `(id) label:`
 * head, or the parse throws with its text. There is at least one.
 */
export function parseGates(outputTxt: string): { passed: boolean; label: string }[] {
  const gates = outputTxt
    .split('\n')
    .filter((l) => /^\s*\[[^\]]*\]\s*\[[^\]]*\]/.test(l))
    .map((l) => {
      const m = l.match(/^\s*\[(PASS|FAIL)\] \[\w+\] (\([0-9a-z]+\) [^:]+):/);
      if (!m) throw new Error(`output.txt: unparsed gate line: ${l.trim()}`);
      return { passed: m[1] === 'PASS', label: m[2] };
    });
  if (gates.length === 0) throw new Error('output.txt: no gate lines');
  return gates;
}
