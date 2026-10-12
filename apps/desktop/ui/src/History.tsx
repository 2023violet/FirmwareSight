/**
 * The P5 History page: three bounded tables over rows this application already wrote.
 *
 * History is a viewer of persisted evidence, and the distinction from Analyze is stated on the
 * screen rather than left to be inferred: Analyze describes the artifact in front of you now,
 * History describes what the store holds. Nothing here opens a firmware file, edits a row, deletes
 * one or re-runs a Gate — the commands this page calls are the only three read commands the shell
 * exposes for history, and there is no write command to mis-click (prompt §17).
 *
 * Four rules shape the component:
 *
 * 1. **The rows are the shell's.** Page, filter and the limit actually applied come back from Rust;
 *    nothing here slices an array it happens to hold, so a filter searches the stored population
 *    rather than the one page currently on screen (prompt §17, `04_TECH/14` 3).
 * 2. **A table's state belongs to that table.** The builds filter cannot leak into the Gate list, so
 *    each table keeps its own offset, draft, applied filter and request counter.
 * 3. **Only the newest response paints.** Each table ignores an answer that a later request has
 *    already superseded, which is what keeps rapid navigation from showing page one under page two.
 * 4. **A digest or an id is shown in full when it is opened.** A shortened value is a convenience;
 *    the detail row carries the whole text so it can be read and copied, never a second encoding
 *    of it (prompt §16).
 */

import { Fragment, useCallback, useEffect, useRef, useState } from 'react';

import styles from './History.module.css';
import { ErrorPanel } from './components/ErrorPanel';
import { Page, ScrollArea } from './components/Layout';
import { PageHeader } from './components/PageHeader';
import { Pager } from './components/Table';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import { StateBadge, type StateName } from './components/StateBadge';
import { formatSize, truncateMiddle, type SizeUnit } from './format';
import { useI18n, type Translate } from './i18n';
import { cx } from './styles/classnames';
import {
  listHistoryBuilds,
  listHistoryGateRuns,
  listHistoryReleases,
} from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type {
  ErrorEnvelopeDto,
  HistoryGateRunRowDto,
  HistoryPageRequestDto,
  HistoryReleaseRowDto,
} from './ipc/types';

/** One row's full text, opened by its own button and closed again. */
type Open = 'build' | 'gate' | 'release';

interface PageState<T> {
  readonly rows: readonly T[];
  readonly total: number;
  readonly offset: number;
  readonly limit: number;
  readonly nextOffset: number | null;
}

/**
 * The bounded-read state one table needs, with the request guard the detail tables already use.
 *
 * `fetch` is passed in a `useCallback` by the caller, so changing the filter or the offset re-runs
 * exactly one read and nothing else.
 */
/**
 * The three stored entities, and the one the page is showing.
 *
 * U1P §12 asks for a history workspace rather than three database tables stacked in one column. The
 * entities stay separate because they are three different facts about the store — a build, a gate run,
 * a release — and merging their rows would be the false simplification §12B forbids. What changes is
 * that the page shows one of them at a time, so a reader looking at Builds sees a list and its detail
 * rather than the third of a screen each of three tables leaves them.
 */
type View = 'builds' | 'runs' | 'releases';

/** The catalog key naming one stored population. */
type ViewKey = 'views.builds' | 'views.runs' | 'views.releases';

const VIEWS: readonly { readonly key: View; readonly labelKey: ViewKey }[] = [
  { key: 'builds', labelKey: 'views.builds' },
  { key: 'runs', labelKey: 'views.runs' },
  { key: 'releases', labelKey: 'views.releases' },
];

/**
 * The three stored populations as the machinery names them, in one fixed English word each.
 *
 * This is not reader-facing copy: it builds the filter field's DOM id, so the id a test or a bookmark
 * depends on does not change when the screen changes language. The word a reader hears in the pager line
 * comes from the catalog (`pager.noun.*`), because `components/Table.tsx` now writes that sentence in
 * whichever language the screen resolved to.
 */
