/**
 * The failure a reader can inspect, in the one shape the product uses everywhere: what happened, the
 * code, why we know, what to do, the engine's own words, and the diagnostics id to quote.
 *
 * Both pages that call the shell show a typed envelope, and a third copy of this markup would be a
 * third place for one of those lines to go missing (DESIGN.md 9, `AGENTS.md` 10).
 *
 * The diagnosis line comes from the catalog keyed by the code, because that is the line a reader who
 * does not work in English has to act on. Nothing here re-says the engine's sentence at runtime: its
 * message, its details and its remediation are shown as written, under labels that are translated and
 * never rewritten, and the code itself stays monospace and selectable so the same failure can be
 * quoted (`ADR-0032` D6). A code this build has no copy for is announced as exactly that, rather than
 * being dressed in a guess.
 */

import styles from './ErrorPanel.module.css';
import type { ReactNode } from 'react';

import type { Catalog } from '../i18n/en';
import { knowsError, useI18n } from '../i18n';
import type { ErrorEnvelopeDto } from '../ipc/types';

/** The catalog's own name for one error code, once the code is known to be in it. */
type ErrorKey = keyof Catalog['errors'];

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
  const { t } = useI18n();
  const known = knowsError(envelope.code);
  return (
    <section className={styles['error']} role="alert" aria-label={label}>
      <h2>{heading}</h2>
      <dl className={styles['list']}>
        <div>
          <dt>{t('common', 'error.whatHappened')}</dt>
          <dd>
            {known
              ? t('errors', envelope.code as ErrorKey)
              : t('errors', 'unknown' satisfies ErrorKey)}
          </dd>
        </div>
        <div>
          <dt>{t('common', 'error.code')}</dt>
          <dd className={styles['mono']}>{envelope.code}</dd>
        </div>
        {envelope.details === null ? null : (
          <div>
            <dt>{t('common', 'error.whyWeKnow')}</dt>
            <dd className={styles['mono']}>{envelope.details}</dd>
          </div>
        )}
        {envelope.remediation === null ? null : (
          <div>
            <dt>{t('common', 'error.whatToDo')}</dt>
            <dd>{envelope.remediation}</dd>
          </div>
        )}
        <div>
          <dt>{t('common', 'error.engineDetail')}</dt>
          <dd className={styles['mono']}>{envelope.message}</dd>
        </div>
        <div>
          <dt>{t('common', 'error.diagnosticsId')}</dt>
          <dd className={styles['mono']}>{envelope.operationId}</dd>
        </div>
      </dl>
      {action === undefined || action === null ? null : <div className={styles['action']}>{action}</div>}
    </section>
  );
}
