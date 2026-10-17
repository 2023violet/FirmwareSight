/**
 * A number the reader can trace (ADR-0034 D1 principle 2).
 *
 * The value is a button because its job is to open the evidence behind it in the Inspector. Beside the
 * value sits a small mark for its evidence class: filled for Observed, half for Derived, hollow for
 * Declared and dashed for Unknown. The mark is decoration, and the class is spoken in the button's
 * name, so the four classes never depend on telling a disc from a ring.
 */

import styles from './FactValue.module.css';

import { useI18n } from '../i18n';
import type { EvidenceClassDto } from '../ipc/types';
import { cx } from '../styles/classnames';

/** Everything the Inspector can show about one fact. Fields the evidence does not carry stay absent. */
export interface FactEvidence {
  /** The machine name of the fact, e.g. `nonvolatile_image_footprint_bytes`. */
  readonly field: string;
  /** The value as shown on screen. */
  readonly value: string;
  readonly classification: EvidenceClassDto;
  readonly source?: string;
  readonly locator?: string;
  readonly rule?: string;
  /** The raw recorded value, when it differs from the shown one (a full hash, an unrounded byte count). */
  readonly raw?: string;
  /** Further labelled facts the evidence carries, such as confidence or an evidence id. */
  readonly details?: readonly (readonly [string, string])[];
}

/**
 * Core's classification word as the four-class union. Stored words are lower-case; anything this build
 * does not recognise is shown as Unknown rather than guessed at.
 */
export function asEvidenceClass(word: string): EvidenceClassDto {
  const lower = word.toLowerCase();
  return lower === 'observed' || lower === 'derived' || lower === 'declared' ? lower : 'unknown';
}

export function FactValue({
  evidence,
  selected = false,
  onInspect,
}: {
  readonly evidence: FactEvidence;
  /** True while the Inspector shows this fact, so the reader can see which number it is about. */
  readonly selected?: boolean;
  readonly onInspect: (evidence: FactEvidence, trigger: HTMLElement) => void;
}) {
  const { t } = useI18n();
  const word = t('common', `evidenceClass.${evidence.classification}`);
  return (
    <button
      type="button"
      className={cx(styles['fact'], selected ? styles['selected'] : undefined)}
      aria-label={t('common', 'fact.open', { value: evidence.value, class: word })}
      aria-current={selected ? 'true' : undefined}
      onClick={(event) => {
        onInspect(evidence, event.currentTarget);
      }}
    >
      <span className={styles['value']}>{evidence.value}</span>
      <span className={cx(styles['mark'], styles[evidence.classification])} aria-hidden="true" />
    </button>
  );
}
