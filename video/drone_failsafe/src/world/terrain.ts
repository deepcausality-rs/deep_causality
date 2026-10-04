// The ground the Rust world exports on a 2 m grid, with bilinear elevation between grid points.
import { world } from "../data";

const at = (ix: number, iy: number) => {
  const cx = Math.max(0, Math.min(world.nx - 1, ix));
  const cy = Math.max(0, Math.min(world.ny - 1, iy));
  return cx * world.ny + cy;
};

export function elevation(x: number, y: number): number {
  const gx = (x - world.x0) / world.step;
  const gy = (y - world.y0) / world.step;
  const ix = Math.floor(gx);
  const iy = Math.floor(gy);
  const sx = gx - ix;
  const sy = gy - iy;
  const e = world.elevation;
  const a = e[at(ix, iy)] * (1 - sx) + e[at(ix + 1, iy)] * sx;
  const b = e[at(ix, iy + 1)] * (1 - sx) + e[at(ix + 1, iy + 1)] * sx;
  return a * (1 - sy) + b * sy;
}

/** 0 water, 1 grass, 2 road, 3 rock, 4 pad, 5 ravine, 6 trees: the nearest grid point's surface. */
export function surface(x: number, y: number): number {
  return world.surface[at(Math.round((x - world.x0) / world.step), Math.round((y - world.y0) / world.step))];
}

export const PADS_ALONG = [150, 450, 750];
export const LINE_ACROSS = 60;
