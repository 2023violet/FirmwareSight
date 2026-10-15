/**
 * The Compare page: two builds this application already analyzed, and what moved between them.
 *
 * Five rules decide the shape of this screen.
 *
 * 1. **Nothing is computed here.** Deltas, change kinds, comparability and ranking come from Core
 *    through the shell; this page chooses words and layout, and filters, sorts and pages by asking
 *    the shell to do the same (`AGENTS.md` 3, `04_TECH/14` 3).
 * 2. **Absence is a word, not a zero.** An added section has no old size, so its Old column reads
 *    `Not present`, and a delta that could not be computed reads `Unknown` with the reason beside it
 *    (DESIGN.md 5; prompt §23, §38).
 * 3. **The full lists are the answer; the ranking is a way to them.** Top growth and largest
 *    additions are capped navigation aids, and each entry is a control that filters the table below
 *    to the row it names (prompt §25, §26, §41).
 * 4. **A failed comparison does not delete the last good one.** The summary moves only when the
 *    shell returns one, and the surviving report says which pair of builds it belongs to and which
 *    pair it does not (prompt §42).
 * 5. **A comparison outlives the page that computed it.** The pair and the result are session facts,
 *    so the shell holds them and navigating to Overview and back neither loses the answer nor recomputes
 *    it. The diff handle that answers for it lives in this process only, which is why a fresh process
 *    starts with no comparison at all (U1P-R3 F1).
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import styles from './Compare.module.css';
import { Button } from './components/Button';
import { ErrorPanel } from './components/ErrorPanel';
import { Page, ScrollArea } from './components/Layout';
import { PageHeader } from './components/PageHeader';
import { EmptyState, Panel } from './components/Panel';
import { Pager, SortHeader } from './components/Table';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import { StateBadge, type StateName } from './components/StateBadge';
import { evidenceBasisCaption } from './evidenceBasis';
import { ABSENT, formatDelta, formatSize, truncateMiddle, type SizeUnit } from './format';
import { useI18n, type Translate } from './i18n';
import {
  compareSnapshots,
  exportCompareHtml,
  exportCompareJson,
  exportCompareVisual,
  listCompareCandidates,
  querySectionChanges,
  querySymbolChanges,
} from './ipc/bridge';
import type {
  ByteDeltaDto,
  CandidatePageDto,
  ChangeKindCountsDto,
  ChangeKindFilterDto,
  CompareCandidateDto,
  CompareSummaryDto,
  ContributorDto,
  DiffCountsDto,
  DiffMemoryDto,
  DiffSideMemoryDto,
  PercentageChangeDto,
  ErrorEnvelopeDto,
  ExportOutcomeDto,
  SectionChangePageDto,
  SectionChangeRowDto,
  SectionChangeSortDto,
  SortDirDto,
  SymbolChangePageDto,
  SymbolChangeRowDto,
  SymbolChangeSortDto,
} from './ipc/types';
import { cx } from './styles/classnames';

/**
 * The additions lists are one small page of the authoritative change table, ranked by the shell on
 * the same quantity the growth lists use. They are not a second copy of the diff (prompt §25).
 */
const ADDITION_LIMIT = 5;

/** Which side of a change row is being shown: only the unpaired side can be genuinely absent. */
type Side = 'base' | 'target';

/** A drill-down request: the row the reader chose, in the table that holds it. */
type Focus = { readonly key: string };

/** The largest additions, read back from the change tables. */
type Additions = {
  readonly sections: readonly ContributorDto[];
  readonly symbols: readonly ContributorDto[];
};

/**
 * All, Added, Removed, Changed - the four views prompt §25 asks the full lists to offer.
 *
 * `key` is the catalog entry naming the label; `token` is a stable, language-independent id used for the
 * radio's DOM id and React key, because those must not move with the interface language. `Added`,
 * `Removed` and `Changed` are the diff's own change-kind words (`ADR-0032` D5): the catalog keeps them
 * spelled exactly as the shell emits them in both languages, while `All` - a filter state with no diff
 * counterpart - is translated.
 */
const KIND_FILTERS: readonly {
  readonly key: 'filter.all' | 'filter.added' | 'filter.removed' | 'filter.changed';
  readonly token: string;
  readonly value: ChangeKindFilterDto | null;
}[] = [
  { key: 'filter.all', token: 'all', value: null },
  { key: 'filter.added', token: 'added', value: 'added' },
  { key: 'filter.removed', token: 'removed', value: 'removed' },
  { key: 'filter.changed', token: 'changed', value: 'changed' },
];

/**
 * What the shell keeps about a comparison, and the only part of one that has to survive a page switch.
 *
 * The pair and the result travel together because they are one fact: a diff that outlives its own selectors
 * is the stale-pair problem wearing a different coat. `diffId` rides inside `summary` and is deliberately
 * *not* mirrored here - it is this process's handle for a computed comparison, so a fresh session starts
 * with no comparison at all and nothing on this page may suggest otherwise.
 */
export type ComparisonSession = {
  readonly baseId: string | null;
  readonly targetId: string | null;
  readonly summary: CompareSummaryDto | null;
};

export const EMPTY_COMPARISON: ComparisonSession = { baseId: null, targetId: null, summary: null };

