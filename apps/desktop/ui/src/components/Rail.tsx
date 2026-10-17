/**
 * The navigation rail (ADR-0034 D2): icon and word per page, full width at 224 px and an icon column at
 * 56 px.
 *
 * It collapses on its own below the `rail_collapse` breakpoint, because at the 1024 px window minimum a
 * 224 px rail leaves the tables less room than they need, and a reader can override that either way for
 * the session. The override is the reader's act, so the window being resized afterwards does not undo it.
 *
 * Collapsing hides the word visually and keeps it in the accessible name: each entry is still announced as
 * "Analyze page", and the `title` gives a pointer reader the same word the icon replaced. Nothing about
 * which page exists or which one is open changes with the width.
 */

import { useEffect, useState } from 'react';

import styles from './Rail.module.css';

import { useI18n } from '../i18n';
import { cx } from '../styles/classnames';
import { Icon, type IconName } from './Icon';

export interface RailEntry<K extends string> {
  readonly key: K;
  readonly label: string;
  readonly icon: IconName;
}

export interface RailGroup<K extends string> {
  /** The landmark name of this list of pages. */
  readonly label: string;
  readonly entries: readonly RailEntry<K>[];
}

/** `layout.breakpoint.rail_collapse` is 1180: below it, the rail starts collapsed. */
export const NARROW_QUERY = '(width < 1180px)';

function narrowNow(): boolean {
  return typeof window !== 'undefined' && typeof window.matchMedia === 'function'
    ? window.matchMedia(NARROW_QUERY).matches
    : false;
}

export function Rail<K extends string>({
  groups,
  current,
  onOpen,
}: {
  readonly groups: readonly RailGroup<K>[];
  readonly current: K;
  readonly onOpen: (key: K) => void;
}) {
  const { t } = useI18n();
  const [narrow, setNarrow] = useState(narrowNow);
  /** `null` until the reader chooses; then their choice wins over the width. */
  const [choice, setChoice] = useState<boolean | null>(null);

  useEffect(() => {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
      return undefined;
    }
    const query = window.matchMedia(NARROW_QUERY);
    const follow = () => {
      setNarrow(query.matches);
    };
    query.addEventListener('change', follow);
    return () => {
      query.removeEventListener('change', follow);
    };
  }, []);

  const collapsed = choice ?? narrow;

  return (
    <div className={cx(styles['rail'], collapsed ? styles['collapsed'] : undefined)} id="fs-rail">
      {groups.map((group) => (
        <nav key={group.label} className={styles['nav']} aria-label={group.label}>
          {group.entries.map((entry) => {
            const active = entry.key === current;
            return (
              <button
                key={entry.key}
                type="button"
                aria-label={t('common', 'nav.pageAria', { label: entry.label })}
                aria-current={active ? 'page' : undefined}
                title={collapsed ? entry.label : undefined}
                className={cx(styles['item'], active ? styles['active'] : undefined)}
                onClick={() => {
                  onOpen(entry.key);
                }}
              >
                <Icon name={entry.icon} size="md" />
                <span className={styles['word']}>{entry.label}</span>
              </button>
            );
          })}
        </nav>
      ))}
      <button
        type="button"
        className={cx(styles['item'], styles['toggle'])}
        aria-controls="fs-rail"
        aria-pressed={!collapsed}
        title={collapsed ? t('common', 'nav.labels') : undefined}
        onClick={() => {
          setChoice(!collapsed);
        }}
      >
        <Icon name="panel" size="md" />
        {/* One fixed name; the pressed state says whether the names are showing, so it is announced once. */}
        <span className={styles['word']}>{t('common', 'nav.labels')}</span>
      </button>
    </div>
  );
}
