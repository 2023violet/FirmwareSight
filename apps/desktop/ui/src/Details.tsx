/**
 * The P1 detail screen: the largest stored payload, then Sections, Symbols and Evidence, one
 * bounded page at a time, plus the Evidence Inspector.
 *
 * Three rules shape this component, and they are all about where a decision is made.
 *
 * 1. **The rows are the shell's.** A filter, a sort, a class and a page offset travel to Rust, and
 *    the page comes back. Nothing here slices an array it happens to already hold: `04_TECH/14` 3
 *    puts the whole table behind the boundary, and a client-side filter would silently only ever
 *    search the rows that fit in one page.
 * 2. **The unit switch is presentation.** It is a prop, not a query dependency, so choosing KiB
 *    re-labels text and issues no request. It is applied to byte counts only; addresses, offsets,
 *    hashes, counts and ordinals are never scaled (US-001 addendum 3).
 * 3. **A failure belongs to this area.** A page that would not load shows an inspectable error here,
 *    while the summary the reader already trusted stays on screen. The snapshot id this component was
 *    given is the last good one, and it does not change because a later analysis failed.
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import styles from './Details.module.css';
import { Pager, SortHeader } from './components/Table';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import { formatSize, formatOptional, type SizeUnit } from './format';
import { queryEvidence, querySections, querySymbols } from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type {
  EvidenceClassDto,
  EvidencePageDto,
  EvidenceRowDto,
  EvidenceSortDto,
  SectionPageDto,
  SectionRowDto,
  SectionSortDto,
  SortDirDto,
  SymbolPageDto,
  SymbolRowDto,
  SymbolSortDto,
} from './ipc/types';
import type { ErrorEnvelopeDto } from './ipc/types';
import { cx } from './styles/classnames';

type Tab = 'sections' | 'symbols' | 'evidence';

/**
 * A page together with the tab it answers.
 *
 * The tab and the rows are one fact, not two: a page kept in its own piece of state is painted
 * under whichever tab is current, and for the render between choosing a tab and the effect that
 * clears the page that put section rows inside a symbol table's columns.
 */
type Loaded =
  | { readonly tab: 'sections'; readonly page: SectionPageDto }
  | { readonly tab: 'symbols'; readonly page: SymbolPageDto }
  | { readonly tab: 'evidence'; readonly page: EvidencePageDto };

function tagged<K extends Loaded['tab']>(
  tab: K,
  outcome: IpcOutcome<Extract<Loaded, { tab: K }>['page']>,
): IpcOutcome<Loaded> {
  // The pairing is checked by `K` at the call site; the assertion only tells TypeScript that a
  // tab and a page chosen together are one of the three combinations `Loaded` allows.
  return outcome.ok ? { ok: true, value: { tab, page: outcome.value } as Loaded } : outcome;
}

const TABS: readonly { readonly key: Tab; readonly label: string }[] = [
  { key: 'sections', label: 'Sections' },
  { key: 'symbols', label: 'Symbols' },
  { key: 'evidence', label: 'Evidence' },
];

/** The contributors list is one small page of the same table, ordered by what it measures. */
const CONTRIBUTOR_LIMIT = 5;

