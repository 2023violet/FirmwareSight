/**
 * The P1 detail screen: the largest stored payload, then Sections, Symbols, Evidence and the M2
 * Objects and Libraries tables (ADR-0033), one bounded page at a time, plus the Evidence Inspector.
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
import { ScrollArea } from './components/Layout';
import { RankBar } from './components/RankBar';
import { Pager, SortHeader } from './components/Table';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import { StateBadge } from './components/StateBadge';
import { formatSize, formatOptional, type SizeUnit } from './format';
import { useI18n } from './i18n';
import type { Catalog } from './i18n/en';
import { queryEvidence, queryObjects, querySections, querySymbols } from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type {
  EvidenceClassDto,
  EvidencePageDto,
  EvidenceRowDto,
  EvidenceSortDto,
  ObjectPageDto,
  ObjectRowDto,
  ObjectSortDto,
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

type Tab = 'sections' | 'symbols' | 'evidence' | 'objects' | 'libraries';

type AnySort = SectionSortDto | SymbolSortDto | EvidenceSortDto | ObjectSortDto;

/**
 * The keys this page owns. Details writes into the `analyze` namespace rather than opening a tenth
 * one (`ADR-0032` D4): it is the same analysis's rows, and a reader switching between the report
 * above and these tables must not meet two vocabularies for one artifact.
 */
type DetailsKey = keyof Catalog['analyze'];

/** The population a pager line counts, in the reader's language. */
const PAGER_NOUN: Readonly<Record<Tab, DetailsKey>> = {
  sections: 'details.filter.noun.sections',
  symbols: 'details.filter.noun.symbols',
  evidence: 'details.filter.noun.evidence',
  objects: 'details.filter.noun.objects',
  libraries: 'details.filter.noun.libraries',
};

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
  | { readonly tab: 'evidence'; readonly page: EvidencePageDto }
  | { readonly tab: 'objects' | 'libraries'; readonly page: ObjectPageDto };

function tagged<K extends Loaded['tab']>(
  tab: K,
  outcome: IpcOutcome<Extract<Loaded, { tab: K }>['page']>,
): IpcOutcome<Loaded> {
  // The pairing is checked by `K` at the call site; the assertion only tells TypeScript that a
  // tab and a page chosen together are one of the combinations `Loaded` allows.
  return outcome.ok ? { ok: true, value: { tab, page: outcome.value } as Loaded } : outcome;
}

const TABS: readonly { readonly key: Tab; readonly labelKey: DetailsKey }[] = [
  { key: 'sections', labelKey: 'details.tab.sections' },
  { key: 'symbols', labelKey: 'details.tab.symbols' },
  { key: 'evidence', labelKey: 'details.tab.evidence' },
  { key: 'objects', labelKey: 'details.tab.objects' },
  { key: 'libraries', labelKey: 'details.tab.libraries' },
];

/** Where a freshly chosen tab starts. The object tables open ranked by Flash, largest first. */
function initialOrder(tab: Tab): { readonly sort: AnySort; readonly direction: SortDirDto } {
  switch (tab) {
    case 'sections':
      return { sort: 'index', direction: 'asc' };
    case 'symbols':
      return { sort: 'ordinal', direction: 'asc' };
    case 'evidence':
      return { sort: 'field', direction: 'asc' };
    case 'objects':
    case 'libraries':
      return { sort: 'flash', direction: 'desc' };
  }
}

/** The contributors list is one small page of the same table, ordered by what it measures. */
const CONTRIBUTOR_LIMIT = 5;

