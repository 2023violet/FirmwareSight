/**
 * A modal sheet for a decision or an edit that needs the reader's whole attention (ADR-0034 D2, R4 D-R4-2).
 *
 * It is always modal, at every width: a dialog with a focus trap over the workspace, closed by its own
 * button, by Esc or by a click outside it, and focus goes back to the control that opened it. The Inspector
 * is a different surface - evidence beside the page - and stays its own component.
 *
 * Focus goes to `initialFocus` when one is given (the decision the sheet asks for), else to the close button.
 */

import { useEffect, useId, useRef, type KeyboardEvent, type ReactNode, type RefObject } from 'react';

import styles from './Sheet.module.css';

import { useI18n } from '../i18n';
import { Icon } from './Icon';

const FOCUSABLE =
  'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function Sheet({
  open,
  title,
  closeLabel,
  returnFocus,
  initialFocus,
  onClose,
  children,
  actions,
}: {
  readonly open: boolean;
  readonly title: string;
  /** The close button's name, when "Close" alone would not say what closes. */
  readonly closeLabel?: string;
  /** The control that opened it; focus goes back there on close. */
  readonly returnFocus: HTMLElement | null;
  readonly initialFocus?: RefObject<HTMLElement | null>;
  readonly onClose: () => void;
  readonly children: ReactNode;
  /** The sheet's own buttons, kept in view at its foot. */
  readonly actions?: ReactNode;
}) {
  const { t } = useI18n();
  const titleId = useId();
  const panel = useRef<HTMLDivElement | null>(null);
  const closeButton = useRef<HTMLButtonElement | null>(null);
  /** Read at close time, so a trigger that changed while open is the one focus returns to. */
  const trigger = useRef(returnFocus);
  trigger.current = returnFocus;
  const latestClose = useRef(onClose);
  latestClose.current = onClose;

  useEffect(() => {
    if (!open) {
      return undefined;
    }
    (initialFocus?.current ?? closeButton.current)?.focus();
    const host = panel.current;
    // Esc closes from anywhere, and focus that lands outside the sheet is sent back into it.
    const onKey = (event: globalThis.KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        latestClose.current();
      }
    };
    const onFocusIn = (event: FocusEvent) => {
      if (host !== null && event.target instanceof Node && !host.contains(event.target)) {
        closeButton.current?.focus();
      }
    };
    document.addEventListener('keydown', onKey);
    document.addEventListener('focusin', onFocusIn);
    return () => {
      document.removeEventListener('keydown', onKey);
      document.removeEventListener('focusin', onFocusIn);
      // However it closed, focus returns to the opener unless the reader already moved it on purpose.
      const active = document.activeElement;
      if (active === null || active === document.body || (host !== null && host.contains(active)) || !active.isConnected) {
        trigger.current?.focus();
      }
    };
    // `initialFocus` is a ref, read once when the sheet opens.
  }, [open, initialFocus]);

  if (!open) {
    return null;
  }

  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key !== 'Tab' || panel.current === null) {
      return;
    }
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
  };

  return (
    <div
      className={styles['scrim']}
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <div
        ref={panel}
        className={styles['sheet']}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        onKeyDown={onKeyDown}
      >
        <div className={styles['head']}>
          <h2 id={titleId} className={styles['title']}>
            {title}
          </h2>
          <button
            ref={closeButton}
            type="button"
            className={styles['close']}
            aria-label={closeLabel ?? t('common', 'sheet.close')}
            onClick={onClose}
          >
            <Icon name="close" />
          </button>
        </div>
        <div className={styles['body']}>{children}</div>
        {actions === undefined ? null : <div className={styles['foot']}>{actions}</div>}
      </div>
    </div>
  );
}
