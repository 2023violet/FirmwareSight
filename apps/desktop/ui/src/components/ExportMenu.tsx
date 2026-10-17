/**
 * Export, as one menu in a page header (ADR-0034 D2, spec §6.2–6.3).
 *
 * A disclosure rather than a popup menu: plain buttons that Tab reaches in order. Esc closes it, focus
 * leaving it closes it, and choosing a format returns focus to the menu's own button, because the button
 * that was pressed is about to leave the DOM (R2 review S3). Analyze and Compare share it, so the two pages
 * export the same way.
 */

import { useRef, useState } from 'react';

import styles from './ExportMenu.module.css';

import { Button } from './Button';
import { Icon } from './Icon';

export interface ExportItem<K extends string> {
  readonly key: K;
  readonly label: string;
}

export function ExportMenu<K extends string>({
  label,
  regionLabel,
  hint,
  items,
  disabled,
  onExport,
}: {
  /** The menu button's word, e.g. "Export". */
  readonly label: string;
  /** The group's accessible name, e.g. "Visual report export". */
  readonly regionLabel: string;
  /** One sentence on what the formats are, shown at the top of the open menu. */
  readonly hint?: string;
  readonly items: readonly ExportItem<K>[];
  readonly disabled: boolean;
  readonly onExport: (key: K) => void;
}) {
  const [open, setOpen] = useState(false);
  const toggle = useRef<HTMLButtonElement | null>(null);
  return (
    <div
      className={styles['menu']}
      role="group"
      aria-label={regionLabel}
      onKeyDown={(event) => {
        if (event.key === 'Escape' && open) {
          event.stopPropagation();
          setOpen(false);
          toggle.current?.focus();
        }
      }}
      onBlur={(event) => {
        // Tabbing or clicking away closes the menu; moving between its own buttons does not.
        if (open && !event.currentTarget.contains(event.relatedTarget as Node | null)) {
          setOpen(false);
        }
      }}
    >
      <Button
        ref={toggle}
        disabled={disabled}
        ariaExpanded={open}
        onClick={() => {
          setOpen((now) => !now);
        }}
      >
        <Icon name="export" />
        {label}
        <Icon name="chevronDown" />
      </Button>
      {open ? (
        <div className={styles['list']}>
          {hint === undefined ? null : <p className={styles['hint']}>{hint}</p>}
          {items.map((item) => (
            <Button
              key={item.key}
              variant="ghost"
              disabled={disabled}
              onClick={() => {
                setOpen(false);
                toggle.current?.focus();
                onExport(item.key);
              }}
            >
              {item.label}
            </Button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
