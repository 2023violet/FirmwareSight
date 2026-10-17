/**
 * The one way to choose a stored build (ADR-0034 D1 principle 5).
 *
 * Compare has a native `select` and Release has another. A native option can show only one line of text,
 * and two builds of the same `firmware.elf` are told apart by their hash and import time, which a
 * single-line option crushes into an unreadable string. This is a button that opens a listbox
 * (WAI-ARIA select-only combobox): one row per build, with the name, short hash, import time and a pin
 * mark on two lines. Up and Down move, Enter or Space choose, Esc closes and keeps the old choice, and
 * typing a letter jumps to the next build whose name starts with it.
 */

import { useEffect, useId, useRef, useState, type KeyboardEvent } from 'react';

import styles from './BuildPicker.module.css';

import { useI18n } from '../i18n';
import { cx } from '../styles/classnames';
import { Icon } from './Icon';

export interface BuildOption {
  readonly id: string;
  readonly fileName: string;
  readonly sha256: string;
  readonly importedAt: string;
  readonly pinned?: boolean;
}

export function BuildPicker({
  label,
  options: builds,
  value,
  onChange: choose,
  disabled = false,
  noneLabel,
}: {
  readonly label: string;
  readonly options: readonly BuildOption[];
  readonly value: string | null;
  /** The chosen build's id, or `null` when the reader chose the "none" entry. */
  readonly onChange: (id: string | null) => void;
  readonly disabled?: boolean;
  /**
   * When given, the list opens with an entry that names no build, such as "No baseline" (R4 D-R4-3). It is
   * the choice shown while `value` is `null`.
   */
  readonly noneLabel?: string;
}) {
  const { t } = useI18n();
  const base = useId();
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const button = useRef<HTMLButtonElement | null>(null);
  const options: readonly (BuildOption & { readonly none?: true })[] =
    noneLabel === undefined
      ? builds
      : [{ id: '', fileName: noneLabel, sha256: '', importedAt: '', none: true }, ...builds];
  const chosen = builds.find((option) => option.id === value) ?? null;
  const onChange = (id: string) => {
    choose(id === '' ? null : id);
  };

  // The highlight starts on the current choice when the list opens, and only then: a parent that passes a
  // fresh options array on every render must not snap the reader's highlight back.
  const latest = useRef({ options, value });
  latest.current = { options, value };
  useEffect(() => {
    if (open) {
      const index = latest.current.options.findIndex((option) => option.id === (latest.current.value ?? ''));
      setActive(index < 0 ? 0 : index);
    }
  }, [open]);

  useEffect(() => {
    if (open) {
      document.getElementById(`${base}-option-${active}`)?.scrollIntoView?.({ block: 'nearest' });
    }
  }, [open, active, base]);

  const pick = (index: number) => {
    const option = options[index];
    if (option !== undefined) {
      onChange(option.id);
    }
    setOpen(false);
    button.current?.focus();
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (!open) {
      if (event.key === 'ArrowDown' || event.key === 'ArrowUp' || event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        setOpen(true);
      }
      return;
    }
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setActive((index) => Math.min(index + 1, options.length - 1));
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActive((index) => Math.max(index - 1, 0));
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      pick(active);
    } else if (event.key === 'Escape') {
      event.preventDefault();
      setOpen(false);
    } else if (event.key === 'Tab') {
      setOpen(false);
    } else if (event.key.length === 1) {
      const letter = event.key.toLowerCase();
      const after = options.findIndex(
        (option, index) => index > active && option.fileName.toLowerCase().startsWith(letter),
      );
      const wrapped = after >= 0 ? after : options.findIndex((option) => option.fileName.toLowerCase().startsWith(letter));
      if (wrapped >= 0) {
        setActive(wrapped);
      }
    }
  };

  const optionId = (index: number) => `${base}-option-${index}`;

  return (
    <div className={styles['picker']}>
      <span className={styles['label']} id={`${base}-label`}>
        {label}
      </span>
      <button
        ref={button}
        type="button"
        role="combobox"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-controls={`${base}-list`}
        // The label names the control; the chosen build is its content (WAI-ARIA select-only combobox).
        aria-labelledby={`${base}-label`}
        data-value={value ?? ''}
        aria-activedescendant={open ? optionId(active) : undefined}
        className={styles['button']}
        disabled={disabled || builds.length === 0}
        onClick={() => {
          setOpen((now) => !now);
        }}
        onKeyDown={onKeyDown}
        onBlur={(event) => {
          if (!event.currentTarget.parentElement?.contains(event.relatedTarget as Node | null)) {
            setOpen(false);
          }
        }}
      >
        <span id={`${base}-value`} className={styles['value']}>
          {chosen !== null
            ? `${chosen.fileName} · ${chosen.sha256.slice(0, 12)}`
            : builds.length === 0
              ? t('common', 'picker.none')
              : noneLabel ?? t('common', 'picker.choose')}
        </span>
        <Icon name="chevronDown" />
      </button>
      {open ? (
        <ul className={styles['list']} role="listbox" id={`${base}-list`} aria-labelledby={`${base}-label`}>
          {options.map((option, index) => (
            <li
              key={option.id}
              id={optionId(index)}
              role="option"
              data-value={option.id}
              aria-selected={option.none === true ? value === null : option.id === value}
              className={cx(styles['option'], index === active ? styles['active'] : undefined)}
              onMouseDown={(event) => {
                // Keep focus on the combobox; the click below makes the choice.
                event.preventDefault();
              }}
              onClick={() => {
                pick(index);
              }}
            >
              {option.none === true ? (
                <span className={styles['line']}>
                  <span className={styles['name']}>{option.fileName}</span>
                </span>
              ) : (
                <>
                  <span className={styles['line']}>
                    <span className={styles['name']}>
                      {option.fileName} · {option.sha256.slice(0, 12)}
                    </span>
                    {option.pinned === true ? (
                      <span className={styles['pin']}>
                        <Icon name="pin" size="inline" />
                        {t('common', 'subject.pinned')}
                      </span>
                    ) : null}
                  </span>
                  <span className={styles['meta']}>{t('common', 'picker.imported', { date: option.importedAt })}</span>
                </>
              )}
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
