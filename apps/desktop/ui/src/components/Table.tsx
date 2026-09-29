/**
 * A sortable column header, and the pager that goes under a bounded table.
 *
 * Both are shared by the Analyze details and the Compare change tables so that one click on a
 * column name means one thing in the whole application. Neither one sorts or slices anything: the
 * header reports which column the reader chose and the pager reports which offset they asked for,
 * and the shell answers both (`04_TECH/14` 3, `AGENTS.md` 3).
 */

import styles from './table.module.css';
import type { SortDirDto } from '../ipc/types';

export function SortHeader({
  label,
  sortable = true,
  active = false,
  direction,
  onSort,
}: {
  readonly label: string;
  /** A column that carries no ordering, like a role or a kind. */
  readonly sortable?: boolean;
  readonly active?: boolean;
  readonly direction: SortDirDto;
  /** Which column this header orders by; the caller closes over it. */
  readonly onSort?: () => void;
}) {
  if (!sortable || onSort === undefined) {
    return <th scope="col" className={styles['head']}>{label}</th>;
  }
  return (
    <th
      scope="col"
      className={styles['head']}
      aria-sort={active ? (direction === 'asc' ? 'ascending' : 'descending') : 'none'}
    >
      <button
        type="button"
        className={styles['sort']}
        aria-label={`Sort by ${label}`}
        onClick={onSort}
      >
        {label}
        {active ? <span className={styles['arrow']}>{direction === 'asc' ? '↑' : '↓'}</span> : null}
      </button>
    </th>
  );
}

/**
 * Where the reader is in the table, and the two moves available.
 *
 * The next offset is the one the shell reported, not one this page computed: a page size the
 * boundary clamped would otherwise silently skip rows.
 */
export function Pager({
  total,
  offset,
  shown,
  nextOffset,
  label,
  onOffset,
}: {
  readonly total: number;
  readonly offset: number;
  readonly shown: number;
  readonly nextOffset: number | null;
  /** What the rows are, so the announcement says which table moved. */
  readonly label: string;
  readonly onOffset: (offset: number) => void;
}) {
  return (
    <div className={styles['pager']}>
      <p className={styles['status']} role="status" aria-live="polite">
        {shown === 0
          ? `No ${label} in this page of ${String(total)}`
          : `Showing ${String(offset + 1)} to ${String(offset + shown)} of ${String(total)} ${label}`}
      </p>
      <button
        type="button"
        className={styles['page']}
        disabled={offset === 0}
        onClick={() => {
          onOffset(0);
        }}
      >
        Previous page
      </button>
      <button
        type="button"
        className={styles['page']}
        disabled={nextOffset === null}
        onClick={() => {
          onOffset(nextOffset ?? offset);
        }}
      >
        Next page
      </button>
    </div>
  );
}
