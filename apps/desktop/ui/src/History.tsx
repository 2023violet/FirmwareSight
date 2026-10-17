/**
 * The History page: three bounded lists over rows this application already wrote, and the one build the
 * project measures against.
 *
 * History is a viewer of persisted evidence. Analyze describes the artifact in front of you now; History
 * describes what the store holds. Nothing here opens a firmware file, edits a row, deletes one or re-runs a
 * Gate. The one write it offers is the pinned baseline (R5 / D3), which appends to its own log and never
 * touches a stored row.
 *
 * Five rules shape the component:
 *
 * 1. **The rows are the shell's.** Page, filter and the limit actually applied come back from Rust;
 *    nothing here slices an array it happens to hold, so a filter searches the stored population rather
 *    than the page on screen (prompt §17, `04_TECH/14` 3). The shell orders each list newest first, and
 *    the tables offer no sort of their own (D-R5-5).
 * 2. **A list's state belongs to that list.** The builds filter cannot leak into the Gate list, so each
 *    keeps its own offset, draft, applied filter and request counter.
 * 3. **Only the newest response paints.** Each list ignores an answer that a later request superseded.
 * 4. **A digest or an id is whole in the Inspector.** A shortened value is a convenience in the table; the
 *    row's Inspector carries the whole text so it can be read and copied (prompt §16).
 * 5. **A pin is a measurement reference, not a favourite** (`03_DESIGN/08` §8). There is one per project,
 *    pinning another build replaces it, and Compare starts from it.
 */

import { useCallback, useEffect, useRef, useState, type ReactNode } from 'react';

import styles from './History.module.css';
import { Button } from './components/Button';
import { DataTable, FilterBar, type Column } from './components/DataTable';
import { ErrorPanel } from './components/ErrorPanel';
import type { FactEvidence } from './components/FactValue';
import { Icon } from './components/Icon';
import { Inspector } from './components/Inspector';
import { Page } from './components/Layout';
import { PageHeader } from './components/PageHeader';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import { StateBadge, type StateName } from './components/StateBadge';
import { Pager } from './components/Table';
import { Tabs, tabPanelProps } from './components/Tabs';
import { formatSize, truncateMiddle, type SizeUnit } from './format';
import { useI18n, type Translate } from './i18n';
import {
  listHistoryBuilds,
  listHistoryGateRuns,
  listHistoryReleases,
  pinBaseline,
  unpinBaseline,
} from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type {
  BaselinePinDto,
  CompareCandidateDto,
  ErrorEnvelopeDto,
  HistoryGateRunRowDto,
  HistoryPageRequestDto,
  HistoryReleaseRowDto,
} from './ipc/types';

interface PageState<T> {
  readonly rows: readonly T[];
  readonly total: number;
  readonly offset: number;
  readonly limit: number;
  readonly nextOffset: number | null;
}

/**
 * The three stored entities, one at a time (U1P §12). They stay separate because they are three different
 * facts about the store — a build, a Gate run, a release — and merging their rows would be the false
 * simplification §12B forbids.
 */
type View = 'builds' | 'runs' | 'releases';
const VIEWS: readonly View[] = ['builds', 'runs', 'releases'];

type Inspect = (evidence: FactEvidence, trigger: HTMLElement) => void;

interface BoundedList<T> {
  readonly state: PageState<T> | null;
  readonly loading: boolean;
  readonly error: ErrorEnvelopeDto | null;
  readonly draft: string;
  readonly applied: string;
  readonly onDraftChange: (value: string) => void;
  readonly applyFilter: (value: string) => void;
  readonly goToOffset: (offset: number) => void;
}

function useBoundedPage<T>(
  fetch: (request: HistoryPageRequestDto) => Promise<IpcOutcome<PageState<T>>>,
): BoundedList<T> {
  const [state, setState] = useState<PageState<T> | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [draft, setDraft] = useState('');
  const [applied, setApplied] = useState('');
  const [offset, setOffset] = useState(0);
  const request = useRef(0);

  useEffect(() => {
    const id = ++request.current;
    setLoading(true);
    // Rows that answer the previous question leave the screen before the new question is asked, so a slow
    // reply cannot be mistaken for the current one.
    setState(null);
    void fetch({
      offset,
      limit: null,
      filter: applied.length === 0 ? null : applied,
    }).then((outcome) => {
      if (request.current !== id) {
        return;
      }
      setLoading(false);
      if (outcome.ok) {
        setState(outcome.value);
        setError(null);
      } else {
        setError(outcome.envelope);
      }
    });
  }, [fetch, offset, applied]);

  const applyFilter = useCallback((value: string) => {
    setApplied(value);
    setOffset(0);
  }, []);

  return { state, loading, error, draft, applied, onDraftChange: setDraft, applyFilter, goToOffset: setOffset };
}

