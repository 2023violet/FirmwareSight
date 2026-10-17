/**
 * What the page is about, always in view (ADR-0034 D1 principle 3).
 *
 * One build on Analyze and Release, a base and a target on Compare. Each build is named by its file and
 * its short hash, and states whether a MAP came with it, because object attribution and exact memory
 * figures depend on that. The freshness flag is the one place a page says it is showing a retained
 * result rather than the current selection; R2–R5 retire the scattered stale paragraphs in its favour.
 */

import type { ReactNode } from 'react';

import styles from './SubjectBar.module.css';

import { useI18n } from '../i18n';
import { cx } from '../styles/classnames';
import { Icon } from './Icon';

export interface SubjectBuild {
  readonly fileName: string;
  /** `null` for a file that has been chosen but not analyzed: no hash has been computed for it yet. */
  readonly sha256: string | null;
  /** `null` when the build has not reported whether a MAP was attached. */
  readonly map: boolean | null;
  /** The attached MAP's file name, when the shell reported one. */
  readonly mapFileName?: string | null;
  readonly pinned?: boolean;
}

/** Latest result, a retained earlier result, or a chosen file with no result of its own yet. */
export type SubjectFreshness = 'latest' | 'retained' | 'pending';

export type Subject =
  | { readonly kind: 'single'; readonly build: SubjectBuild }
  | { readonly kind: 'pair'; readonly base: SubjectBuild; readonly target: SubjectBuild };

export function SubjectBar({
  subject,
  freshness,
  actions,
}: {
  /** `null` before anything is chosen. */
  readonly subject: Subject | null;
  readonly freshness: SubjectFreshness;
  readonly actions?: ReactNode;
}) {
  const { t } = useI18n();
  return (
    <div className={styles['bar']} role="region" aria-label={t('common', 'subject.label')}>
      {subject === null ? (
        <>
          <span className={styles['label']}>{t('common', 'subject.current')}</span>
          <span className={styles['none']}>{t('common', 'subject.none')}</span>
        </>
      ) : subject.kind === 'single' ? (
        <>
          <span className={styles['label']}>{t('common', 'subject.current')}</span>
          <Build build={subject.build} />
        </>
      ) : (
        <>
          <span className={styles['label']}>{t('common', 'subject.base')}</span>
          <Build build={subject.base} />
          <span className={styles['label']}>{t('common', 'subject.target')}</span>
          <Build build={subject.target} />
        </>
      )}
      {subject === null ? null : freshness === 'latest' ? (
        <span className={styles['chip']}>{t('common', 'subject.latest')}</span>
      ) : freshness === 'pending' ? (
        <span className={styles['chip']}>{t('common', 'subject.pending')}</span>
      ) : (
        <span className={cx(styles['chip'], styles['retained'])}>
          <Icon name="history" size="inline" />
          {t('common', 'subject.retained')}
        </span>
      )}
      {actions === undefined ? null : <div className={styles['actions']}>{actions}</div>}
    </div>
  );
}

function Build({ build }: { readonly build: SubjectBuild }) {
  const { t } = useI18n();
  return (
    <span className={styles['build']}>
      <span
        className={styles['name']}
        title={build.sha256 === null ? build.fileName : `${build.fileName} · sha256 ${build.sha256}`}
      >
        {build.sha256 === null ? build.fileName : `${build.fileName} · ${build.sha256.slice(0, 12)}`}
      </span>
      {build.pinned === true ? (
        <span className={styles['chip']}>
          <Icon name="pin" size="inline" />
          {t('common', 'subject.pinned')}
        </span>
      ) : null}
      {build.map === null ? (
        <span className={cx(styles['chip'], styles['unknown'])}>
          <Icon name="unknown" size="inline" />
          {t('common', 'subject.mapUnknown')}
        </span>
      ) : build.map ? (
        <span className={styles['chip']}>
          <Icon name="file" size="inline" />
          {build.mapFileName === undefined || build.mapFileName === null
            ? t('common', 'subject.map')
            : t('common', 'subject.mapNamed', { name: build.mapFileName })}
        </span>
      ) : (
        <span className={cx(styles['chip'], styles['unknown'])}>
          <Icon name="unknown" size="inline" />
          {t('common', 'subject.noMap')}
        </span>
      )}
    </span>
  );
}