export function Compare({
  unit,
  onUnitChange,
  lastAnalyzedSnapshotId,
  session,
  onSessionChange,
  onGoToAnalyze,
}: {
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
  readonly lastAnalyzedSnapshotId: string | null;
  readonly session: ComparisonSession;
  readonly onSessionChange: (
    next: ComparisonSession | ((previous: ComparisonSession) => ComparisonSession),
  ) => void;
  readonly onGoToAnalyze: () => void;
}) {
  const { t } = useI18n();
  const [candidates, setCandidates] = useState<CandidatePageDto | null>(null);
  const [candidateError, setCandidateError] = useState<ErrorEnvelopeDto | null>(null);
  // The pair and the result are the shell's, so leaving the page cannot throw them away (U1P-R3 F1).
  const { baseId, targetId, summary } = session;
  // Every write is an update function rather than a literal built from `session`, because two writes issued
  // from one render would otherwise both spread the same snapshot and the second would silently undo the
  // first - and a comparison that lands late must merge into whatever pair the reader has since chosen,
  // not resurrect the pair they pressed Compare on.
  const setPair = useCallback(
    (next: { readonly baseId: string | null; readonly targetId: string | null }) => {
      onSessionChange((previous) => ({ ...previous, baseId: next.baseId, targetId: next.targetId }));
    },
    [onSessionChange],
  );
  const setBaseId = useCallback(
    (id: string | null) => {
      onSessionChange((previous) => ({ ...previous, baseId: id }));
    },
    [onSessionChange],
  );
  const setTargetId = useCallback(
    (id: string | null) => {
      onSessionChange((previous) => ({ ...previous, targetId: id }));
    },
    [onSessionChange],
  );
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [comparing, setComparing] = useState(false);
  const [sectionFocus, setSectionFocus] = useState<Focus | null>(null);
  const [symbolFocus, setSymbolFocus] = useState<Focus | null>(null);
  const [additions, setAdditions] = useState<Additions | null>(null);
  const [exportNote, setExportNote] = useState<string | null>(null);
  const [exportError, setExportError] = useState<ErrorEnvelopeDto | null>(null);
  const [exporting, setExporting] = useState(false);

  const exportRequest = useRef(0);
  useEffect(() => {
    exportRequest.current += 1;
    setExporting(false);
    setExportNote(null);
    setExportError(null);
    return () => { exportRequest.current += 1; };
  }, [summary?.diffId]);

  const additionsRequest = useRef(0);

  /**
   * The stored builds, one bounded page at a time, read by Rust from SQLite rather than from the
   * filesystem: a build whose original file has since moved stays comparable.
   */
  useEffect(() => {
    let abandoned = false;
    void listCompareCandidates({ offset: null, limit: null }).then((outcome) => {
      if (abandoned) {
        return;
      }
      if (outcome.ok) {
        setCandidates(outcome.value);
        setCandidateError(null);
      } else {
        setCandidateError(outcome.envelope);
      }
    });
    return () => {
      abandoned = true;
    };
  }, []);

  /**
   * Name a default pair once the list exists (prompt §18).
   *
   * The build Analyze last proved is the one the reader most likely wants to compare, so it becomes
   * the target and the next distinct build becomes the base. Choosing is not comparing: no diff runs
   * until the reader presses Compare.
   *
   * The guard that keeps this from running is the same one that protects a retained session: on re-entry both
   * ids are already named, so the pair the reader left behind comes back unchanged and the restored comparison
   * is not silently re-pointed at a different pair (U1P-R3 §11).
   */
  useEffect(() => {
    if (
      candidates === null ||
      candidates.rows.length === 0 ||
      baseId !== null ||
      targetId !== null
    ) {
      return;
    }
    const mostRecent = candidates.rows[0];
    if (mostRecent === undefined) {
      return;
    }
    const preferred =
      candidates.rows.find((row) => row.snapshotId === lastAnalyzedSnapshotId) ?? mostRecent;
    const other = candidates.rows.find((row) => row.snapshotId !== preferred.snapshotId) ?? null;
    setPair({
      baseId: other === null ? null : other.snapshotId,
      targetId: preferred.snapshotId,
    });
  }, [baseId, candidates, lastAnalyzedSnapshotId, setPair, targetId]);

  /** More stored builds, appended to the list the selectors already hold. */
  const loadMore = useCallback(async () => {
    if (candidates === null || candidates.nextOffset === null) {
      return;
    }
    const outcome = await listCompareCandidates({ offset: candidates.nextOffset, limit: null });
    if (!outcome.ok) {
      setCandidateError(outcome.envelope);
      return;
    }
    setCandidates({
      rows: [...candidates.rows, ...outcome.value.rows],
      total: candidates.total,
      offset: candidates.offset,
      limit: candidates.limit,
      nextOffset: outcome.value.nextOffset,
    });
  }, [candidates]);

  /**
   * Compare the chosen pair, and keep the previous comparison on the failure path.
   *
   * `summary` only ever moves when the shell returns one, so a failed attempt changes the error and
   * the label on the surviving report - nothing else.
   *
   * The result is written through the shell rather than into the page, which is what makes a comparison that
   * finishes after the reader has navigated away still arrive instead of landing on an unmounted setter. The
   * write merges into whatever the selectors name by then, so an obsolete response cannot drag the pair back
   * with it; the report simply says which pair it belongs to.
   */
  const compare = useCallback(async () => {
    if (baseId === null || targetId === null || baseId === targetId) {
      return;
    }
    setComparing(true);
    const outcome = await compareSnapshots({ baseSnapshotId: baseId, targetSnapshotId: targetId });
    setComparing(false);

    if (outcome.ok) {
      onSessionChange((previous) => ({ ...previous, summary: outcome.value }));
      setError(null);
      setExportNote(null);
      setExportError(null);
      setSectionFocus(null);
      setSymbolFocus(null);
      return;
    }
    setError(outcome.envelope);
  }, [baseId, onSessionChange, targetId]);

  const swap = useCallback(() => {
    setPair({ baseId: targetId, targetId: baseId });
  }, [baseId, setPair, targetId]);

  /* The largest additions come from the same bounded query the table below uses, so a capped list
     and the full list cannot disagree about which rows exist. If this read fails, that table asks
     the shell the same question and reports the failure where its own rows belong. */
  useEffect(() => {
    if (summary === null) {
      setAdditions(null);
      return;
    }
    const id = ++additionsRequest.current;
    const diffId = summary.diffId;
    void Promise.all([
      querySectionChanges({
        diffId,
        filter: null,
        changeKind: 'added',
        sort: 'memorySize',
        direction: 'desc',
        offset: 0,
        limit: ADDITION_LIMIT,
      }),
      querySymbolChanges({
        diffId,
        filter: null,
        changeKind: 'added',
        sort: 'size',
        direction: 'desc',
        offset: 0,
        limit: ADDITION_LIMIT,
      }),
    ]).then(([sections, symbols]) => {
      if (additionsRequest.current !== id) {
        return;
      }
      if (!sections.ok || !symbols.ok) {
        setAdditions(null);
        return;
      }
      setAdditions({
        sections: sections.value.rows.map(sectionAddition),
        symbols: symbols.value.rows.map(symbolAddition),
      });
    });
  }, [summary]);

  const exportAs = useCallback(
    async (format: 'json' | 'html' | 'png' | 'svg' | 'visual-html') => {
      if (summary === null) {
        return;
      }
      const request = ++exportRequest.current;
      setExporting(true);
      const outcome = format === 'json'
        ? await exportCompareJson(summary.diffId)
        : format === 'html'
          ? await exportCompareHtml(summary.diffId)
          : await exportCompareVisual(summary.diffId, format === 'visual-html' ? 'html' : format);
      if (request !== exportRequest.current) {
        // The comparison changed while the Save dialog was open. Rust refuses that export rather than
        // writing the comparison the reader moved away from (M1, AC24); the screen says so instead of
        // staying silent. Any other late outcome belongs to the old comparison and is not shown here.
        if (!outcome.ok && outcome.envelope.code === 'ERR-EXPORT-6002') {
          setExportNote(t('compare', 'export.staleNote'));
        }
        return;
      }
      setExporting(false);

      if (!outcome.ok) {
        // A failed export does not invalidate the comparison (prompt §42), so nothing above moves.
        setExportError(outcome.envelope);
        setExportNote(null);
        return;
      }
      setExportError(null);
      setExportNote(exportWords(outcome.value, t));
    },
    [summary, t],
  );

  const rows = candidates?.rows ?? [];
  const base = rows.find((row) => row.snapshotId === baseId) ?? null;
  const target = rows.find((row) => row.snapshotId === targetId) ?? null;
  const bothChosen = base !== null && target !== null;
  const samePair = bothChosen && baseId === targetId;
  const hasTwoBuilds = (candidates?.total ?? 0) >= 2;
  // The report answers for the pair it was computed from, not for the pair the selectors happen to
  // name now. A swap or a failed attempt leaves the previous diff standing, so the screen says whose
  // facts the reader is looking at (prompt §18).
  const stale =
    summary !== null &&
    (error !== null ||
      baseId !== summary.base.snapshotId ||
      targetId !== summary.target.snapshotId);

  return (
    <Page>
      <PageHeader
        title={t('compare', 'title')}
        subhead={t('compare', 'subhead')}
      />

      {candidateError === null ? null : (
        <ErrorPanel
          envelope={candidateError}
          label={t('compare', 'buildListError.label')}
          heading={t('compare', 'buildListError.heading')}
        />
      )}

      {candidates === null ? (
        <p className={styles['status']} role="status" aria-live="polite">
          {t('compare', 'status.readingBuilds')}
        </p>
      ) : null}

      {candidates === null || hasTwoBuilds || candidateError !== null ? null : (
        // Fewer than two builds is a fact about the session, and the screen says so instead of
        // inventing a comparison out of one build (prompt §19).
        <EmptyState
          label={t('compare', 'empty.label')}
          message={t('compare', 'empty.message')}
          nextStep={<Button onClick={onGoToAnalyze}>{t('compare', 'empty.action')}</Button>}
        />
      )}

      {candidates === null || !hasTwoBuilds ? null : (
        <section className={styles['selectors']} aria-label={t('compare', 'selectors.region')}>
          <SnapshotPicker
            id="fs-base"
            label={t('compare', 'picker.baseLabel')}
            rows={rows}
            selected={baseId}
            lastAnalyzedSnapshotId={lastAnalyzedSnapshotId}
            unit={unit}
            disabled={comparing}
            onSelect={setBaseId}
          />
          <span className={styles['pairArrow']} aria-hidden="true">
            {'→'}
          </span>
          <SnapshotPicker
            id="fs-target"
            label={t('compare', 'picker.targetLabel')}
            rows={rows}
            selected={targetId}
            lastAnalyzedSnapshotId={lastAnalyzedSnapshotId}
            unit={unit}
            disabled={comparing}
            onSelect={setTargetId}
          />
          <div className={styles['pickerActions']}>
            {candidates.nextOffset === null ? null : (
              <Button
                onClick={() => {
                  void loadMore();
                }}
                disabled={comparing}
              >
                {t('compare', 'picker.showMore')}
              </Button>
            )}
            <Button onClick={swap} disabled={!bothChosen || samePair || comparing}>
              {t('compare', 'picker.swap')}
            </Button>
            <Button
              variant="primary"
              onClick={() => {
                void compare();
              }}
              disabled={!bothChosen || samePair || comparing}
            >
              {t('compare', 'picker.compareAction')}
            </Button>
          </div>
        </section>
      )}

      {samePair ? (
        // Refusing is not a failure to report: the pair is named, and the action stays locked until
        // the reader changes it (prompt §18).
        <p className={styles['warning']} role="note">
          {base === null
            ? t('compare', 'refusal.sameBuildNoLabel')
            : t('compare', 'refusal.sameBuild', { label: buildLabel(base) })}
        </p>
      ) : null}

      {comparing ? (
        <p className={styles['status']} role="status" aria-live="polite">
          {t('compare', 'status.comparing')}
        </p>
      ) : null}

      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label={t('compare', 'comparisonError.label')}
          heading={t('compare', 'comparisonError.heading')}
        />
      )}

      {/* Two stored builds and no comparison is the state every re-entry of this page used to land in, and
          it used to land in an empty band of page: the pickers, a hairline, and nothing under them. It is a
          real fact about the session - there is a pair to compare and no result yet - so it gets the words
          for that fact (U1P-R3 F1). It claims no numbers, no delta and no verdict, because a promise of a
          result is not one, and it does not replace the Compare action it describes. */}
      {candidates === null || !hasTwoBuilds || summary !== null || comparing ? null : (
        <Panel title={t('compare', 'ready.title')}>
          <p className={styles['readyMessage']}>
            {t('compare', 'ready.message')}
          </p>
        </Panel>
      )}

      {/* Before there is a result the unit governs the recorded figures beside the two selectors, so the
          switch sits under them; once there is one it belongs with the figures it re-labels, which is the
          row it heads (U1P-R3 §6). Exactly one switch is on screen at a time. */}
      {summary === null ? <SizeUnitSwitch unit={unit} onSelect={onUnitChange} /> : null}

      {summary === null ? null : (
        <section
          className={styles['report']}
          aria-label={stale ? t('compare', 'report.regionStale') : t('compare', 'report.regionFresh')}
        >
          {stale ? (
            <p className={styles['stale']} role="note">
              {error === null
                ? t('compare', 'report.staleMoved', {
                    base: buildLabel(summary.base),
                    target: buildLabel(summary.target),
                  })
                : t('compare', 'report.staleFailed', {
                    base: buildLabel(summary.base),
                    target: buildLabel(summary.target),
                  })}
            </p>
          ) : null}

          {/* U1P §10: the result leads. What moved, then what it cost in flash and RAM, then the section
              diff itself, with the growth lists beside it - and the long evidence prose, which explains a
              comparison rather than being one, moves below. The order is the only thing that changed: every
              region keeps its own name, its own rows and its own words. */}
          <WhatMoved counts={summary.counts} unit={unit} onUnitChange={onUnitChange} />
          <MemoryComparison memory={summary.memory} unit={unit} />

          {/* One pair of tables for one comparison: each resets its own filter when the handle it
              was given changes. */}
          <SectionChanges
            diffId={summary.diffId}
            unit={unit}
            focus={sectionFocus}
          />

          <Ranking
            summary={summary}
            additions={additions}
            unit={unit}
            onPickSection={(key) => {
              setSectionFocus({ key });
            }}
            onPickSymbol={(key) => {
              setSymbolFocus({ key });
            }}
          />

          <SymbolChanges
            diffId={summary.diffId}
            unit={unit}
            focus={symbolFocus}
          />

          <EvidenceNotice summary={summary} />

          <Exports
            note={exportNote}
            error={exportError}
            exporting={exporting}
            onExport={(format) => {
              void exportAs(format);
            }}
          />

          <footer className={styles['footer']}>
            <span className={styles['monoSmall']}>
              {t('compare', 'footer.baseLine', {
                id: summary.base.snapshotId,
                hash: truncateMiddle(summary.base.sha256, 8),
              })}
            </span>
            <span className={styles['monoSmall']}>
              {t('compare', 'footer.targetLine', {
                id: summary.target.snapshotId,
                hash: truncateMiddle(summary.target.sha256, 8),
              })}
            </span>
            <span>
              {t('compare', 'footer.diffIdNote')}
            </span>
            <span>
              {t('compare', 'footer.storageNote')}
            </span>
          </footer>
        </section>
      )}
    </Page>
  );
}