/** The five-state word a stored disposition uses, with the label a reader needs beside it. */
function dispositionBadge(state: string, t: Translate): { readonly name: StateName; readonly label: string } {
  switch (state) {
    case 'PASS':
      return { name: 'PASS', label: t('history', 'disposition.pass') };
    case 'REVIEW':
      return { name: 'REVIEW', label: t('history', 'disposition.review') };
    case 'BLOCK':
      return { name: 'BLOCK', label: t('history', 'disposition.block') };
    default:
      // A stored word the reader cannot map is shown as unknown rather than promoted to a verdict.
      return { name: 'UNKNOWN', label: t('history', 'disposition.unknown') };
  }
}

/** A build as a reader names it: file and short digest. */
function buildName(row: { readonly fileName: string; readonly sha256: string }): string {
  return `${row.fileName} · ${row.sha256.slice(0, 12)}`;
}

export function History({
  unit,
  onUnitChange,
  pin,
  pinReadError = null,
  onRetryPinRead,
  onPinChange,
  currentSnapshotId,
  onCompare,
}: {
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
  /** The project's pinned baseline, as the shell last reported it. */
  readonly pin: BaselinePinDto | null;
  /** The pin could not be read: it is Unknown, not "none" (R5 review S2-1). */
  readonly pinReadError?: ErrorEnvelopeDto | null;
  readonly onRetryPinRead?: () => void;
  readonly onPinChange: (pin: BaselinePinDto | null) => void;
  /** The build Analyze last proved, which "the current build" means here. */
  readonly currentSnapshotId: string | null;
  /** Open Compare on this pair and run it once (D-R5-4). */
  readonly onCompare: (baseId: string, targetId: string) => void;
}) {
  const { t } = useI18n();
  const [view, setView] = useState<View>('builds');
  const [selected, setSelected] = useState<readonly CompareCandidateDto[]>([]);
  const [pinError, setPinError] = useState<ErrorEnvelopeDto | null>(null);
  const [pinBusy, setPinBusy] = useState(false);
  const [inspected, setInspected] = useState<{ readonly evidence: FactEvidence; readonly trigger: HTMLElement } | null>(
    null,
  );
  const inspect = useCallback<Inspect>((evidence, trigger) => {
    setInspected({ evidence, trigger });
  }, []);
  const closeInspector = useCallback(() => {
    setInspected(null);
  }, []);

  const builds = useBoundedPage(
    useCallback(async (request: HistoryPageRequestDto) => await listHistoryBuilds(request), []),
  );
  const runs = useBoundedPage(
    useCallback(async (request: HistoryPageRequestDto) => await listHistoryGateRuns(request), []),
  );
  const releases = useBoundedPage(
    useCallback(async (request: HistoryPageRequestDto) => await listHistoryReleases(request), []),
  );

  // A row's detail belongs to the list it came from, in the unit it was written in.
  useEffect(() => {
    setInspected(null);
  }, [view, unit]);

  const pinBuild = async (snapshotId: string) => {
    setPinBusy(true);
    const outcome = await pinBaseline(snapshotId);
    setPinBusy(false);
    if (!outcome.ok) {
      setPinError(outcome.envelope);
      return;
    }
    setPinError(null);
    onPinChange(outcome.value);
  };

  const unpin = async () => {
    setPinBusy(true);
    const outcome = await unpinBaseline();
    setPinBusy(false);
    if (!outcome.ok) {
      setPinError(outcome.envelope);
      return;
    }
    setPinError(null);
    onPinChange(null);
  };

  const toggleSelected = (row: CompareCandidateDto) => {
    setSelected((now) =>
      now.some((entry) => entry.snapshotId === row.snapshotId)
        ? now.filter((entry) => entry.snapshotId !== row.snapshotId)
        : // Two builds make a comparison; a third tick replaces the oldest tick rather than growing a set
          // the one action cannot use.
          [...now, row].slice(-2),
    );
  };

  const compareSelected = () => {
    const [first, second] = selected;
    if (first === undefined || second === undefined) {
      return;
    }
    // The build stored earlier is the base, so the delta reads forward in time.
    const [base, target] = first.importedAt <= second.importedAt ? [first, second] : [second, first];
    onCompare(base.snapshotId, target.snapshotId);
  };

  const buildColumns: readonly Column<CompareCandidateDto>[] = [
    {
      key: 'select',
      header: t('history', 'builds.col.select'),
      cell: (row) => (
        <input
          type="checkbox"
          className={styles['check']}
          aria-label={t('history', 'builds.selectRow', { name: buildName(row) })}
          checked={selected.some((entry) => entry.snapshotId === row.snapshotId)}
          onChange={() => {
            toggleSelected(row);
          }}
        />
      ),
    },
    {
      key: 'artifact',
      header: t('history', 'builds.col.artifact'),
      cell: (row) => (
        <span className={styles['artifact']}>
          <span className={styles['name']}>{row.fileName}</span>
          {pin?.snapshotId === row.snapshotId ? (
            <span className={styles['pinned']}>
              <Icon name="pin" size="inline" />
              {t('common', 'subject.pinned')}
            </span>
          ) : null}
        </span>
      ),
    },
    { key: 'sha', header: t('history', 'builds.col.sha'), cell: (row) => <span className={styles['mono']}>{truncateMiddle(row.sha256, 8)}</span> },
    { key: 'stored', header: t('history', 'col.stored'), cell: (row) => <span className={styles['mono']}>{row.importedAt}</span> },
    { key: 'format', header: t('history', 'builds.col.format'), cell: (row) => row.architecture },
    {
      key: 'nonvolatile',
      header: t('history', 'builds.col.nonvolatile'),
      cell: (row) => <Budget state={row.nonvolatile.state} bytes={row.nonvolatile.bytes} unit={unit} />,
    },
    {
      key: 'ram',
      header: t('history', 'builds.col.ram'),
      cell: (row) => <Budget state={row.runtimeRam.state} bytes={row.runtimeRam.bytes} unit={unit} />,
    },
    {
      key: 'act',
      header: t('history', 'builds.col.actions'),
      cell: (row) => {
        const name = buildName(row);
        const isPinned = pin?.snapshotId === row.snapshotId;
        const canCompare = currentSnapshotId !== null && currentSnapshotId !== row.snapshotId;
        return (
          <span className={styles['rowActions']}>
            {isPinned ? null : (
              <Button
                variant="ghost"
                ariaLabel={t('history', 'pin.pinRow', { name })}
                disabled={pinBusy}
                onClick={() => void pinBuild(row.snapshotId)}
              >
                {t('history', 'pin.pinShort')}
              </Button>
            )}
            <Button
              variant="ghost"
              ariaLabel={t('history', 'builds.compareRow', { name })}
              disabled={!canCompare}
              onClick={() => {
                if (currentSnapshotId !== null) {
                  onCompare(row.snapshotId, currentSnapshotId);
                }
              }}
            >
              {t('history', 'builds.compareShort')}
            </Button>
          </span>
        );
      },
    },
  ];

  const runColumns: readonly Column<HistoryGateRunRowDto>[] = [
    { key: 'run', header: t('history', 'runs.col.run'), cell: (row) => <span className={styles['mono']}>{truncateMiddle(row.runId, 8)}</span> },
    { key: 'build', header: t('history', 'col.build'), cell: (row) => row.fileName },
    {
      key: 'baseline',
      header: t('history', 'runs.col.baseline'),
      cell: (row) => row.baselineFileName ?? t('history', 'runs.baseline.none'),
    },
    {
      key: 'disposition',
      header: t('history', 'runs.col.disposition'),
      cell: (row) => {
        const badge = dispositionBadge(row.disposition, t);
        return <StateBadge state={badge.name} label={badge.label} />;
      },
    },
    {
      key: 'findings',
      header: t('history', 'runs.col.findings'),
      cell: (row) => (
        <span className={styles['counts']}>
          <StateBadge state="PASS" label={t('history', 'count.pass')} count={row.counts.pass} />
          <StateBadge state="REVIEW" label={t('history', 'count.review')} count={row.counts.review} />
          <StateBadge state="BLOCK" label={t('history', 'count.block')} count={row.counts.block} />
          <StateBadge state="UNKNOWN" label={t('history', 'count.unknown')} count={row.counts.unknown} />
          <StateBadge state="N/A" label={t('history', 'count.na')} count={row.counts.notApplicable} />
        </span>
      ),
    },
    { key: 'stored', header: t('history', 'col.stored'), cell: (row) => <span className={styles['mono']}>{row.storedAt}</span> },
  ];

  const releaseColumns: readonly Column<HistoryReleaseRowDto>[] = [
    { key: 'release', header: t('history', 'releases.col.release'), cell: (row) => <span className={styles['mono']}>{truncateMiddle(row.releaseId, 8)}</span> },
    { key: 'version', header: t('history', 'releases.col.version'), cell: (row) => <span className={styles['mono']}>{row.releaseVersion}</span> },
    { key: 'build', header: t('history', 'col.build'), cell: (row) => row.fileName },
    { key: 'run', header: t('history', 'releases.col.gateRun'), cell: (row) => <span className={styles['mono']}>{truncateMiddle(row.gateRunId, 8)}</span> },
    { key: 'manifest', header: t('history', 'col.manifestSha'), cell: (row) => <span className={styles['mono']}>{truncateMiddle(row.manifestSha256, 8)}</span> },
    { key: 'stored', header: t('history', 'col.stored'), cell: (row) => <span className={styles['mono']}>{row.storedAt}</span> },
  ];

  return (
    <div className={styles['withInspector']}>
      <Page>
        <PageHeader
          title={t('history', 'title')}
          subhead={t('history', 'subhead')}
          actions={<SizeUnitSwitch unit={unit} onSelect={onUnitChange} />}
        />

        <section className={styles['pin']} aria-label={t('history', 'pin.region')}>
          <Icon name="pin" />
          {pinReadError !== null ? (
            <>
              <p className={styles['pinText']}>
                <StateBadge state="UNKNOWN" label={t('history', 'pin.unknown')} /> {t('history', 'pin.unknownFact')}
              </p>
              <span className={styles['pinActions']}>
                <Button onClick={() => onRetryPinRead?.()}>{t('history', 'pin.retry')}</Button>
              </span>
            </>
          ) : pin === null ? (
            <p className={styles['pinText']}>{t('history', 'pin.none')}</p>
          ) : (
            <>
              <p className={styles['pinText']}>
                <span className={styles['axis']}>{t('history', 'pin.label')}</span>{' '}
                <span className={styles['mono']}>{buildName(pin)}</span>{' '}
                <span className={styles['note']}>
                  {t('history', 'pin.sinceLabel')} <span className={styles['mono']}>{pin.pinnedAt}</span>
                </span>
              </p>
              <span className={styles['pinActions']}>
                <Button
                  disabled={currentSnapshotId === null || currentSnapshotId === pin.snapshotId}
                  onClick={() => {
                    if (currentSnapshotId !== null) {
                      onCompare(pin.snapshotId, currentSnapshotId);
                    }
                  }}
                >
                  {t('history', 'pin.compare')}
                </Button>
                <Button disabled={pinBusy} onClick={() => void unpin()}>
                  {t('history', 'pin.unpin')}
                </Button>
              </span>
            </>
          )}
          <p className={styles['pinNote']}>
            {currentSnapshotId === null ? t('history', 'pin.noCurrent') : t('history', 'pin.note')}
          </p>
        </section>
        {pinError === null ? null : (
          <ErrorPanel envelope={pinError} label={t('history', 'pin.errorLabel')} heading={t('history', 'pin.errorHeading')} />
        )}
        {pinReadError === null ? null : (
          <ErrorPanel envelope={pinReadError} label={t('history', 'pin.readErrorLabel')} heading={t('history', 'pin.errorHeading')} />
        )}

        <Tabs
          idBase="fs-history"
          label={t('history', 'views.group')}
          items={VIEWS.map((key) => ({ key, label: t('history', `views.${key}`) }))}
          current={view}
          onSelect={setView}
        />
        <div {...tabPanelProps('fs-history', view)} className={styles['panel']}>
          {view === 'builds' ? (
            <section className={styles['list']} aria-label={t('history', 'builds.region')}>
              <h2>{t('history', 'views.builds')}</h2>
              <FilterBar
                label={t('history', 'filter.label', { noun: t('history', 'filter.noun.builds'), subject: t('history', 'filter.subject.builds') })}
                formLabel={t('history', 'filter.formAria', { noun: t('history', 'filter.noun.builds') })}
                value={builds.draft}
                onChange={builds.onDraftChange}
                onApply={builds.applyFilter}
                applyLabel={t('history', 'filter.apply')}
              >
                <Button variant="primary" disabled={selected.length !== 2} onClick={compareSelected}>
                  {t('history', 'builds.compareSelected', { count: selected.length })}
                </Button>
              </FilterBar>
              <ListBody
                list={builds}
                loading={t('history', 'builds.loading')}
                errorLabel={t('history', 'builds.errorLabel')}
                errorHeading={t('history', 'builds.errorHeading')}
                empty={t('history', 'builds.empty')}
                noMatch={(term) => t('history', 'builds.noMatch', { term })}
                pagerNoun={t('history', 'pager.noun.builds')}
                render={(rows) => (
                  <DataTable
                    caption={t('history', 'builds.caption')}
                    columns={buildColumns}
                    rows={rows}
                    rowKey={(row) => row.buildId}
                    rowHeader="artifact"
                    actionFirst
                    density="compact"
                    onRowAction={(row, trigger) => {
                      inspect(buildEvidence(row, unit, t), trigger);
                    }}
                    rowActionLabel={(row) => t('history', 'row.inspect', { name: buildName(row) })}
                  />
                )}
              />
            </section>
          ) : view === 'runs' ? (
            <section className={styles['list']} aria-label={t('history', 'runs.region')}>
              <h2>{t('history', 'views.runs')}</h2>
              <FilterBar
                label={t('history', 'filter.label', { noun: t('history', 'filter.noun.runs'), subject: t('history', 'filter.subject.runs') })}
                formLabel={t('history', 'filter.formAria', { noun: t('history', 'filter.noun.runs') })}
                value={runs.draft}
                onChange={runs.onDraftChange}
                onApply={runs.applyFilter}
                applyLabel={t('history', 'filter.apply')}
              />
              <ListBody
                list={runs}
                loading={t('history', 'runs.loading')}
                errorLabel={t('history', 'runs.errorLabel')}
                errorHeading={t('history', 'runs.errorHeading')}
                empty={t('history', 'runs.empty')}
                noMatch={(term) => t('history', 'runs.noMatch', { term })}
                pagerNoun={t('history', 'pager.noun.runs')}
                render={(rows) => (
                  <DataTable
                    caption={t('history', 'runs.caption')}
                    columns={runColumns}
                    rows={rows}
                    rowKey={(row) => row.runId}
                    actionFirst
                    density="compact"
                    onRowAction={(row, trigger) => {
                      inspect(runEvidence(row, t), trigger);
                    }}
                    rowActionLabel={(row) => t('history', 'row.inspect', { name: truncateMiddle(row.runId, 8) })}
                  />
                )}
              />
            </section>
          ) : (
            <section className={styles['list']} aria-label={t('history', 'releases.region')}>
              <h2>{t('history', 'views.releases')}</h2>
              <FilterBar
                label={t('history', 'filter.label', { noun: t('history', 'filter.noun.releases'), subject: t('history', 'filter.subject.releases') })}
                formLabel={t('history', 'filter.formAria', { noun: t('history', 'filter.noun.releases') })}
                value={releases.draft}
                onChange={releases.onDraftChange}
                onApply={releases.applyFilter}
                applyLabel={t('history', 'filter.apply')}
              />
              <ListBody
                list={releases}
                loading={t('history', 'releases.loading')}
                errorLabel={t('history', 'releases.errorLabel')}
                errorHeading={t('history', 'releases.errorHeading')}
                empty={t('history', 'releases.empty')}
                noMatch={(term) => t('history', 'releases.noMatch', { term })}
                pagerNoun={t('history', 'pager.noun.releases')}
                render={(rows) => (
                  <DataTable
                    caption={t('history', 'releases.caption')}
                    columns={releaseColumns}
                    rows={rows}
                    rowKey={(row) => row.releaseId}
                    actionFirst
                    density="compact"
                    onRowAction={(row, trigger) => {
                      inspect(releaseEvidence(row, t), trigger);
                    }}
                    rowActionLabel={(row) => t('history', 'row.inspect', { name: truncateMiddle(row.releaseId, 8) })}
                  />
                )}
              />
            </section>
          )}
        </div>
      </Page>
      <Inspector
        evidence={inspected?.evidence ?? null}
        returnFocus={inspected?.trigger ?? null}
        onClose={closeInspector}
      />
    </div>
  );
}