export function Details({
  snapshotId,
  unit,
  onUnitChange,
  stale = false,
}: {
  readonly snapshotId: string;
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
  /**
   * Whether these tables belong to a retained snapshot rather than to the attempt now in the row.
   *
   * The report above already says so; this is the same fact in the accessible name of a region a
   * screen-reader jumps to directly, so the label does not depend on having read the paragraph first.
   */
  readonly stale?: boolean;
}) {
  const [tab, setTab] = useState<Tab>('sections');
  const [contributors, setContributors] = useState<readonly SectionRowDto[] | null>(null);

  // One page of state per active tab, reset when the reader changes tab: the filter that belongs to
  // the symbol table must not silently apply to the sections table.
  const [offset, setOffset] = useState(0);
  const [filter, setFilter] = useState<string | null>(null);
  const [filterDraft, setFilterDraft] = useState('');
  const [classification, setClassification] = useState<EvidenceClassDto | null>(null);
  const [sort, setSort] = useState<AnySort>('index');
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
          : tab === 'evidence'
            ? queryEvidence({
                snapshotId,
                filter,
                classification,
                sort: sort as EvidenceSortDto,
                direction,
                offset,
                limit: null,
              }).then((outcome) => tagged(tab, outcome))
            : queryObjects({
                snapshotId,
                libraries: tab === 'libraries',
                filter,
                sort: sort as ObjectSortDto,
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
    (column: AnySort) => {
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

  const { t } = useI18n();
  const tabId = (key: Tab) => `fs-tab-${key}`;

  return (
    <section
      className={styles['area']}
      aria-label={t('analyze', stale ? 'details.region.stale' : 'details.region.live')}
    >
      <SizeUnitSwitch unit={unit} onSelect={onUnitChange} />

      <div role="tablist" aria-label={t('analyze', 'details.tablist')} className={styles['tabs']}>
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
              const order = initialOrder(entry.key);
              setSort(order.sort);
              setDirection(order.direction);
              setInspected(null);
            }}
          >
            {t('analyze', entry.labelKey)}
          </button>
        ))}
      </div>

      {loading ? (
        <p className={styles['status']} role="status" aria-live="polite">
          {t('analyze', 'details.loading')}
        </p>
      ) : null}

      {error === null ? null : <DetailError envelope={error} />}

      {loaded === null || loaded.tab !== tab || error !== null ? null : (loaded.tab === 'objects' ||
          loaded.tab === 'libraries') &&
        !loaded.page.available ? (
        <div role="tabpanel" id={`fs-panel-${tab}`} aria-labelledby={tabId(tab)} tabIndex={0}>
          <ObjectsUnavailable reason={loaded.page.reason} />
        </div>
      ) : (
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
          {loaded.tab === 'objects' || loaded.tab === 'libraries' ? (
            <ObjectTable
              libraries={loaded.tab === 'libraries'}
              page={loaded.page}
              unit={unit}
              sort={sort as ObjectSortDto}
              direction={direction}
              onSort={chooseSort}
            />
          ) : null}

          <Pager
            total={loaded.page.total}
            offset={loaded.page.offset}
            shown={loaded.page.rows.length}
            nextOffset={loaded.page.nextOffset}
            label={t('analyze', PAGER_NOUN[tab])}
            onOffset={setOffset}
          />
        </div>
      )}

      {/* U1P §9 moved this below the table rather than above it. The ranked payload list is context that
          explains the rows a reader is looking at, and on the reference's Analyze screen the table is the
          first object on the page, not the second. */}
      <Contributors rows={contributors} unit={unit} />
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
  // The bar is a ratio against the largest row in this same list, so it says "half of the biggest",
  // never "half of the image". The number beside it is the fact; the bar only accelerates it, which is
  // why it carries no colour-only meaning and is hidden from the accessibility tree.
  const { t } = useI18n();
  const largest = Math.max(...rows.map((row) => row.fileSize));
  return (
    <section className={styles['contributors']} aria-label={t('analyze', 'details.contributors.title')}>
      <h2>{t('analyze', 'details.contributors.title')}</h2>
      <p className={styles['hint']}>
        {t('analyze', 'details.contributors.hint', { count: String(rows.length) })}
      </p>
      <ol className={styles['contributorList']}>
        {rows.map((row) => (
          <li key={row.index}>
            <RankBar
              label={
                <>
                  {row.name ?? t('analyze', 'details.unknown')}
                  <span className={styles['role']}>{row.role}</span>
                </>
              }
              ratio={largest > 0 ? row.fileSize / largest : null}
              value={formatSize(row.fileSize, unit)}
            />
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
  const { t } = useI18n();
  const noun = t('analyze', PAGER_NOUN[tab]);
  const subject = t('analyze', tab === 'evidence' ? 'details.filter.subject.field' : 'details.filter.subject.name');
  // The id is built from the tab key, not from the translated noun: a DOM id does not move with the
  // language, and no reader sees it.
  const id = `fs-filter-${tab}`;

  return (
    <form
      className={styles['filter']}
      aria-label={t('analyze', 'details.filter.aria', { noun })}
      onSubmit={(event) => {
        event.preventDefault();
        onApply(draft, classification);
      }}
    >
      <label className={styles['filterLabel']} htmlFor={id}>
        {t('analyze', 'details.filter.label', { noun, subject })}
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
            {t('analyze', 'details.filter.class')}
          </label>
          <select
            id="fs-evidence-class"
            aria-label={t('analyze', 'details.filter.class')}
            value={classification ?? ''}
            onChange={(event) => {
              const value = event.target.value;
              onClassification(value === '' ? null : (value as EvidenceClassDto));
            }}
          >
            {/* observed / derived / declared / unknown are Core's own evidence classes, and the
                option's value is that word, so translating the label would make the control read one
                thing and send another. They are deliberately not in the catalog (`ADR-0032` D5). */}
            <option value="">{t('analyze', 'details.filter.allClasses')}</option>
            <option value="observed">observed</option>
            <option value="derived">derived</option>
            <option value="declared">declared</option>
            <option value="unknown">unknown</option>
          </select>
        </span>
      ) : null}
      <button type="submit" className={styles['apply']}>
        {t('analyze', 'details.filter.apply')}
      </button>
    </form>
  );
}

/** A fact that was stored unknown, rendered as the word and its reason - never as `0`. */
function Unknown({ reason }: { readonly reason?: string | null }) {
  const { t } = useI18n();
  return (
    <span className={styles['unknown']}>
      {t('analyze', 'details.unknown')}
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
  const { t } = useI18n();
  return (
    <ScrollArea label={t('analyze', 'details.sections.table')}>
      <table className={styles['table']}>
        <caption className={styles['caption']}>
          {t('analyze', 'details.sections.caption')}
        </caption>
        <thead>
          <tr>
            <SortHeader label={t('analyze', 'details.col.index')} sortable direction={direction} active={sort === 'index'} onSort={() => onSort('index')} />
            <SortHeader label={t('analyze', 'details.col.name')} sortable direction={direction} active={sort === 'name'} onSort={() => onSort('name')} />
            <SortHeader
                label={t('analyze', 'details.col.role')}
                sortable={false}
                direction={direction}
              />
            <SortHeader
                label={t('analyze', 'details.col.flags')}
                sortable={false}
                direction={direction}
              />
            <SortHeader
                label={t('analyze', 'details.col.virtualAddress')}
                sortable={false}
                direction={direction}
              />
            <SortHeader
                label={t('analyze', 'details.col.loadAddress')}
                sortable={false}
                direction={direction}
              />
            <SortHeader
                label={t('analyze', 'details.col.fileOffset')}
                sortable={false}
                direction={direction}
              />
            <SortHeader label={t('analyze', 'details.col.fileSize')} sortable direction={direction} active={sort === 'fileSize'} onSort={() => onSort('fileSize')} />
            <SortHeader label={t('analyze', 'details.col.memorySize')} sortable direction={direction} active={sort === 'memorySize'} onSort={() => onSort('memorySize')} />
            <SortHeader
                label={t('analyze', 'details.col.region')}
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
              <td className={styles['mono']}>
                {row.fileOffset ?? <Unknown reason={row.fileOffsetUnknownReason} />}
              </td>
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
    </ScrollArea>
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
  const { t } = useI18n();
  return (
    <ScrollArea label={t('analyze', 'details.symbols.table')}>
      <table className={styles['table']}>
        <caption className={styles['caption']}>
          {t('analyze', 'details.symbols.caption')}
        </caption>
        <thead>
          <tr>
            <SortHeader label={t('analyze', 'details.col.ordinal')} sortable direction={direction} active={sort === 'ordinal'} onSort={() => onSort('ordinal')} />
            <SortHeader label={t('analyze', 'details.col.name')} sortable direction={direction} active={sort === 'name'} onSort={() => onSort('name')} />
            <SortHeader label={t('analyze', 'details.col.address')} sortable direction={direction} active={sort === 'address'} onSort={() => onSort('address')} />
            <SortHeader label={t('analyze', 'details.col.size')} sortable direction={direction} active={sort === 'size'} onSort={() => onSort('size')} />
            <SortHeader
                label={t('analyze', 'details.col.kind')}
                sortable={false}
                direction={direction}
              />
            <SortHeader
                label={t('analyze', 'details.col.binding')}
                sortable={false}
                direction={direction}
              />
            <SortHeader
                label={t('analyze', 'details.col.section')}
                sortable={false}
                direction={direction}
              />
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.ordinal}>
              <td className={styles['mono']} title={t('analyze', 'details.ordinalHint')}>
                {row.ordinal}
              </td>
              <td>{row.name === null ? <Unknown reason={row.nameUnknownReason} /> : row.name}</td>
              <td className={styles['mono']}>
                {row.address ?? <Unknown reason={row.addressUnknownReason} />}
              </td>
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
    </ScrollArea>
  );
}

/**
 * A build without object rows. UNKNOWN, not N/A: the question applies to every image, and a MAP
 * would answer it. The sentence says what is missing, and the hint says how to get it.
 */
function ObjectsUnavailable({ reason }: { readonly reason: string | null }) {
  const { t } = useI18n();
  return (
    <div className={styles['objectsUnavailable']}>
      <StateBadge state="UNKNOWN" label={t('analyze', 'details.objects.unavailable')} note={reason ?? undefined} />
      <p className={styles['hint']}>{t('analyze', 'details.objects.nextStep')}</p>
    </div>
  );
}

/** Basis points as the reader's percentage; a share too small to show is said to be so, not shown as 0. */
function share(basisPoints: number | null, bytes: number | null): string | null {
  if (basisPoints === null) {
    return null;
  }
  if (basisPoints === 0 && bytes !== null && bytes > 0) {
    return '<0.01%';
  }
  return `${(basisPoints / 100).toFixed(2)}%`;
}

function ObjectTable({
  libraries,
  page,
  unit,
  sort,
  direction,
  onSort,
}: {
  readonly libraries: boolean;
  readonly page: ObjectPageDto;
  readonly unit: SizeUnit;
  readonly sort: ObjectSortDto;
  readonly direction: SortDirDto;
  readonly onSort: (column: ObjectSortDto) => void;
}) {
  const { t } = useI18n();
  const noShare = t('analyze', 'details.objects.noShare');
  return (
    <>
      <ScrollArea label={t('analyze', libraries ? 'details.libraries.table' : 'details.objects.table')}>
        <table className={styles['table']}>
          <caption className={styles['caption']}>
            {t('analyze', libraries ? 'details.libraries.caption' : 'details.objects.caption')}
          </caption>
          <thead>
            <tr>
              <SortHeader
                label={t('analyze', libraries ? 'details.col.library' : 'details.col.object')}
                sortable
                direction={direction}
                active={sort === 'name'}
                onSort={() => onSort('name')}
              />
              {libraries ? null : (
                <SortHeader label={t('analyze', 'details.col.library')} sortable={false} direction={direction} />
              )}
              <SortHeader label={t('analyze', 'details.col.flash')} sortable direction={direction} active={sort === 'flash'} onSort={() => onSort('flash')} />
              <SortHeader label={t('analyze', 'details.col.ram')} sortable direction={direction} active={sort === 'ram'} onSort={() => onSort('ram')} />
              <SortHeader label={t('analyze', 'details.col.flashShare')} sortable={false} direction={direction} />
              <SortHeader label={t('analyze', 'details.col.mapRows')} sortable={false} direction={direction} />
            </tr>
          </thead>
          <tbody>
            {page.rows.map((row: ObjectRowDto) => (
              <tr key={row.key}>
                <td className={styles['mono']}>{row.key}</td>
                {libraries ? null : (
                  <td className={styles['mono']}>{row.archive ?? t('analyze', 'details.objects.application')}</td>
                )}
                <td className={styles['mono']}>
                  {row.nonvolatileBytes === null ? <Unknown /> : formatSize(row.nonvolatileBytes, unit)}
                </td>
                <td className={styles['mono']}>
                  {row.runtimeRamBytes === null ? <Unknown /> : formatSize(row.runtimeRamBytes, unit)}
                </td>
                <td className={styles['mono']}>
                  {share(row.nonvolatileShareBasisPoints, row.nonvolatileBytes) ?? <Unknown reason={noShare} />}
                </td>
                <td className={styles['mono']}>{row.mapRows}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </ScrollArea>
      <p className={styles['hint']}>
        {page.unattributedNonvolatile === null || page.unattributedRuntimeRam === null
          ? t('analyze', 'details.objects.remainderUnknown')
          : t('analyze', 'details.objects.remainder', {
              flash: formatSize(page.unattributedNonvolatile, unit),
              ram: formatSize(page.unattributedRuntimeRam, unit),
            })}
      </p>
    </>
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
  const { t } = useI18n();
  return (
    <>
      {inspected === null ? null : <Inspector row={inspected} />}

      <ScrollArea label={t('analyze', 'details.evidence.table')}>
        <table className={styles['table']}>
          <caption className={styles['caption']}>
            {t('analyze', 'details.evidence.caption')}
          </caption>
          <thead>
            <tr>
              <SortHeader
                label={t('analyze', 'details.col.detail')}
                sortable={false}
                direction={direction}
              />
              <SortHeader label={t('analyze', 'details.col.field')} sortable direction={direction} active={sort === 'field'} onSort={() => onSort('field')} />
              <SortHeader
                label={t('analyze', 'details.col.classification')}
                sortable
                direction={direction}
                active={sort === 'classification'}
                onSort={() => onSort('classification')}
              />
              <SortHeader
                label={t('analyze', 'details.col.value')}
                sortable={false}
                direction={direction}
              />
              <SortHeader
                label={t('analyze', 'details.col.source')}
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
                    aria-label={t('analyze', 'details.inspectAria', { field: row.field })}
                    onClick={() => {
                      onInspect(row);
                    }}
                  >
                    {t('analyze', 'details.inspect')}
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
      </ScrollArea>
    </>
  );
}

/**
 * One legacy identifier, rendered as what the evidence is actually of.
 *
 * `ElfProgramHeader` is the Debug name the storage column holds (`db.rs` writes `format!("{:?}", …)`) and
 * `elf.program-header` is its `analysis:1` wire twin; both name a program header, while the facts they
 * accompany come from an ELF address plus section flags. Option E keeps both names forever, so the
 * inaccuracy is bounded here instead of being renamed: no stored value, DTO field, exported JSON or golden
 * changes, and a value outside this table stays on screen exactly as it is stored.
 */
const SOURCE_TYPE_CAPTIONS = new Map<string, DetailsKey>([
  ['ElfProgramHeader', 'details.source.programHeader'],
  ['elf.program-header', 'details.source.programHeader'],
]);

/**
 * The whole provenance of one fact.
 *
 * A locator a reader can go back and check is the point of the evidence model, so it is shown in
 * full rather than truncated: shortening it here would defeat the reason it was stored. It is
 * painted above the table because a table taller than the window buries the answer below it.
 */
function Inspector({ row }: { readonly row: EvidenceRowDto }) {
  const { t } = useI18n();
  const sourceCaption = SOURCE_TYPE_CAPTIONS.get(row.sourceType);
  return (
    <section className={styles['inspector']} aria-label={t('analyze', 'details.inspector.aria')}>
      <h2>{row.field}</h2>
      <dl className={styles['inspectorList']}>
        <div>
          <dt>{t('analyze', 'details.col.classification')}</dt>
          <dd>{row.classification}</dd>
        </div>
        <div>
          <dt>{t('analyze', 'details.col.source')}</dt>
          <dd className={sourceCaption === undefined ? styles['mono'] : undefined}>
            {sourceCaption === undefined ? row.sourceType : t('analyze', sourceCaption)}
          </dd>
        </div>
        <div>
          <dt>{t('analyze', 'details.col.locator')}</dt>
          <dd className={styles['mono']}>{row.sourceLocator}</dd>
        </div>
        <div>
          <dt>{t('analyze', 'details.col.value')}</dt>
          <dd className={styles['mono']}>{row.rawValue}</dd>
        </div>
        <div>
          <dt>{t('analyze', 'details.col.rule')}</dt>
          <dd className={styles['mono']}>{row.rule}</dd>
        </div>
        <div>
          <dt>{t('analyze', 'details.col.confidence')}</dt>
          <dd>{formatOptional(row.confidence)}</dd>
        </div>
        <div>
          <dt>{t('analyze', 'details.inspector.evidenceId')}</dt>
          <dd className={styles['mono']}>{row.id}</dd>
        </div>
      </dl>
    </section>
  );
}

function DetailError({ envelope }: { readonly envelope: ErrorEnvelopeDto }) {
  const { t } = useI18n();
  return (
    <section className={styles['error']} role="alert" aria-label={t('analyze', 'details.error.aria')}>
      <h2>{t('analyze', 'details.error.heading')}</h2>
      <p>{envelope.message}</p>
      <p className={styles['monoSmall']}>{envelope.code}</p>
      {envelope.details === null ? null : <p className={styles['monoSmall']}>{envelope.details}</p>}
      {envelope.remediation === null ? null : <p>{envelope.remediation}</p>}
      <p className={styles['monoSmall']}>{t('common', 'error.diagnosticsId')} {envelope.operationId}</p>
    </section>
  );
}