export function Details({
  snapshotId,
  unit,
  onUnitChange,
}: {
  readonly snapshotId: string;
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
}) {
  const [tab, setTab] = useState<Tab>('sections');
  const [contributors, setContributors] = useState<readonly SectionRowDto[] | null>(null);

  // One page of state per active tab, reset when the reader changes tab: the filter that belongs to
  // the symbol table must not silently apply to the sections table.
  const [offset, setOffset] = useState(0);
  const [filter, setFilter] = useState<string | null>(null);
  const [filterDraft, setFilterDraft] = useState('');
  const [classification, setClassification] = useState<EvidenceClassDto | null>(null);
  const [sort, setSort] = useState<SectionSortDto | SymbolSortDto | EvidenceSortDto>('index');
  const [direction, setDirection] = useState<SortDirDto>('asc');
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [loading, setLoading] = useState(false);
  const [inspected, setInspected] = useState<EvidenceRowDto | null>(null);

  /**
   * Only the newest response may paint, and each surface counts its own requests: the contributor
   * list and the active table are two different questions, and an answer to one is never stale for
   * the other.
   */
  const pageRequest = useRef(0);
  const contributorRequest = useRef(0);

  useEffect(() => {
    const id = ++contributorRequest.current;
    void querySections({
      snapshotId,
      filter: null,
      sort: 'fileSize',
      direction: 'desc',
      offset: 0,
      limit: CONTRIBUTOR_LIMIT,
    }).then((outcome) => {
      if (contributorRequest.current !== id) {
        return;
      }
      setContributors(outcome.ok ? outcome.value.rows : null);
    });
  }, [snapshotId]);

  useEffect(() => {
    const id = ++pageRequest.current;
    setLoading(true);
    // The rows on screen answer the *previous* question, so they go before the new one is asked.
    setLoaded(null);
    const call =
      tab === 'sections'
        ? querySections({
            snapshotId,
            filter,
            sort: sort as SectionSortDto,
            direction,
            offset,
            limit: null,
          }).then((outcome) => tagged(tab, outcome))
        : tab === 'symbols'
          ? querySymbols({
              snapshotId,
              filter,
              sort: sort as SymbolSortDto,
              direction,
              offset,
              limit: null,
            }).then((outcome) => tagged(tab, outcome))
          : queryEvidence({
              snapshotId,
              filter,
              classification,
              sort: sort as EvidenceSortDto,
              direction,
              offset,
              limit: null,
            }).then((outcome) => tagged(tab, outcome));

    void call.then((outcome) => {
      if (pageRequest.current !== id) {
        return;
      }
      setLoading(false);
      if (outcome.ok) {
        setLoaded(outcome.value);
        setError(null);
      } else {
        setLoaded(null);
        setError(outcome.envelope);
      }
    });
  }, [tab, snapshotId, filter, classification, sort, direction, offset]);

  /**
   * Choosing a column orders by it; choosing it again reverses the order. Sorting happens in
   * SQLite, so the second click re-reads the page rather than flipping the rows in memory.
   */
  const chooseSort = useCallback(
    (column: SectionSortDto | SymbolSortDto | EvidenceSortDto) => {
      setOffset(0);
      if (sort === column) {
        setDirection(direction === 'asc' ? 'desc' : 'asc');
        return;
      }
      setSort(column);
      setDirection('asc');
    },
    [direction, sort],
  );

  const tabId = (key: Tab) => `fs-tab-${key}`;

  return (
    <section className={styles['area']} aria-label="Snapshot details">
      <Contributors rows={contributors} unit={unit} />

      <SizeUnitSwitch unit={unit} onSelect={onUnitChange} />

      <div role="tablist" aria-label="Analyze details" className={styles['tabs']}>
        {TABS.map((entry) => (
          <button
            key={entry.key}
            type="button"
            role="tab"
            id={tabId(entry.key)}
            aria-selected={tab === entry.key}
            className={cx(styles['tab'], tab === entry.key ? styles['tabActive'] : undefined)}
            onClick={() => {
              setTab(entry.key);
              setOffset(0);
              setFilter(null);
              setFilterDraft('');
              setClassification(null);
              setSort(entry.key === 'sections' ? 'index' : entry.key === 'symbols' ? 'ordinal' : 'field');
              setDirection('asc');
              setInspected(null);
            }}
          >
            {entry.label}
          </button>
        ))}
      </div>

      {loading ? (
        <p className={styles['status']} role="status" aria-live="polite">
          Loading…
        </p>
      ) : null}

      {error === null ? null : <DetailError envelope={error} />}

      {loaded === null || loaded.tab !== tab || error !== null ? null : (
        <div role="tabpanel" id={`fs-panel-${tab}`} aria-labelledby={tabId(tab)} tabIndex={0}>
          <FilterForm
            tab={tab}
            draft={filterDraft}
            classification={classification}
            onDraftChange={setFilterDraft}
            onApply={(value, class_) => {
              setFilterDraft(value);
              setFilter(value.length === 0 ? null : value);
              setClassification(class_);
              setOffset(0);
            }}
            onClassification={(class_) => {
              setClassification(class_);
              setOffset(0);
            }}
          />

          {loaded.tab === 'sections' ? (
            <SectionTable
              rows={loaded.page.rows}
              unit={unit}
              sort={sort as SectionSortDto}
              direction={direction}
              onSort={chooseSort}
            />
          ) : null}
          {loaded.tab === 'symbols' ? (
            <SymbolTable
              rows={loaded.page.rows}
              unit={unit}
              sort={sort as SymbolSortDto}
              direction={direction}
              onSort={chooseSort}
            />
          ) : null}
          {loaded.tab === 'evidence' ? (
            <EvidenceTable
              rows={loaded.page.rows}
              sort={sort as EvidenceSortDto}
              direction={direction}
              onSort={chooseSort}
              inspected={inspected}
              onInspect={setInspected}
            />
          ) : null}

          <Pager
            total={loaded.page.total}
            offset={loaded.page.offset}
            shown={loaded.page.rows.length}
            nextOffset={loaded.page.nextOffset}
            label={tab}
            onOffset={setOffset}
          />
        </div>
      )}
    </section>
  );
}