/** One list's loading, error, empty and pager states around the table it renders. */
function ListBody<T>({
  list,
  loading,
  errorLabel,
  errorHeading,
  empty,
  noMatch,
  pagerNoun,
  render,
}: {
  readonly list: BoundedList<T>;
  readonly loading: string;
  readonly errorLabel: string;
  readonly errorHeading: string;
  readonly empty: string;
  readonly noMatch: (term: string) => string;
  readonly pagerNoun: string;
  readonly render: (rows: readonly T[]) => ReactNode;
}) {
  return (
    <>
      {list.loading ? (
        <p className={styles['status']} role="status" aria-live="polite">
          {loading}
        </p>
      ) : null}
      {list.error === null ? null : <ErrorPanel envelope={list.error} label={errorLabel} heading={errorHeading} />}
      {list.state === null || list.error !== null ? null : (
        <>
          {list.state.rows.length === 0 ? (
            <p className={styles['empty']}>{list.applied.length === 0 ? empty : noMatch(list.applied)}</p>
          ) : (
            render(list.state.rows)
          )}
          <Pager
            total={list.state.total}
            offset={list.state.offset}
            shown={list.state.rows.length}
            nextOffset={list.state.nextOffset}
            label={pagerNoun}
            onOffset={list.goToOffset}
          />
        </>
      )}
    </>
  );
}

