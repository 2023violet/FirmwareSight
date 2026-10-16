/**
 * A change, stated as a fact rather than a judgement (ADR-0034 D1 principle 4).
 *
 * Growth is not bad and shrinkage is not good until a policy says so, and only the Gate's five states
 * carry that colour. So a delta is primary text with a ▲ or ▼ glyph, and the glyph is backed by words
 * for a screen reader ("increase of 64 B"). A side that does not exist is "— not present", never 0:
 * an object that was added had no size before, not a size of zero (DESIGN.md 5, Diff Row).
 */

import styles from './DeltaValue.module.css';

import { useI18n } from '../i18n';

/** `null` means there is no difference to state because one side does not exist. */
export function DeltaValue({
  delta,
  format,
}: {
  readonly delta: number | null;
  /** Formats the magnitude in the reader's unit, e.g. `64 B` or `0.06 KiB`. Never given a sign. */
  readonly format: (magnitude: number) => string;
}) {
  const { t } = useI18n();
  if (delta === null) {
    return <Absent />;
  }
  if (delta === 0) {
    return (
      <span className={styles['delta']}>
        <span aria-hidden="true">{format(0)}</span>
        <span className={styles['spoken']}>{t('common', 'delta.none')}</span>
      </span>
    );
  }
  const amount = format(Math.abs(delta));
  return (
    <span className={styles['delta']}>
      <span className={styles['glyph']} aria-hidden="true">
        {delta > 0 ? '▲' : '▼'}
      </span>
      <span aria-hidden="true">{amount}</span>
      {/* Spoken words rather than an aria-label: a label on a plain span is not reliably announced. */}
      <span className={styles['spoken']}>
        {t('common', delta > 0 ? 'delta.increase' : 'delta.decrease', { amount })}
      </span>
    </span>
  );
}

/** The one way to say a side is missing: a dash and the words, in secondary text. */
export function Absent() {
  const { t } = useI18n();
  return (
    <span className={styles['absent']}>
      <span aria-hidden="true">— </span>
      {t('common', 'delta.notPresent')}
    </span>
  );
}
