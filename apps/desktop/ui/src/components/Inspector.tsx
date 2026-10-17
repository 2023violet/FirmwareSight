/**
 * The evidence Inspector (ADR-0034 D2): where any FactValue leads.
 *
 * From the `inspector_dock` breakpoint (1280 px) up, it is a column beside the workspace and the page
 * stays usable while it is open. Below that there is no room for a column, so it becomes a sheet over the
 * workspace: a modal dialog with a scrim and a focus trap. Either way, focus moves to the close button
 * when it opens, Esc closes it, and focus returns to the value that opened it, because a reader who
 * checked one number usually wants the next one beside it.
 *
 * It shows only fields the evidence carries. A missing locator is left out, not printed as "—", because
 * an empty row reads as "the locator is empty", which is a different fact.
 */

import { useEffect, useRef, useState, type KeyboardEvent } from 'react';

import styles from './Inspector.module.css';

import { useI18n } from '../i18n';
import { cx } from '../styles/classnames';
import type { FactEvidence } from './FactValue';
import { Icon } from './Icon';

/** `layout.breakpoint.inspector_dock` is 1280: from there up the Inspector is a column. */
export const DOCK_QUERY = '(width >= 1280px)';

function dockedNow(): boolean {
  return typeof window !== 'undefined' && typeof window.matchMedia === 'function'
    ? window.matchMedia(DOCK_QUERY).matches
    : true;
}

