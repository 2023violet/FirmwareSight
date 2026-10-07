/**
 * The three button levels `DESIGN.md` 5 names, in one place.
 *
 * Before U1 each page restyled its own controls - `.primary` lived in three CSS modules and `.control`
 * in four - which is how one product ended up with three slightly different secondary buttons. The
 * levels carry no other meaning: a Primary is the one move this region recommends, a Secondary is a
 * real alternative, and a Ghost is navigation or a low-weight read.
 *
 * The disabled Primary is the reason this file has a comment. `text.disabled` and `accent.disabled` are
 * the same token value (#98A2B3), so a label painted with the first on a background of the second is
 * invisible; Analyze shipped that bug once already (`Analyze.module.css:107-115` records it). Secondary
 * text on a subtle surface keeps the word legible without borrowing the accent.
 */

import styles from './Button.module.css';
import type { ReactNode, Ref } from 'react';

import { cx } from '../styles/classnames';

export function Button({
  variant = 'secondary',
  type = 'button',
  disabled = false,
  title,
  ariaLabel,
  ariaPressed,
  ariaExpanded,
  ref,
  onClick,
  children,
}: {
  readonly variant?: 'primary' | 'secondary' | 'ghost';
  readonly type?: 'button' | 'submit';
  readonly disabled?: boolean;
  /** Overflow relief for a label that can be long: the full text, never a path (ADR-0025). */
  readonly title?: string;
  readonly ariaLabel?: string;
  readonly ariaPressed?: boolean;
  /** A button that opens a form or a disclosure has to say so while it is closed. */
  readonly ariaExpanded?: boolean;
  /** React 19 takes a ref as a prop. The bundle replace confirmation focuses this control by hand. */
  readonly ref?: Ref<HTMLButtonElement>;
  readonly onClick?: () => void;
  readonly children: ReactNode;
}) {
  return (
    <button
      ref={ref}
      type={type}
      className={cx(styles['button'], styles[variant])}
      disabled={disabled}
      title={title}
      aria-label={ariaLabel}
      aria-pressed={ariaPressed}
      aria-expanded={ariaExpanded}
      onClick={onClick}
    >
      {children}
    </button>
  );
}

/**
 * A link-styled action that is still a button.
 *
 * The mockups end several rows with a plain blue verb ("View all symbols"). It is an action, not
 * navigation to a document, so it stays a `button` for keyboard and screen-reader semantics and borrows
 * only the accent colour.
 */
export function LinkButton({
  onClick,
  children,
}: {
  readonly onClick: () => void;
  readonly children: ReactNode;
}) {
  return (
    <button type="button" className={cx(styles['button'], styles['link'])} onClick={onClick}>
      {children}
    </button>
  );
}
