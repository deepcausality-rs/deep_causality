// The world in 3D: a block of the valley cut across the slope at the drone. The cut face is the
// cross-section; the top surface carries what the drone has judged, the crew, the line and the drone.
import React from "react";
import { C, MONO, WORLD_W, H } from "../tokens";
import { camera, lambert, project, type Camera, type V3 } from "../world/projection";
import { elevation, LINE_ACROSS, PADS_ALONG } from "../world/terrain";
import { droneAt, patchesAt, trackTo, world, type Part } from "../data";
import { Worker } from "./Worker";

export type WorldOptions = {
  part: Part;
  t: number;
  /** The drone judges the ground from part 2 on. */
  knows?: boolean;
  /** Read the map with the daytime rule: water becomes unsure. */
  dayRule?: boolean;
  keepOut?: boolean;
  wind?: boolean;
  track?: boolean;
  target?: [number, number] | null;
  pins?: { patch: [number, number]; letter: string; danger: boolean }[];
  link?: { to: [number, number]; label: string; danger: boolean } | null;
  labels?: { at: V3; text: string }[];
  /** Turn the camera about the block, in degrees. */
  orbit?: number;
  /** Raise the camera's aim, in m, and widen its view, in degrees. */
  lift?: number;
  fov?: number;
  /** The view's size in px, where the block's centre lands, and how large the block draws. */
  width?: number;
  height?: number;
  cx?: number;
  cy?: number;
  zoom?: number;
  /** Hold the block and the camera still, cut at this distance along the line, in m; the drone
   * flies into the block and through it. Without it, the cut follows the drone. */
  fixedCut?: number;
  /** Effect Ethos verdicts to mark on the ground: approved, worth a look, or forbidden. */
  verdicts?: { patch: [number, number]; kind: "approved" | "look" | "forbidden" }[];
};

const X0 = -24;
const X1 = 100;
const DEPTH = 48;
const BASE = -10;
const LIGHT: V3 = [-0.5, 0.35, 0.8];
const TONES: Record<number, string> = {
  0: "#1b4249", // safe
  1: "#1d2935", // steep
  2: "#12313b", // water
  3: "#2c3744", // person
  4: "#26303c", // unsure
  5: "#1a2b24", // trees
};
const UNSEEN = "#111922";

function shade(hex: string, s: number): string {
  const v = [1, 3, 5].map((k) => Math.max(0, Math.min(255, Math.round(parseInt(hex.slice(k, k + 2), 16) * s))));
  return `rgb(${v[0]},${v[1]},${v[2]})`;
}
function mix(a: string, b: string, t: number): string {
  const pa = [1, 3, 5].map((k) => parseInt(a.slice(k, k + 2), 16));
  const pb = [1, 3, 5].map((k) => parseInt(b.slice(k, k + 2), 16));
  return "#" + pa.map((v, k) => Math.round(v + (pb[k] - v) * t).toString(16).padStart(2, "0")).join("");
}
const pts = (ps: number[][]) => ps.map((p) => `${p[0].toFixed(1)},${p[1].toFixed(1)}`).join(" ");

