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
  /**
   * `null` when the build has not reported whether a MAP was attached. Left out where the page has no MAP
   * fact to report at all (Release's stored candidates), so no chip claims one either way (R4 D-R4-4).
   */
  readonly map?: boolean | null;
  /** The attached MAP's file name, when the shell reported one. */
  readonly mapFileName?: string | null;
  readonly pinned?: boolean;
}

/**
 * Latest result; a retained earlier result; a chosen file with no result of its own yet; a retained result
 * after the newest attempt failed; or a result whose selection has since gained or lost its MAP.
 */
export type SubjectFreshness =
  | 'latest'
  | 'retained'
  | 'pending'
  | 'failed'
  | 'mapChanged'
  | 'pairMoved'
  | 'compareFailed'
  | 'settingsMoved'
  | 'gateFailed';

/** The words for every retained-result freshness, so a new one cannot be added without its sentence. */
const RETAINED_WORD = {
  retained: 'subject.retained',
  failed: 'subject.failed',
  mapChanged: 'subject.mapChanged',
  pairMoved: 'subject.pairMoved',
  compareFailed: 'subject.compareFailed',
  settingsMoved: 'subject.settingsMoved',
  gateFailed: 'subject.gateFailed',
} as const;

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
          {t('common', RETAINED_WORD[freshness])}
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
      {build.map === undefined ? null : build.map === null ? (
        <span className={cx(styles['chip'], styles['unknown'])}>
          <Icon name="unknown" size="inline" />
          {t('common', 'subject.mapUnknown')}
        </span>
      ) : build.map ? (
        <span className={styles['chip']} title={build.mapFileName ?? undefined}>
          <Icon name="file" size="inline" />
          <span className={styles['chipText']}>
            {build.mapFileName === undefined || build.mapFileName === null
              ? t('common', 'subject.map')
              : t('common', 'subject.mapNamed', { name: build.mapFileName })}
          </span>
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
