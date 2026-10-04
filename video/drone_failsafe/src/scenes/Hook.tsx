// Melbourne 2023 and Orlando 2024: two fail-safes that fired on one condition. Numbers from the
// accident reports the tutorial cites: ATSB AO-2023-033 and NTSB DCA25LA065.
import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C, MONO, SANS } from "../tokens";
import { type SceneSpec } from "../data";
import { Frame1080 } from "../views/Hud";

const DRONES = 500;
const IN_HARBOUR = 427;
// A fixed shuffle picks which dots fall, so every render draws the same frame.
const order = Array.from({ length: DRONES }, (_, k) => k).sort((a, b) => ((a * 7919) % 503) - ((b * 7919) % 503));
const falls = new Set(order.slice(0, IN_HARBOUR));

const fade = (frame: number, a: number, b: number) => interpolate(frame, [a, a + 12, b - 12, b], [0, 1, 1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

export const Hook: React.FC<{ spec: SceneSpec }> = ({ spec }) => {
  const frame = useCurrentFrame();
  const at = (k: number) => spec.cues[k].from;
  const end = spec.durationInFrames;
  const melbourne = fade(frame, 0, at(3) - 6);
  const orlando = fade(frame, at(3), at(5) - 4);
  const closing = fade(frame, at(5), end + 12);
  const fallProgress = interpolate(frame, [at(2), at(2) + spec.cues[2].durationInFrames], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const drift = interpolate(frame, [at(1), at(2)], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  // The counter counts the dots that have reached the water.
  const fallOf = (k: number) => (falls.has(k) ? Math.max(0, Math.min(1, (fallProgress * 1.25 - (order.indexOf(k) / IN_HARBOUR) * 0.7) / 0.3)) : 0);
  const counted = Array.from({ length: DRONES }, (_, k) => k).filter((k) => fallOf(k) >= 1).length;
  const shift = interpolate(frame, [at(4), at(4) + 40], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  const struck = interpolate(frame, [at(4) + spec.cues[4].durationInFrames * 0.55, at(4) + spec.cues[4].durationInFrames * 0.55 + 10], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  return (
    <AbsoluteFill style={{ backgroundColor: C.bg0 }}>
      <Frame1080>
        <g opacity={melbourne}>
          <text x={120} y={180} fill={C.fg2} fontFamily={MONO} fontSize={22} letterSpacing={2}>MELBOURNE · DOCKLANDS · 14 JULY 2023</text>
          <text x={120} y={270} fill={C.fg0} fontFamily={SANS} fontSize={64}>500 show drones</text>
          <text x={120} y={330} fill={C.fg1} fontFamily={SANS} fontSize={30} opacity={drift}>wind at more than twice their limit</text>
          <text x={120} y={520} fill={C.fg0} fontFamily={MONO} fontSize={96} opacity={fallProgress > 0 ? 1 : 0}>{counted}</text>
          <text x={120} y={572} fill={C.fg1} fontFamily={SANS} fontSize={30} opacity={fallProgress > 0 ? 1 : 0}>in the harbour after their fail-safes fired</text>
          {Array.from({ length: DRONES }, (_, k) => {
            const [col, row] = [k % 25, Math.floor(k / 25)];
            const x0 = 1020 + col * 26 + Math.sin(k * 1.7) * 10 * drift;
            const y0 = 200 + row * 20 + Math.cos(k * 2.3) * 6 * drift;
            const fall = fallOf(k);
            const y = y0 + (860 - y0) * fall * fall;
            return <circle key={k} cx={x0} cy={y} r={3.2} fill={fall >= 1 ? C.fg2 : C.accent} opacity={fall >= 1 ? 0.6 : 0.9} />;
          })}
          <rect x={1000} y={850} width={680} height={40} fill="#0f2c35" opacity={0.9} />
          <text x={1680} y={918} fill={C.fg2} fontFamily={MONO} fontSize={16} textAnchor="end">Victoria Harbour</text>
          <text x={120} y={1000} fill={C.fg2} fontFamily={MONO} fontSize={16}>ATSB AO-2023-033 · ABC News, 16 July 2023</text>
        </g>
        <g opacity={orlando}>
          <text x={120} y={180} fill={C.fg2} fontFamily={MONO} fontSize={22} letterSpacing={2}>ORLANDO · LAKE EOLA · 21 DECEMBER 2024</text>
          <text x={120} y={270} fill={C.fg0} fontFamily={SANS} fontSize={64}>setup errors</text>
          <text x={120} y={330} fill={C.fg1} fontFamily={SANS} fontSize={30}>shifted the show about 18 m toward the crowd</text>
          <text x={120} y={500} fill={C.danger} fontFamily={SANS} fontSize={44} opacity={struck}>A drone struck a seven-year-old boy.</text>
          <rect x={1060 + 0} y={300 + shift * 117} width={520} height={260} fill="none" stroke={C.accent} strokeWidth={2} />
          <text x={1070} y={290 + shift * 117} fill={C.accent} fontFamily={MONO} fontSize={16}>show area</text>
          <line x1={1000} y1={760} x2={1680} y2={760} stroke={C.fg1} strokeWidth={2} strokeDasharray="8 8" />
          <text x={1000} y={792} fill={C.fg1} fontFamily={MONO} fontSize={16}>spectators</text>
          <line x1={1620} y1={560} x2={1620} y2={560 + shift * 117} stroke={C.fg1} strokeWidth={1.5} opacity={shift} />
          <text x={1632} y={565 + shift * 60} fill={C.fg1} fontFamily={MONO} fontSize={16} opacity={shift}>18 m</text>
          <text x={120} y={1000} fill={C.fg2} fontFamily={MONO} fontSize={16}>NTSB DCA25LA065, September 2026</text>
        </g>
        <g opacity={closing}>
          <text x={120} y={300} fill={C.fg2} fontFamily={MONO} fontSize={22} letterSpacing={2}>EACH FAIL-SAFE FIRED ON ONE CONDITION</text>
          <text x={120} y={390} fill={C.fg0} fontFamily={SANS} fontSize={56}>a lost position</text>
          <text x={120} y={460} fill={C.fg0} fontFamily={SANS} fontSize={56}>a crossed line</text>
          <g opacity={interpolate(frame, [at(6), at(6) + 15], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })}>
            <text x={120} y={640} fill={C.accent} fontFamily={MONO} fontSize={40} letterSpacing={3}>UNSAFE CONTROL ACTION</text>
            <text x={120} y={700} fill={C.fg1} fontFamily={SANS} fontSize={28}>unsafe "in a particular context and worst-case environment"</text>
            <text x={120} y={1000} fill={C.fg2} fontFamily={MONO} fontSize={16}>N. G. Leveson and J. P. Thomas, STPA Handbook, MIT, 2018</text>
          </g>
        </g>
      </Frame1080>
    </AbsoluteFill>
  );
};
