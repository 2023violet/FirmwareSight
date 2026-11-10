/**
 * The History trend charts (ADR-0041 D4): a line over stored builds, and a sparkline for one region or
 * object. Hand-written SVG, token colours only (F5).
 *
 * The chart is an aid, never the only encoding: every value it draws is also in the table beside it, so
 * the SVG is hidden from assistive technology and the table carries the names, numbers and states. A
 * pointer can still pick a point; the keyboard picks the same build from the table.
 */

import styles from './LineChart.module.css';
import { plot } from '../lib/trendGeometry';

const CHART = { width: 640, height: 168, pad: 14 };
const SPARK = { width: 160, height: 36, pad: 4 };
const POINT_RADIUS = 4;
const SPARK_RADIUS = 2;

export interface ChartMark {
  readonly index: number;
  readonly label: string;
}

export function LineChart({
  values,
  floors = [],
  marks = [],
  selected = [],
  focused = null,
  onPick,
}: {
  readonly values: readonly (number | null)[];
  /** Indices whose value is a floor (a partial total): drawn hollow. */
  readonly floors?: readonly number[];
  /** Labels above a point: the pinned baseline, a release version. */
  readonly marks?: readonly ChartMark[];
  readonly selected?: readonly number[];
  readonly focused?: number | null;
  readonly onPick?: (index: number) => void;
}) {
  const geometry = plot(values, CHART);
  return (
    <svg
      className={styles['chart']}
      viewBox={`0 0 ${CHART.width} ${CHART.height}`}
      aria-hidden="true"
      focusable="false"
    >
      <line className={styles['axis']} x1={CHART.pad} x2={CHART.width - CHART.pad} y1={CHART.height - CHART.pad} y2={CHART.height - CHART.pad} />
      {geometry.gaps.map((x) => (
        <line key={`gap-${x}`} className={styles['gap']} x1={x} x2={x} y1={CHART.pad} y2={CHART.height - CHART.pad} />
      ))}
      {geometry.segments.map((d) => (
        <path key={d} className={styles['line']} d={d} />
      ))}
      {marks.map((mark) => {
        const x = geometry.xs[mark.index];
        return x === undefined ? null : (
          <g key={`${mark.index}-${mark.label}`}>
            <line className={styles['mark']} x1={x} x2={x} y1={CHART.pad} y2={CHART.height - CHART.pad} />
            <text className={styles['markLabel']} x={x} y={CHART.pad - 3} textAnchor="middle">
              {mark.label}
            </text>
          </g>
        );
      })}
      {geometry.points.map((point) => (
        <circle
          key={point.index}
          data-index={point.index}
          className={[
            styles['point'],
            floors.includes(point.index) ? styles['floor'] : '',
            selected.includes(point.index) ? styles['selected'] : '',
            focused === point.index ? styles['focused'] : '',
          ].join(' ')}
          cx={point.x}
          cy={point.y}
          r={POINT_RADIUS}
          onClick={onPick === undefined ? undefined : () => { onPick(point.index); }}
        />
      ))}
    </svg>
  );
}

export function Sparkline({
  values,
  reference = [],
}: {
  readonly values: readonly (number | null)[];
  /** A capacity per point; drawn as a dashed line. */
  readonly reference?: readonly (number | null)[];
}) {
  const geometry = plot(values, SPARK, reference);
  const capacity = geometry.reference
    .map((y, index) => (y === null ? null : `${geometry.xs[index] ?? 0},${y}`))
    .filter((p): p is string => p !== null);
  return (
    <svg
      className={styles['spark']}
      viewBox={`0 0 ${SPARK.width} ${SPARK.height}`}
      preserveAspectRatio="none"
      aria-hidden="true"
      focusable="false"
    >
      {capacity.length > 1 ? <polyline className={styles['capacity']} points={capacity.join(' ')} /> : null}
      {geometry.gaps.map((x) => (
        <line key={`gap-${x}`} className={styles['gap']} x1={x} x2={x} y1={SPARK.pad} y2={SPARK.height - SPARK.pad} />
      ))}
      {geometry.segments.map((d) => (
        <path key={d} className={styles['line']} d={d} />
      ))}
      {geometry.points.map((point) => (
        <circle key={point.index} className={styles['sparkPoint']} cx={point.x} cy={point.y} r={SPARK_RADIUS} />
      ))}
    </svg>
  );
}
