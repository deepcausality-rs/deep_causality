// A pinhole camera: projects world points (across, along, up), in m, to the frame, in px.
export type V3 = [number, number, number];
export type Camera = { eye: V3; f: V3; r: V3; u: V3; foc: number; cx: number; cy: number };

const sub = (a: V3, b: V3): V3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const dot = (a: V3, b: V3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const cross = (a: V3, b: V3): V3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const norm = (a: V3): V3 => {
  const l = Math.hypot(a[0], a[1], a[2]);
  return [a[0] / l, a[1] / l, a[2] / l];
};

/** `zoom` scales the image about (cx, cy); 1 frames the block for the 1920 px frame. */
export function camera(eye: V3, target: V3, fovDeg: number, cx: number, cy: number, zoom = 1): Camera {
  const f = norm(sub(target, eye));
  const r = norm(cross(f, [0, 0, 1]));
  const u = cross(r, f);
  const foc = (960 * zoom) / Math.tan((fovDeg * Math.PI) / 360);
  return { eye, f, r, u, foc, cx, cy };
}

/** The point's position in the frame, and its depth along the view. */
export function project(c: Camera, p: V3): [number, number, number] {
  const d = sub(p, c.eye);
  const z = dot(d, c.f);
  return [c.cx + (dot(d, c.r) / z) * c.foc, c.cy - (dot(d, c.u) / z) * c.foc, z];
}

export const lambert = (a: V3, b: V3, c: V3, light: V3): number => {
  let n = norm(cross(sub(b, a), sub(c, a)));
  if (n[2] < 0) n = [-n[0], -n[1], -n[2]];
  return Math.max(0, dot(n, norm(light)));
};
