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
  noun,
  subject,
  draft,
  onDraftChange,
  onApply,
}: {
  readonly noun: string;
  readonly subject: string;
  readonly draft: string;
  readonly onDraftChange: (value: string) => void;
  readonly onApply: (value: string) => void;
}) {
  const id = `fs-history-filter-${noun}`;
  return (
    <form
      className={styles['filter']}
      aria-label={`Filter ${noun}`}
      onSubmit={(event) => {
        event.preventDefault();
        onApply(draft);
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
      <button type="submit" className={styles['filterApply']}>
        Apply
      </button>
      {draft.length === 0 ? (
        <span className={styles['filterHint']}>Shows every stored row</span>
      ) : (
        <span className={styles['filterHint']}>Filtering by “{draft}”</span>
      )}
    </form>
  );
}

/** The five-state word a stored disposition uses, with the label a reader needs beside it. */
function dispositionBadge(state: string): { readonly name: StateName; readonly label: string } {
  switch (state) {
    case 'PASS':
      return { name: 'PASS', label: 'Passed' };
    case 'REVIEW':
      return { name: 'REVIEW', label: 'Needs review' };
    case 'BLOCK':
      return { name: 'BLOCK', label: 'Blocked' };
    default:
      // A stored word the reader cannot map is shown as unknown rather than promoted to a verdict.
      return { name: 'UNKNOWN', label: 'Unknown disposition' };
  }
}

export function History({
  unit,
  onUnitChange,
}: {
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
}) {
  const [open, setOpen] = useState<{ readonly kind: Open; readonly id: string } | null>(null);

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
      <PageHeader
        title="History"
        subhead="What this computer has already stored: the builds that were analyzed, the Gate runs that were recorded, and the releases that were published. These are persisted facts, not the artifact currently open on Analyze, and a row stays readable after the firmware file it came from has moved or been deleted. Nothing on this page changes a stored row."
      />

      <SizeUnitSwitch unit={unit} onSelect={onUnitChange} />

      <section className={styles['table']} aria-label="Stored builds">
        <h2>Builds</h2>
        <FilterForm
          noun="builds"
          subject="build, snapshot, digest or architecture"
          draft={builds.draft}
          onDraftChange={builds.onDraftChange}
          onApply={builds.applyFilter}
        />
        {builds.loading ? (
          <p className={styles['status']} role="status" aria-live="polite">
            Loading builds…
          </p>
        ) : null}
        {builds.error !== null ? (
          <ErrorPanel envelope={builds.error} label="Build history error" heading="Builds could not be read" />
        ) : null}
        {builds.state === null || builds.error !== null ? null : (
          <>
            {builds.state.rows.length === 0 ? (
              <p className={styles['empty']}>
                {builds.applied.length === 0
                  ? 'No build has been stored yet. Analyze an artifact and it appears here.'
                  : `No stored build matches “${builds.applied}”.`}
              </p>
            ) : (
              <ScrollArea label="Stored builds table">
                <table className={styles['grid']}>
                  <caption className={styles['caption']}>
                    Ordered by when FirmwareSight stored the build, newest first.
                  </caption>
                  <thead>
                    <tr>
                      {/* The action leads its row. A table of stored ids, digests and times is wider
                          than the frozen minimum window, so an action at the end of it is the thing
                          that falls off the right edge (F2R-02); the Evidence table already puts its
                          row action first for the same reason. */}
                      <th scope="col">
                        <span className={styles['srOnly']}>Row actions</span>
                      </th>
                      <th scope="col">Artifact</th>
                      <th scope="col">Snapshot</th>
                      <th scope="col">SHA-256</th>
                      <th scope="col">Stored</th>
                      <th scope="col">Format</th>
                      <th scope="col">Non-volatile</th>
                      <th scope="col">RAM</th>
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
                              Details
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
                                  <dt>Build id</dt>
                                  <dd className={styles['mono']}>{row.buildId}</dd>
                                </div>
                                <div>
                                  <dt>Snapshot id</dt>
                                  <dd className={styles['mono']}>{row.snapshotId}</dd>
                                </div>
                                <div>
                                  <dt>Artifact SHA-256</dt>
                                  <dd className={styles['mono']}>{row.sha256}</dd>
                                </div>
                                <div>
                                  <dt>Size</dt>
                                  <dd className={styles['mono']}>{formatSize(row.byteSize, unit)}</dd>
                                </div>
                                <div>
                                  <dt>Architecture</dt>
                                  <dd>{row.architecture}</dd>
                                </div>
                                <div>
                                  <dt>Stored</dt>
                                  <dd className={styles['mono']}>{row.importedAt}</dd>
                                </div>
                              </dl>
                              <p className={styles['detailNote']}>
                                The file name is all FirmwareSight keeps visible here. Where the artifact
                                lives is never shown, and this row does not need the file to still exist.
                              </p>
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
              label="builds"
              onOffset={builds.goToOffset}
            />
          </>
        )}
      </section>

      <section className={styles['table']} aria-label="Stored Gate runs">
        <h2>Gate runs</h2>
        <FilterForm
          noun="Gate runs"
          subject="run, build, policy digest or disposition"
          draft={runs.draft}
          onDraftChange={runs.onDraftChange}
          onApply={runs.applyFilter}
        />
        {runs.loading ? (
          <p className={styles['status']} role="status" aria-live="polite">
            Loading Gate runs…
          </p>
        ) : null}
        {runs.error !== null ? (
          <ErrorPanel envelope={runs.error} label="Gate history error" heading="Gate runs could not be read" />
        ) : null}
        {runs.state === null || runs.error !== null ? null : (
          <>
            {runs.state.rows.length === 0 ? (
              <p className={styles['empty']}>
                {runs.applied.length === 0
                  ? 'No Gate run has been recorded yet. Release judges a stored build and writes the run here.'
                  : `No stored Gate run matches “${runs.applied}”.`}
              </p>
            ) : (
              <ScrollArea label="Stored Gate runs table">
                <table className={styles['grid']}>
                  <caption className={styles['caption']}>
                    Ordered by when FirmwareSight stored the verdict, newest first. A stored run is
                    immutable, so this list cannot be edited and a re-run adds a row rather than
                    replacing one.
                  </caption>
                  <thead>
                    <tr>
                      <th scope="col">
                        <span className={styles['srOnly']}>Row actions</span>
                      </th>
                      <th scope="col">Run</th>
                      <th scope="col">Build</th>
                      <th scope="col">Baseline</th>
                      <th scope="col">Disposition</th>
                      <th scope="col">Findings by state</th>
                      <th scope="col">Stored</th>
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
              label="Gate runs"
              onOffset={runs.goToOffset}
            />
          </>
        )}
      </section>

      <section className={styles['table']} aria-label="Stored release records">
        <h2>Release records</h2>
        <FilterForm
          noun="release records"
          subject="release, version, build, Gate run or manifest digest"
          draft={releases.draft}
          onDraftChange={releases.onDraftChange}
          onApply={releases.applyFilter}
        />
        {releases.loading ? (
          <p className={styles['status']} role="status" aria-live="polite">
            Loading release records…
          </p>
        ) : null}
        {releases.error !== null ? (
          <ErrorPanel
            envelope={releases.error}
            label="Release history error"
            heading="Release records could not be read"
          />
        ) : null}
        {releases.state === null || releases.error !== null ? null : (
          <>
            {releases.state.rows.length === 0 ? (
              <p className={styles['empty']}>
                {releases.applied.length === 0
                  ? 'No release has been published from this computer yet.'
                  : `No stored release record matches “${releases.applied}”.`}
              </p>
            ) : (
              <ScrollArea label="Stored release records table">
                <table className={styles['grid']}>
                  <caption className={styles['caption']}>
                    Ordered by when FirmwareSight recorded the release, newest first. Each row names the
                    Gate run that qualified it.
                  </caption>
                  <thead>
                    <tr>
                      <th scope="col">
                        <span className={styles['srOnly']}>Row actions</span>
                      </th>
                      <th scope="col">Release</th>
                      <th scope="col">Version</th>
                      <th scope="col">Build</th>
                      <th scope="col">Gate run</th>
                      <th scope="col">Manifest SHA-256</th>
                      <th scope="col">Stored</th>
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
              label="release records"
              onOffset={releases.goToOffset}
            />
          </>
        )}
      </section>
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
  const name: StateName = state === 'exact' ? 'PASS' : state === 'partial' ? 'REVIEW' : 'UNKNOWN';
  const label = state === 'exact' ? 'Exact' : state === 'partial' ? 'Partial floor' : 'Unknown';
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
  const badge = dispositionBadge(row.disposition);
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
            Details
          </button>
        </td>
        <td className={styles['mono']}>{truncateMiddle(row.runId, 8)}</td>
        <td title={row.fileName}>{row.fileName}</td>
        <td>{row.baselineFileName === null ? 'None' : row.baselineFileName}</td>
        <td>
          <StateBadge state={badge.name} label={badge.label} />
        </td>
        <td>
          <span className={styles['counts']}>
            <StateBadge state="PASS" label="Pass" count={counts.pass} />
            <StateBadge state="REVIEW" label="Review" count={counts.review} />
            <StateBadge state="BLOCK" label="Block" count={counts.block} />
            <StateBadge state="UNKNOWN" label="Unknown" count={counts.unknown} />
            <StateBadge state="N/A" label="Not applicable" count={counts.notApplicable} />
          </span>
        </td>
        <td className={styles['mono']}>{row.storedAt}</td>
      </tr>
      {open ? (
        <tr className={styles['detail']}>
          <td colSpan={7}>
            <dl className={styles['facts']}>
              <div>
                <dt>Gate run id</dt>
                <dd className={styles['mono']}>{row.runId}</dd>
              </div>
              <div>
                <dt>Build id</dt>
                <dd className={styles['mono']}>{row.buildId}</dd>
              </div>
              {row.baselineBuildId === null ? null : (
                <div>
                  <dt>Baseline build id</dt>
                  <dd className={styles['mono']}>{row.baselineBuildId}</dd>
                </div>
              )}
              <div>
                <dt>Stored</dt>
                <dd className={styles['mono']}>{row.storedAt}</dd>
              </div>
            </dl>
            <p className={styles['detailNote']}>
              The disposition and the counts are what this run stored. Reading history never
              re-evaluates it, and re-running the Gate is a decision on the Release page against the
              build you choose there.
            </p>
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
            Details
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
                <dt>Release id</dt>
                <dd className={styles['mono']}>{row.releaseId}</dd>
              </div>
              <div>
                <dt>Gate run id</dt>
                <dd className={styles['mono']}>{row.gateRunId}</dd>
              </div>
              <div>
                <dt>Build id</dt>
                <dd className={styles['mono']}>{row.buildId}</dd>
              </div>
              <div>
                <dt>Manifest SHA-256</dt>
                <dd className={styles['mono']}>{row.manifestSha256}</dd>
              </div>
              <div>
                <dt>Stored</dt>
                <dd className={styles['mono']}>{row.storedAt}</dd>
              </div>
            </dl>
            <p className={styles['detailNote']}>
              A release row records what was published and which Gate run qualified it. Where the
              bundle folder was written is not part of this record and is never shown here.
            </p>
          </td>
        </tr>
      ) : null}
    </>
  );
}
