/**
 * The boxes every reference screen is built from: a panel, a summary card, and the state a panel is in
 * when it has nothing to show.
 *
 * All three are hairline-and-surface, never elevated: `DESIGN.md` 4 builds hierarchy from alignment,
 * grouping, hairlines and type rather than from card shadows, and §9 forbids the shadow outright. The
 * tree contained zero `box-shadow` declarations before this round and still does.
 *
 * The uppercase micro-label the references use for a card title is a device this product had not used
 * (no `text-transform` existed in the UI before U1). It needs no token value, so it is admitted; it is
 * recorded in the validation report as a pattern taken from the references rather than as something the
 * design system already prescribed.
 */

import styles from './Panel.module.css';
import type { ReactNode } from 'react';

import { cx } from '../styles/classnames';

export function Panel({
  title,
  hint,
  labelledBy,
  children,
  className,
  bare,
}: {
  readonly title?: ReactNode;
  /** The quiet explanation opposite the title - what this panel is standing on. */
  readonly hint?: ReactNode;
  readonly labelledBy?: string;
  readonly children: ReactNode;
  readonly className?: string;
  /** Inside a `Band`: keep the content, surrender the box, because the band owns the hairlines. */
  readonly bare?: boolean;
}) {
  // Same rule as `PageSection`: a `<section>` is only a named region when something names it, so a
  // string title becomes the accessible label unless the caller pointed at an element that carries it.
  const namedByString = labelledBy === undefined && typeof title === 'string';
  return (
    <section
      className={cx(styles['panel'], bare ? styles['panelBare'] : undefined, className)}
      aria-label={namedByString ? title : undefined}
      aria-labelledby={labelledBy}
    >
      {title === undefined ? null : (
        <div className={styles['panelHead']}>
          <h2 id={labelledBy} className={styles['panelTitle']}>
            {title}
          </h2>
          {hint === undefined ? null : <p className={styles['panelHint']}>{hint}</p>}
        </div>
      )}
      {children}
    </section>
  );
}

/**
 * One number with the context that makes it mean something (`DESIGN.md` 9: numbers carry context).
 *
 * The value is always mono and tabular so two screens put the same figure on the same edge, and the
 * context line is never optional in practice: a bare "486.2 KB" is a fact without a budget, which is the
 * difference between a summary and a decoration.
 */
export function SummaryCard({
  label,
  value,
  context,
  state,
}: {
  readonly label: string;
  /** The figure. A cell may carry a state instead of a number, which is still an answer. */
  readonly value?: ReactNode;
  readonly context?: ReactNode;
  /** A state glyph beside the value, when the number's meaning depends on it. */
  readonly state?: ReactNode;
}) {
  return (
    <div className={styles['summary']}>
      <p className={styles['summaryLabel']}>{label}</p>
      <p className={styles['summaryValue']}>
        {state}
        <span className={styles['mono']}>{value}</span>
      </p>
      {context === undefined ? null : <p className={styles['summaryContext']}>{context}</p>}
    </div>
  );
}

/**
 * The divided band: one hairline container, several cells, hairlines between them.
 *
 * Both reference rows on FS-UI-01 are drawn this way - the capability cells and the metric cells share one
 * box and are separated by rules, not by four boxes floating in a page gutter. Cells keep whatever semantics
 * they bring (a `Panel` inside a band is still a named region, a `SummaryCard` is still a figure with its
 * context), which is the point: the band is a layout, not a component with an opinion about its contents.
 */
export function Band({
  label,
  narrow,
  children,
}: {
  readonly label?: string;
  /** Short-figure cells: three or six across at 1440 instead of two or three. */
  readonly narrow?: boolean;
  readonly children: ReadonlyArray<ReactNode>;
}) {
  // A `group` rather than a bare region, which is the reasoning `SummaryRow` recorded: the row is a set of
  // related facts, not a landmark a reader navigates to, and an `aria-label` on a plain `div` is dropped by
  // every screen reader unless the role names it.
  return (
    <div
      className={cx(styles['band'], narrow ? styles['bandNarrow'] : undefined)}
      role="group"
      aria-label={label}
    >
      {children}
    </div>
  );
}

/**
 * A term/value list, the shape four pages each wrote for themselves.
 *
 * Analyze, Details, Compare and Release all needed "a label in one column, a fact in the next, a hairline
 * between rows". Each kept its own `.row`/`.term`/`.value` triple, so the label column width drifted
 * between pages and one page's label column was a different width from the next page's for no reason
 * anyone chose. The label column is the inspector minimum again, because that is the width the product
 * already trusts for a label column (`Analyze.module.css:139` said so first).
 */
export function FactList({ children }: { readonly children: ReactNode }) {
  return <div className={styles['facts']}>{children}</div>;
}

export function FactRow({
  term,
  children,
  title,
}: {
  readonly term: ReactNode;
  readonly children: ReactNode;
  /** The untruncated text behind a value that may be ellipsised. */
  readonly title?: string;
}) {
  return (
    <div className={styles['factRow']}>
      <span className={styles['factTerm']}>{term}</span>
      <span className={styles['factValue']} title={title}>
        {children}
      </span>
    </div>
  );
}

/** A figure that must read as a figure: mono, tabular, never re-flowed mid-column. */
export function Figure({ children }: { readonly children: ReactNode }) {
  return <span className={styles['figure']}>{children}</span>;
}

/** The quiet trailing clause after a value - the reason, the count, the caveat. */
export function Qualifier({ children }: { readonly children: ReactNode }) {
  return <span className={styles['qualifier']}>{children}</span>;
}

/**
 * "Nothing here yet", said once and the same way.
 *
 * Before U1 three pages each held their own `.empty` paragraph. A first-run reader should not meet three
 * dialects of the same sentence, and a next step belongs in an empty state because an empty state without
 * one is a dead end (prompt §7.2 puts this above new interaction).
 */
export function EmptyState({
  message,
  nextStep,
  label,
  role,
}: {
  readonly message: ReactNode;
  readonly nextStep?: ReactNode;
  /** Names the region, which is how a reader finds out *which* thing is empty. */
  readonly label?: string;
  /** `status` when the emptiness is a live result of a filter or a load. */
  readonly role?: 'status' | 'note';
}) {
  return (
    <section className={styles['empty']} aria-label={label} role={role}>
      <p className={styles['emptyMessage']}>{message}</p>
      {nextStep === undefined || nextStep === null ? null : (
        <div className={styles['emptyNext']}>{nextStep}</div>
      )}
    </section>
  );
}