/**
 * Names one stored build the way the pickers do. Two builds of one artifact share a file name, so a
 * name alone cannot tell the pair on screen from the pair just selected - which is exactly what the
 * "this is not that comparison" note has to do (prompt §18).
 */
function buildLabel(row: { readonly fileName: string; readonly sha256: string }): string {
  return `${row.fileName} · ${row.sha256.slice(0, 12)}`;
}

/** One selector, with the recorded facts of the build it currently names. */
function SnapshotPicker({
  id,
  label,
  rows,
  selected,
  lastAnalyzedSnapshotId,
  unit,
  disabled,
  onSelect,
}: {
  readonly id: string;
  readonly label: string;
  readonly rows: readonly CompareCandidateDto[];
  readonly selected: string | null;
  readonly lastAnalyzedSnapshotId: string | null;
  readonly unit: SizeUnit;
  readonly disabled: boolean;
  readonly onSelect: (snapshotId: string | null) => void;
}) {
  const { t } = useI18n();
  const chosen = rows.find((row) => row.snapshotId === selected) ?? null;

  return (
    <div className={styles['picker']}>
      <label className={styles['pickerLabel']} htmlFor={id}>
        {label}
      </label>
      <select
        id={id}
        className={styles['select']}
        value={selected ?? ''}
        disabled={disabled}
        onChange={(event) => {
          onSelect(event.target.value === '' ? null : event.target.value);
        }}
      >
        <option value="">{t('compare', 'picker.chooseOption')}</option>
        {rows.map((row) => (
          <option
            key={row.snapshotId}
            value={row.snapshotId}
            title={`${row.fileName} · sha256 ${row.sha256} · imported ${row.importedAt}`}
          >
            {buildLabel(row)}
            {row.snapshotId === lastAnalyzedSnapshotId ? t('compare', 'picker.lastAnalyzedSuffix') : ''}
          </option>
        ))}
      </select>

      {chosen === null ? (
        <p className={styles['sideFacts']}>{t('compare', 'picker.noBuildNamed')}</p>
      ) : (
        <p className={styles['sideFacts']}>
          <span className={styles['mono']}>{formatSize(chosen.byteSize, unit)}</span>
          <span>{chosen.architecture}</span>
          <span>
            {t('compare', 'picker.nonvolatilePrefix')} {budgetWords(chosen.nonvolatile.state, chosen.nonvolatile.bytes, unit)}
          </span>
          <span>
            {t('compare', 'picker.runtimeRamPrefix')} {budgetWords(chosen.runtimeRam.state, chosen.runtimeRam.bytes, unit)}
          </span>
          <span className={styles['monoSmall']}>{t('compare', 'picker.importedLine', { date: chosen.importedAt })}</span>
        </p>
      )}
    </div>
  );
}

