/** Number helpers shared by the tutorial run figures (`CorridorRun`, `WeatherRun`, `RetroRun`). */
export const f1 = (v: number) => v.toFixed(1);
export const f2 = (v: number) => v.toFixed(2);
export const r2 = (v: number) => Math.round(v * 100) / 100;
export const pct = (v: number, of: number) => `${((v / of) * 100).toFixed(2)}%`;
