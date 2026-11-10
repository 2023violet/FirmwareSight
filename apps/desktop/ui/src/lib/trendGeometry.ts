/**
 * Geometry for the History trend charts (ADR-0041 D4). Pure layout: it places values Core already decided
 * and computes no byte, delta or headroom (AGENTS.md §3).
 *
 * Two rules matter more than the arithmetic:
 *
 * - **Unknown is a gap.** A `null` value breaks the line and gets no point; it is never placed at zero.
 * - **The scale is the data's.** The y range is the known values (and any reference line), padded so a
 *   flat series still reads as a line and not as the floor.
 */

export interface PlotBox {
  readonly width: number;
  readonly height: number;
  /** Inner padding on every side, in viewBox units. */
  readonly pad: number;
}

export interface PlottedPoint {
  readonly index: number;
  readonly x: number;
  readonly y: number;
  readonly value: number;
}

export interface Plot {
  /** One SVG path per unbroken run of known values. */
  readonly segments: readonly string[];
  readonly points: readonly PlottedPoint[];
  /** x of every Unknown value, so the chart can mark the gap. */
  readonly gaps: readonly number[];
  /** y of each reference value (a capacity line), in input order; `null` where none. */
  readonly reference: readonly (number | null)[];
  readonly min: number;
  readonly max: number;
  /** x of every index, known or not. */
  readonly xs: readonly number[];
}

export function plot(
  values: readonly (number | null)[],
  box: PlotBox,
  reference: readonly (number | null)[] = [],
): Plot {
  const known = [...values, ...reference].filter((v): v is number => v !== null);
  let min = known.length === 0 ? 0 : Math.min(...known);
  let max = known.length === 0 ? 1 : Math.max(...known);
  if (min === max) {
    // A flat series sits mid-chart instead of on an edge.
    const spread = Math.max(1, Math.abs(max) * 0.05);
    min -= spread;
    max += spread;
  } else {
    const pad = (max - min) * 0.08;
    min -= pad;
    max += pad;
  }
  const innerW = box.width - box.pad * 2;
  const innerH = box.height - box.pad * 2;
  const xOf = (index: number) =>
    values.length <= 1 ? box.pad + innerW / 2 : box.pad + (innerW * index) / (values.length - 1);
  const yOf = (value: number) => box.pad + innerH - ((value - min) / (max - min)) * innerH;

  const xs = values.map((_, index) => round(xOf(index)));
  const points: PlottedPoint[] = [];
  const gaps: number[] = [];
  const segments: string[] = [];
  let run: string[] = [];
  values.forEach((value, index) => {
    if (value === null) {
      gaps.push(round(xOf(index)));
      if (run.length > 0) {
        segments.push(run.join(' '));
        run = [];
      }
      return;
    }
    const x = round(xOf(index));
    const y = round(yOf(value));
    points.push({ index, x, y, value });
    run.push(`${run.length === 0 ? 'M' : 'L'}${x} ${y}`);
  });
  if (run.length > 0) {
    segments.push(run.join(' '));
  }

  return {
    segments,
    points,
    gaps,
    reference: reference.map((v) => (v === null ? null : round(yOf(v)))),
    min,
    max,
    xs,
  };
}

function round(value: number): number {
  return Math.round(value * 100) / 100;
}