export const WorldView: React.FC<WorldOptions> = (o) => {
  const drone = droneAt(o.part, o.t);
  const fixed = o.fixedCut !== undefined;
  const cutY = o.fixedCut ?? Math.max(0, Math.min(470, drone.y));
  // A following camera aims higher as the drone climbs, so it stays in the frame; a still one holds
  // its aim.
  const droneZ = elevation(drone.x, drone.y) + drone.agl;
  const target: V3 = [38, cutY + 22, 10 + (o.lift ?? (fixed ? 0 : Math.max(0, (droneZ - 30) * 0.55)))];
  const a = ((o.orbit ?? 0) * Math.PI) / 180;
  const off: V3 = [-105, -277, 155];
  const rot: V3 = [off[0] * Math.cos(a) - off[1] * Math.sin(a), off[0] * Math.sin(a) + off[1] * Math.cos(a), off[2]];
  const cam: Camera = camera([target[0] + rot[0], target[1] + rot[1], target[2] + rot[2]], target, o.fov ?? 30, o.cx ?? 640, o.cy ?? 610, o.zoom ?? 1);
  const P = (p: V3) => project(cam, p);
  const known = o.knows ? patchesAt(o.part, o.t) : new Map();

  // Rows of the surface from the cut back, columns across the slope, 2 m apart.
  const ys: number[] = [cutY];
  for (let y = Math.ceil(cutY / 2) * 2; y < cutY + DEPTH; y += 2) if (y > cutY + 0.01) ys.push(y);
  ys.push(cutY + DEPTH);
  const quads: { d: number; p: number[][]; fill: string; edges: [boolean, boolean]; seen: boolean }[] = [];
  for (let k = 0; k + 1 < ys.length; k++) {
    for (let x = X0; x < X1; x += 2) {
      const [y0, y1] = [ys[k], ys[k + 1]];
      const c3: V3[] = [
        [x, y0, elevation(x, y0)],
        [x + 2, y0, elevation(x + 2, y0)],
        [x + 2, y1, elevation(x + 2, y1)],
        [x, y1, elevation(x, y1)],
      ];
      const lam = 0.5 + 0.5 * lambert(c3[0], c3[1], c3[3], LIGHT);
      const patch = known.get(`${Math.floor((x + 1) / 4)},${Math.floor((y0 + y1) / 8)}`);
      let fill = UNSEEN;
      if (patch) {
        const g = o.dayRule && patch.g === 2 ? 4 : patch.g;
        const certainty = Math.max(0.45, Math.min(1, 1.3 - patch.sigma * 0.2));
        fill = mix(UNSEEN, TONES[g], certainty);
      }
      const p = c3.map(P);
      quads.push({
        d: (p[0][2] + p[1][2] + p[2][2] + p[3][2]) / 4,
        p,
        fill: shade(fill, lam * 1.15),
        edges: [x % 4 === 0, Math.abs(y0 / 4 - Math.round(y0 / 4)) < 1e-6],
        seen: !!patch,
      });
    }
  }
  quads.sort((m, n) => n.d - m.d);

  // The cut face: the cross-section at the drone.
  const profile: V3[] = [];
  for (let x = X0; x <= X1; x += 0.5) profile.push([x, cutY, elevation(x, cutY)]);
  const face = [...profile.map(P), P([X1, cutY, BASE]), P([X0, cutY, BASE])];
  const left = [P([X0, cutY, 0]), P([X0, cutY + DEPTH, 0]), P([X0, cutY + DEPTH, BASE]), P([X0, cutY, BASE])];

  const lines: React.ReactNode[] = [];
  const seg = (key: string, p: V3[], stroke: string, w: number, extra: React.SVGProps<SVGPolylineElement> = {}) =>
    lines.push(<polyline key={key} points={pts(p.map(P))} fill="none" stroke={stroke} strokeWidth={w} strokeLinecap="round" strokeLinejoin="round" {...extra} />);
  const onGround = (x: number, y: number, lift = 0.15): V3 => [x, y, elevation(x, y) + lift];
  const inBlock = (y: number) => y >= cutY && y <= cutY + DEPTH;

  // Water ripples on the creek, anchored to the world so they slide with the block.
  for (let wy = Math.ceil(cutY / 9) * 9 + 3; wy < cutY + DEPTH - 2; wy += 9) {
    for (const wx of [-18, -8, 2]) {
      seg(`w${wx},${wy}`, [onGround(wx, wy, 0.05), onGround(wx + 2.5, wy + 1.6, 0.05), onGround(wx + 5, wy, 0.05), onGround(wx + 7.5, wy + 1.6, 0.05)], C.accent, 1.3, { opacity: 0.45 });
    }
  }
  // The power line along the slope and the tower on a pad inside the block. The height is set
  // dressing; the simulation models neither.
  const padZ = elevation(LINE_ACROSS, PADS_ALONG[1]);
  const top = padZ + 20;
  for (const [xo, zo] of [[-5, 0], [0, -7], [5, 0]]) {
    seg(`c${xo}`, [[LINE_ACROSS + xo, cutY, top + zo], [LINE_ACROSS + xo, cutY + DEPTH, top + zo]], C.fg1, 1.1, { opacity: 0.6 });
  }
  for (const py of PADS_ALONG.filter((py) => inBlock(py))) {
    for (const dx of [-3.5, 3.5]) for (const dy of [-3.5, 3.5]) seg(`t${py}${dx}${dy}`, [[LINE_ACROSS + dx, py + dy, padZ], [LINE_ACROSS + dx * 0.25, py + dy * 0.25, top]], C.fg2, 1.4);
    for (const z of [top, top - 7]) seg(`a${py}${z}`, [[LINE_ACROSS - 5, py, z], [LINE_ACROSS + 5, py, z]], C.fg2, 1.4);
  }
  // The drone's ground track.
  if (o.track) {
    const tr = trackTo(o.part, o.t).filter(([, y]) => inBlock(y));
    if (tr.length > 1) seg("track", tr.map(([x, y]) => onGround(x, y, 0.3)), C.fg1, 1.6, { strokeDasharray: "5 6", opacity: 0.75 });
  }
  // The landing patch the controller chose.
  if (o.target) {
    const [i, j] = o.target;
    const r: V3[] = [onGround(i * 4, j * 4), onGround(i * 4 + 4, j * 4), onGround(i * 4 + 4, j * 4 + 4), onGround(i * 4, j * 4 + 4), onGround(i * 4, j * 4)];
    if (inBlock(j * 4 + 2)) seg("target", r, C.accent, 2.6);
  }
  // A 13 m keep-out ring around each person.
  const crew = world.crew.filter(([, y]) => y > cutY - 13 && y < cutY + DEPTH + 13);
  if (o.keepOut) {
    crew.forEach(([cx, cy], n) => {
      const ring: V3[] = [];
      for (let s = 0; s <= 72; s++) {
        const [rx, ry] = [cx + 13 * Math.cos((s * Math.PI) / 36), cy + 13 * Math.sin((s * Math.PI) / 36)];
        if (inBlock(ry)) ring.push(onGround(rx, ry, 0.1));
      }
      if (ring.length > 1) seg(`ring${n}`, ring, C.fg2, 1.1, { strokeDasharray: "5 6", opacity: 0.75 });
    });
  }

  // Verdicts of the Effect Ethos, marked on the patches they judged.
  for (const [n, v] of (o.verdicts ?? []).entries()) {
    const [i, j] = v.patch;
    if (!inBlock(j * 4 + 2)) continue;
    if (v.kind === "approved" || v.kind === "look") {
      const r: V3[] = [onGround(i * 4 + 0.4, j * 4 + 0.4, 0.2), onGround(i * 4 + 3.6, j * 4 + 0.4, 0.2), onGround(i * 4 + 3.6, j * 4 + 3.6, 0.2), onGround(i * 4 + 0.4, j * 4 + 3.6, 0.2), onGround(i * 4 + 0.4, j * 4 + 0.4, 0.2)];
      seg(`v${n}`, r, v.kind === "approved" ? C.accent : C.fg1, 1.3, v.kind === "look" ? { strokeDasharray: "3 3", opacity: 0.7 } : { opacity: 0.55 });
    } else {
      const col = C.fg1;
      seg(`va${n}`, [onGround(i * 4 + 0.8, j * 4 + 0.8, 0.2), onGround(i * 4 + 3.2, j * 4 + 3.2, 0.2)], col, 1.8);
      seg(`vb${n}`, [onGround(i * 4 + 3.2, j * 4 + 0.8, 0.2), onGround(i * 4 + 0.8, j * 4 + 3.2, 0.2)], col, 1.8);
    }
  }

  // Objects drawn over the terrain: the crew, pins, the drone, its shadow, cone and height. Workers
  // and the drone are drawn at three times their size so they read at this distance.
  const objects: React.ReactNode[] = [];
  crew
    .filter(([, y]) => inBlock(y))
    .sort((a, b) => P([b[0], b[1], 0])[2] - P([a[0], a[1], 0])[2])
    .forEach(([cx, cy], n) => {
      const g = elevation(cx, cy);
      const feet = P([cx, cy, g]);
      const head = P([cx, cy, g + 1.8 * 3]);
      objects.push(<Worker key={`crew${n}`} x={feet[0]} y={feet[1]} height={Math.max(14, feet[1] - head[1])} />);
    });
  for (const pin of o.pins ?? []) {
    const [i, j] = pin.patch;
    const [px, py] = [i * 4 + 2, j * 4 + 2];
    if (!inBlock(py)) continue;
    const col = pin.danger ? C.danger : C.fg0;
    const g = elevation(px, py);
    const [b, t2] = [P([px, py, g]), P([px, py, g + 7])];
    objects.push(
      <g key={`pin${pin.letter}`}>
        <line x1={b[0]} y1={b[1]} x2={t2[0]} y2={t2[1]} stroke={col} strokeWidth={1.6} />
        <circle cx={b[0]} cy={b[1]} r={3} fill={col} />
        <circle cx={t2[0]} cy={t2[1] - 11} r={12} fill={C.bg0} stroke={col} strokeWidth={2} />
        <text x={t2[0]} y={t2[1] - 6.5} fill={col} fontFamily={MONO} fontSize={14} fontWeight={600} textAnchor="middle">{pin.letter}</text>
      </g>,
    );
  }
  // A following cut draws the drone just behind the cut face; a still block draws it where it is.
  const droneY = fixed ? drone.y : cutY + 1;
  const ground = elevation(drone.x, droneY);
  const D: V3 = [drone.x, droneY, ground + drone.agl];
  if (o.link) {
    const [lx, ly] = o.link.to;
    const [la, lb] = [P([drone.x, droneY, ground + 0.3]), P([lx, ly, elevation(lx, ly) + 0.3])];
    const col = o.link.danger ? C.danger : C.fg2;
    const [mx, my] = [(la[0] + lb[0]) / 2, (la[1] + lb[1]) / 2];
    objects.push(
      <g key="link">
        <line x1={la[0]} y1={la[1]} x2={lb[0]} y2={lb[1]} stroke={col} strokeWidth={o.link.danger ? 2.4 : 1.4} strokeDasharray="4 6" />
        <rect x={mx - 32} y={my - 34} width={64} height={28} rx={4} fill={C.bg1} stroke={o.link.danger ? C.danger : C.line1} />
        <text x={mx} y={my - 14} fill={o.link.danger ? C.danger : C.fg1} fontFamily={MONO} fontSize={17} textAnchor="middle">{o.link.label}</text>
      </g>,
    );
  }
  if (o.knows && drone.agl > 0.5 && fixed) {
    // The sensor's view: a pyramid from the drone down to its square footprint on the ground.
    const half = Math.max(drone.agl * 0.577, 2);
    const corners: [number, number][] = [[-1, -1], [1, -1], [1, 1], [-1, 1]].map(([a, b]) => [drone.x + a * half, droneY + b * half]);
    corners.forEach(([x, y], n) => {
      const [c0, c1] = [P(D), P(onGround(x, y, 0.2))];
      objects.push(<line key={`ray${n}`} x1={c0[0]} y1={c0[1]} x2={c1[0]} y2={c1[1]} stroke={C.accent} strokeWidth={1.1} opacity={0.45} />);
    });
    const edge: V3[] = [];
    corners.forEach(([x, y], n) => {
      const [nx, ny] = corners[(n + 1) % 4];
      for (let s = 0; s < 12; s++) edge.push(onGround(x + ((nx - x) * s) / 12, y + ((ny - y) * s) / 12, 0.2));
    });
    edge.push(edge[0]);
    objects.push(<polyline key="foot" points={pts(edge.map(P))} fill="none" stroke={C.accent} strokeWidth={1.6} opacity={0.75} />);
  }
  if (o.knows && drone.agl > 0.5 && !fixed) {
    const half = Math.max(drone.agl * 0.577, 2);
    const [c0, c1, c2] = [P(D), P([drone.x - half, cutY, elevation(drone.x - half, cutY)]), P([drone.x + half, cutY, elevation(drone.x + half, cutY)])];
    objects.push(<polygon key="cone" points={pts([c0, c1, c2])} fill={C.accent} opacity={0.08} />);
    objects.push(<polyline key="conel" points={pts([c1, c0, c2])} fill="none" stroke={C.accent} strokeWidth={1.2} opacity={0.5} />);
    const foot: V3[] = [];
    for (let x = drone.x - half; x <= drone.x + half + 0.01; x += 0.5) foot.push([x, cutY, elevation(x, cutY) + 0.1]);
    objects.push(<polyline key="foot" points={pts(foot.map(P))} fill="none" stroke={C.accent} strokeWidth={2.4} opacity={0.8} />);
  }
  if (drone.agl > 0.5) {
    const sh: number[][] = [];
    for (let s = 0; s < 36; s++) sh.push(P([drone.x + 2.2 * Math.cos((s * Math.PI) / 18), droneY + 2.2 * Math.sin((s * Math.PI) / 18), ground + 0.1]));
    objects.push(<polygon key="shadow" points={pts(sh)} fill="#000" opacity={0.5} />);
    const [g0, g1] = [P(D), P([drone.x, droneY, ground + 0.2])];
    objects.push(<line key="drop" x1={g0[0]} y1={g0[1] + 6} x2={g1[0]} y2={g1[1]} stroke={C.accent} strokeWidth={1.6} strokeDasharray="5 5" />);
    const mid = P([drone.x, droneY, ground + drone.agl / 2]);
    objects.push(<text key="agl" x={mid[0] + 12} y={mid[1] + 5} fill={C.accent} fontFamily={MONO} fontSize={17}>{`${Math.round(drone.agl)} m`}</text>);
  }
  const arm = 1.8;
  const rotors = [[1, 1], [1, -1], [-1, 1], [-1, -1]].map(([ax, ay], n) => {
    const tip: V3 = [D[0] + ax * arm, D[1] + ay * arm, D[2]];
    const ring: number[][] = [];
    for (let s = 0; s < 24; s++) ring.push(P([tip[0] + 0.95 * Math.cos((s * Math.PI) / 12), tip[1] + 0.95 * Math.sin((s * Math.PI) / 12), tip[2] + 0.25]));
    const [b0, b1] = [P(D), P(tip)];
    return (
      <g key={`rotor${n}`}>
        <line x1={b0[0]} y1={b0[1]} x2={b1[0]} y2={b1[1]} stroke={C.fg0} strokeWidth={2.6} />
        <polygon points={pts(ring)} fill={C.accent} fillOpacity={0.22} stroke={C.accent} strokeWidth={1.6} />
      </g>
    );
  });
  const body = P(D);

  // Wind, downslope, sliding with flight time.
  const wind: React.ReactNode[] = [];
  if (o.wind) {
    const phase = (o.t * 2) % 6;
    [[78, 10], [92, 30], [48, 38]].forEach(([wx, dy], n) => {
      const line: V3[] = [];
      for (let s = 0; s < 6; s++) line.push(onGround(wx - phase - s * 3, cutY + dy, 3));
      const p = line.map(P);
      const [e0, e1] = [p[p.length - 1], p[p.length - 2]];
      const ang = Math.atan2(e0[1] - e1[1], e0[0] - e1[0]);
      wind.push(
        <g key={`wind${n}`} opacity={0.45}>
          <polyline points={pts(p)} fill="none" stroke={C.accent} strokeWidth={1.3} />
          {[-0.5, 0.5].map((s) => (
            <line key={s} x1={e0[0]} y1={e0[1]} x2={e0[0] - 8 * Math.cos(ang + s)} y2={e0[1] - 8 * Math.sin(ang + s)} stroke={C.accent} strokeWidth={1.3} />
          ))}
        </g>,
      );
    });
  }

  return (
    <svg width={o.width ?? WORLD_W} height={o.height ?? H} viewBox={`0 0 ${o.width ?? WORLD_W} ${o.height ?? H}`} style={{ position: "absolute", left: 0, top: 0 }}>
      {quads.map((q, n) => (
        <g key={n}>
          <polygon points={pts(q.p)} fill={q.fill} stroke={q.fill} strokeWidth={0.8} />
          {q.seen && q.edges[0] && <line x1={q.p[0][0]} y1={q.p[0][1]} x2={q.p[3][0]} y2={q.p[3][1]} stroke="#3a5263" strokeWidth={0.7} />}
          {q.seen && q.edges[1] && <line x1={q.p[0][0]} y1={q.p[0][1]} x2={q.p[1][0]} y2={q.p[1][1]} stroke="#3a5263" strokeWidth={0.7} />}
        </g>
      ))}
      <polygon points={pts(left)} fill="#0b1219" stroke={C.line2} />
      <polygon points={pts(face)} fill="#0d151d" />
      {Array.from({ length: Math.floor((X1 - X0) / 10) + 1 }, (_, k) => X0 + 4 + k * 10).map((x) => {
        const [b, t2] = [P([x, cutY, BASE]), P([x, cutY, elevation(x, cutY)])];
        return <line key={`v${x}`} x1={b[0]} y1={b[1]} x2={t2[0]} y2={t2[1]} stroke="#16202a" strokeWidth={0.8} />;
      })}
      <polygon points={pts([P([X0, cutY, 0]), P([12, cutY, 0]), P([12, cutY, -2.5]), P([X0, cutY, -2.5])])} fill="#10343e" opacity={0.9} />
      <polyline points={pts(profile.map(P))} fill="none" stroke={C.fg1} strokeWidth={2.2} />
      {lines}
      {wind}
      {objects}
      {rotors}
      <circle cx={body[0]} cy={body[1]} r={5.5} fill={C.fg0} />
      {(o.labels ?? []).map((l, n) => {
        const p = P(l.at);
        return (
          <text key={`l${n}`} x={p[0]} y={p[1]} fill={C.fg2} fontFamily={MONO} fontSize={15} textAnchor="middle">
            {l.text}
          </text>
        );
      })}
    </svg>
  );
};