const FOCUSABLE = 'button:not([disabled]), [href], input:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function Inspector({
  evidence,
  returnFocus,
  onClose,
}: {
  /** `null` closes the Inspector. */
  readonly evidence: FactEvidence | null;
  /** The control that opened it; focus goes back there on close. */
  readonly returnFocus: HTMLElement | null;
  readonly onClose: () => void;
}) {
  const { t } = useI18n();
  const [docked, setDocked] = useState(dockedNow);
  const [copy, setCopy] = useState<'idle' | 'copied' | 'failed'>('idle');
  const panel = useRef<HTMLElement | null>(null);
  const closeButton = useRef<HTMLButtonElement | null>(null);
  const open = evidence !== null;
  /** Read at close time, so a trigger that changed while open is the one focus returns to. */
  const trigger = useRef(returnFocus);
  trigger.current = returnFocus;

  useEffect(() => {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
      return undefined;
    }
    const query = window.matchMedia(DOCK_QUERY);
    const follow = () => {
      setDocked(query.matches);
    };
    query.addEventListener('change', follow);
    return () => {
      query.removeEventListener('change', follow);
    };
  }, []);

  // A new fact is a new read: focus the panel's first control and forget the last copy result.
  useEffect(() => {
    if (open) {
      setCopy('idle');
      closeButton.current?.focus();
    }
  }, [open, evidence]);

  // However it closes - its own button, Esc, or the page clearing the evidence - focus goes back to the
  // value that opened it, unless the reader has already moved focus somewhere else on purpose.
  useEffect(() => {
    if (!open) {
      return undefined;
    }
    const host = panel.current;
    return () => {
      const active = document.activeElement;
      if (active === null || active === document.body || (host !== null && host.contains(active)) || !active.isConnected) {
        trigger.current?.focus();
      }
    };
  }, [open]);

  // The sheet is modal. Its keydown handler only hears keys while focus is inside it, so a click on plain
  // text in the sheet (focus falls to <body>) would let Tab leave and Esc stop working. Watching the
  // document closes both holes: Esc closes from anywhere, and focus that lands outside is sent back.
  useEffect(() => {
    if (!open || docked) {
      return undefined;
    }
    const onKey = (event: globalThis.KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
      }
    };
    const onFocusIn = (event: FocusEvent) => {
      const host = panel.current;
      if (host !== null && event.target instanceof Node && !host.contains(event.target)) {
        closeButton.current?.focus();
      }
    };
    document.addEventListener('keydown', onKey);
    document.addEventListener('focusin', onFocusIn);
    return () => {
      document.removeEventListener('keydown', onKey);
      document.removeEventListener('focusin', onFocusIn);
    };
  }, [open, docked, onClose]);

  if (evidence === null) {
    return null;
  }

  const close = () => {
    onClose();
    trigger.current?.focus();
  };

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      // The document listener handles the sheet; this one covers the docked column.
      event.stopPropagation();
      event.nativeEvent.stopImmediatePropagation();
      close();
      return;
    }
    // The sheet is modal, so Tab cycles inside it; the docked column is not, so Tab leaves it normally.
    if (event.key === 'Tab' && !docked && panel.current !== null) {
      const items = Array.from(panel.current.querySelectorAll<HTMLElement>(FOCUSABLE));
      const first = items[0];
      const last = items[items.length - 1];
      if (first === undefined || last === undefined) {
        return;
      }
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }
  };

  const raw = evidence.raw ?? evidence.value;
  const rows: [string, string | undefined][] = [
    [t('common', 'inspector.source'), evidence.source],
    [t('common', 'inspector.locator'), evidence.locator],
    [t('common', 'inspector.rule'), evidence.rule],
    ...(evidence.details ?? []).map(([label, value]): [string, string] => [label, value]),
  ];

  const body = (
    <aside
      ref={panel}
      className={cx(styles['panel'], docked ? styles['docked'] : styles['sheet'])}
      role={docked ? 'complementary' : 'dialog'}
      aria-modal={docked ? undefined : true}
      aria-labelledby="fs-inspector-title"
      tabIndex={-1}
      onKeyDown={onKeyDown}
    >
      <div className={styles['head']}>
        <Icon name="evidence" />
        <h2 id="fs-inspector-title" className={styles['title']}>
          {t('common', 'inspector.title')}
        </h2>
        <button
          ref={closeButton}
          type="button"
          className={styles['close']}
          aria-label={t('common', 'inspector.close')}
          onClick={close}
        >
          <Icon name="close" />
        </button>
      </div>
      <div className={styles['body']}>
        <div>
          <p className={styles['field']}>{evidence.field}</p>
          <p className={styles['value']}>{evidence.value}</p>
        </div>
        <dl className={styles['facts']}>
          <div className={styles['pair']}>
            <dt>{t('common', 'inspector.class')}</dt>
            <dd>{t('common', `evidenceClass.${evidence.classification}`)}</dd>
          </div>
          {rows.map(([label, value]) =>
            value === undefined ? null : (
              <div key={label} className={styles['pair']}>
                <dt>{label}</dt>
                <dd className={styles['mono']}>{value}</dd>
              </div>
            ),
          )}
        </dl>
        <div className={styles['rawBlock']}>
          <p className={styles['field']}>{t('common', 'inspector.raw')}</p>
          <p className={styles['raw']}>{raw}</p>
        </div>
        <div className={styles['actions']}>
          <button
            type="button"
            className={styles['copy']}
            onClick={() => {
              const clipboard = typeof navigator === 'undefined' ? undefined : navigator.clipboard;
              if (clipboard === undefined) {
                setCopy('failed');
                return;
              }
              clipboard.writeText(raw).then(
                () => {
                  setCopy('copied');
                },
                () => {
                  setCopy('failed');
                },
              );
            }}
          >
            <Icon name="copy" />
            {t('common', 'inspector.copy')}
          </button>
          <p role="status" className={styles['copyStatus']}>
            {copy === 'copied' ? t('common', 'inspector.copied') : null}
            {copy === 'failed' ? t('common', 'inspector.copyFailed') : null}
          </p>
        </div>
      </div>
    </aside>
  );

  // One outer element in both modes: crossing 1280 px while open must not remount the panel, or focus
  // inside it would be lost (R1 review S2-2). Docked, the host is layout-transparent.
  return (
    <div
      className={docked ? styles['host'] : styles['scrim']}
      onMouseDown={(event) => {
        if (!docked && event.target === event.currentTarget) {
          close();
        }
      }}
    >
      {body}
    </div>
  );
}