function budgetWords(state: string, bytes: number | null, unit: SizeUnit): string {
  return `${state} ${formatSize(bytes, unit)}`;
}

/**
 * The two memory totals, each with both sides and the signed difference.
 *
 * Comparability is the strength of *this* comparison, not of one side: an exact total measured
 * against a partial one is a partial delta, which is why the word sits beside the figures rather
 * than under a heading of its own (prompt §38, `04_TECH/23`).
 */
function MemoryComparison({
  memory,
  unit,
}: {
  readonly memory: DiffMemoryDto;
  readonly unit: SizeUnit;
}) {
  const { t } = useI18n();
  return (
    <section className={styles['section']} aria-label={t('compare', 'memory.region')}>
      <div className={styles['sectionHead']}>
        <h2>{t('compare', 'memory.heading')}</h2>
        <p className={styles['caption']}>
          {t('compare', 'memory.caption', { absent: ABSENT })}
        </p>
      </div>
      <div className={styles['rows']}>
        <MemoryRow term={t('compare', 'memory.nonvolatileTerm')} change={memory.nonvolatile} percent={memory.nonvolatilePercent} unit={unit} />
        <MemoryRow term={t('compare', 'memory.runtimeRamTerm')} change={memory.runtimeRam} percent={memory.runtimeRamPercent} unit={unit} />
        <div className={styles['row']}>
          <span className={styles['term']}>{t('compare', 'memory.comparabilityTerm')}</span>
          <span className={styles['value']}>
            <Comparability value={memory.comparability} />
            <span className={styles['note']}>
              {t('compare', 'memory.comparabilityNote')}
            </span>
          </span>
        </div>
        <SideEvidenceRow label={t('compare', 'evidence.baseLabel')} side={memory.base} />
        <SideEvidenceRow label={t('compare', 'evidence.targetLabel')} side={memory.target} />
      </div>
    </section>
  );
}

function MemoryRow({
  term,
  change,
  percent,
  unit,
}: {
  readonly term: string;
  readonly change: ByteDeltaDto;
  readonly percent: PercentageChangeDto;
  readonly unit: SizeUnit;
}) {
  const { t } = useI18n();
  return (
    <div className={styles['row']}>
      <span className={styles['term']}>{term}</span>
      <span className={styles['value']}>
        <Figure axis={t('compare', 'axis.old')} text={formatSize(change.base, unit)} />
        <Figure axis={t('compare', 'axis.new')} text={formatSize(change.target, unit)} />
        <DeltaFigure change={change} unit={unit} />
        <span className={styles['note']}>
          {t('compare', 'memory.percentLabel')} {percent.value ?? percentStatus(percent.status, t)}
        </span>
      </span>
    </div>
  );
}

function percentStatus(status: string, t: Translate): string {
  switch (status) {
    case 'missing_side':
      return t('compare', 'memory.percentMissingSide');
    case 'not_exact':
      return t('compare', 'memory.percentNotExact');
    case 'zero_baseline':
      return t('compare', 'memory.percentZeroBaseline');
    case 'delta_unavailable':
      return t('compare', 'memory.percentDeltaUnavailable');
    default:
      return t('compare', 'memory.percentUnknown');
  }
}

/**
 * One number with the word that says which axis it belongs to.
 *
 * A delta may take the diff colour, because the sign and the word are both on the page; a side value
 * never does, since `Not present` and `Unknown` are not colours.
 */
function Figure({
  axis,
  text,
  delta,
  reason,
}: {
  readonly axis: string;
  readonly text: string;
  readonly delta?: number | null;
  readonly reason?: string | null;
}) {
  const sign =
    delta === undefined || delta === null
      ? undefined
      : delta > 0
        ? styles['deltaPositive']
        : delta < 0
          ? styles['deltaNegative']
          : undefined;
  return (
    <span className={styles['figure']}>
      {/* The axis word carries its own spacing, so the pair reads as one sentence in the DOM and not
          only as two columns on the screen. */}
      <span className={styles['axis']}>{`${axis} `}</span>
      <span className={cx(styles['mono'], sign)}>{text}</span>
      {reason === undefined || reason === null ? null : (
        <span className={styles['note']}> {reason}</span>
      )}
    </span>
  );
}

function DeltaFigure({
  change,
  unit,
}: {
  readonly change: ByteDeltaDto;
  readonly unit: SizeUnit;
}) {
  const { t } = useI18n();
  return (
    <Figure
      axis={t('compare', 'axis.delta')}
      text={formatDelta(change.delta, unit)}
      delta={change.delta}
      reason={change.delta === null ? change.reason : null}
    />
  );
}

/// What Compare adds to the shared evidence-basis caption: its own word for a side that recorded none.
///
/// The mapping of the five bases the memory model can name lives in `evidenceBasis.ts`, in the
/// vocabulary Analyze already uses, so this page and Release cannot drift apart. A missing basis gets
/// Compare's own word from the catalog rather than a hardcoded English string.
function basisCaption(basis: string | null, t: Translate): string {
  if (basis === null) {
    return t('compare', 'evidence.unknownBasis');
  }
  return evidenceBasisCaption(basis, t);
}

function SideEvidenceRow({
  label,
  side,
}: {
  readonly label: string;
  readonly side: DiffSideMemoryDto;
}) {
  const { t } = useI18n();
  return (
    <div className={styles['row']}>
      <span className={styles['term']}>{label}</span>
      <span className={styles['value']}>
        <StateBadge
          state={side.mapBacked ? 'PASS' : 'UNKNOWN'}
          label={side.mapBacked ? t('compare', 'evidence.linkerMapUsed') : t('compare', 'evidence.noLinkerMap')}
        />
        <span className={styles['monoSmall']}>{t('compare', 'evidence.layoutPrefix')} {side.layoutSource}</span>
        <span className={styles['monoSmall']}>
          {t('compare', 'evidence.weakestBasisPrefix')} {basisCaption(side.weakestEvidenceBasis, t)}
        </span>
        <span className={styles['note']}>
          {side.footprintRowPresent
            ? t('compare', 'evidence.footprintRecorded')
            : t('compare', 'evidence.footprintMissing')}
        </span>
      </span>
    </div>
  );
}

