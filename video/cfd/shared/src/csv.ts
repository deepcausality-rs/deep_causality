/**
 * The reader for the CSV tables an example commits with its run, shared by every cut's data module.
 * It throws on input it cannot read whole, so a malformed trace fails the load before any frame
 * renders.
 *
 * Free of React and three.js, so scripts run under plain Node import it.
 */

export type Row = Record<string, number>;

/**
 * Parse the two-row-header CSV `write_rows` emits: names, then `#units`, then data. Every data row
 * has one cell per name, and every cell is a number or the literal `NaN` some columns carry.
 * `name` is the file name the errors report.
 */
export function parseRows(csv: string, name: string): Row[] {
  const lines = csv.trim().split(/\r?\n/);
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
