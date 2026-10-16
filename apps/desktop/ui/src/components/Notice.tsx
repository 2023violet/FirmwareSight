/**
 * The one shape for a message that is not a fact table (ADR-0034 D1 principle 5).
 *
 * Four tones and no others. `info` is a neutral statement, `stale` says the screen is showing a retained
 * result, `unknown` is the DESIGN.md 6 Unknown (neutral grey, hollow icon, a fact sentence and a next
 * step), and `error` is a failure that the reader can recover from where they stand. An error carries the
 * same four answers ErrorPanel gives — what happened, why we know, what to do, diagnostics ID — so a page
 * moving onto this component loses none of them.
 */

import type { ReactNode } from 'react';

import styles from './Notice.module.css';

import { cx } from '../styles/classnames';
import { Icon, type IconName } from './Icon';

export type NoticeTone = 'info' | 'stale' | 'unknown' | 'error';

const GLYPH: Record<NoticeTone, IconName> = {
  info: 'info',
  stale: 'review',
  unknown: 'unknown',
  error: 'block',
};

export function Notice({
  tone,
  title,
  children,
  details,
  actions,
}: {
  readonly tone: NoticeTone;
  readonly title: string;
  /** The fact sentence. */
  readonly children?: ReactNode;
  /** Labelled facts, e.g. What happened / Why we know / What to do / Diagnostics ID. */
  readonly details?: readonly (readonly [string, ReactNode])[];
  /** The next step, as one or two buttons. */
  readonly actions?: ReactNode;
}) {
  return (
    <div className={cx(styles['notice'], styles[tone])} role={tone === 'error' ? 'alert' : 'status'}>
      <Icon name={GLYPH[tone]} className={styles['icon']} />
      <div className={styles['content']}>
        <h3 className={styles['title']}>{title}</h3>
        {children === undefined ? null : <div className={styles['text']}>{children}</div>}
        {details === undefined || details.length === 0 ? null : (
          <dl className={styles['details']}>
            {details.map(([label, value]) => (
              <div key={label} className={styles['pair']}>
                <dt>{label}</dt>
                <dd>{value}</dd>
              </div>
            ))}
          </dl>
        )}
        {actions === undefined ? null : <div className={styles['actions']}>{actions}</div>}
      </div>
    </div>
  );
}
