/**
 * The one answer a page gives first (ADR-0034 D1 principle 1).
 *
 * A sentence, at most one figure in the figure size, and a reference line that puts the figure in
 * context: against what, what share, and who contributed most. A bare number with nothing to measure it
 * against is the KPI-card habit this replaces. The figure size is reserved for this component, once per
 * page (ADR-0034 D3).
 */

import { useId, type ReactNode, type RefObject } from 'react';

import styles from './LeadAnswer.module.css';

export function LeadAnswer({
  state,
  sentence,
  figure,
  reference,
  action,
  note,
  label,
  focusRef,
}: {
  /** A StateBadge, when the answer is a Gate verdict. */
  readonly state?: ReactNode;
  readonly sentence: string;
  /** The one figure, usually a FactValue. */
  readonly figure?: ReactNode;
  /** What the figure is measured against. */
  readonly reference?: ReactNode;
  /** The single next move this answer recommends. */
  readonly action?: ReactNode;
  /** A standing qualification, such as "this is not a certification". */
  readonly note?: ReactNode;
  /** A name for the region when the sentence is not one, such as "Gate verdict" (R4 D-R4-5). */
  readonly label?: string;
  /** Lets the page move focus to the answer after an action the reader asked for. */
  readonly focusRef?: RefObject<HTMLElement | null>;
}) {
  const id = useId();
  return (
    <section
      ref={focusRef}
      tabIndex={focusRef === undefined ? undefined : -1}
      className={styles['lead']}
      aria-label={label}
      aria-labelledby={label === undefined ? id : undefined}
    >
      <div className={styles['row']}>
        {state}
        {/* Sentence, then the figure it is about: "Flash used 196 bytes" (spec §6.2). */}
        <p id={id} className={styles['sentence']}>{sentence}</p>
        {figure === undefined ? null : <span className="figure">{figure}</span>}
        {action === undefined ? null : <div className={styles['action']}>{action}</div>}
      </div>
      {reference === undefined ? null : <p className={styles['reference']}>{reference}</p>}
      {note === undefined ? null : <p className={styles['note']}>{note}</p>}
    </section>
  );
}