const POPULATION: Readonly<Record<View, string>> = {
  builds: 'builds',
  runs: 'Gate runs',
  releases: 'release records',
};

function useBoundedPage<T>(
  fetch: (request: HistoryPageRequestDto) => Promise<IpcOutcome<PageState<T>>>,
): {
  readonly state: PageState<T> | null;
  readonly loading: boolean;
  readonly error: ErrorEnvelopeDto | null;
  readonly draft: string;
  readonly applied: string;
  readonly offset: number;
  onDraftChange: (value: string) => void;
  applyFilter: (value: string) => void;
  goToOffset: (offset: number) => void;
} {
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
    // Rows that answer the previous question leave the screen before the new question is asked, so
    // a slow reply cannot be mistaken for the current one.
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

  const goToOffset = useCallback((value: number) => {
    setOffset(value);
  }, []);

  return {
    state,
    loading,
    error,
    draft,
    applied,
    offset,
    onDraftChange: setDraft,
    applyFilter,
    goToOffset,
  };
}

function FilterForm({
  population,
  noun,
  subject,
  draft,
  onDraftChange,
  onApply,
}: {
  /** The machine word behind the field's DOM id, so a language switch does not move it. */
  readonly population: string;
  readonly noun: string;
  readonly subject: string;
  readonly draft: string;
  readonly onDraftChange: (value: string) => void;
  readonly onApply: (value: string) => void;
}) {
  const { t } = useI18n();
  const id = `fs-history-filter-${population}`;
  return (
    <form
      className={styles['filter']}
      aria-label={t('history', 'filter.formAria', { noun })}
      onSubmit={(event) => {
        event.preventDefault();
        onApply(draft);
      }}
    >
      <label className={styles['filterLabel']} htmlFor={id}>
        {t('history', 'filter.label', { noun, subject })}
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
      <button type="submit" className={styles['filterApply']}>
        {t('history', 'filter.apply')}
      </button>
      {draft.length === 0 ? (
        <span className={styles['filterHint']}>{t('history', 'filter.showsAll')}</span>
      ) : (
        <span className={styles['filterHint']}>{t('history', 'filter.filteringBy', { draft })}</span>
      )}
    </form>
  );
}

