// What the drone knows, from above: its 4 m patch map around it, its sensor footprint, the patch it
// chose and, in part 4, the verdicts of the latest Effect Ethos round.
import React from "react";
import { C, MONO, PANEL_W, PANEL_X } from "../tokens";
import { droneAt, patchesAt, roundAt, type Part } from "../data";

const MAP_W = 500;
const MAP_H = 275;
const PX = 25 / 4; // px per m
const ACROSS_M = MAP_W / PX;
const ALONG_M = MAP_H / PX;

export type TopOptions = {
  part: Part;
  t: number;
  dayRule?: boolean;
  target?: [number, number] | null;
  showRound?: boolean;
  pins?: { patch: [number, number]; letter: string; danger: boolean }[];
};

export const TopView: React.FC<TopOptions> = (o) => {
  const top = 112;
  const drone = droneAt(o.part, o.t);
  const known = patchesAt(o.part, o.t);
  const ox = PANEL_X + 25;
  const oy = top + 56;
  // The drone sits at 40 % of the width, so more of the uphill side shows; north is along the line.
  const xLeft = drone.x - ACROSS_M * 0.4;
  const yTop = drone.y + ALONG_M / 2;
  const sx = (x: number) => ox + (x - xLeft) * PX;
  const sy = (y: number) => oy + (yTop - y) * PX;
  const cell = 4 * PX;
  const glyphs: React.ReactNode[] = [];
  for (const p of known.values()) {
    const [x0, y1] = [sx(p.i * 4), sy(p.j * 4 + 4)];
    if (x0 < ox - cell || x0 > ox + MAP_W || y1 < oy - cell || y1 > oy + MAP_H) continue;
    const g = o.dayRule && p.g === 2 ? 4 : p.g;
    const [cx, cy] = [x0 + cell / 2, y1 + cell / 2];
    const op = Math.max(0.45, Math.min(1, 1.3 - p.sigma * 0.2));
    glyphs.push(
      <g key={`${p.i},${p.j}`} opacity={op}>
        <rect x={x0} y={y1} width={cell} height={cell} fill={C.bg1} stroke={C.line1} strokeWidth={0.8} />
        {g === 0 && <circle cx={cx} cy={cy} r={2.4} fill={C.fg1} />}
        {g === 1 && <line x1={x0 + 7} y1={y1 + cell - 7} x2={x0 + cell - 7} y2={y1 + 7} stroke={C.fg2} strokeWidth={1.1} />}
        {g === 2 && <path d={`M ${x0 + 5} ${cy} q 3.75 -4 7.5 0 t 7.5 0`} fill="none" stroke={C.accent} strokeWidth={1.3} opacity={0.65} />}
        {g === 3 && (
          <>
            <circle cx={cx} cy={cy} r={7} fill="none" stroke={C.fg0} strokeWidth={1.6} />
            <circle cx={cx} cy={cy} r={2} fill={C.fg0} />
          </>
        )}
        {g === 4 && <text x={cx} y={cy + 4.5} fill={C.fg2} fontFamily={MONO} fontSize={12} textAnchor="middle">?</text>}
        {g === 5 && <text x={cx} y={cy + 4.5} fill={C.fg2} fontFamily={MONO} fontSize={12} textAnchor="middle">T</text>}
      </g>,
    );
  }
  const marks: React.ReactNode[] = [];
  if (o.showRound) {
    const round = roundAt(o.t);
    if (round && o.t - round.t < 6) {
      round.rulings.forEach((r, n) => {
        const [cx, cy] = [sx(r.patch[0] * 4 + 2), sy(r.patch[1] * 4 + 2)];
        if (cx < ox || cx > ox + MAP_W || cy < oy || cy > oy + MAP_H) return;
        const col = r.review === "Approved" ? C.accent : r.review === "Look" ? C.fg1 : C.fg2;
        marks.push(
          r.review === "Rejected" ? (
            <g key={`r${n}`} stroke={col} strokeWidth={1.4}>
              <line x1={cx - 5} y1={cy - 5} x2={cx + 5} y2={cy + 5} />
              <line x1={cx - 5} y1={cy + 5} x2={cx + 5} y2={cy - 5} />
            </g>
          ) : (
            <circle key={`r${n}`} cx={cx} cy={cy} r={9} fill="none" stroke={col} strokeWidth={1.6} strokeDasharray={r.review === "Look" ? "3 3" : undefined} />
          ),
        );
      });
    }
  }
  if (o.target) {
    const [i, j] = o.target;
    marks.push(<rect key="target" x={sx(i * 4)} y={sy(j * 4 + 4)} width={cell} height={cell} fill="none" stroke={C.accent} strokeWidth={2.2} rx={2} />);
  }
  for (const pin of o.pins ?? []) {
    const [cx, cy] = [sx(pin.patch[0] * 4 + 2), sy(pin.patch[1] * 4 + 2)];
    const col = pin.danger ? C.danger : C.fg0;
    marks.push(
      <g key={`pin${pin.letter}`}>
        <circle cx={cx} cy={cy} r={9} fill={C.bg0} stroke={col} strokeWidth={1.6} />
        <text x={cx} y={cy + 4} fill={col} fontFamily={MONO} fontSize={11} fontWeight={600} textAnchor="middle">{pin.letter}</text>
      </g>,
    );
  }
  const half = Math.max(drone.agl * 0.577, 2);
  const [dx, dy] = [sx(drone.x), sy(drone.y)];
  return (
    <g>
      <rect x={PANEL_X} y={top} width={PANEL_W} height={404} rx={8} fill={C.bg1} stroke={C.line1} />
      <text x={PANEL_X + 22} y={top + 34} fill={C.fg2} fontFamily={MONO} fontSize={14} letterSpacing={1.4}>WHAT THE DRONE KNOWS · TOP VIEW</text>
      <clipPath id="map">
        <rect x={ox} y={oy} width={MAP_W} height={MAP_H} />
      </clipPath>
      <rect x={ox} y={oy} width={MAP_W} height={MAP_H} fill={C.bg0} stroke={C.line1} />
      <g clipPath="url(#map)">
        {glyphs}
        {marks}
        {drone.agl > 0.5 && <rect x={dx - half * PX} y={dy - half * PX} width={2 * half * PX} height={2 * half * PX} fill="none" stroke={C.accent} strokeWidth={1.4} opacity={0.8} rx={2} />}
        {[[1, 1], [1, -1], [-1, 1], [-1, -1]].map(([a, b]) => (
          <g key={`${a}${b}`}>
            <line x1={dx} y1={dy} x2={dx + a * 6} y2={dy + b * 6} stroke={C.accent} strokeWidth={2} />
            <circle cx={dx + a * 6} cy={dy + b * 6} r={3.2} fill="none" stroke={C.accent} strokeWidth={1.4} />
          </g>
        ))}
      </g>
      <text x={ox} y={oy + MAP_H + 30} fill={C.fg2} fontFamily={MONO} fontSize={13}>
        {o.dayRule ? "read with the daytime rule: water turns to ?" : "· safe   / steep   ~ water   ○ person   ? unsure   ▢ sensor footprint"}
      </text>
    </g>
  );
};

/** Part 1's view from above: the controller reads its own telemetry and nothing of the ground. */
export const NoGroundView: React.FC = () => {
  const top = 112;
  return (
    <g>
      <rect x={PANEL_X} y={top} width={PANEL_W} height={404} rx={8} fill={C.bg1} stroke={C.line1} />
      <text x={PANEL_X + 22} y={top + 34} fill={C.fg2} fontFamily={MONO} fontSize={14} letterSpacing={1.4}>WHAT THE DRONE KNOWS · TOP VIEW</text>
      <rect x={PANEL_X + 25} y={top + 56} width={MAP_W} height={MAP_H} fill={C.bg0} stroke={C.line1} />
      <text x={PANEL_X + PANEL_W / 2} y={top + 56 + MAP_H / 2 - 6} fill={C.fg1} fontFamily={MONO} fontSize={17} textAnchor="middle">telemetry only</text>
      <text x={PANEL_X + PANEL_W / 2} y={top + 56 + MAP_H / 2 + 20} fill={C.fg2} fontFamily={MONO} fontSize={14} textAnchor="middle">satellites · link · battery</text>
    </g>
  );
};