function Contributors({
  rows,
  unit,
}: {
  readonly rows: readonly SectionRowDto[] | null;
  readonly unit: SizeUnit;
}) {
  if (rows === null || rows.length === 0) {
    return null;
  }
  return (
    <section className={styles['contributors']} aria-label="Largest stored payload">
      <h2>Largest stored payload</h2>
      <p className={styles['hint']}>
        The {rows.length} sections holding the most bytes in the file. This counts stored bytes,
        which is not the same question as what a device budget charges.
      </p>
      <ol className={styles['contributorList']}>
        {rows.map((row) => (
          <li key={row.index} className={styles['contributor']}>
            <span className={styles['name']}>
              <span>{row.name ?? 'Unknown'}</span>
              <span className={styles['role']}>{row.role}</span>
            </span>
            <span className={cx(styles['mono'], styles['number'])}>{formatSize(row.fileSize, unit)}</span>
          </li>
        ))}
      </ol>
    </section>
  );
}

function FilterForm({
  tab,
  draft,
  classification,
  onDraftChange,
  onApply,
  onClassification,
}: {
  readonly tab: Tab;
  readonly draft: string;
  readonly classification: EvidenceClassDto | null;
  readonly onDraftChange: (value: string) => void;
  readonly onApply: (value: string, classification: EvidenceClassDto | null) => void;
  readonly onClassification: (classification: EvidenceClassDto | null) => void;
}) {
  const noun = tab === 'sections' ? 'sections' : tab === 'symbols' ? 'symbols' : 'evidence';
  const subject = tab === 'evidence' ? 'field' : 'name';
  const id = `fs-filter-${tab}`;

  return (
    <form
      className={styles['filter']}
      aria-label={`Filter ${noun}`}
      onSubmit={(event) => {
        event.preventDefault();
        onApply(draft, classification);
      }}
    >
      <label className={styles['filterLabel']} htmlFor={id}>
        Filter {noun} by {subject}
      </label>
      <input
        id={id}
        className={styles['filterInput']}
        type="text"
        autoComplete="off"
        value={draft}
        onChange={(event) => {
          onDraftChange(event.target.value);
        }}
      />
      {tab === 'evidence' ? (
        <span className={styles['class']}>
          <label className={styles['filterLabel']} htmlFor="fs-evidence-class">
            Evidence class
          </label>
          <select
            id="fs-evidence-class"
            aria-label="Evidence class"
            value={classification ?? ''}
            onChange={(event) => {
              const value = event.target.value;
              onClassification(value === '' ? null : (value as EvidenceClassDto));
            }}
          >
            <option value="">All classes</option>
            <option value="observed">observed</option>
            <option value="derived">derived</option>
            <option value="declared">declared</option>
            <option value="unknown">unknown</option>
          </select>
        </span>
      ) : null}
      <button type="submit" className={styles['apply']}>
        Apply filter
      </button>
    </form>
  );
}

/** A fact that was stored unknown, rendered as the word and its reason - never as `0`. */
function Unknown({ reason }: { readonly reason?: string | null }) {
  return (
    <span className={styles['unknown']}>
      Unknown
      {reason === undefined || reason === null || reason.length === 0 ? null : (
        <span className={styles['note']}>{reason}</span>
      )}
    </span>
  );
}