/**
 * The facts that say how much of this comparison rests on evidence (prompt §43).
 *
 * Only what the shell recorded appears: a warning the diff raised, the attribution capability, and
 * the per-side layout basis in the Memory block. No per-row provenance is claimed, because the
 * stored facts do not support one.
 */
function EvidenceNotice({ summary }: { readonly summary: CompareSummaryDto }) {
  const { t } = useI18n();
  const { memory, warnings, objectChanges } = summary;
  const lines: readonly string[] = [
    ...(memory.evidenceWarning === null ? [] : [memory.evidenceWarning]),
    ...warnings.map((warning) => `${warning.code}: ${warning.message}`),
  ];

  return (
    <section className={styles['section']} aria-label={t('compare', 'notice.region')}>
      <h2>{t('compare', 'notice.heading')}</h2>
      {lines.length === 0 ? (
        <p className={styles['status']}>{t('compare', 'notice.noWarning')}</p>
      ) : (
        <ul className={styles['noticeList']}>
          {lines.map((line) => (
            <li key={line} className={styles['noticeItem']}>
              {line}
            </li>
          ))}
        </ul>
      )}
      <p className={styles['status']}>{t('compare', 'notice.producerCompatibilityUnproven')}</p>
      <p className={styles['status']}>{t('compare', 'notice.remainderUnknown')}</p>
      <div className={styles['row']}>
        {/* L19: this page answers "did any object file change between the two builds", which is not
            the question Analyze's capability row answers. The two labels must stay distinct. */}
        <span className={styles['term']}>{t('compare', 'notice.objectTerm')}</span>
        <span className={styles['value']}>
          <StateBadge
            state={objectChanges.available ? 'PASS' : 'N/A'}
            label={objectChanges.available ? t('compare', 'notice.attributionAvailable') : t('compare', 'notice.attributionUnavailable')}
            note={objectChanges.available ? undefined : objectChanges.reason}
          />
        </span>
      </div>
    </section>
  );
}

/**
 * How many rows moved, in diff vocabulary.
 *
 * Added, Removed and Changed are not Gate states and borrow no Gate icon: each is a word with a
 * count, and the unpaired count says what the pairing could not decide (prompt §40).
 *
 * The unit switch moved into this row (U1P-R3 §6) because it answers a question about the figures below,
 * and a full-width row of its own above them pushed the first section rows off the first viewport. The
 * counts do not convert, so the switch sits here as the page's unit rather than as a control for this row.
 */
function WhatMoved({
  counts,
  unit,
  onUnitChange,
}: {
  readonly counts: DiffCountsDto;
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
}) {
  const { t } = useI18n();
  return (
    <section className={styles['section']} aria-label={t('compare', 'moved.region')}>
      <div className={styles['sectionHead']}>
        <h2>{t('compare', 'moved.heading')}</h2>
        <SizeUnitSwitch unit={unit} onSelect={onUnitChange} />
      </div>
      <div className={styles['rows']}>
        <CountRow term={t('compare', 'moved.sectionsTerm')} counts={counts.sections} unchanged={counts.unchangedSections} />
        <CountRow term={t('compare', 'moved.symbolsTerm')} counts={counts.symbols} unchanged={counts.unchangedSymbols} />
      </div>
    </section>
  );
}

function CountRow({
  term,
  counts,
  unchanged,
}: {
  readonly term: string;
  readonly counts: ChangeKindCountsDto;
  readonly unchanged: number;
}) {
  const { t } = useI18n();
  return (
    <div className={styles['row']}>
      <span className={styles['term']}>{term}</span>
      <span className={cx(styles['value'], styles['counts'])}>
        <Kind kind="Added" count={counts.added} />
        <Kind kind="Removed" count={counts.removed} />
        <Kind kind="Changed" count={counts.changed} />
        <span className={styles['note']}>{t('compare', 'moved.unchanged', { count: unchanged })}</span>
        {counts.ambiguous === 0 ? null : (
          <span className={styles['unknown']}>
            {t('compare', 'moved.unpaired', { count: counts.ambiguous })}
            <span className={styles['note']}>
              {t('compare', 'moved.unpairedNote')}
            </span>
          </span>
        )}
      </span>
    </div>
  );
}

function Kind({ kind, count }: { readonly kind: string; readonly count?: number }) {
  return (
    <span className={cx(styles['kind'], kindClass(kind))}>
      {kind}
      {count === undefined ? null : ` ${String(count)}`}
    </span>
  );
}

/** The word is the state; the diff roles only reinforce it. An unrecognised word stays neutral. */
function kindClass(kind: string): string | undefined {
  switch (kind) {
    case 'Added':
      return styles['kindAdded'];
    case 'Removed':
      return styles['kindRemoved'];
    case 'Changed':
      return styles['kindChanged'];
    default:
      return undefined;
  }
}

/**
 * The two capped lists, kept apart because they answer different questions (prompt §26).
 *
 * Growth is a Changed row that got bigger; an addition is a row that did not exist before. Folding
 * them into one ranking would imply an addition grew from zero, which the diff refuses to claim.
 */
function Ranking({
  summary,
  additions,
  unit,
  onPickSection,
  onPickSymbol,
}: {
  readonly summary: CompareSummaryDto;
  readonly additions: Additions | null;
  readonly unit: SizeUnit;
  readonly onPickSection: (key: string) => void;
  readonly onPickSymbol: (key: string) => void;
}) {
  const { t } = useI18n();
  return (
    <section className={styles['section']} aria-label={t('compare', 'ranking.region')}>
      <h2>{t('compare', 'ranking.heading')}</h2>
      <p className={styles['hint']}>
        {t('compare', 'ranking.basisHint')}
      </p>
      <div className={styles['ranking']}>
        <div className={styles['list']}>
          <h3>{t('compare', 'ranking.growthHeading')}</h3>
          <p className={styles['hint']}>
            {t('compare', 'ranking.growthHint')}
          </p>
          <Contributors
            nounKey="contributors.sections"
            rows={summary.topSections}
            unit={unit}
            emptyKey="contributors.emptySectionsGrowth"
            onPick={onPickSection}
          />
          <Contributors
            nounKey="contributors.symbols"
            rows={summary.topSymbols}
            unit={unit}
            emptyKey="contributors.emptySymbolsGrowth"
            onPick={onPickSymbol}
          />
        </div>
        <div className={styles['list']}>
          <h3>{t('compare', 'ranking.additionsHeading')}</h3>
          <p className={styles['hint']}>
            {t('compare', 'ranking.additionsHint', { absent: ABSENT })}
          </p>
          {additions === null ? (
            <p className={styles['status']}>{t('compare', 'ranking.readingAdded')}</p>
          ) : (
            <>
              <Contributors
                nounKey="contributors.sections"
                rows={additions.sections}
                unit={unit}
                emptyKey="contributors.emptySectionsAdded"
                onPick={onPickSection}
              />
              <Contributors
                nounKey="contributors.symbols"
                rows={additions.symbols}
                unit={unit}
                emptyKey="contributors.emptySymbolsAdded"
                onPick={onPickSymbol}
              />
            </>
          )}
        </div>
      </div>
    </section>
  );
}

