import styles from './StateBadge.module.css';
import { cx } from '../styles/classnames';

/** The only semantic state set in this product (DESIGN.md 5). */
export type StateName = 'PASS' | 'REVIEW' | 'BLOCK' | 'UNKNOWN' | 'N/A';

interface Props {
  readonly state: StateName;
  /** Plain-language label. The state name alone is never the whole message. */
  readonly label: string;
  readonly count?: number | undefined;
  /** What is known and what to do next, shown under the badge. */
  readonly note?: string | undefined;
}

/**
 * A state is icon + label (+ optional count). Colour is an aid, never the carrier (DESIGN.md 9),
 * and UNKNOWN keeps a hollow icon in neutral grey so it can never be mistaken for PASS.
 */
export function StateBadge({ state, label, count, note }: Props) {
  return (
    <span className={cx(styles['badge'], styles[stateClass(state)])}>
      <Icon state={state} />
      <span className={styles['label']}>{label}</span>
      {count === undefined ? null : <span className={styles['count']}>{count}</span>}
      {note === undefined ? null : <span className={styles['note']}>{note}</span>}
    </span>
  );
}

function stateClass(state: StateName): string {
  switch (state) {
    case 'PASS':
      return 'pass';
    case 'REVIEW':
      return 'review';
    case 'BLOCK':
      return 'block';
    case 'UNKNOWN':
      return 'unknown';
    case 'N/A':
      return 'na';
  }
}

function Icon({ state }: { readonly state: StateName }) {
  // 1em sizing keeps the glyph on the type scale instead of inventing a pixel size that no
  // token defines.
  const svg = {
    viewBox: '0 0 12 12',
    width: '1em',
    height: '1em',
    'aria-hidden': true,
    focusable: false,
  } as const;

  switch (state) {
    case 'PASS':
      return (
        <svg {...svg} className={cx(styles['solid'])}>
          <circle cx="6" cy="6" r="5.5" fill="currentColor" />
          <path
            d="M3.4 6.2 5.1 7.9 8.6 4.4"
            fill="none"
            stroke="var(--fs-color-text-on-status)"
            strokeWidth="1.4"
          />
        </svg>
      );
    case 'REVIEW':
      return (
        <svg {...svg} className={cx(styles['solid'])}>
          <path d="M6 1 11 10.5H1Z" fill="currentColor" />
          <path
            d="M6 4.4v3"
            stroke="var(--fs-color-text-on-status)"
            strokeWidth="1.4"
            strokeLinecap="butt"
          />
          <circle cx="6" cy="9" r="0.75" fill="var(--fs-color-text-on-status)" />
        </svg>
      );
    case 'BLOCK':
      return (
        <svg {...svg} className={cx(styles['solid'])}>
          <circle cx="6" cy="6" r="5.5" fill="currentColor" />
          <path d="M3.2 8.8 8.8 3.2" stroke="var(--fs-color-text-on-status)" strokeWidth="1.4" />
        </svg>
      );
    case 'UNKNOWN':
      return (
        <svg {...svg} className={cx(styles['hollow'])}>
          <circle
            cx="6"
            cy="6"
            r="5"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.2"
            strokeDasharray="2 1.6"
          />
        </svg>
      );
    case 'N/A':
      return (
        <svg {...svg} className={cx(styles['hollow'])}>
          <path d="M2.5 6h7" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
        </svg>
      );
  }
}
