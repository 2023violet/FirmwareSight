/**
 * A ranked bar: one row of a magnitude list, drawn at the ratio its own value has to the largest value
 * in the same list.
 *
 * `DESIGN.md` 7 puts ranked bars second only to tables, and the chart tokens it provides for them
 * (`--fs-color-chart-blue-1..5`, `--fs-color-chart-other`) were in the frozen token file with no user.
 * So this is the design system being used, not extended.
 *
 * Two rules the reference screens also follow, kept deliberate:
 *
 * - the number is always beside the bar, and the bar is `aria-hidden`. Length and colour are an
 *   acceleration of a fact the text already states; a screen reader, a reader who cannot see the blue
 *   scale, and a reader with a colour-blind simulation all get the same answer from the words.
 * - an unknown magnitude gets no bar. Drawing a stub for a value Core refused to state would be a guess
 *   with pixels on it, and `AGENTS.md` 8 says an unknown stays an unknown.
 *
 * The only number computed here is `value / largest`, which is the data's own ratio. Every static length,
 * colour and radius stays a token.
 */

import styles from './RankBar.module.css';
import type { ReactNode } from 'react';

import { cx } from '../styles/classnames';

export function RankBar({
  label,
  ratio,
  value,
  unknownHint,
  onPick,
}: {
  readonly label: ReactNode;
  /** 0 to 1, or `null` when the magnitude is not known. */
  readonly ratio: number | null;
  readonly value: ReactNode;
  /** What to say instead of a bar when `ratio` is null. */
  readonly unknownHint?: ReactNode;
  /** When present the row becomes the navigation aid the Compare ranking already is. */
  readonly onPick?: () => void;
}) {
  const bar =
    ratio === null ? (
      <span className={styles['absent']}>{unknownHint ?? 'no magnitude'}</span>
    ) : (
      <span className={styles['track']} aria-hidden="true">
        <span className={styles['fill']} style={{ width: `${Math.round(clamped(ratio) * 100)}%` }} />
      </span>
    );

  if (onPick === undefined) {
    return (
      <div className={styles['row']}>
        <span className={styles['label']}>{label}</span>
        {bar}
        <span className={styles['value']}>{value}</span>
      </div>
    );
  }

  return (
    <button type="button" className={cx(styles['row'], styles['pickable'])} onClick={onPick}>
      <span className={styles['label']}>{label}</span>
      {bar}
      <span className={styles['value']}>{value}</span>
    </button>
  );
}

function clamped(ratio: number): number {
  if (!(ratio > 0)) {
    return 0;
  }
  return ratio > 1 ? 1 : ratio;
}
