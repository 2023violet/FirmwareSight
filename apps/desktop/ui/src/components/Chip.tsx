/**
 * A quiet pill for a fact, and the count line a verdict is standing on.
 *
 * `Chip` is for the things that are not gate states - which input produced this screen, how many rows a
 * table holds, which file is attached. States themselves belong to `StateBadge`, which gained the same
 * pill as a variant rather than being reimplemented here, because two components spelling "REVIEW"
 * differently is exactly the drift this file exists to prevent.
 */

import styles from './Chip.module.css';
import type { ReactNode } from 'react';

import type { StateName } from './StateBadge';
import { StateIcon } from './StateBadge';
import { cx } from '../styles/classnames';
import { useI18n } from '../i18n';

export function Chip({
  children,
  mono = false,
  label,
}: {
  readonly children: ReactNode;
  /** Numbers, names and ids are mono (`DESIGN.md` 4); prose is not. */
  readonly mono?: boolean;
  /** A spoken label when the visible text is an abbreviation a reader should not have to expand. */
  readonly label?: string;
}) {
  return (
    <span className={cx(styles['chip'], mono ? styles['mono'] : undefined)} aria-label={label}>
      {children}
    </span>
  );
}

/**
 * `✓4 ⚠2 ×1 ○1 –1`: how many findings landed in each factual state.
 *
 * The order is the one `03_DESIGN/06_UI_REFERENCE_SCREENS.md` fixes for the Gate page - BLOCK before
 * REVIEW before UNKNOWN before PASS before N/A - so the strip and the grouped list below it read in the
 * same direction. The counts arrive from `GateCountsDto`; nothing is summed, averaged or inferred here,
 * because Core owns the aggregate (`AGENTS.md` 3).
 *
 * Each item is glyph + number, and the whole strip is one `role="img"` with a sentence as its name,
 * because "✓4" is not an announcement and an `aria-label` on a bare `span` is not an accessible name.
 */
export function StatusStrip({
  counts,
  label,
}: {
  readonly counts: Readonly<Partial<Record<StateName, number>>>;
  readonly label: string;
}) {
  const { t } = useI18n();
  const order: readonly StateName[] = ['BLOCK', 'REVIEW', 'UNKNOWN', 'PASS', 'N/A'];
  const spoken = order
    .filter((state) => (counts[state] ?? 0) > 0)
    .map((state) =>
      t('common', 'strip.item', { count: counts[state] ?? 0, state: t('common', SPOKEN[state]) }),
    )
    .join(t('common', 'strip.join'));

  return (
    <span
      className={styles['strip']}
      role="img"
      aria-label={
        spoken === ''
          ? t('common', 'chip.none', { label })
          : t('common', 'chip.spoken', { label, value: spoken })
      }
    >
      {order.map((state) => (
        <span key={state} className={cx(styles['stripItem'], styles[classFor(state)])}>
          <StateIcon state={state} />
          <span className={styles['stripCount']}>{String(counts[state] ?? 0)}</span>
        </span>
      ))}
    </span>
  );
}

/**
 * The state word as it is read out, per catalog.
 *
 * These are the Gate's own vocabulary, so the Chinese entry keeps `PASS` / `REVIEW` / `BLOCK` /
 * `UNKNOWN` / `N/A` exactly as the engine writes them (`ADR-0032` D5): a screen reader that announced
 * 通过 for a REVIEW row would be renaming a verdict, which is the one thing this product's whole
 * state model forbids. English keeps the lowercase prose it has always announced, so nothing a test or
 * a reader depended on moves.
 */
const SPOKEN: Readonly<Record<StateName, 'strip.pass' | 'strip.review' | 'strip.block' | 'strip.unknown' | 'strip.na'>> = {
  PASS: 'strip.pass',
  REVIEW: 'strip.review',
  BLOCK: 'strip.block',
  UNKNOWN: 'strip.unknown',
  'N/A': 'strip.na',
};

function classFor(state: StateName): string {
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
