/**
 * The product's one icon set (ADR-0034 D4).
 *
 * Drawn here rather than taken from a library: ADR-0011 keeps UI kits out of the frontend, the set the
 * product needs is about two dozen shapes, and a glyph that lives in this file can be read, reviewed and
 * inlined into an export by the same hand. Every glyph sits on a 20-unit grid, is stroked with the token
 * width and takes its colour from the text around it, so an icon can never carry a colour the label
 * beside it does not.
 *
 * The five state glyphs are the exception to "stroke only". A state has to survive greyscale, a colour
 * blind reader and forced colours, so each one is a different *shape*: a filled disc, a triangle, an
 * octagon, a dashed ring and a struck ring. Two of them could share a colour and still not be confused.
 */

import styles from './Icon.module.css';

import { cx } from '../styles/classnames';

/** The inner mark on a filled state glyph: the status surface colour, so it reads in both themes. */
const ON_STATUS = 'var(--fs-color-text-on-status)';

const GLYPHS = {
  // Navigation
  overview: (
    <>
      <rect x="3" y="3" width="6" height="6" rx="1" />
      <rect x="11" y="3" width="6" height="6" rx="1" />
      <rect x="3" y="11" width="6" height="6" rx="1" />
      <rect x="11" y="11" width="6" height="6" rx="1" />
    </>
  ),
  analyze: <path d="M3 16h14M5 13V8M9 13V4M13 13v-3" />,
  compare: <path d="M4 7h10l-3-3M16 13H6l3 3" />,
  release: (
    <>
      <path d="M10 3l6 3v4c0 4-3 6-6 7-3-1-6-3-6-7V6z" />
      <path d="M7.5 10l2 2 3.5-4" />
    </>
  ),
  history: (
    <>
      <circle cx="10" cy="10" r="7" />
      <path d="M10 6v4l3 2" />
    </>
  ),
  help: (
    <>
      <circle cx="10" cy="10" r="7" />
      <path d="M8 8a2 2 0 113 1.7c-.7.4-1 .8-1 1.6M10 14h.01" />
    </>
  ),
  settings: (
    <>
      <circle cx="10" cy="10" r="2.5" />
      <path d="M10 2.5v2M10 15.5v2M2.5 10h2M15.5 10h2M4.7 4.7l1.4 1.4M13.9 13.9l1.4 1.4M4.7 15.3l1.4-1.4M13.9 6.1l1.4-1.4" />
    </>
  ),
  // States: one shape each, so the glyph alone tells them apart
  pass: (
    <>
      <circle cx="10" cy="10" r="7.5" fill="currentColor" stroke="none" />
      <path d="M6.6 10.2l2.3 2.3 4.4-4.8" stroke={ON_STATUS} />
    </>
  ),
  review: (
    <>
      <path d="M10 2.5l8 14H2z" fill="currentColor" />
      <path d="M10 8v3.6" stroke={ON_STATUS} />
      <circle cx="10" cy="14" r="1" fill={ON_STATUS} stroke="none" />
    </>
  ),
  block: (
    <>
      <path d="M7 2.5h6l4.5 4.5v6L13 17.5H7L2.5 13V7z" fill="currentColor" />
      <path d="M7.6 7.6l4.8 4.8M12.4 7.6l-4.8 4.8" stroke={ON_STATUS} />
    </>
  ),
  unknown: (
    <>
      <circle cx="10" cy="10" r="7" strokeDasharray="2.6 2.2" />
      <path d="M8.3 8.2a1.8 1.8 0 112.6 1.6c-.6.3-.9.7-.9 1.3M10 13.6h.01" />
    </>
  ),
  na: (
    <>
      <circle cx="10" cy="10" r="7" />
      <path d="M5.2 14.8l9.6-9.6" />
    </>
  ),
  // Actions
  swap: <path d="M7 16V4L4 7M13 4v12l3-3" />,
  pin: <path d="M8 3h4l-.5 5 3 2.5h-9l3-2.5zM10 10.5V17" />,
  copy: (
    <>
      <rect x="7" y="7" width="9" height="9" rx="1.5" />
      <path d="M4 13V5.5A1.5 1.5 0 015.5 4H13" />
    </>
  ),
  close: <path d="M5 5l10 10M15 5L5 15" />,
  chevronRight: <path d="M8 5l5 5-5 5" />,
  chevronDown: <path d="M5 8l5 5 5-5" />,
  filter: <path d="M3 5h14l-5.5 6v4.5l-3 1.5v-6z" />,
  export: <path d="M10 3v9M6.5 8.5L10 12l3.5-3.5M4 15h12" />,
  evidence: (
    <>
      <circle cx="9" cy="9" r="5" />
      <path d="M13 13l4 4M9 7v4M7 9h4" />
    </>
  ),
  file: <path d="M5 3h6l4 4v10H5zM11 3v4h4" />,
  info: (
    <>
      <circle cx="10" cy="10" r="7" />
      <path d="M10 9v5M10 6.5h.01" />
    </>
  ),
  panel: (
    <>
      <rect x="3" y="4" width="14" height="12" rx="1.5" />
      <path d="M8 4v12" />
    </>
  ),
  search: (
    <>
      <circle cx="9" cy="9" r="5" />
      <path d="M13 13l4 4" />
    </>
  ),
} as const;

export type IconName = keyof typeof GLYPHS;

/** Every name the set holds, for the test that renders each one and the component gallery. */
export const ICON_NAMES = Object.keys(GLYPHS) as readonly IconName[];

export function Icon({
  name,
  size = 'sm',
  label,
  className,
}: {
  readonly name: IconName;
  /** `sm` 16 and `md` 20 are the token sizes; `inline` follows the text it sits in (1em). */
  readonly size?: 'sm' | 'md' | 'inline';
  /**
   * A spoken name, only for an icon that stands alone. Beside a word the icon is decoration and stays
   * hidden, because a screen reader hearing "check mark, PASS" has been told the same thing twice.
   */
  readonly label?: string | undefined;
  readonly className?: string | undefined;
}) {
  return (
    <svg
      viewBox="0 0 20 20"
      className={cx(styles['icon'], styles[size], className)}
      data-icon={name}
      role={label === undefined ? undefined : 'img'}
      aria-label={label}
      aria-hidden={label === undefined ? true : undefined}
      focusable={false}
    >
      {GLYPHS[name]}
    </svg>
  );
}
