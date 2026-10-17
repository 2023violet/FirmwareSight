/**
 * The command palette (`Ctrl+K`, spec §3.3, R5).
 *
 * A modal dialog holding one combobox and the listbox it filters (WAI-ARIA combobox with list autocomplete).
 * Up and Down move the active command, Enter runs it, Esc closes, and focus returns to wherever it was.
 * The palette runs only what the shell already offers through the pages' own controls: it is a faster way to
 * the same actions, never a new capability (D-R5-6).
 *
 * A command that cannot run right now (comparing with a pinned baseline when none is pinned) is listed with
 * the reason instead of being hidden, so the palette does not teach that the action does not exist.
 */

import { useEffect, useId, useRef, useState, type KeyboardEvent } from 'react';
import { createPortal } from 'react-dom';

import styles from './CommandPalette.module.css';

import { useI18n } from '../i18n';
import { cx } from '../styles/classnames';

export interface PaletteCommand {
  readonly id: string;
  /** What the command does, in the reader's language. */
  readonly label: string;
  /** A group word shown beside it, such as "Go to" or "Theme". */
  readonly group: string;
  /** A keyboard shortcut that does the same, shown as text. */
  readonly shortcut?: string;
  /** Why it cannot run now; `undefined` when it can. */
  readonly unavailable?: string;
  readonly run: () => void;
}

export function CommandPalette({
  open,
  commands,
  onClose,
}: {
  readonly open: boolean;
  readonly commands: readonly PaletteCommand[];
  readonly onClose: () => void;
}) {
  const { t } = useI18n();
  const base = useId();
  const [query, setQuery] = useState('');
  const [active, setActive] = useState(0);
  const input = useRef<HTMLInputElement | null>(null);
  const opener = useRef<Element | null>(null);

  // Opening remembers where focus was and starts clean; closing gives focus back.
  useEffect(() => {
    if (!open) {
      return undefined;
    }
    opener.current = document.activeElement;
    setQuery('');
    setActive(0);
    input.current?.focus();
    return () => {
      const back = opener.current;
      if (back instanceof HTMLElement && back.isConnected) {
        back.focus();
      }
    };
  }, [open]);

  // The active command stays in view while the arrows move through a list longer than the panel.
  useEffect(() => {
    if (open) {
      document.getElementById(`${base}-option-${active}`)?.scrollIntoView?.({ block: 'nearest' });
    }
  }, [open, active, base]);

  if (!open) {
    return null;
  }

  const needle = query.trim().toLowerCase();
  const shown = commands.filter(
    (command) =>
      needle === '' ||
      command.label.toLowerCase().includes(needle) ||
      command.group.toLowerCase().includes(needle),
  );
  const current = Math.min(active, Math.max(shown.length - 1, 0));
  const optionId = (index: number) => `${base}-option-${index}`;

  const run = (command: PaletteCommand | undefined) => {
    if (command === undefined || command.unavailable !== undefined) {
      return;
    }
    onClose();
    command.run();
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setActive(Math.min(current + 1, shown.length - 1));
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActive(Math.max(current - 1, 0));
    } else if (event.key === 'Enter') {
      event.preventDefault();
      run(shown[current]);
    } else if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      onClose();
    } else if (event.key === 'Tab') {
      // One control in a modal dialog: Tab stays on it.
      event.preventDefault();
    }
  };

  return createPortal(
    <div
      className={styles['scrim']}
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <div
        className={styles['palette']}
        role="dialog"
        aria-modal="true"
        aria-label={t('common', 'palette.title')}
        // Keys are heard on the dialog, and a press on its padding or list keeps focus in the input, so Esc
        // and the arrows never stop working (R5 review S3).
        onKeyDown={onKeyDown}
        onMouseDown={(event) => {
          if (event.target !== input.current) {
            event.preventDefault();
          }
        }}
      >
        <input
          ref={input}
          className={styles['input']}
          role="combobox"
          aria-expanded="true"
          aria-controls={`${base}-list`}
          aria-autocomplete="list"
          aria-activedescendant={shown.length === 0 ? undefined : optionId(current)}
          aria-label={t('common', 'palette.input')}
          placeholder={t('common', 'palette.placeholder')}
          value={query}
          onChange={(event) => {
            setQuery(event.target.value);
            setActive(0);
          }}
        />
        <ul className={styles['list']} role="listbox" id={`${base}-list`} aria-label={t('common', 'palette.list')}>
          {shown.length === 0 ? (
            <li className={styles['empty']} role="presentation">
              {t('common', 'palette.none')}
            </li>
          ) : (
            shown.map((command, index) => (
              <li
                key={command.id}
                id={optionId(index)}
                role="option"
                aria-selected={index === current}
                aria-disabled={command.unavailable === undefined ? undefined : true}
                className={cx(styles['option'], index === current ? styles['active'] : undefined)}
                onMouseDown={(event) => {
                  event.preventDefault();
                }}
                onMouseMove={() => {
                  setActive(index);
                }}
                onClick={() => {
                  run(command);
                }}
              >
                <span className={styles['group']}>{command.group}</span>
                <span className={styles['label']}>{command.label}</span>
                {command.unavailable === undefined ? null : (
                  <span className={styles['reason']}>{command.unavailable}</span>
                )}
                {command.shortcut === undefined ? null : <kbd className={styles['kbd']}>{command.shortcut}</kbd>}
              </li>
            ))
          )}
        </ul>
      </div>
    </div>,
    document.body,
  );
}