function SectionTable({
  rows,
  unit,
  sort,
  direction,
  onSort,
}: {
  readonly rows: readonly SectionRowDto[];
  readonly unit: SizeUnit;
  readonly sort: SectionSortDto;
  readonly direction: SortDirDto;
  readonly onSort: (column: SectionSortDto) => void;
}) {
  return (
    <table className={styles['table']}>
      <caption className={styles['caption']}>
        Sections of the analyzed artifact, as the parser recorded them
      </caption>
      <thead>
        <tr>
          <SortHeader label="Index" sortable direction={direction} active={sort === 'index'} onSort={() => onSort('index')} />
          <SortHeader label="Name" sortable direction={direction} active={sort === 'name'} onSort={() => onSort('name')} />
          <SortHeader
              label="Role"
              sortable={false}
              direction={direction}
            />
          <SortHeader
              label="Flags"
              sortable={false}
              direction={direction}
            />
          <SortHeader
              label="Virtual address"
              sortable={false}
              direction={direction}
            />
          <SortHeader
              label="Load address"
              sortable={false}
              direction={direction}
            />
          <SortHeader
              label="File offset"
              sortable={false}
              direction={direction}
            />
          <SortHeader label="File size" sortable direction={direction} active={sort === 'fileSize'} onSort={() => onSort('fileSize')} />
          <SortHeader label="Memory size" sortable direction={direction} active={sort === 'memorySize'} onSort={() => onSort('memorySize')} />
          <SortHeader
              label="Region"
              sortable={false}
              direction={direction}
            />
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr key={row.index}>
            <td className={styles['mono']}>{row.index}</td>
            <td>
              {row.name === null ? <Unknown reason={row.nameUnknownReason} /> : row.name}
            </td>
            <td>{row.role}</td>
            <td className={styles['mono']}>{flags(row)}</td>
            <td className={styles['mono']}>
              {row.virtualAddress ?? <Unknown reason={row.virtualAddressUnknownReason} />}
            </td>
            <td className={styles['mono']}>
              {row.loadAddress ?? <Unknown reason={row.loadAddressUnknownReason} />}
            </td>
            <td className={styles['mono']}>{row.fileOffset ?? <Unknown />}</td>
            <td className={styles['mono']}>{formatSize(row.fileSize, unit)}</td>
            <td className={styles['mono']}>
              {row.memorySize === null ? (
                <Unknown reason={row.memorySizeUnknownReason} />
              ) : (
                formatSize(row.memorySize, unit)
              )}
            </td>
            <td>
              {row.region === null ? <Unknown reason={row.regionUnknownReason} /> : row.region}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function flags(row: SectionRowDto): string {
  return `${row.alloc ? 'A' : '-'}${row.write ? 'W' : '-'}${row.execute ? 'X' : '-'}`;
}

function SymbolTable({
  rows,
  unit,
  sort,
  direction,
  onSort,
}: {
  readonly rows: readonly SymbolRowDto[];
  readonly unit: SizeUnit;
  readonly sort: SymbolSortDto;
  readonly direction: SortDirDto;
  readonly onSort: (column: SymbolSortDto) => void;
}) {
  return (
    <table className={styles['table']}>
      <caption className={styles['caption']}>
        Symbols, ordered and filtered by the shell rather than by this page
      </caption>
      <thead>
        <tr>
          <SortHeader label="Ordinal" sortable direction={direction} active={sort === 'ordinal'} onSort={() => onSort('ordinal')} />
          <SortHeader label="Name" sortable direction={direction} active={sort === 'name'} onSort={() => onSort('name')} />
          <SortHeader label="Address" sortable direction={direction} active={sort === 'address'} onSort={() => onSort('address')} />
          <SortHeader label="Size" sortable direction={direction} active={sort === 'size'} onSort={() => onSort('size')} />
          <SortHeader
              label="Kind"
              sortable={false}
              direction={direction}
            />
          <SortHeader
              label="Binding"
              sortable={false}
              direction={direction}
            />
          <SortHeader
              label="Section"
              sortable={false}
              direction={direction}
            />
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr key={row.ordinal}>
            <td className={styles['mono']} title="A row position in this build, not an identity">
              {row.ordinal}
            </td>
            <td>{row.name === null ? <Unknown reason={row.nameUnknownReason} /> : row.name}</td>
            <td className={styles['mono']}>{row.address ?? <Unknown />}</td>
            <td className={styles['mono']}>
              {row.size === null ? (
                <Unknown reason={row.sizeUnknownReason} />
              ) : (
                formatSize(row.size, unit)
              )}
            </td>
            <td>{row.kind}</td>
            <td>{row.binding}</td>
            <td className={styles['mono']}>{row.sectionRef}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function EvidenceTable({
  rows,
  sort,
  direction,
  onSort,
  inspected,
  onInspect,
}: {
  readonly rows: readonly EvidenceRowDto[];
  readonly sort: EvidenceSortDto;
  readonly direction: SortDirDto;
  readonly onSort: (column: EvidenceSortDto) => void;
  readonly inspected: EvidenceRowDto | null;
  readonly onInspect: (row: EvidenceRowDto) => void;
}) {
  return (
    <>
      {inspected === null ? null : <Inspector row={inspected} />}

      <table className={styles['table']}>
        <caption className={styles['caption']}>
          Recorded facts and the class each one belongs to
        </caption>
        <thead>
          <tr>
            <SortHeader
              label="Detail"
              sortable={false}
              direction={direction}
            />
            <SortHeader label="Field" sortable direction={direction} active={sort === 'field'} onSort={() => onSort('field')} />
            <SortHeader
              label="Classification"
              sortable
              direction={direction}
              active={sort === 'classification'}
              onSort={() => onSort('classification')}
            />
            <SortHeader
              label="Value"
              sortable={false}
              direction={direction}
            />
            <SortHeader
              label="Source"
              sortable={false}
              direction={direction}
            />
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.id}>
              <td>
                <button
                  type="button"
                  className={styles['inspect']}
                  aria-label={`Inspect ${row.field}`}
                  onClick={() => {
                    onInspect(row);
                  }}
                >
                  Inspect
                </button>
              </td>
              <td>{row.field}</td>
              <td>{row.classification}</td>
              <td className={cx(styles['mono'], styles['value'])}>{row.rawValue}</td>
              <td className={styles['mono']}>{row.sourceLocator}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}

/**
 * The whole provenance of one fact.
 *
 * A locator a reader can go back and check is the point of the evidence model, so it is shown in
 * full rather than truncated: shortening it here would defeat the reason it was stored. It is
 * painted above the table because a table taller than the window buries the answer below it.
 */
function Inspector({ row }: { readonly row: EvidenceRowDto }) {
  return (
    <section className={styles['inspector']} aria-label="Evidence detail">
      <h2>{row.field}</h2>
      <dl className={styles['inspectorList']}>
        <div>
          <dt>Classification</dt>
          <dd>{row.classification}</dd>
        </div>
        <div>
          <dt>Source</dt>
          <dd className={styles['mono']}>{row.sourceType}</dd>
        </div>
        <div>
          <dt>Locator</dt>
          <dd className={styles['mono']}>{row.sourceLocator}</dd>
        </div>
        <div>
          <dt>Value</dt>
          <dd className={styles['mono']}>{row.rawValue}</dd>
        </div>
        <div>
          <dt>Rule</dt>
          <dd className={styles['mono']}>{row.rule}</dd>
        </div>
        <div>
          <dt>Confidence</dt>
          <dd>{formatOptional(row.confidence)}</dd>
        </div>
        <div>
          <dt>Evidence id</dt>
          <dd className={styles['mono']}>{row.id}</dd>
        </div>
      </dl>
    </section>
  );
}

function DetailError({ envelope }: { readonly envelope: ErrorEnvelopeDto }) {
  return (
    <section className={styles['error']} role="alert" aria-label="Detail query failed">
      <h2>The details would not load</h2>
      <p>{envelope.message}</p>
      <p className={styles['monoSmall']}>{envelope.code}</p>
      {envelope.details === null ? null : <p className={styles['monoSmall']}>{envelope.details}</p>}
      {envelope.remediation === null ? null : <p>{envelope.remediation}</p>}
      <p className={styles['monoSmall']}>Diagnostics ID {envelope.operationId}</p>
    </section>
  );
}