/** The five-state word a stored disposition uses, with the label a reader needs beside it. */
function dispositionBadge(
  state: string,
  t: Translate,
): { readonly name: StateName; readonly label: string } {
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

export function History({
  unit,
  onUnitChange,
}: {
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
}) {
  const { t } = useI18n();
  const [open, setOpen] = useState<{ readonly kind: Open; readonly id: string } | null>(null);
  /** Which one of the three stored entities the page is showing. See the view switch below. */
  const [view, setView] = useState<View>('builds');

  const builds = useBoundedPage(
    useCallback(
      async (request: HistoryPageRequestDto) => await listHistoryBuilds(request),
      [],
    ),
  );
  const runs = useBoundedPage(
    useCallback(
      async (request: HistoryPageRequestDto) => await listHistoryGateRuns(request),
      [],
    ),
  );
  const releases = useBoundedPage(
    useCallback(
      async (request: HistoryPageRequestDto) => await listHistoryReleases(request),
      [],
    ),
  );

  const toggle = (kind: Open, id: string) => {
    setOpen((current) => (current !== null && current.kind === kind && current.id === id ? null : { kind, id }));
  };

  return (
    <Page>
      <PageHeader title={t('history', 'title')} subhead={t('history', 'subhead')} />

      <SizeUnitSwitch unit={unit} onSelect={onUnitChange} />

      {/* One list at a time. A button group with `aria-pressed` rather than an ARIA tablist, because each
          list below is a named region a reader may want to navigate to, and a `tabpanel` role would take
          that landmark away to buy a pattern the switch does not need (§22). */}
      <div className={styles['views']} role="group" aria-label={t('history', 'views.group')}>
        {VIEWS.map((entry) => (
          <button
            key={entry.key}
            type="button"
            aria-pressed={view === entry.key}
            className={cx(styles['view'], view === entry.key ? styles['viewActive'] : undefined)}
            onClick={() => {
              setView(entry.key);
              setOpen(null);
            }}
          >
            {t('history', entry.labelKey)}
          </button>
        ))}
      </div>

      {view === 'builds' ? (
      <section className={styles['table']} aria-label={t('history', 'builds.region')}>
        <h2>{t('history', 'views.builds')}</h2>
        <FilterForm
          population={POPULATION.builds}
          noun={t('history', 'filter.noun.builds')}
          subject={t('history', 'filter.subject.builds')}
          draft={builds.draft}
          onDraftChange={builds.onDraftChange}
          onApply={builds.applyFilter}
        />
        {builds.loading ? (
          <p className={styles['status']} role="status" aria-live="polite">
            {t('history', 'builds.loading')}
          </p>
        ) : null}
        {builds.error !== null ? (
          <ErrorPanel
            envelope={builds.error}
            label={t('history', 'builds.errorLabel')}
            heading={t('history', 'builds.errorHeading')}
          />
        ) : null}
        {builds.state === null || builds.error !== null ? null : (
          <>
            {builds.state.rows.length === 0 ? (
              <p className={styles['empty']}>
                {builds.applied.length === 0
                  ? t('history', 'builds.empty')
                  : t('history', 'builds.noMatch', { term: builds.applied })}
              </p>
            ) : (
              <ScrollArea label={t('history', 'builds.table')}>
                <table className={styles['grid']}>
                  <caption className={styles['caption']}>{t('history', 'builds.caption')}</caption>
                  <thead>
                    <tr>
                      {/* The action leads its row. A table of stored ids, digests and times is wider
                          than the frozen minimum window, so an action at the end of it is the thing
                          that falls off the right edge (F2R-02); the Evidence table already puts its
                          row action first for the same reason. */}
                      <th scope="col">
                        <span className={styles['srOnly']}>{t('history', 'row.actions')}</span>
                      </th>
                      <th scope="col">{t('history', 'builds.col.artifact')}</th>
                      <th scope="col">{t('history', 'builds.col.snapshot')}</th>
                      <th scope="col">{t('history', 'builds.col.sha')}</th>
                      <th scope="col">{t('history', 'col.stored')}</th>
                      <th scope="col">{t('history', 'builds.col.format')}</th>
                      <th scope="col">{t('history', 'builds.col.nonvolatile')}</th>
                      <th scope="col">{t('history', 'builds.col.ram')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {builds.state.rows.map((row) => (
                      <Fragment key={row.buildId}>
                        <tr>
                          <td>
                            <button
                              type="button"
                              className={styles['action']}
                              aria-expanded={open !== null && open.kind === 'build' && open.id === row.buildId}
                              onClick={() => {
                                toggle('build', row.buildId);
                              }}
                            >
                              {t('history', 'row.details')}
                            </button>
                          </td>
                          <td title={row.fileName}>{row.fileName}</td>
                          <td className={styles['mono']}>{truncateMiddle(row.snapshotId, 8)}</td>
                          <td className={styles['mono']}>{truncateMiddle(row.sha256, 8)}</td>
                          <td className={styles['mono']}>{row.importedAt}</td>
                          <td>{row.architecture}</td>
                          <td>
                            <Budget state={row.nonvolatile.state} bytes={row.nonvolatile.bytes} unit={unit} />
                          </td>
                          <td>
                            <Budget state={row.runtimeRam.state} bytes={row.runtimeRam.bytes} unit={unit} />
                          </td>
                        </tr>
                        {open !== null && open.kind === 'build' && open.id === row.buildId ? (
                          <tr key={`${row.buildId}-detail`} className={styles['detail']}>
                            <td colSpan={8}>
                              <dl className={styles['facts']}>
                                <div>
                                  <dt>{t('history', 'detail.buildId')}</dt>
                                  <dd className={styles['mono']}>{row.buildId}</dd>
                                </div>
                                <div>
                                  <dt>{t('history', 'detail.snapshotId')}</dt>
                                  <dd className={styles['mono']}>{row.snapshotId}</dd>
                                </div>
                                <div>
                                  <dt>{t('history', 'detail.artifactSha')}</dt>
                                  <dd className={styles['mono']}>{row.sha256}</dd>
                                </div>
                                <div>
                                  <dt>{t('history', 'detail.size')}</dt>
                                  <dd className={styles['mono']}>{formatSize(row.byteSize, unit)}</dd>
                                </div>
                                <div>
                                  <dt>{t('history', 'detail.architecture')}</dt>
                                  <dd>{row.architecture}</dd>
                                </div>
                                <div>
                                  <dt>{t('history', 'col.stored')}</dt>
                                  <dd className={styles['mono']}>{row.importedAt}</dd>
                                </div>
                              </dl>
                              <p className={styles['detailNote']}>{t('history', 'builds.note')}</p>
                            </td>
                          </tr>
                        ) : null}
                      </Fragment>
                    ))}
                  </tbody>
                </table>
              </ScrollArea>
            )}
            <Pager
              total={builds.state.total}
              offset={builds.state.offset}
              shown={builds.state.rows.length}
              nextOffset={builds.state.nextOffset}
              label={t('history', 'pager.noun.builds')}
              onOffset={builds.goToOffset}
            />
          </>
        )}
      </section>
      ) : null}

      {view === 'runs' ? (
      <section className={styles['table']} aria-label={t('history', 'runs.region')}>
        <h2>{t('history', 'views.runs')}</h2>
        <FilterForm
          population={POPULATION.runs}
          noun={t('history', 'filter.noun.runs')}
          subject={t('history', 'filter.subject.runs')}
          draft={runs.draft}
          onDraftChange={runs.onDraftChange}
          onApply={runs.applyFilter}
        />
        {runs.loading ? (
          <p className={styles['status']} role="status" aria-live="polite">
            {t('history', 'runs.loading')}
          </p>
        ) : null}
        {runs.error !== null ? (
          <ErrorPanel
            envelope={runs.error}
            label={t('history', 'runs.errorLabel')}
            heading={t('history', 'runs.errorHeading')}
          />
        ) : null}
        {runs.state === null || runs.error !== null ? null : (
          <>
            {runs.state.rows.length === 0 ? (
              <p className={styles['empty']}>
                {runs.applied.length === 0
                  ? t('history', 'runs.empty')
                  : t('history', 'runs.noMatch', { term: runs.applied })}
              </p>
            ) : (
              <ScrollArea label={t('history', 'runs.table')}>
                <table className={styles['grid']}>
                  <caption className={styles['caption']}>{t('history', 'runs.caption')}</caption>
                  <thead>
                    <tr>
                      <th scope="col">
                        <span className={styles['srOnly']}>{t('history', 'row.actions')}</span>
                      </th>
                      <th scope="col">{t('history', 'runs.col.run')}</th>
                      <th scope="col">{t('history', 'col.build')}</th>
                      <th scope="col">{t('history', 'runs.col.baseline')}</th>
                      <th scope="col">{t('history', 'runs.col.disposition')}</th>
                      <th scope="col">{t('history', 'runs.col.findings')}</th>
                      <th scope="col">{t('history', 'col.stored')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {runs.state.rows.map((row) => (
                      <GateRows
                        key={row.runId}
                        row={row}
                        open={open !== null && open.kind === 'gate' && open.id === row.runId}
                        onToggle={() => {
                          toggle('gate', row.runId);
                        }}
                      />
                    ))}
                  </tbody>
                </table>
              </ScrollArea>
            )}
            <Pager
              total={runs.state.total}
              offset={runs.state.offset}
              shown={runs.state.rows.length}
              nextOffset={runs.state.nextOffset}
              label={t('history', 'pager.noun.runs')}
              onOffset={runs.goToOffset}
            />
          </>
        )}
      </section>
      ) : null}

      {view === 'releases' ? (
      <section className={styles['table']} aria-label={t('history', 'releases.region')}>
        <h2>{t('history', 'views.releases')}</h2>
        <FilterForm
          population={POPULATION.releases}
          noun={t('history', 'filter.noun.releases')}
          subject={t('history', 'filter.subject.releases')}
          draft={releases.draft}
          onDraftChange={releases.onDraftChange}
          onApply={releases.applyFilter}
        />
        {releases.loading ? (
          <p className={styles['status']} role="status" aria-live="polite">
            {t('history', 'releases.loading')}
          </p>
        ) : null}
        {releases.error !== null ? (
          <ErrorPanel
            envelope={releases.error}
            label={t('history', 'releases.errorLabel')}
            heading={t('history', 'releases.errorHeading')}
          />
        ) : null}
        {releases.state === null || releases.error !== null ? null : (
          <>
            {releases.state.rows.length === 0 ? (
              <p className={styles['empty']}>
                {releases.applied.length === 0
                  ? t('history', 'releases.empty')
                  : t('history', 'releases.noMatch', { term: releases.applied })}
              </p>
            ) : (
              <ScrollArea label={t('history', 'releases.table')}>
                <table className={styles['grid']}>
                  <caption className={styles['caption']}>{t('history', 'releases.caption')}</caption>
                  <thead>
                    <tr>
                      <th scope="col">
                        <span className={styles['srOnly']}>{t('history', 'row.actions')}</span>
                      </th>
                      <th scope="col">{t('history', 'releases.col.release')}</th>
                      <th scope="col">{t('history', 'releases.col.version')}</th>
                      <th scope="col">{t('history', 'col.build')}</th>
                      <th scope="col">{t('history', 'releases.col.gateRun')}</th>
                      <th scope="col">{t('history', 'col.manifestSha')}</th>
                      <th scope="col">{t('history', 'col.stored')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {releases.state.rows.map((row) => (
                      <ReleaseRows
                        key={row.releaseId}
                        row={row}
                        open={open !== null && open.kind === 'release' && open.id === row.releaseId}
                        onToggle={() => {
                          toggle('release', row.releaseId);
                        }}
                      />
                    ))}
                  </tbody>
                </table>
              </ScrollArea>
            )}
            <Pager
              total={releases.state.total}
              offset={releases.state.offset}
              shown={releases.state.rows.length}
              nextOffset={releases.state.nextOffset}
              label={t('history', 'pager.noun.releases')}
              onOffset={releases.goToOffset}
            />
          </>
        )}
      </section>
      ) : null}
    </Page>
  );
}

/** A recorded budget: its state word, its value when one was recorded, and no invented zero. */
function Budget({
  state,
  bytes,
  unit,
}: {
  readonly state: string;
  readonly bytes: number | null;
  readonly unit: SizeUnit;
}) {
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

function GateRows({
  row,
  open,
  onToggle,
}: {
  readonly row: HistoryGateRunRowDto;
  readonly open: boolean;
  readonly onToggle: () => void;
}) {
  const { t } = useI18n();
  const badge = dispositionBadge(row.disposition, t);
  const counts = row.counts;
  return (
    <>
      <tr>
        <td>
          <button
            type="button"
            className={styles['action']}
            aria-expanded={open}
            onClick={onToggle}
          >
            {t('history', 'row.details')}
          </button>
        </td>
        <td className={styles['mono']}>{truncateMiddle(row.runId, 8)}</td>
        <td title={row.fileName}>{row.fileName}</td>
        <td>{row.baselineFileName === null ? t('history', 'runs.baseline.none') : row.baselineFileName}</td>
        <td>
          <StateBadge state={badge.name} label={badge.label} />
        </td>
        <td>
          <span className={styles['counts']}>
            <StateBadge state="PASS" label={t('history', 'count.pass')} count={counts.pass} />
            <StateBadge state="REVIEW" label={t('history', 'count.review')} count={counts.review} />
            <StateBadge state="BLOCK" label={t('history', 'count.block')} count={counts.block} />
            <StateBadge state="UNKNOWN" label={t('history', 'count.unknown')} count={counts.unknown} />
            <StateBadge state="N/A" label={t('history', 'count.na')} count={counts.notApplicable} />
          </span>
        </td>
        <td className={styles['mono']}>{row.storedAt}</td>
      </tr>
      {open ? (
        <tr className={styles['detail']}>
          <td colSpan={7}>
            <dl className={styles['facts']}>
              <div>
                <dt>{t('history', 'detail.gateRunId')}</dt>
                <dd className={styles['mono']}>{row.runId}</dd>
              </div>
              <div>
                <dt>{t('history', 'detail.buildId')}</dt>
                <dd className={styles['mono']}>{row.buildId}</dd>
              </div>
              {row.baselineBuildId === null ? null : (
                <div>
                  <dt>{t('history', 'detail.baselineBuildId')}</dt>
                  <dd className={styles['mono']}>{row.baselineBuildId}</dd>
                </div>
              )}
              <div>
                <dt>{t('history', 'col.stored')}</dt>
                <dd className={styles['mono']}>{row.storedAt}</dd>
              </div>
            </dl>
            <p className={styles['detailNote']}>{t('history', 'runs.note')}</p>
          </td>
        </tr>
      ) : null}
    </>
  );
}

function ReleaseRows({
  row,
  open,
  onToggle,
}: {
  readonly row: HistoryReleaseRowDto;
  readonly open: boolean;
  readonly onToggle: () => void;
}) {
  const { t } = useI18n();
  return (
    <>
      <tr>
        <td>
          <button
            type="button"
            className={styles['action']}
            aria-expanded={open}
            onClick={onToggle}
          >
            {t('history', 'row.details')}
          </button>
        </td>
        <td className={styles['mono']}>{truncateMiddle(row.releaseId, 8)}</td>
        <td className={styles['mono']}>{row.releaseVersion}</td>
        <td title={row.fileName}>{row.fileName}</td>
        <td className={styles['mono']}>{truncateMiddle(row.gateRunId, 8)}</td>
        <td className={styles['mono']}>{truncateMiddle(row.manifestSha256, 8)}</td>
        <td className={styles['mono']}>{row.storedAt}</td>
      </tr>
      {open ? (
        <tr className={styles['detail']}>
          <td colSpan={7}>
            <dl className={styles['facts']}>
              <div>
                <dt>{t('history', 'detail.releaseId')}</dt>
                <dd className={styles['mono']}>{row.releaseId}</dd>
              </div>
              <div>
                <dt>{t('history', 'detail.gateRunId')}</dt>
                <dd className={styles['mono']}>{row.gateRunId}</dd>
              </div>
              <div>
                <dt>{t('history', 'detail.buildId')}</dt>
                <dd className={styles['mono']}>{row.buildId}</dd>
              </div>
              <div>
                <dt>{t('history', 'col.manifestSha')}</dt>
                <dd className={styles['mono']}>{row.manifestSha256}</dd>
              </div>
              <div>
                <dt>{t('history', 'col.stored')}</dt>
                <dd className={styles['mono']}>{row.storedAt}</dd>
              </div>
            </dl>
            <p className={styles['detailNote']}>{t('history', 'releases.note')}</p>
          </td>
        </tr>
      ) : null}
    </>
  );
}
