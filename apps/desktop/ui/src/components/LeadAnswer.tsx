/**
 * The one answer a page gives first (ADR-0034 D1 principle 1).
 *
 * A sentence, at most one figure in the figure size, and a reference line that puts the figure in
 * context: against what, what share, and who contributed most. A bare number with nothing to measure it
 * against is the KPI-card habit this replaces. The figure size is reserved for this component, once per
 * page (ADR-0034 D3).
 */

import { useId, type ReactNode } from 'react';

import styles from './LeadAnswer.module.css';

export function LeadAnswer({
  state,
  sentence,
  figure,
  reference,
  action,
  note,
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
}) {
  const id = useId();
  return (
    <section className={styles['lead']} aria-labelledby={id}>
      <div className={styles['row']}>
        {state}
        {figure === undefined ? null : <span className="figure">{figure}</span>}
        <p id={id} className={styles['sentence']}>{sentence}</p>
        {action === undefined ? null : <div className={styles['action']}>{action}</div>}
      </div>
      {reference === undefined ? null : <p className={styles['reference']}>{reference}</p>}
      {note === undefined ? null : <p className={styles['note']}>{note}</p>}
    </section>
  );
}
