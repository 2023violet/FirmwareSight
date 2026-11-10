import { describe, expect, it } from 'vitest';

import { plot } from './trendGeometry';

const BOX = { width: 100, height: 50, pad: 5 };

describe('trend geometry', () => {
  it('breaks the line at an Unknown value and never places it at zero', () => {
    const result = plot([10, null, 30, 40], BOX);
    expect(result.segments).toHaveLength(2);
    expect(result.points.map((p) => p.index)).toEqual([0, 2, 3]);
    expect(result.gaps).toEqual([result.xs[1]]);
    expect(result.min).toBeGreaterThan(0);
  });

  it('is deterministic and spans the box', () => {
    const a = plot([1, 2, 3], BOX);
    expect(plot([1, 2, 3], BOX)).toEqual(a);
    expect(a.xs[0]).toBe(5);
    expect(a.xs[2]).toBe(95);
    expect(a.points[0]?.y).toBeGreaterThan(a.points[2]?.y ?? Infinity);
  });

  it('keeps a flat series and a single point inside the box', () => {
    const flat = plot([7, 7], BOX);
    expect(flat.points.every((p) => p.y > 5 && p.y < 45)).toBe(true);
    const one = plot([7], BOX);
    expect(one.xs[0]).toBe(50);
  });

  it('scales a reference line with the values', () => {
    const result = plot([60, 90], BOX, [100, 100]);
    expect(result.max).toBeGreaterThan(100);
    const capY = result.reference[0] ?? 0;
    expect(capY).toBeLessThan(result.points[1]?.y ?? 0);
  });

  it('draws nothing for an all-Unknown series', () => {
    const result = plot([null, null], BOX);
    expect(result.segments).toEqual([]);
    expect(result.gaps).toHaveLength(2);
  });
});
