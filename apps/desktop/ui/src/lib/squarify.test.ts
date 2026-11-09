import { describe, expect, it } from 'vitest';

import { squarify } from './squarify';

const bounds = { x: 0, y: 0, width: 600, height: 400 };

describe('squarify', () => {
  it('gives every known item an area proportional to its value and fills the bounds', () => {
    const items = [{ v: 6 }, { v: 6 }, { v: 4 }, { v: 3 }, { v: 2 }, { v: 2 }, { v: 1 }];
    const placed = squarify(items, (i) => i.v, bounds);
    expect(placed).toHaveLength(items.length);
    const total = items.reduce((s, i) => s + i.v, 0);
    for (const p of placed) {
      expect(p.width * p.height).toBeCloseTo((p.item.v / total) * 600 * 400, 6);
      expect(p.x).toBeGreaterThanOrEqual(-1e-9);
      expect(p.y).toBeGreaterThanOrEqual(-1e-9);
      expect(p.x + p.width).toBeLessThanOrEqual(600 + 1e-6);
      expect(p.y + p.height).toBeLessThanOrEqual(400 + 1e-6);
    }
    const area = placed.reduce((s, p) => s + p.width * p.height, 0);
    expect(area).toBeCloseTo(600 * 400, 6);
  });

  it('is deterministic and draws nothing for an Unknown or zero value', () => {
    const items = [{ v: 10 }, { v: null }, { v: 0 }, { v: 5 }];
    const a = squarify(items, (i) => i.v, bounds);
    const b = squarify(items, (i) => i.v, bounds);
    expect(a).toEqual(b);
    expect(a.map((p) => p.item.v)).toEqual([10, 5]);
  });

  it('keeps rectangles reasonably square', () => {
    const items = Array.from({ length: 30 }, (_, i) => ({ v: 30 - i }));
    const placed = squarify(items, (i) => i.v, bounds);
    const aspects = placed.map((p) => Math.max(p.width / p.height, p.height / p.width));
    expect(Math.max(...aspects)).toBeLessThan(6);
  });

  it('returns nothing for empty bounds or an empty list', () => {
    expect(squarify([{ v: 1 }], (i) => i.v, { x: 0, y: 0, width: 0, height: 10 })).toEqual([]);
    expect(squarify([], () => 1, bounds)).toEqual([]);
  });
});
