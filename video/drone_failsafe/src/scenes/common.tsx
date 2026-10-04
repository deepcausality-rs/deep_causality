// Pieces the scenes share: fades, an eyebrow, a block of text.
import React from "react";
import { interpolate } from "remotion";
import { C, MONO, SANS } from "../tokens";

export const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;

/** 0 before `from`, rising to 1 over `len` frames. */
export const appear = (frame: number, from: number, len = 14) => interpolate(frame, [from, from + len], [0, 1], clamp);

export const Eyebrow: React.FC<{ text: string; style?: React.CSSProperties }> = ({ text, style }) => (
  <div style={{ fontFamily: MONO, fontSize: 20, letterSpacing: 2, color: C.fg2, textTransform: "uppercase", ...style }}>{text}</div>
);

export const Text: React.FC<{ size: number; color?: string; weight?: number; style?: React.CSSProperties; children: React.ReactNode }> = ({ size, color = C.fg0, weight = 400, style, children }) => (
  <div style={{ fontFamily: SANS, fontSize: size, lineHeight: 1.22, color, fontWeight: weight, textWrap: "balance", ...style } as React.CSSProperties}>{children}</div>
);
