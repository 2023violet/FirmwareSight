/**
 * The failure a reader can inspect, in the one shape the product uses everywhere: what happened, the
 * code, why we know, what to do, and the diagnostics id to quote.
 *
 * Both pages that call the shell show a typed envelope, and a third copy of this markup would be a
 * third place for one of those four lines to go missing (DESIGN.md 9, `AGENTS.md` 10).
 */

import styles from './ErrorPanel.module.css';
import type { ReactNode } from 'react';

import type { ErrorEnvelopeDto } from '../ipc/types';

export function ErrorPanel({
  envelope,
  label,
  heading,
  action,
}: {
  readonly envelope: ErrorEnvelopeDto;
  /** Names the region for a screen reader: which action failed. */
  readonly label: string;
  readonly heading: string;
  /**
   * The move that answers the failure, when the page has one to offer.
   *
   * U1P §13C asks for the recovery action to be as prominent as the diagnosis. It stays an optional slot
   * rather than a second copy of the markup because a page that has no recovery to offer must be able to
   * say so by leaving it out, not by hiding a button it never had.
   */
  readonly action?: ReactNode;
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
      {action === undefined || action === null ? null : <div className={styles['action']}>{action}</div>}
    </section>
  );
}