/** A recorded budget: its state word, its value when one was recorded, and no invented zero. */
function Budget({ state, bytes, unit }: { readonly state: string; readonly bytes: number | null; readonly unit: SizeUnit }) {
  const { t } = useI18n();
  const name: StateName = state === 'exact' ? 'PASS' : state === 'partial' ? 'REVIEW' : 'UNKNOWN';
  const label =
    state === 'exact'
      ? t('history', 'budget.exact')
      : state === 'partial'
        ? t('history', 'budget.partial')
        : t('history', 'budget.unknown');
  return (
    <span className={styles['budget']}>
      <StateBadge state={name} label={label} />
      <span className={styles['mono']}>{formatSize(bytes, unit)}</span>
    </span>
  );
}

function buildEvidence(row: CompareCandidateDto, unit: SizeUnit, t: Translate): FactEvidence {
  return {
    field: row.fileName,
    value: row.sha256,
    classification: 'observed',
    source: t('history', 'inspector.source'),
    details: [
      [t('history', 'detail.buildId'), row.buildId],
      [t('history', 'detail.snapshotId'), row.snapshotId],
      [t('history', 'detail.size'), formatSize(row.byteSize, unit)],
      [t('history', 'detail.architecture'), row.architecture],
      [t('history', 'col.stored'), row.importedAt],
      [t('history', 'detail.note'), t('history', 'builds.note')],
    ],
  };
}