function Contributors({
  nounKey,
  rows,
  unit,
  emptyKey,
  onPick,
}: {
  readonly nounKey: 'contributors.sections' | 'contributors.symbols';
  readonly rows: readonly ContributorDto[];
  readonly unit: SizeUnit;
  readonly emptyKey:
    | 'contributors.emptySectionsGrowth'
    | 'contributors.emptySymbolsGrowth'
    | 'contributors.emptySectionsAdded'
    | 'contributors.emptySymbolsAdded';
  readonly onPick: (key: string) => void;
}) {
  const { t } = useI18n();
  const noun = t('compare', nounKey);
  return (
    <div className={styles['contribGroup']}>
      <span className={styles['contribNoun']}>{noun}</span>
      {rows.length === 0 ? (
        <p className={styles['status']}>{t('compare', emptyKey)}</p>
      ) : (
        <ol className={styles['contribList']}>
          {rows.map((row, index) => (
            <li key={`${row.key}:${String(index)}`}>
              <button
                type="button"
                className={styles['pick']}
                aria-label={t('compare', 'contributors.pickAria', { noun, key: row.key })}
                onClick={() => {
                  onPick(row.key);
                }}
              >
                <span className={styles['wrap']}>
                  {row.key}
                  <span className={styles['note']}> {row.changeKind}</span>
                </span>
                <span className={styles['pickFigure']}>
                  {/* A growth row shows its signed delta; an addition has no delta to show, so its
                      size stands in, labelled by the list it is in. */}
                  {row.delta === null ? formatSize(row.bytes, unit) : formatDelta(row.delta, unit)}
                </span>
              </button>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}

/**
 * The full section list, filtered, ordered and paged by the shell.
 *
 * A row the diff has no number for keeps its word: `Not present` on the side the row did not exist
 * on, `Unknown` and the reason where a number was simply never recorded (prompt §23).
 */
function SectionChanges({
  diffId,
  unit,
  focus,
}: {
  readonly diffId: string;
  readonly unit: SizeUnit;
  readonly focus: Focus | null;
}) {
  const { t } = useI18n();
  const [offset, setOffset] = useState(0);
  const [filter, setFilter] = useState<string | null>(null);
  const [draft, setDraft] = useState('');
  const [kind, setKind] = useState<ChangeKindFilterDto | null>(null);
  const [sort, setSort] = useState<SectionChangeSortDto>('key');
  const [direction, setDirection] = useState<SortDirDto>('asc');
  const [page, setPage] = useState<SectionChangePageDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [loading, setLoading] = useState(false);
  const request = useRef(0);
  const area = useRef<HTMLElement | null>(null);
  const [seenDiffId, setSeenDiffId] = useState(diffId);

  // A new comparison must not inherit the previous pair's filter. React re-renders before it
  // commits, so no request goes out with the old name still in the box.
  if (seenDiffId !== diffId) {
    setSeenDiffId(diffId);
    setFilter(null);
    setDraft('');
    setKind(null);
    setOffset(0);
  }

  // A drill-down names the row the reader wants and moves the keyboard here; the filter travels to
  // the shell like any other, so the row is filtered from the whole table, not from one page.
  useEffect(() => {
    if (focus === null) {
      return;
    }
    setFilter(focus.key);
    setDraft(focus.key);
    setKind(null);
    setOffset(0);
    area.current?.focus();
  }, [focus]);

  useEffect(() => {
    const id = ++request.current;
    setLoading(true);
    setPage(null);
    void querySectionChanges({
      diffId,
      filter,
      changeKind: kind,
      sort,
      direction,
      offset,
      limit: null,
    }).then((outcome) => {
      if (request.current !== id) {
        return;
      }
      setLoading(false);
      if (outcome.ok) {
        setPage(outcome.value);
        setError(null);
      } else {
        setError(outcome.envelope);
      }
    });
  }, [diffId, filter, kind, sort, direction, offset]);

  const chooseSort = useCallback(
    (column: SectionChangeSortDto) => {
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

  return (
    <section className={styles['area']} ref={area} tabIndex={-1} aria-label={t('compare', 'section.region')}>
      <h2>{t('compare', 'section.heading')}</h2>
      <ChangeFilters
        nounKey="contributors.sections"
        kind={kind}
        draft={draft}
        onKind={
          (value) => {
            setKind(value);
            setOffset(0);
          }
        }
        onDraftChange={setDraft}
        onApply={(value) => {
          setDraft(value);
          setFilter(value.length === 0 ? null : value);
          setOffset(0);
        }}
      />
      {loading ? (
        <p className={styles['status']} role="status" aria-live="polite">
          {t('compare', 'section.loading')}
        </p>
      ) : null}
      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label={t('compare', 'section.errorLabel')}
          heading={t('compare', 'section.errorHeading')}
        />
      )}

      {page === null ? null : (
        <>
          <ScrollArea label={t('compare', 'section.tableLabel')}>
            <table className={styles['table']}>
              <caption className={styles['caption']}>
                {t('compare', 'section.caption')}
              </caption>
              <thead>
                <tr>
                  <Sortable
                    label={t('compare', 'col.change')}
                    column="changeKind"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <Sortable
                    label={t('compare', 'col.name')}
                    column="key"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <SortHeader label={t('compare', 'col.oldFile')} sortable={false} direction={direction} />
                  <SortHeader label={t('compare', 'col.newFile')} sortable={false} direction={direction} />
                  <Sortable
                    label={t('compare', 'col.deltaFile')}
                    column="delta"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <SortHeader label={t('compare', 'col.oldRam')} sortable={false} direction={direction} />
                  <SortHeader label={t('compare', 'col.newRam')} sortable={false} direction={direction} />
                  <SortHeader label={t('compare', 'col.deltaRam')} sortable={false} direction={direction} />
                  <Sortable
                    label={t('compare', 'col.ramSize')}
                    column="memorySize"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <Sortable
                    label={t('compare', 'col.fileSize')}
                    column="fileSize"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <SortHeader label={t('compare', 'col.alsoDiffers')} sortable={false} direction={direction} />
                </tr>
              </thead>
              <tbody>
                {page.rows.map((row, index) => (
                  <tr key={`${row.key}:${String(index)}`}>
                    <td>
                      <Kind kind={row.changeKind} />
                    </td>
                    <td className={styles['wrap']}>
                      {row.key}
                      {row.nameKnown ? null : (
                        <span className={styles['note']}>{t('compare', 'section.noNameNote')}</span>
                      )}
                      {row.ambiguous ? (
                        <span className={styles['note']}>{t('compare', 'section.ambiguousNote')}</span>
                      ) : null}
                    </td>
                    <td className={styles['mono']}>
                      <SideValue change={row.fileSize} side="base" kind={row.changeKind} unit={unit} />
                    </td>
                    <td className={styles['mono']}>
                      <SideValue
                        change={row.fileSize}
                        side="target"
                        kind={row.changeKind}
                        unit={unit}
                      />
                    </td>
                    <td className={styles['mono']}>
                      <DeltaFigure change={row.fileSize} unit={unit} />
                    </td>
                    <td className={styles['mono']}>
                      <SideValue
                        change={row.memorySize}
                        side="base"
                        kind={row.changeKind}
                        unit={unit}
                      />
                    </td>
                    <td className={styles['mono']}>
                      <SideValue
                        change={row.memorySize}
                        side="target"
                        kind={row.changeKind}
                        unit={unit}
                      />
                    </td>
                    <td className={styles['mono']}>
                      <DeltaFigure change={row.memorySize} unit={unit} />
                    </td>
                    <td className={styles['mono']}>
                      {sizeText(row.memorySize, unit)}
                      {' / '}
                      {sizeText(row.fileSize, unit)}
                    </td>
                    <td>
                      <Differs row={row} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </ScrollArea>

          <Pager
            total={page.total}
            offset={page.offset}
            shown={page.rows.length}
            nextOffset={page.nextOffset}
            label={t('compare', 'section.pagerLabel')}
            onOffset={setOffset}
          />
        </>
      )}
    </section>
  );
}

/** The full symbol list, with the same filters and the fields a symbol row carries. */
function SymbolChanges({
  diffId,
  unit,
  focus,
}: {
  readonly diffId: string;
  readonly unit: SizeUnit;
  readonly focus: Focus | null;
}) {
  const { t } = useI18n();
  const [offset, setOffset] = useState(0);
  const [filter, setFilter] = useState<string | null>(null);
  const [draft, setDraft] = useState('');
  const [kind, setKind] = useState<ChangeKindFilterDto | null>(null);
  const [sort, setSort] = useState<SymbolChangeSortDto>('name');
  const [direction, setDirection] = useState<SortDirDto>('asc');
  const [page, setPage] = useState<SymbolChangePageDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [loading, setLoading] = useState(false);
  const request = useRef(0);
  const area = useRef<HTMLElement | null>(null);
  const [seenDiffId, setSeenDiffId] = useState(diffId);

  // Same rule as the sections table: the pair changed, so the reading of it starts clean.
  if (seenDiffId !== diffId) {
    setSeenDiffId(diffId);
    setFilter(null);
    setDraft('');
    setKind(null);
    setOffset(0);
  }

  useEffect(() => {
    if (focus === null) {
      return;
    }
    setFilter(focus.key);
    setDraft(focus.key);
    setKind(null);
    setOffset(0);
    area.current?.focus();
  }, [focus]);

  useEffect(() => {
    const id = ++request.current;
    setLoading(true);
    setPage(null);
    void querySymbolChanges({
      diffId,
      filter,
      changeKind: kind,
      sort,
      direction,
      offset,
      limit: null,
    }).then((outcome) => {
      if (request.current !== id) {
        return;
      }
      setLoading(false);
      if (outcome.ok) {
        setPage(outcome.value);
        setError(null);
      } else {
        setError(outcome.envelope);
      }
    });
  }, [diffId, filter, kind, sort, direction, offset]);

  const chooseSort = useCallback(
    (column: SymbolChangeSortDto) => {
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

  return (
    <section className={styles['area']} ref={area} tabIndex={-1} aria-label={t('compare', 'symbol.region')}>
      <h2>{t('compare', 'symbol.heading')}</h2>
      <ChangeFilters
        nounKey="contributors.symbols"
        kind={kind}
        draft={draft}
        onKind={
          (value) => {
            setKind(value);
            setOffset(0);
          }
        }
        onDraftChange={setDraft}
        onApply={(value) => {
          setDraft(value);
          setFilter(value.length === 0 ? null : value);
          setOffset(0);
        }}
      />
      {loading ? (
        <p className={styles['status']} role="status" aria-live="polite">
          {t('compare', 'symbol.loading')}
        </p>
      ) : null}
      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label={t('compare', 'symbol.errorLabel')}
          heading={t('compare', 'symbol.errorHeading')}
        />
      )}

      {page === null ? null : (
        <>
          <ScrollArea label={t('compare', 'symbol.tableLabel')}>
            <table className={styles['table']}>
              <caption className={styles['caption']}>
                {t('compare', 'symbol.caption')}
              </caption>
              <thead>
                <tr>
                  <Sortable
                    label={t('compare', 'col.change')}
                    column="changeKind"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <Sortable
                    label={t('compare', 'col.name')}
                    column="name"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <SortHeader label={t('compare', 'col.kindBinding')} sortable={false} direction={direction} />
                  <SortHeader label={t('compare', 'col.oldAddress')} sortable={false} direction={direction} />
                  <SortHeader label={t('compare', 'col.newAddress')} sortable={false} direction={direction} />
                  <SortHeader label={t('compare', 'col.oldSize')} sortable={false} direction={direction} />
                  <SortHeader label={t('compare', 'col.newSize')} sortable={false} direction={direction} />
                  <Sortable
                    label={t('compare', 'col.deltaSize')}
                    column="sizeDelta"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <Sortable
                    label={t('compare', 'col.size')}
                    column="size"
                    sort={sort}
                    direction={direction}
                    onSort={chooseSort}
                  />
                  <SortHeader label={t('compare', 'col.section')} sortable={false} direction={direction} />
                  <SortHeader label={t('compare', 'col.alsoDiffers')} sortable={false} direction={direction} />
                </tr>
              </thead>
              <tbody>
                {page.rows.map((row, index) => (
                  <tr key={`${row.name}:${String(index)}`}>
                    <td>
                      <Kind kind={row.changeKind} />
                    </td>
                    <td className={styles['wrap']}>
                      {row.name}
                      {row.ambiguous ? (
                        <span className={styles['note']}>{t('compare', 'symbol.ambiguousNote')}</span>
                      ) : null}
                    </td>
                    <td>
                      {row.kind} / {row.binding}
                    </td>
                    <td className={styles['mono']}>{sideAddress(row, 'base')}</td>
                    <td className={styles['mono']}>{sideAddress(row, 'target')}</td>
                    <td className={styles['mono']}>
                      <SideValue change={row.size} side="base" kind={row.changeKind} unit={unit} />
                    </td>
                    <td className={styles['mono']}>
                      <SideValue change={row.size} side="target" kind={row.changeKind} unit={unit} />
                    </td>
                    <td className={styles['mono']}>
                      <DeltaFigure change={row.size} unit={unit} />
                    </td>
                    <td className={styles['mono']}>{sizeText(row.size, unit)}</td>
                    <td className={styles['mono']}>
                      {row.base === null ? ABSENT : row.base.sectionRef}
                      {' → '}
                      {row.target === null ? ABSENT : row.target.sectionRef}
                    </td>
                    <td>
                      <Differs row={row} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </ScrollArea>

          <Pager
            total={page.total}
            offset={page.offset}
            shown={page.rows.length}
            nextOffset={page.nextOffset}
            label={t('compare', 'symbol.pagerLabel')}
            onOffset={setOffset}
          />
        </>
      )}
    </section>
  );
}

/**
 * The two export actions.
 *
 * The save path is chosen in a dialog the Rust side opens, so this page learns only the file name
 * and what happened. Declining to pick a folder, and declining to overwrite a file, are reported as
 * the normal decisions they are (prompt §35, §36).
 */
function Exports({
  note,
  error,
  exporting,
  onExport,
}: {
  readonly note: string | null;
  readonly error: ErrorEnvelopeDto | null;
  readonly exporting: boolean;
  readonly onExport: (format: 'json' | 'html' | 'png' | 'svg' | 'visual-html') => void;
}) {
  const { t } = useI18n();
  return (
    <section className={styles['block']} aria-label={t('compare', 'export.region')}>
      <h2>{t('compare', 'export.heading')}</h2>
      <p className={styles['hint']}>
        {t('compare', 'export.hint')}
      </p>
      <div className={styles['exports']}>
        <Button
          disabled={exporting}
          onClick={() => {
            onExport('json');
          }}
        >
          {t('compare', 'export.json')}
        </Button>
        <Button
          disabled={exporting}
          onClick={() => {
            onExport('html');
          }}
        >
          {t('compare', 'export.html')}
        </Button>
        <Button disabled={exporting} onClick={() => onExport('png')}>{t('compare', 'export.png')}</Button>
        <Button disabled={exporting} onClick={() => onExport('svg')}>{t('compare', 'export.svg')}</Button>
        <Button disabled={exporting} onClick={() => onExport('visual-html')}>{t('compare', 'export.printHtml')}</Button>
        {exporting ? (
          <p className={styles['status']} role="status" aria-live="polite">
            {t('compare', 'export.waiting')}
          </p>
        ) : null}
        {note === null ? null : (
          <p className={styles['status']} role="status" aria-live="polite">
            {note}
          </p>
        )}
      </div>
      {error === null ? null : (
        <ErrorPanel envelope={error} label={t('compare', 'export.errorLabel')} heading={t('compare', 'export.errorHeading')} />
      )}
    </section>
  );
}

function exportWords(outcome: ExportOutcomeDto, t: Translate): string {
  switch (outcome.status) {
    case 'written':
      return t('compare', 'export.written', {
        name: outcome.fileName ?? t('compare', 'export.fallbackName'),
        format: outcome.format.toUpperCase(),
      });
    case 'cancelled':
      return t('compare', 'export.cancelled');
    case 'kept-existing':
      return t('compare', 'export.keptExisting');
    default:
      // An outcome this build does not know is reported as what it is, not as a guess.
      return t('compare', 'export.unknownOutcome', { status: outcome.status });
  }
}

/** A column header that orders by one of this table's sort keys. */
function Sortable<S extends string>({
  label,
  column,
  sort,
  direction,
  onSort,
}: {
  readonly label: string;
  readonly column: S;
  readonly sort: S;
  readonly direction: SortDirDto;
  readonly onSort: (column: S) => void;
}) {
  return (
    <SortHeader
      label={label}
      direction={direction}
      active={sort === column}
      onSort={() => {
        onSort(column);
      }}
    />
  );
}

function ChangeFilters({
  nounKey,
  kind,
  draft,
  onKind,
  onDraftChange,
  onApply,
}: {
  readonly nounKey: 'contributors.sections' | 'contributors.symbols';
  readonly kind: ChangeKindFilterDto | null;
  readonly draft: string;
  readonly onKind: (kind: ChangeKindFilterDto | null) => void;
  readonly onDraftChange: (value: string) => void;
  readonly onApply: (value: string) => void;
}) {
  const { t } = useI18n();
  const noun = t('compare', nounKey);
  const inputId = `fs-change-filter-${noun}`;

  return (
    <form
      className={styles['filters']}
      aria-label={t('compare', 'filter.formAria', { noun })}
      onSubmit={(event) => {
        event.preventDefault();
        onApply(draft);
      }}
    >
      <span className={styles['radioGroup']} role="group" aria-label={t('compare', 'filter.groupAria', { noun })}>
        {KIND_FILTERS.map((entry) => (
          <span key={entry.token} className={styles['radio']}>
            <input
              id={`fs-change-${noun}-${entry.token}`}
              type="radio"
              name={`fs-change-${noun}`}
              checked={kind === entry.value}
              onChange={() => {
                onKind(entry.value);
              }}
            />
            <label htmlFor={`fs-change-${noun}-${entry.token}`}>{t('compare', entry.key)}</label>
          </span>
        ))}
      </span>
      <label className={styles['filterLabel']} htmlFor={inputId}>
        {t('compare', 'filter.byName', { noun })}
      </label>
      <input
        id={inputId}
        className={styles['filterInput']}
        type="text"
        autoComplete="off"
        value={draft}
        onChange={(event) => {
          onDraftChange(event.target.value);
        }}
      />
      <Button type="submit">{t('compare', 'filter.apply')}</Button>
    </form>
  );
}

/**
 * One side of a change row.
 *
 * `Not present` is reserved for the side the diff says the row did not exist on. Every other empty
 * cell is a number that was never recorded, which is Unknown and not absence (prompt §23).
 */
function SideValue({
  change,
  side,
  kind,
  unit,
}: {
  readonly change: ByteDeltaDto;
  readonly side: Side;
  readonly kind: string;
  readonly unit: SizeUnit;
}) {
  const value = side === 'base' ? change.base : change.target;
  if (value !== null) {
    return <span className={styles['mono']}>{formatSize(value, unit)}</span>;
  }
  if (isAbsentSide(kind, side)) {
    return <span className={styles['absent']}>{ABSENT}</span>;
  }
  return (
    <span className={styles['unknown']}>
      Unknown
      {change.reason === null ? null : <span className={styles['note']}> {change.reason}</span>}
    </span>
  );
}

/** The row's own size, whichever side carries it: the quantity the size columns order by. */
function sizeText(change: ByteDeltaDto, unit: SizeUnit): string {
  return formatSize(change.target ?? change.base, unit);
}

function sideAddress(row: SymbolChangeRowDto, side: Side): string {
  const record = side === 'base' ? row.base : row.target;
  if (record !== null) {
    return record.address ?? 'Unknown';
  }
  return isAbsentSide(row.changeKind, side) ? ABSENT : 'Unknown';
}

/** Only an unpaired row is absent on one side; a Changed row with no number is unknown, not absent. */
function isAbsentSide(kind: string, side: Side): boolean {
  return (kind === 'Added' && side === 'base') || (kind === 'Removed' && side === 'target');
}

/** What else the diff saw, and what it refused to call a change. */
function Differs({ row }: { readonly row: SectionChangeRowDto | SymbolChangeRowDto }) {
  const { t } = useI18n();
  return (
    <>
      {row.differingFields.length === 0 ? null : (
        <span className={styles['monoSmall']}>{row.differingFields.join(', ')}</span>
      )}
      {row.indeterminateFields.length === 0 ? null : (
        <span className={styles['note']}>
          {row.differingFields.length === 0 ? '' : ' '}
          {t('compare', 'differs.notDecidable')} {row.indeterminateFields.join(', ')}
        </span>
      )}
    </>
  );
}

function Comparability({ value }: { readonly value: string }) {
  const state: StateName = value === 'exact' ? 'PASS' : value === 'partial' ? 'REVIEW' : 'UNKNOWN';
  return <StateBadge state={state} label={value} />;
}

function sectionAddition(row: SectionChangeRowDto): ContributorDto {
  return {
    key: row.key,
    changeKind: row.changeKind,
    delta: null,
    bytes: row.memorySize.target ?? row.memorySize.base,
  };
}

function symbolAddition(row: SymbolChangeRowDto): ContributorDto {
  return {
    key: row.name,
    changeKind: row.changeKind,
    delta: null,
    bytes: row.size.target ?? row.size.base,
  };
}
