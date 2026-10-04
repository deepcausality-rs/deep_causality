// A member of the repair crew, standing: hard hat, vest with a reflective band, arms and legs. Drawn
// facing the camera at its feet's position in the frame, `height` px tall.
import React from "react";
import { C } from "../tokens";

export const Worker: React.FC<{ x: number; y: number; height: number; tone?: string }> = ({ x, y, height, tone = C.fg0 }) => {
  // The figure is designed 100 units tall, feet at the origin, up is negative.
  const s = height / 100;
  return (
    <g transform={`translate(${x.toFixed(1)} ${y.toFixed(1)}) scale(${s.toFixed(3)})`}>
      <ellipse cx={0} cy={0} rx={16} ry={4} fill="#000" opacity={0.5} />
      {/* legs */}
      <rect x={-9} y={-46} width={7.5} height={46} rx={3} fill={tone} />
      <rect x={1.5} y={-46} width={7.5} height={46} rx={3} fill={tone} />
      {/* arms */}
      <rect x={-18} y={-78} width={6.5} height={32} rx={3.2} fill={tone} transform="rotate(8 -14.75 -78)" />
      <rect x={11.5} y={-78} width={6.5} height={32} rx={3.2} fill={tone} transform="rotate(-8 14.75 -78)" />
      {/* torso in a vest, with its reflective band */}
      <rect x={-12.5} y={-81} width={25} height={38} rx={5} fill={tone} />
      <rect x={-12.5} y={-62} width={25} height={4.5} fill={C.bg0} opacity={0.55} />
      {/* head */}
      <circle cx={0} cy={-89} r={7.5} fill={tone} />
      {/* hard hat: dome and brim */}
      <path d="M -8.5 -93 A 8.5 8 0 0 1 8.5 -93 Z" fill={tone} />
      <rect x={-12} y={-94.5} width={24} height={3} rx={1.5} fill={tone} />
      <rect x={-12} y={-94.5} width={24} height={3} rx={1.5} fill="none" stroke={C.bg0} strokeWidth={1} opacity={0.5} />
    </g>
  );
};
