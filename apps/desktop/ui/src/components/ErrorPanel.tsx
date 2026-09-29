/**
 * The failure a reader can inspect, in the one shape the product uses everywhere: what happened, the
 * code, why we know, what to do, and the diagnostics id to quote.
 *
 * Both pages that call the shell show a typed envelope, and a third copy of this markup would be a
 * third place for one of those four lines to go missing (DESIGN.md 9, `AGENTS.md` 10).
 */

import styles from './ErrorPanel.module.css';
import type { ErrorEnvelopeDto } from '../ipc/types';

export function ErrorPanel({
  envelope,
  label,
  heading,
}: {
  readonly envelope: ErrorEnvelopeDto;
  /** Names the region for a screen reader: which action failed. */
  readonly label: string;
  readonly heading: string;
}) {
  return (
    <section className={styles['error']} role="alert" aria-label={label}>
      <h2>{heading}</h2>
      <dl className={styles['list']}>
        <div>
          <dt>What happened</dt>
          <dd>{envelope.message}</dd>
        </div>
        <div>
          <dt>Code</dt>
          <dd className={styles['mono']}>{envelope.code}</dd>
        </div>
        {envelope.details === null ? null : (
          <div>
            <dt>Why we know</dt>
            <dd className={styles['mono']}>{envelope.details}</dd>
          </div>
        )}
        {envelope.remediation === null ? null : (
          <div>
            <dt>What to do</dt>
            <dd>{envelope.remediation}</dd>
          </div>
        )}
        <div>
          <dt>Diagnostics ID</dt>
          <dd className={styles['mono']}>{envelope.operationId}</dd>
        </div>
      </dl>
    </section>
  );
}
