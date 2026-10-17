/**
 * The one table (ADR-0034 D1 principle 5), and the filter row that goes above it.
 *
 * The pages hand-write fourteen `<table>`s with four local table styles. This one carries what every one
 * of them needs: a caption, `th scope`, sticky headers, numeric columns set right in tabular mono, token
 * row density and an optional row action that opens the Inspector.
 *
 * Sorting comes in two forms. A small table the page already holds in full is sorted here. A large one is
 * sorted by the shell, where the order is a fact the boundary computes (`04_TECH/14` 3). Then the page
 * passes `sort` and `onSort`, and this table only reports which column the reader chose. Filtering
 * follows the same split: `FilterBar` reports the text, and either this table or the page applies it.
 */

import { useId, useMemo, useState, type ReactNode } from 'react';

import styles from './DataTable.module.css';

import { useI18n } from '../i18n';
import { cx } from '../styles/classnames';
import { Icon } from './Icon';

export interface Column<T> {
  readonly key: string;
  readonly header: string;
  readonly cell: (row: T) => ReactNode;
  /** Right-aligned tabular mono: sizes, counts, addresses. */
  readonly numeric?: boolean;
  /** Present when this table orders its own rows by the column. */
  readonly sortValue?: (row: T) => string | number;
  /**
   * Server mode (R3): the column can be ordered, but by the shell. The table only reports the choice
   * through `onSort`; it never reorders rows it was handed (`04_TECH/14` §3).
   */
  readonly sortable?: boolean;
  /** The direction a first click asks for. Names read A→Z; quantities read largest first. */
  readonly firstDirection?: 'asc' | 'desc';
}

function canSort<T>(column: Column<T>): boolean {
  return column.sortable ?? column.sortValue !== undefined;
}

export interface SortState {
  readonly key: string;
  readonly direction: 'asc' | 'desc';
}

