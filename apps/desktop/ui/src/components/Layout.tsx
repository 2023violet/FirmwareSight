/**
 * The page container and the wrapper that owns a table's scroll axis.
 *
 * `ScrollArea` is F2R-01's rule expressed once. That round found Analyze's Sections table rendering one
 * character per line because `display: block` was set on the `<table>` itself, which pins the box to the
 * pane and lets its columns sit *below their own minimum*; `Details.module.css` kept the measurement, and
 * U1 moved the wrapper itself into this file so the four pages that need a scroll share one.
 *
 * U1 also planned a content/detail pair here. It is not in this file: a side detail column needs a
 * selection to describe, Compare and History do not hold one, and shipping the box before the interaction
 * would be the half-built component `U1_UI_GAP_AUDIT.md` §D already refuses to fake. `U1_VALIDATION_REPORT.md`
 * carries that as a gap still open.
 */

import styles from './Layout.module.css';
import type { ReactNode } from 'react';

/** A table, a filter row, or anything wider than the pane it sits in. */
export function ScrollArea({ children, label }: { readonly children: ReactNode; readonly label?: string }) {
  return (
    <div className={styles['viewport']} role="group" aria-label={label}>
      {children}
    </div>
  );
}

/**
 * The page container: a `main`, one measure, one rhythm.
 *
 * Six CSS modules declared their own `.page` and one page (`Release.tsx:309`) rendered a `div` where the
 * other four rendered `main`, so the landmark a screen reader lands on depended on which page opened
 * first. This is the one container, and it is always `main`.
 */
export function Page({ children }: { readonly children: ReadonlyArray<ReactNode> }) {
  return <main className={styles['page']}>{children}</main>;
}
