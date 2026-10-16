/**
 * The one way to switch views inside a page (ADR-0034 D1 principle 5).
 *
 * The shipped pages switch views three ways: a tablist, a row of `aria-pressed` buttons and the rail
 * itself. This is the WAI-ARIA tabs pattern, with automatic activation: arrow keys move and select,
 * Home and End jump to the ends, and only the selected tab is in the Tab order (roving tabindex). The
 * panel is the caller's; `tabPanelProps` gives it the matching id and label.
 */

import { useRef, type KeyboardEvent } from 'react';

import styles from './Tabs.module.css';

export interface TabItem<K extends string> {
  readonly key: K;
  readonly label: string;
  /** A row count shown beside the label, e.g. Sections 8. */
  readonly count?: number;
}

export function tabId(idBase: string, key: string): string {
  return `${idBase}-tab-${key}`;
}

export function tabPanelProps(idBase: string, key: string) {
  return {
    id: `${idBase}-panel-${key}`,
    role: 'tabpanel',
    'aria-labelledby': tabId(idBase, key),
    tabIndex: 0,
  } as const;
}

export function Tabs<K extends string>({
  idBase,
  label,
  items,
  current,
  onSelect,
}: {
  readonly idBase: string;
  /** The tablist's name, e.g. "Analyze views". */
  readonly label: string;
  readonly items: readonly TabItem<K>[];
  readonly current: K;
  readonly onSelect: (key: K) => void;
}) {
  const buttons = useRef<(HTMLButtonElement | null)[]>([]);

  const move = (index: number) => {
    const target = items[index];
    if (target !== undefined) {
      onSelect(target.key);
      buttons.current[index]?.focus();
    }
  };

  const onKeyDown = (event: KeyboardEvent, index: number) => {
    const last = items.length - 1;
    const next =
      event.key === 'ArrowRight'
        ? (index + 1) % items.length
        : event.key === 'ArrowLeft'
          ? (index - 1 + items.length) % items.length
          : event.key === 'Home'
            ? 0
            : event.key === 'End'
              ? last
              : null;
    if (next !== null) {
      event.preventDefault();
      move(next);
    }
  };

  return (
    <div className={styles['list']} role="tablist" aria-label={label}>
      {items.map((item, index) => {
        const selected = item.key === current;
        return (
          <button
            key={item.key}
            ref={(element) => {
              buttons.current[index] = element;
            }}
            type="button"
            role="tab"
            id={tabId(idBase, item.key)}
            aria-selected={selected}
            aria-controls={tabPanelProps(idBase, item.key).id}
            tabIndex={selected ? 0 : -1}
            className={styles['tab']}
            onClick={() => {
              onSelect(item.key);
            }}
            onKeyDown={(event) => {
              onKeyDown(event, index);
            }}
          >
            {item.label}
            {item.count === undefined ? null : <span className={styles['count']}>{item.count}</span>}
          </button>
        );
      })}
    </div>
  );
}