export function DataTable<T>({
  caption,
  columns,
  rows,
  rowKey,
  density = 'default',
  sort,
  onSort,
  filterText,
  onRowAction,
  rowActionLabel,
  empty,
  rowHeader,
  actionFirst = false,
}: {
  readonly caption: string;
  readonly columns: readonly Column<T>[];
  readonly rows: readonly T[];
  readonly rowKey: (row: T) => string;
  readonly density?: 'compact' | 'default';
  /** Controlled sort, for a table the shell orders. Omit both to sort here. */
  readonly sort?: SortState | null;
  readonly onSort?: (next: SortState) => void;
  /** Rows whose text matches this are kept, when the table filters its own rows. */
  readonly filterText?: { readonly query: string; readonly of: (row: T) => string };
  readonly onRowAction?: (row: T, trigger: HTMLElement) => void;
  readonly rowActionLabel?: (row: T) => string;
  /** Shown in place of the body when no row is left. */
  readonly empty?: string;
  /** The column whose cell names the row (`th scope="row"`). Defaults to the first column. */
  readonly rowHeader?: string;
  /**
   * Put the row action first. A wide table of ids, digests and times is wider than the minimum window, and
   * the last column is the one that scrolls out of view (F2R-02, History).
   */
  readonly actionFirst?: boolean;
}) {
  const { t } = useI18n();
  const [localSort, setLocalSort] = useState<SortState | null>(null);
  const controlled = onSort !== undefined;
  const active = controlled ? (sort ?? null) : localSort;

  const shown = useMemo(() => {
    const query = filterText?.query.trim().toLowerCase() ?? '';
    const kept =
      filterText === undefined || query === ''
        ? [...rows]
        : rows.filter((row) => filterText.of(row).toLowerCase().includes(query));
    if (controlled || active === null) {
      return kept;
    }
    const column = columns.find((c) => c.key === active.key);
    if (column?.sortValue === undefined) {
      return kept;
    }
    const value = column.sortValue;
    const sign = active.direction === 'asc' ? 1 : -1;
    // A stable sort keeps the page's own order among equal values, so ties never shuffle.
    return kept.sort((a, b) => {
      const left = value(a);
      const right = value(b);
      return (left < right ? -1 : left > right ? 1 : 0) * sign;
    });
  }, [rows, filterText, controlled, active, columns]);

  const choose = (column: Column<T>) => {
    const key = column.key;
    const next: SortState =
      active?.key === key
        ? { key, direction: active.direction === 'asc' ? 'desc' : 'asc' }
        : { key, direction: column.firstDirection ?? 'desc' };
    if (controlled) {
      onSort(next);
    } else {
      setLocalSort(next);
    }
  };

  const hasAction = onRowAction !== undefined;
  const actionCell = (row: T) =>
    onRowAction === undefined ? null : (
      <td className={styles['actionCell']}>
        <button
          type="button"
          className={styles['rowAction']}
          aria-label={rowActionLabel?.(row) ?? t('common', 'table.inspect')}
          onClick={(event) => {
            onRowAction(row, event.currentTarget);
          }}
        >
          <Icon name="evidence" />
        </button>
      </td>
    );

  return (
    <div className={styles['wrap']}>
      <table className={cx(styles['table'], styles[density])}>
        <caption className={styles['caption']}>{caption}</caption>
        <thead>
          <tr>
            {hasAction && actionFirst ? (
              <th scope="col" className={styles['actionHead']}>
                <span className={styles['spoken']}>{t('common', 'table.actions')}</span>
              </th>
            ) : null}
            {columns.map((column) => {
              const sorted = active?.key === column.key;
              return (
                <th
                  key={column.key}
                  scope="col"
                  className={column.numeric === true ? styles['num'] : undefined}
                  aria-sort={
                    !canSort(column)
                      ? undefined
                      : sorted
                        ? active.direction === 'asc'
                          ? 'ascending'
                          : 'descending'
                        : 'none'
                  }
                >
                  {!canSort(column) ? (
                    column.header
                  ) : (
                    // The header's own words are its name: a "Sort by" label here would be read as the
                    // column header of every cell. `aria-sort` on the th already says it is sortable.
                    <button
                      type="button"
                      className={styles['sort']}
                      onClick={() => {
                        choose(column);
                      }}
                    >
                      {column.header}
                      {sorted ? (
                        <span className={styles['arrow']} aria-hidden="true">
                          {active.direction === 'asc' ? '↑' : '↓'}
                        </span>
                      ) : null}
                    </button>
                  )}
                </th>
              );
            })}
            {hasAction && !actionFirst ? (
              <th scope="col" className={styles['actionHead']}>
                <span className={styles['spoken']}>{t('common', 'table.actions')}</span>
              </th>
            ) : null}
          </tr>
        </thead>
        <tbody>
          {shown.length === 0 ? (
            <tr>
              <td className={styles['empty']} colSpan={columns.length + (hasAction ? 1 : 0)}>
                {empty ?? t('common', 'table.empty')}
              </td>
            </tr>
          ) : (
            shown.map((row) => (
              <tr key={rowKey(row)}>
                {hasAction && actionFirst ? actionCell(row) : null}
                {columns.map((column, index) =>
                  (rowHeader === undefined ? index === 0 : column.key === rowHeader) ? (
                    <th key={column.key} scope="row" className={cx(styles['rowHead'], column.numeric === true ? styles['num'] : undefined)}>
                      {column.cell(row)}
                    </th>
                  ) : (
                    <td key={column.key} className={column.numeric === true ? styles['num'] : undefined}>
                      {column.cell(row)}
                    </td>
                  ),
                )}
                {hasAction && !actionFirst ? actionCell(row) : null}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

/**
 * The filter row. It reports what the reader typed, and the table or the page applies it. `/` focuses
 * it from anywhere on the page (R2 wires the key) by finding `[data-fs-filter]`, so the input needs no
 * page-chosen id.
 */
export function FilterBar({
  label,
  value,
  onChange,
  shown,
  total,
  children,
  onApply,
  applyLabel,
  formLabel,
}: {
  /** What is being filtered, already translated, e.g. "Filter sections". */
  readonly label: string;
  readonly value: string;
  readonly onChange: (value: string) => void;
  readonly shown?: number;
  readonly total?: number;
  /** Extra controls on the right, such as an Objects / Libraries switch. */
  readonly children?: ReactNode;
  /**
   * Apply mode (R3), for a filter the shell applies: typing edits a draft, and the filter is sent when the
   * reader presses Enter or the Apply button - one request per decision, not one per keystroke.
   */
  readonly onApply?: (value: string) => void;
  readonly applyLabel?: string;
  /** The form's accessible name in apply mode. */
  readonly formLabel?: string;
}) {
  const { t } = useI18n();
  const id = useId();
  const field = (
    <>
      <label htmlFor={id} className={styles['filterLabel']}>
        <Icon name="filter" />
        <span className={styles['spoken']}>{label}</span>
      </label>
      <input
        id={id}
        type="search"
        data-fs-filter=""
        className={styles['input']}
        placeholder={label}
        value={value}
        autoComplete="off"
        onChange={(event) => {
          onChange(event.target.value);
        }}
      />
    </>
  );
  const count =
    shown === undefined || total === undefined ? null : (
      <span className={styles['count']} role="status">
        {t('common', 'filter.count', { shown: String(shown), total: String(total) })}
      </span>
    );

  if (onApply !== undefined) {
    return (
      <form
        className={styles['filter']}
        aria-label={formLabel}
        onSubmit={(event) => {
          event.preventDefault();
          onApply(value);
        }}
      >
        {field}
        <button type="submit" className={styles['apply']}>
          {applyLabel ?? t('common', 'filter.apply')}
        </button>
        {count}
        {children === undefined ? null : <div className={styles['extra']}>{children}</div>}
      </form>
    );
  }

  return (
    <div className={styles['filter']}>
      {field}
      {value === '' ? null : (
        <button
          type="button"
          className={styles['clear']}
          onClick={() => {
            onChange('');
          }}
        >
          {t('common', 'filter.clear')}
        </button>
      )}
      {count}
      {children === undefined ? null : <div className={styles['extra']}>{children}</div>}
    </div>
  );
}
