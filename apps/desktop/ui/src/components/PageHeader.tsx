/**
 * How a page opens: what it is, what this screen is standing on right now, and the moves available.
 *
 * U1-A0 §B.3 counted six declarations of `.page`, five of `.header` and four of `.caption` across the
 * page CSS modules, which is the mechanism by which five pages drift into five different grammars. This
 * is the one grammar: title, a metadata line whose items are separated the same way everywhere, an
 * optional prose subhead, and an action slot that sits where the eye starts.
 *
 * The metadata line is data, not decoration, so it carries the facts the mockups show in it - artifact
 * name, version, run id, timestamp - and the caller passes each value already captioned. Nothing here
 * computes: `AGENTS.md` 3 keeps derivation in Core.
 */

import styles from './PageHeader.module.css';
import type { ReactNode } from 'react';

import { cx } from '../styles/classnames';

export function PageHeader({
  title,
  meta,
  subhead,
  actions,
}: {
  readonly title: string;
  /** The facts under the title, in the order a reader resolves them. */
  readonly meta?: ReadonlyArray<ReactNode>;
  /** One or two sentences about what this page does. Optional: a page whose header already names the
   *  artifact does not also need a paragraph. */
  readonly subhead?: ReactNode;
  readonly actions?: ReactNode;
}) {
  return (
    <header className={styles['header']}>
      <div className={styles['heading']}>
        <h1>{title}</h1>
        {meta === undefined || meta.length === 0 ? null : (
          <p className={styles['meta']}>
            {meta.map((item, index) => (
              <span key={index} className={styles['metaItem']}>
                {index === 0 ? null : (
                  <span className={styles['dot']} aria-hidden="true">
                    {' · '}
                  </span>
                )}
                {item}
              </span>
            ))}
          </p>
        )}
      </div>
      {actions === undefined || actions === null ? null : <ActionBar>{actions}</ActionBar>}
      {subhead === undefined ? null : <p className={styles['subhead']}>{subhead}</p>}
    </header>
  );
}

/**
 * The row that holds a region's moves.
 *
 * `DESIGN.md` 5 allows one Primary per region, which is a rule about ordering rather than about markup,
 * so the caller keeps that responsibility and this component keeps the alignment: moves sit at the end
 * of the header on a wide page and drop below the title rather than squeezing it when the pane narrows
 * to the frozen 1024 px minimum.
 */
export function ActionBar({ label, children }: { readonly label?: string; readonly children: ReactNode }) {
  return (
    <div className={styles['actions']} role="group" aria-label={label}>
      {children}
    </div>
  );
}

/** A page body that wants the same rhythm as the header without restating it. */
export function PageSection({
  title,
  hint,
  labelledBy,
  children,
  className,
}: {
  readonly title: ReactNode;
  /** The trailing explanation the reference screens put opposite a section title. */
  readonly hint?: ReactNode;
  readonly labelledBy?: string;
  readonly children: ReactNode;
  readonly className?: string;
}) {
  // A region needs a name either way. When the caller points at an element that carries it, the label
  // attribute would fight the reference; when it does not, the title is a string and the region is named
  // by it. Without this a section that used to answer to `getByRole('region', { name: 'Capabilities' })`
  // silently stops being findable at all.
  const namedByString = labelledBy === undefined && typeof title === 'string';
  return (
    <section
      className={cx(styles['section'], className)}
      aria-label={namedByString ? title : undefined}
      aria-labelledby={labelledBy}
    >
      <div className={styles['sectionHead']}>
        <h2 id={labelledBy}>{title}</h2>
        {hint === undefined ? null : <p className={styles['sectionHint']}>{hint}</p>}
      </div>
      {children}
    </section>
  );
}