function runEvidence(row: HistoryGateRunRowDto, t: Translate): FactEvidence {
  const badge = dispositionBadge(row.disposition, t);
  return {
    field: t('history', 'detail.gateRunId'),
    value: row.runId,
    // A stored verdict is the policy's judgement of observed facts; a word this build cannot map is Unknown.
    classification: badge.name === 'UNKNOWN' ? 'unknown' : 'derived',
    source: t('history', 'inspector.source'),
    details: [
      [t('history', 'runs.col.disposition'), row.disposition],
      [t('history', 'detail.buildId'), row.buildId],
      [t('history', 'detail.baselineBuildId'), row.baselineBuildId ?? t('history', 'runs.baseline.none')],
      [
        t('history', 'runs.col.findings'),
        `PASS ${row.counts.pass} · REVIEW ${row.counts.review} · BLOCK ${row.counts.block} · UNKNOWN ${row.counts.unknown} · N/A ${row.counts.notApplicable}`,
      ],
      [t('history', 'col.stored'), row.storedAt],
    ],
  };
}

function releaseEvidence(row: HistoryReleaseRowDto, t: Translate): FactEvidence {
  return {
    field: t('history', 'detail.releaseId'),
    value: row.releaseId,
    classification: 'observed',
    source: t('history', 'inspector.source'),
    details: [
      [t('history', 'releases.col.version'), row.releaseVersion],
      [t('history', 'detail.gateRunId'), row.gateRunId],
      [t('history', 'detail.buildId'), row.buildId],
      [t('history', 'col.manifestSha'), row.manifestSha256],
      [t('history', 'col.stored'), row.storedAt],
    ],
  };
}
