/**
 * Squarified treemap layout (Bruls, Huizing and van Wijk, 2000): rectangles whose areas are proportional
 * to the values Core decided, laid out to keep each rectangle close to square so labels fit.
 *
 * Geometry only. The values are bytes Core already computed (ADR-0039); this never adds, rounds or
 * re-derives one, and an item without a known positive value gets no rectangle (an Unknown is listed,
 * not drawn — `AGENTS.md` 8). The same input always gives the same rectangles.
 */

export interface Rect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

export interface Placed<T> extends Rect {
  readonly item: T;
}

/** Lay `items` out in `bounds`, largest first. Items whose `value` is not a positive number are skipped. */
export function squarify<T>(items: readonly T[], value: (item: T) => number | null, bounds: Rect): Placed<T>[] {
  const sized = items
    .map((item) => ({ item, v: value(item) }))
    .filter((e): e is { item: T; v: number } => e.v !== null && Number.isFinite(e.v) && e.v > 0)
    .sort((a, b) => b.v - a.v);
  const total = sized.reduce((sum, e) => sum + e.v, 0);
  if (total <= 0 || bounds.width <= 0 || bounds.height <= 0) {
    return [];
  }
  const scale = (bounds.width * bounds.height) / total;
  const areas = sized.map((e) => ({ item: e.item, area: e.v * scale }));

  const out: Placed<T>[] = [];
  let free: Rect = bounds;
  let row: { item: T; area: number }[] = [];

  const worst = (candidate: readonly { area: number }[], side: number): number => {
    const sum = candidate.reduce((s, e) => s + e.area, 0);
    let max = 0;
    let min = Infinity;
    for (const e of candidate) {
      max = Math.max(max, e.area);
      min = Math.min(min, e.area);
    }
    const s2 = sum * sum;
    const w2 = side * side;
    return Math.max((w2 * max) / s2, s2 / (w2 * min));
  };

  const layRow = (cells: readonly { item: T; area: number }[], space: Rect): Rect => {
    const sum = cells.reduce((s, e) => s + e.area, 0);
    const horizontal = space.width >= space.height; // the row runs down the short side
    if (horizontal) {
      const width = sum / space.height;
      let y = space.y;
      for (const c of cells) {
        const height = c.area / width;
        out.push({ item: c.item, x: space.x, y, width, height });
        y += height;
      }
      return { x: space.x + width, y: space.y, width: space.width - width, height: space.height };
    }
    const height = sum / space.width;
    let x = space.x;
    for (const c of cells) {
      const width = c.area / height;
      out.push({ item: c.item, x, y: space.y, width, height });
      x += width;
    }
    return { x: space.x, y: space.y + height, width: space.width, height: space.height - height };
  };

  for (const cell of areas) {
    const side = Math.min(free.width, free.height);
    if (row.length === 0 || worst([...row, cell], side) <= worst(row, side)) {
      row.push(cell);
    } else {
      free = layRow(row, free);
      row = [cell];
    }
  }
  if (row.length > 0) {
    layRow(row, free);
  }
  return out;
}
