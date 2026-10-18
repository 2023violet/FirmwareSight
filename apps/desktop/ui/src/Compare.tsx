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
 *    (DESIGN.md 5; prompt §23, §38). An unpaired row is never a Changed row (R3, D-R3-2).
 * 3. **The full lists are the answer; the ranking is a way to them.** Top growth and largest
 *    additions are capped navigation aids, and each entry is a control that opens the table that holds
 *    its row, filtered to it (prompt §25, §26, §41).
 * 4. **A failed comparison does not delete the last good one.** The summary moves only when the
 *    shell returns one, and the surviving report says which pair of builds it belongs to and which
 *    pair it does not (prompt §42).
 * 5. **A comparison outlives the page that computed it.** The pair and the result are session facts,
 *    so the shell holds them and navigating to Overview and back neither loses the answer nor recomputes
 *    it. The diff handle that answers for it lives in this process only, which is why a fresh process
 *    starts with no comparison at all (U1P-R3 F1).
 *
 * R3 (`06_DELIVERY/17`, ADR-0034) gives the page the Instrument Ledger anatomy: an Export menu, the pair
 * selector, a SubjectBar naming the pair on screen, one answer (the Flash change), four views, and one
 * change table shared by sections, symbols and objects, each row of which opens in the Inspector.
 */

import { useCallback, useEffect, useLayoutEffect, useRef, useState, type ReactNode } from 'react';

import styles from './Compare.module.css';
import { BuildPicker } from './components/BuildPicker';
import { Button } from './components/Button';
import { DataTable, FilterBar, type Column, type SortState } from './components/DataTable';
import { DeltaValue } from './components/DeltaValue';
import { ErrorPanel } from './components/ErrorPanel';
import { ExportMenu } from './components/ExportMenu';
import { FactValue, type FactEvidence } from './components/FactValue';
import { Inspector } from './components/Inspector';
import { Page } from './components/Layout';
import { LeadAnswer } from './components/LeadAnswer';
import { Notice } from './components/Notice';
import { PageHeader } from './components/PageHeader';
import { EmptyState, Panel } from './components/Panel';
import { Pager } from './components/Table';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import { StateBadge, type StateName } from './components/StateBadge';
import { SubjectBar, type Subject, type SubjectFreshness } from './components/SubjectBar';
import { Tabs, tabPanelProps } from './components/Tabs';
import { evidenceBasisCaption } from './evidenceBasis';
import { ABSENT, formatSize, truncateMiddle, type SizeUnit } from './format';
import { useI18n, type Translate } from './i18n';
import type { Catalog } from './i18n/en';
import {
  compareSnapshots,
  exportCompareHtml,
  exportCompareJson,
  exportCompareMarkdown,
  exportCompareVisual,
  listCompareCandidates,
  pinBaseline,
  queryObjectChanges,
  querySectionChanges,
  querySymbolChanges,
  type IpcOutcome,
} from './ipc/bridge';
import type {
  BaselinePinDto,
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
  ObjectChangeCountsDto,
  ObjectChangeRowDto,
  ObjectChangeSortDto,
  SectionChangeRowDto,
  SectionRankingExclusionsDto,
  SectionChangeSortDto,
  SortDirDto,
  SymbolChangeRowDto,
  SymbolChangeSortDto,
} from './ipc/types';
import { cx } from './styles/classnames';

type CompareKey = keyof Catalog['compare'];

/**
 * The symbol additions are one small page of the authoritative change table, ranked by the shell on
 * the same quantity the growth lists use. They are not a second copy of the diff (prompt §25). The
 * section additions are Core's own list (`summary.addedSections`): sorting the table would recommend
 * host metadata and unpaired names (NEXT-CORE-01).
 */
const ADDITION_LIMIT = 5;

/** Which side of a change row is being shown: only the unpaired side can be genuinely absent. */
type Side = 'base' | 'target';

/** A drill-down request: the row the reader chose, in the table that holds it. */
type Focus = { readonly key: string; readonly libraries?: boolean };

/** The largest additions, read back from the change tables. */
type Additions = {
  readonly sections: readonly ContributorDto[];
  readonly symbols: readonly ContributorDto[];
};

/** The four views (ADR-0034 D2, spec §6.3). */
type View = 'summary' | 'sections' | 'symbols' | 'objects';
const VIEWS: readonly View[] = ['summary', 'sections', 'symbols', 'objects'];
const VIEW_LABEL: Readonly<Record<View, CompareKey>> = {
  summary: 'views.summary',
  sections: 'views.sections',
  symbols: 'views.symbols',
  objects: 'views.objects',
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

type ExportFormat = 'json' | 'html' | 'png' | 'svg' | 'visual-html' | 'markdown';

export function Compare({
  unit,
  onUnitChange,
  lastAnalyzedSnapshotId,
  session,
  onSessionChange,
  onGoToAnalyze,
  pin = null,
  onPinChange,
  autoCompare = false,
  onAutoCompareStarted,
}: {
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
  readonly lastAnalyzedSnapshotId: string | null;
  readonly session: ComparisonSession;
  readonly onSessionChange: (
    next: ComparisonSession | ((previous: ComparisonSession) => ComparisonSession),
  ) => void;
  readonly onGoToAnalyze: () => void;
  /** The project's pinned baseline (R5 / D3): the default base, marked wherever a build is named. */
  readonly pin?: BaselinePinDto | null;
  readonly onPinChange?: (pin: BaselinePinDto | null) => void;
  /** Another page named the pair and asked for it to be compared once (D-R5-4). */
  readonly autoCompare?: boolean;
  readonly onAutoCompareStarted?: () => void;
}) {
  const { t } = useI18n();
  const [candidates, setCandidates] = useState<CandidatePageDto | null>(null);
  const [pinError, setPinError] = useState<ErrorEnvelopeDto | null>(null);
  const [pinning, setPinning] = useState(false);
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
  const [view, setView] = useState<View>('summary');
  const [sectionFocus, setSectionFocus] = useState<Focus | null>(null);
  const [symbolFocus, setSymbolFocus] = useState<Focus | null>(null);
  const [additions, setAdditions] = useState<Additions | null>(null);
  const [exportNote, setExportNote] = useState<string | null>(null);
  // Not reset by the comparison-change effect: a 6002 answer that lands before the new comparison renders
  // must survive that render (R4 review S2-3). The next export clears it.
  const [exportStale, setExportStale] = useState(false);
  const [exportError, setExportError] = useState<ErrorEnvelopeDto | null>(null);
  const [exporting, setExporting] = useState(false);
  /** The fact in the Inspector, with the control that opened it so focus can go back there. */
  const [inspected, setInspected] = useState<{
    readonly evidence: FactEvidence;
    readonly trigger: HTMLElement;
  } | null>(null);
  const inspect = useCallback((evidence: FactEvidence, trigger: HTMLElement) => {
    setInspected({ evidence, trigger });
  }, []);
  const closeInspector = useCallback(() => {
    setInspected(null);
  }, []);

  const exportRequest = useRef(0);
  // A layout effect, so the counter moves in the same commit that shows the new comparison (CI #123): a late
  // Save-dialog answer can never be read as this comparison's.
  useLayoutEffect(() => {
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
    // The pinned baseline is the measurement reference, so it is the default base whenever it is a different
    // build from the target (D-R5-3); otherwise the next distinct build is.
    // It is used even when it is not on the loaded page of candidates (R5 review S2-2): the comparison needs
    // only its id, and the picker names it from the pin itself.
    const pinnedId = pin === null || pin.snapshotId === preferred.snapshotId ? null : pin.snapshotId;
    const other = candidates.rows.find((row) => row.snapshotId !== preferred.snapshotId) ?? null;
    setPair({
      baseId: pinnedId ?? (other === null ? null : other.snapshotId),
      targetId: preferred.snapshotId,
    });
  }, [baseId, candidates, lastAnalyzedSnapshotId, pin, setPair, targetId]);

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

  // A pair another page named is compared once, as soon as both sides are named (D-R5-4).
  useEffect(() => {
    if (!autoCompare || baseId === null || targetId === null) {
      return;
    }
    onAutoCompareStarted?.();
    void compare();
  }, [autoCompare, baseId, targetId, compare, onAutoCompareStarted]);

  const pinBase = useCallback(async () => {
    if (baseId === null) {
      return;
    }
    setPinning(true);
    const outcome = await pinBaseline(baseId);
    setPinning(false);
    if (!outcome.ok) {
      setPinError(outcome.envelope);
      return;
    }
    setPinError(null);
    onPinChange?.(outcome.value);
  }, [baseId, onPinChange]);

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
    const sections = summary.addedSections;
    void querySymbolChanges({
      diffId,
      filter: null,
      changeKind: 'added',
      sort: 'size',
      direction: 'desc',
      offset: 0,
      limit: ADDITION_LIMIT,
    }).then((symbols) => {
      if (additionsRequest.current !== id) {
        return;
      }
      if (!symbols.ok) {
        setAdditions(null);
        return;
      }
      setAdditions({ sections, symbols: symbols.value.rows.map(symbolAddition) });
    });
  }, [summary]);

  const exportAs = useCallback(
    async (format: ExportFormat) => {
      if (summary === null) {
        return;
      }
      const request = ++exportRequest.current;
      setExporting(true);
      setExportStale(false);
      const outcome = format === 'json'
        ? await exportCompareJson(summary.diffId)
        : format === 'markdown'
          ? await exportCompareMarkdown(summary.diffId)
        : format === 'html'
          ? await exportCompareHtml(summary.diffId)
          : await exportCompareVisual(summary.diffId, format === 'visual-html' ? 'html' : format);
      const staleAnswer = !outcome.ok && outcome.envelope.code === 'ERR-EXPORT-6002';
      if (request !== exportRequest.current) {
        // The comparison changed while the Save dialog was open. Rust refuses that export rather than
        // writing the comparison the reader moved away from (M1, AC24); the screen says so instead of
        // staying silent. Any other late outcome belongs to the old comparison and is not shown here.
        if (staleAnswer) {
          setExportStale(true);
        }
        return;
      }
      setExporting(false);
      if (staleAnswer) {
        // Rust is the authority that the comparison changed, even when its answer arrives before this page
        // has rendered the change (CI #123): it is the stale note, never an error.
        setExportError(null);
        setExportNote(null);
        setExportStale(true);
        return;
      }

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
  // Builds this page can name although they are not on the loaded page of candidates: the pin, and the two
  // sides of the comparison on screen (R5 review S2-2). Each is named by file and digest only.
  const known: readonly KnownBuild[] = [
    ...(pin === null ? [] : [{ id: pin.snapshotId, fileName: pin.fileName, sha256: pin.sha256, meta: t('compare', 'picker.knownPinned', { at: pin.pinnedAt }) }]),
    ...(summary === null
      ? []
      : [summary.base, summary.target].map((side) => ({
          id: side.snapshotId,
          fileName: side.fileName,
          sha256: side.sha256,
          meta: t('compare', 'picker.knownCompared'),
        }))),
  ];
  const base = rows.find((row) => row.snapshotId === baseId) ?? null;
  // The ids decide, not the loaded page: a pinned or compared build off the page is still a chosen build.
  const bothChosen = baseId !== null && targetId !== null;
  const samePair = bothChosen && baseId === targetId;
  const hasTwoBuilds = (candidates?.total ?? 0) >= 2;
  // The report answers for the pair it was computed from, not for the pair the selectors happen to
  // name now. A swap or a failed attempt leaves the previous diff standing, so the screen says whose
  // facts the reader is looking at (prompt §18).
  const moved =
    summary !== null &&
    (baseId !== summary.base.snapshotId || targetId !== summary.target.snapshotId);
  const stale = summary !== null && (error !== null || moved);
  const freshness: SubjectFreshness = error !== null ? 'compareFailed' : moved ? 'pairMoved' : 'latest';

  // Evidence belongs to one comparison, read as current or as retained, in one view: a change of any of the
  // three closes the Inspector rather than leave a fact beside tables that no longer describe it (R2 S2-2).
  const shownDiff = summary?.diffId ?? null;
  useEffect(() => {
    setInspected(null);
  }, [shownDiff, stale, view, unit]);

  /** A ranking pick opens the table that holds the row, filtered to it (prompt §25, now across views). */
  const pickSection = useCallback((key: string) => {
    setView('sections');
    setSectionFocus({ key });
  }, []);
  const pickSymbol = useCallback((key: string) => {
    setView('symbols');
    setSymbolFocus({ key });
  }, []);

  return (
    <div className={styles['withInspector']}>
      <Page>
        <PageHeader
          title={t('compare', 'title')}
          subhead={t('compare', 'subhead')}
          actions={
            summary === null ? undefined : (
              <ExportMenu<ExportFormat>
                label={t('compare', 'export.menu')}
                regionLabel={t('compare', 'export.region')}
                hint={t('compare', 'export.hint')}
                items={[
                  { key: 'json', label: t('compare', 'export.json') },
                  { key: 'html', label: t('compare', 'export.html') },
                  { key: 'png', label: t('compare', 'export.png') },
                  { key: 'svg', label: t('compare', 'export.svg') },
                  { key: 'visual-html', label: t('compare', 'export.printHtml') },
                  { key: 'markdown', label: t('compare', 'export.markdown') },
                ]}
                disabled={exporting}
                onExport={(format) => {
                  void exportAs(format);
                }}
              />
            )
          }
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
            nextStep={<Button variant="primary" onClick={onGoToAnalyze}>{t('compare', 'empty.action')}</Button>}
          />
        )}

        {candidates === null || !hasTwoBuilds ? null : (
          <section className={styles['selectors']} aria-label={t('compare', 'selectors.region')}>
            <PairSide
              label={t('compare', 'picker.baseLabel')}
              rows={rows}
              known={known}
              pinnedId={pin?.snapshotId ?? null}
              selected={baseId}
              lastAnalyzedSnapshotId={lastAnalyzedSnapshotId}
              unit={unit}
              disabled={comparing}
              onSelect={setBaseId}
            />
            <div className={styles['swap']}>
              <Button onClick={swap} disabled={!bothChosen || samePair || comparing}>
                {t('compare', 'picker.swap')}
              </Button>
            </div>
            <PairSide
              label={t('compare', 'picker.targetLabel')}
              rows={rows}
              known={known}
              pinnedId={pin?.snapshotId ?? null}
              selected={targetId}
              lastAnalyzedSnapshotId={lastAnalyzedSnapshotId}
              unit={unit}
              disabled={comparing}
              onSelect={setTargetId}
            />
            <div className={styles['pickerActions']}>
              {onPinChange === undefined || baseId === null || baseId === pin?.snapshotId ? null : (
                <Button
                  onClick={() => {
                    void pinBase();
                  }}
                  disabled={pinning || comparing}
                >
                  {t('compare', 'picker.pinBase')}
                </Button>
              )}
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

        {pinError === null ? null : (
          <ErrorPanel envelope={pinError} label={t('compare', 'pin.errorLabel')} heading={t('compare', 'pin.errorHeading')} />
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

        {exporting ? (
          <p className={styles['status']} role="status" aria-live="polite">
            {t('compare', 'export.waiting')}
          </p>
        ) : null}
        {exportNote === null ? null : (
          <p className={styles['status']} role="status" aria-live="polite">
            {exportNote}
          </p>
        )}
        {exportStale ? (
          <p className={styles['status']} role="status" aria-live="polite">
            {t('compare', 'export.staleNote')}
          </p>
        ) : null}
        {exportError === null ? null : (
          <ErrorPanel envelope={exportError} label={t('compare', 'export.errorLabel')} heading={t('compare', 'export.errorHeading')} />
        )}

        {/* Two stored builds and no comparison is a real fact about the session - there is a pair to compare and
            no result yet - so it gets the words for that fact (U1P-R3 F1). It claims no numbers, no delta and no
            verdict, and it does not replace the Compare action it describes. */}
        {candidates === null || !hasTwoBuilds || summary !== null || comparing ? null : (
          <Panel title={t('compare', 'ready.title')}>
            <p className={styles['readyMessage']}>{t('compare', 'ready.message')}</p>
          </Panel>
        )}

        {/* Before there is a result the unit governs the recorded figures beside the two selectors, so the
            switch sits under them; once there is one it moves into the SubjectBar, with the pair it re-labels.
            Exactly one switch is on screen at a time. */}
        {summary === null && hasTwoBuilds ? <SizeUnitSwitch unit={unit} onSelect={onUnitChange} /> : null}

        {summary === null ? null : (
          <>
            <SubjectBar
              subject={pairSubject(summary, pin?.snapshotId ?? null)}
              freshness={freshness}
              actions={<SizeUnitSwitch unit={unit} onSelect={onUnitChange} />}
            />
            <section
              className={styles['report']}
              aria-label={stale ? t('compare', 'report.regionStale') : t('compare', 'report.regionFresh')}
            >
              {stale ? (
                <Notice tone="stale" level={2} title={t('compare', 'report.staleTitle')}>
                  <p role="note">
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
                </Notice>
              ) : null}

              <CompareLead
                summary={summary}
                retained={stale}
                unit={unit}
                inspecting={inspected?.evidence ?? null}
                onInspect={inspect}
              />

              <Tabs
                idBase="fs-compare"
                label={t('compare', 'views.label')}
                items={VIEWS.map((key) => ({ key, label: t('compare', VIEW_LABEL[key]) }))}
                current={view}
                onSelect={setView}
              />
              <div {...tabPanelProps('fs-compare', view)} className={styles['panel']}>
                {view === 'summary' ? (
                  <>
                    <WhatMoved counts={summary.counts} />
                    <MemoryComparison memory={summary.memory} unit={unit} />
                    <Ranking
                      summary={summary}
                      additions={additions}
                      unit={unit}
                      onPickSection={pickSection}
                      onPickSymbol={pickSymbol}
                    />
                    <EvidenceNotice summary={summary} />
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
                      <span>{t('compare', 'footer.diffIdNote')}</span>
                      <span>{t('compare', 'footer.storageNote')}</span>
                    </footer>
                  </>
                ) : view === 'sections' ? (
                  <SectionChanges
                    diffId={summary.diffId}
                    unit={unit}
                    focus={sectionFocus}
                    onFocusConsumed={() => {
                      setSectionFocus(null);
                    }}
                    onInspect={inspect}
                    onReadingChange={closeInspector}
                  />
                ) : view === 'symbols' ? (
                  <SymbolChanges
                    diffId={summary.diffId}
                    unit={unit}
                    focus={symbolFocus}
                    onFocusConsumed={() => {
                      setSymbolFocus(null);
                    }}
                    onInspect={inspect}
                    onReadingChange={closeInspector}
                  />
                ) : (
                  <ObjectChanges summary={summary} unit={unit} onInspect={inspect} onReadingChange={closeInspector} />
                )}
              </div>
            </section>
          </>
        )}
      </Page>
      <Inspector
        evidence={inspected?.evidence ?? null}
        returnFocus={inspected?.trigger ?? null}
        onClose={closeInspector}
      />
    </div>
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

/** The pair the SubjectBar names: the comparison on screen, never the pickers' next one. */
function pairSubject(summary: CompareSummaryDto, pinnedId: string | null): Subject {
  return {
    kind: 'pair',
    base: {
      fileName: summary.base.fileName,
      sha256: summary.base.sha256,
      map: summary.memory.base.mapBacked,
      pinned: summary.base.snapshotId === pinnedId,
    },
    target: {
      fileName: summary.target.fileName,
      sha256: summary.target.sha256,
      map: summary.memory.target.mapBacked,
      pinned: summary.target.snapshotId === pinnedId,
    },
  };
}

/** One selector, with the recorded facts of the build it currently names. */
/** A build named by file and digest, for a picker that does not hold its candidate row. */
interface KnownBuild {
  readonly id: string;
  readonly fileName: string;
  readonly sha256: string;
  readonly meta: string;
}

function PairSide({
  label,
  rows,
  known,
  pinnedId,
  selected,
  lastAnalyzedSnapshotId,
  unit,
  disabled,
  onSelect,
}: {
  readonly label: string;
  readonly rows: readonly CompareCandidateDto[];
  readonly known: readonly KnownBuild[];
  readonly pinnedId: string | null;
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
      <BuildPicker
        label={label}
        options={rows.map((row) => ({
          id: row.snapshotId,
          fileName: row.fileName,
          sha256: row.sha256,
          importedAt: row.snapshotId === lastAnalyzedSnapshotId
            ? `${row.importedAt}${t('compare', 'picker.lastAnalyzedSuffix')}`
            : row.importedAt,
          pinned: row.snapshotId === pinnedId,
        })).concat(
          known
            .filter((build, index, all) => !rows.some((row) => row.snapshotId === build.id) && all.findIndex((other) => other.id === build.id) === index)
            .map((build) => ({
              id: build.id,
              fileName: build.fileName,
              sha256: build.sha256,
              importedAt: '',
              meta: build.meta,
              pinned: build.id === pinnedId,
            })),
        )}
        value={selected}
        disabled={disabled}
        onChange={onSelect}
      />
      {chosen === null ? (
        <p className={styles['sideFacts']}>
          {selected !== null && known.some((build) => build.id === selected)
            ? t('compare', 'picker.knownFacts')
            : t('compare', 'picker.noBuildNamed')}
        </p>
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

/** A delta as text, neutral: ▲ or ▼ and the magnitude, or Unknown. Used where a figure must be one string. */
function deltaText(delta: number | null, unit: SizeUnit, t: Translate): string {
  if (delta === null) {
    return t('compare', 'lead.unknown');
  }
  if (delta === 0) {
    return formatSize(0, unit);
  }
  return `${delta > 0 ? '▲' : '▼'}${formatSize(Math.abs(delta), unit)}`;
}

/**
 * The page's one answer: the Flash change, as a value the reader can trace, measured against the two sides,
 * the percentage, RAM, comparability and the largest growth source (spec §6.3).
 */
function CompareLead({
  summary,
  retained,
  unit,
  inspecting,
  onInspect,
}: {
  readonly summary: CompareSummaryDto;
  readonly retained: boolean;
  readonly unit: SizeUnit;
  readonly inspecting: FactEvidence | null;
  readonly onInspect: (evidence: FactEvidence, trigger: HTMLElement) => void;
}) {
  const { t } = useI18n();
  const { memory } = summary;
  const flash = memory.nonvolatile;
  // The largest growth source: an object when the MAP attributes one (spec §6.3), else Core's top section.
  // Which sections count is Core's ranking; this page does not second-guess it (D-R3-1).
  const topObject = summary.objectChanges.available ? (summary.objectChanges.topObjects[0] ?? null) : null;
  const topSection = summary.topSections[0] ?? null;
  const evidence: FactEvidence = {
    field: 'nonvolatile delta',
    value: deltaText(flash.delta, unit, t),
    classification: 'derived',
    source: 'diff',
    rule: t('compare', 'lead.rule', { comparability: memory.comparability }),
    raw: `${flash.base ?? ABSENT} → ${flash.target ?? ABSENT} (${flash.delta ?? t('compare', 'lead.unknown')})`,
    details: [
      [t('compare', 'axis.old'), formatSize(flash.base, unit)],
      [t('compare', 'axis.new'), formatSize(flash.target, unit)],
      ...(flash.reason === null ? [] : [[t('compare', 'lead.reason'), flash.reason] as const]),
      [t('compare', 'memory.comparabilityTerm'), memory.comparability],
    ],
  };
  const reference = [
    `${formatSize(flash.base, unit)} → ${formatSize(flash.target, unit)}`,
    memory.nonvolatilePercent.value ?? percentStatus(memory.nonvolatilePercent.status, t),
    `RAM ${deltaText(memory.runtimeRam.delta, unit, t)}`,
    t('compare', 'lead.comparability', { value: memory.comparability }),
    ...(topObject !== null && topObject.delta !== null
      ? [t('compare', 'lead.topObject', { key: topObject.key, delta: deltaText(topObject.delta, unit, t) })]
      : topSection === null || topSection.delta === null
        ? []
        : [t('compare', 'lead.topSource', { key: topSection.key, delta: deltaText(topSection.delta, unit, t) })]),
  ].join(' · ');
  return (
    <LeadAnswer
      sentence={t('compare', retained ? 'lead.flashRetained' : 'lead.flash')}
      figure={
        <FactValue evidence={evidence} selected={inspecting?.field === evidence.field} onInspect={onInspect} />
      }
      state={
        flash.delta === null ? <StateBadge state="UNKNOWN" label={t('compare', 'lead.unknown')} /> : undefined
      }
      reference={reference}
    />
  );
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

/** One number with the word that says which axis it belongs to. Side values are never coloured. */
function Figure({ axis, text }: { readonly axis: string; readonly text: string }) {
  return (
    <span className={styles['figure']}>
      {/* The axis word carries its own spacing, so the pair reads as one sentence in the DOM and not
          only as two columns on the screen. */}
      <span className={styles['axis']}>{`${axis} `}</span>
      <span className={styles['mono']}>{text}</span>
    </span>
  );
}

/**
 * The change itself: neutral, ▲ or ▼ in primary text (ADR-0034 D1.4, D-R3-3), or Unknown with Core's reason.
 * An unpaired or one-sided row has a null delta, and that is Unknown or absent - never 0.
 */
function DeltaFigure({
  change,
  unit,
}: {
  readonly change: ByteDeltaDto;
  readonly unit: SizeUnit;
}) {
  const { t } = useI18n();
  return (
    <span className={styles['figure']}>
      <span className={styles['axis']}>{`${t('compare', 'axis.delta')} `}</span>
      <DeltaCell change={change} unit={unit} />
    </span>
  );
}

/** A delta cell: the neutral delta, or Unknown with Core's reason when the delta could not be computed. */
function DeltaCell({ change, unit }: { readonly change: ByteDeltaDto; readonly unit: SizeUnit }) {
  return change.delta === null ? (
    <span className={styles['unknown']}>
      Unknown
      {change.reason === null ? null : <span className={styles['note']}> {change.reason}</span>}
    </span>
  ) : (
    <DeltaValue delta={change.delta} format={(bytes) => formatSize(bytes, unit)} />
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
 */
function WhatMoved({ counts }: { readonly counts: DiffCountsDto }) {
  const { t } = useI18n();
  return (
    <section className={styles['section']} aria-label={t('compare', 'moved.region')}>
      <h2>{t('compare', 'moved.heading')}</h2>
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
      <SectionsLeftOut counts={summary.sectionRankingExclusions} />
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

/**
 * What Core's section lists leave out, and why (NEXT-CORE-01). Said only when something was left out, so
 * a section missing from a list never reads as "it did not change": every such row is in the Sections
 * view.
 */
function SectionsLeftOut({ counts }: { readonly counts: SectionRankingExclusionsDto }) {
  const { t } = useI18n();
  const total = counts.hostMetadata + counts.notLoaded + counts.undecided + counts.unpairedAdditions;
  if (total === 0) {
    return null;
  }
  return (
    <p className={styles['hint']}>
      {t('compare', 'ranking.sectionsLeftOut', {
        hostMetadata: counts.hostMetadata,
        notLoaded: counts.notLoaded,
        undecided: counts.undecided,
        unpaired: counts.unpairedAdditions,
      })}
    </p>
  );
}

type ContributorNoun =
  | 'contributors.sections'
  | 'contributors.symbols'
  | 'contributors.objects'
  | 'contributors.libraries';

function Contributors({
  nounKey,
  rows,
  unit,
  emptyKey,
  onPick,
}: {
  readonly nounKey: ContributorNoun;
  readonly rows: readonly ContributorDto[];
  readonly unit: SizeUnit;
  readonly emptyKey:
    | 'contributors.emptySectionsGrowth'
    | 'contributors.emptySymbolsGrowth'
    | 'contributors.emptySectionsAdded'
    | 'contributors.emptySymbolsAdded'
    | 'contributors.emptyObjectsGrowth'
    | 'contributors.emptyLibrariesGrowth'
    | 'contributors.emptyObjectsAdded';
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
                  {/* A growth row shows its delta; an addition has no delta to show, so its size stands in,
                      labelled by the list it is in. */}
                  {row.delta === null ? (
                    formatSize(row.bytes, unit)
                  ) : (
                    <DeltaValue delta={row.delta} format={(bytes) => formatSize(bytes, unit)} />
                  )}
                </span>
              </button>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}

/** A page of change rows, as every change query answers. */
interface ChangePage<Row> {
  readonly rows: readonly Row[];
  readonly total: number;
  readonly offset: number;
  readonly nextOffset: number | null;
}

/** What one change query needs from the table: the reader's choices, in the shell's words. */
interface ChangeQuery<Sort extends string> {
  readonly filter: string | null;
  readonly changeKind: ChangeKindFilterDto | null;
  readonly sort: Sort;
  readonly direction: SortDirDto;
  readonly offset: number;
}

/** A row with its place on the page, so a key can use the index where the diff repeats a name. */
interface Indexed<Row> {
  readonly row: Row;
  readonly index: number;
}

/** Lift columns written for a bare row to the indexed entries the change table renders. */
function lift<Row>(columns: readonly Column<Row>[]): Column<Indexed<Row>>[] {
  // A change column never sorts locally (`sortValue`); the shell orders the rows, so only the server-mode
  // fields are carried across.
  return columns.map((column) => ({
    key: column.key,
    header: column.header,
    cell: (entry: Indexed<Row>) => column.cell(entry.row),
    ...(column.numeric === undefined ? {} : { numeric: column.numeric }),
    ...(column.sortable === undefined ? {} : { sortable: column.sortable }),
    ...(column.firstDirection === undefined ? {} : { firstDirection: column.firstDirection }),
  }));
}

/**
 * The one change table (D-R3-4): filters, sorts and pages by asking the shell, and shows the page it gets.
 *
 * Sections, symbols and objects used to be three hand-written copies of this; each now names its columns and
 * its query and nothing else. The reading starts clean whenever `resetKey` changes (a new comparison, or
 * objects ↔ libraries), and a `focus` filters the whole table to one row, which is how a ranking pick lands.
 */
function ChangeTable<Row, Sort extends string>({
  resetKey,
  noun,
  regionLabel,
  heading,
  caption,
  pagerLabel,
  loadingLabel,
  errorLabel,
  errorHeading,
  initialSort,
  initialDirection,
  columns,
  rowKey,
  rowHeader,
  rowName,
  query,
  focus,
  onFocusConsumed,
  detail,
  onInspect,
  onReadingChange,
  before,
  extra,
}: {
  readonly resetKey: string;
  readonly noun: string;
  readonly regionLabel: string;
  readonly heading: string;
  readonly caption: string;
  readonly pagerLabel: string;
  readonly loadingLabel?: string;
  readonly errorLabel: string;
  readonly errorHeading: string;
  readonly initialSort: Sort;
  readonly initialDirection: SortDirDto;
  readonly columns: readonly Column<Row>[];
  readonly rowKey: (row: Row, index: number) => string;
  readonly rowHeader: string;
  readonly rowName: (row: Row) => string;
  readonly query: (request: ChangeQuery<Sort>) => Promise<IpcOutcome<ChangePage<Row>>>;
  readonly focus: Focus | null;
  /**
   * Called once the pick has been applied, so the page forgets it (R3 review S2-1): a table mounts each time
   * its view opens, and a remembered pick would re-filter it and pull focus out of the tab list every time.
   */
  readonly onFocusConsumed: () => void;
  readonly detail: (row: Row) => FactEvidence;
  readonly onInspect: (evidence: FactEvidence, trigger: HTMLElement) => void;
  /** Called whenever the rows on screen are about to change, so a row's detail never outlives its row. */
  readonly onReadingChange: () => void;
  /** Content between the heading and the filters, such as the object rankings. */
  readonly before?: ReactNode;
  /** Controls beside the filter, such as Objects | Libraries. */
  readonly extra?: ReactNode;
}) {
  const { t } = useI18n();
  const [offset, setOffset] = useState(0);
  const [filter, setFilter] = useState<string | null>(null);
  const [draft, setDraft] = useState('');
  const [kind, setKind] = useState<ChangeKindFilterDto | null>(null);
  const [sort, setSort] = useState<Sort>(initialSort);
  const [direction, setDirection] = useState<SortDirDto>(initialDirection);
  const [page, setPage] = useState<ChangePage<Row> | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [loading, setLoading] = useState(false);
  const request = useRef(0);
  const area = useRef<HTMLElement | null>(null);
  const [seenKey, setSeenKey] = useState(resetKey);

  // A new comparison must not inherit the previous pair's filter. React re-renders before it commits, so no
  // request goes out with the old name still in the box.
  if (seenKey !== resetKey) {
    setSeenKey(resetKey);
    setFilter(null);
    setDraft('');
    setKind(null);
    setOffset(0);
    setSort(initialSort);
    setDirection(initialDirection);
  }

  // A drill-down names the row the reader wants and moves the keyboard here; the filter travels to the shell
  // like any other, so the row is filtered from the whole table, not from one page.
  useEffect(() => {
    if (focus === null) {
      return;
    }
    setFilter(focus.key);
    setDraft(focus.key);
    setKind(null);
    setOffset(0);
    area.current?.focus();
    latestFocusConsumed.current();
  }, [focus]);

  // The callbacks are rebuilt by the parent on every render; the request is a function of the reader's
  // choices, so only those (and the reset key) decide when to ask again. Kept current in a layout effect,
  // not during render, so a render React discards can never leave a stale one behind.
  const latestQuery = useRef(query);
  const latestFocusConsumed = useRef(onFocusConsumed);
  const latestReadingChange = useRef(onReadingChange);
  useLayoutEffect(() => {
    latestQuery.current = query;
    latestFocusConsumed.current = onFocusConsumed;
    latestReadingChange.current = onReadingChange;
  });
  useEffect(() => {
    latestReadingChange.current();
    const id = ++request.current;
    setLoading(true);
    setPage(null);
    void latestQuery.current({ filter, changeKind: kind, sort, direction, offset }).then((outcome) => {
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
  }, [resetKey, filter, kind, sort, direction, offset]);

  const onSort = (next: SortState) => {
    setOffset(0);
    setSort(next.key as Sort);
    setDirection(next.direction);
  };

  return (
    <section className={styles['area']} ref={area} tabIndex={-1} aria-label={regionLabel}>
      <h2>{heading}</h2>
      {before}
      <FilterBar
        label={t('compare', 'filter.byName', { noun })}
        formLabel={t('compare', 'filter.formAria', { noun })}
        applyLabel={t('compare', 'filter.apply')}
        value={draft}
        onChange={setDraft}
        onApply={(value) => {
          setDraft(value);
          setFilter(value.length === 0 ? null : value);
          setOffset(0);
        }}
      >
        <span className={styles['radioGroup']} role="group" aria-label={t('compare', 'filter.groupAria', { noun })}>
          {KIND_FILTERS.map((entry) => (
            <span key={entry.token} className={styles['radio']}>
              <input
                id={`fs-change-${resetKey}-${entry.token}`}
                type="radio"
                name={`fs-change-${resetKey}`}
                checked={kind === entry.value}
                onChange={() => {
                  setKind(entry.value);
                  setOffset(0);
                }}
              />
              <label htmlFor={`fs-change-${resetKey}-${entry.token}`}>{t('compare', entry.key)}</label>
            </span>
          ))}
        </span>
        {extra}
      </FilterBar>
      {loading && loadingLabel !== undefined ? (
        <p className={styles['status']} role="status" aria-live="polite">
          {loadingLabel}
        </p>
      ) : null}
      {error === null ? null : <ErrorPanel envelope={error} label={errorLabel} heading={errorHeading} />}

      {page === null ? null : (
        <>
          <DataTable<Indexed<Row>>
            caption={caption}
            columns={lift(columns)}
            rows={page.rows.map((row, index) => ({ row, index }))}
            rowKey={(entry) => rowKey(entry.row, entry.index)}
            rowHeader={rowHeader}
            density="compact"
            sort={{ key: sort, direction }}
            onSort={onSort}
            onRowAction={(entry, trigger) => {
              onInspect(detail(entry.row), trigger);
            }}
            rowActionLabel={(entry) => t('compare', 'row.inspect', { name: rowName(entry.row) })}
          />
          <Pager
            total={page.total}
            offset={page.offset}
            shown={page.rows.length}
            nextOffset={page.nextOffset}
            label={pagerLabel}
            onOffset={setOffset}
          />
        </>
      )}
    </section>
  );
}

/** The change detail the Inspector shows for one row: its kind, its quantities, and what the diff could not decide. */
function changeDetail(
  t: Translate,
  unit: SizeUnit,
  diffId: string,
  name: string,
  kind: string,
  quantities: readonly (readonly [string, ByteDeltaDto])[],
  notes: readonly (readonly [string, string])[],
): FactEvidence {
  return {
    field: name,
    value: kind,
    classification: 'derived',
    source: 'diff',
    locator: diffId,
    rule: t('compare', 'row.rule'),
    details: [
      ...quantities.map(([label, change]): readonly [string, string] => [
        label,
        `${sideText(change, 'base', kind, unit)} → ${sideText(change, 'target', kind, unit)} · ${
          change.delta === null
            ? `Unknown${change.reason === null ? '' : ` (${change.reason})`}`
            : deltaText(change.delta, unit, t)
        }`,
      ]),
      ...notes,
    ],
  };
}

/** One side of a change row as text, under the same rule `SideValue` draws. */
function sideText(change: ByteDeltaDto, side: Side, kind: string, unit: SizeUnit): string {
  const value = side === 'base' ? change.base : change.target;
  if (value !== null) {
    return formatSize(value, unit);
  }
  return isAbsentSide(kind, side) ? ABSENT : 'Unknown';
}

/** The fields a row also differs in, and those the diff could not decide, as Inspector lines. */
function differsNotes(
  t: Translate,
  row: {
    readonly differingFields: readonly string[];
    readonly indeterminateFields: readonly string[];
    readonly ambiguous: boolean;
  },
  ambiguousNote: string,
): (readonly [string, string])[] {
  return [
    ...(row.differingFields.length === 0
      ? []
      : [[t('compare', 'col.alsoDiffers'), row.differingFields.join(', ')] as const]),
    ...(row.indeterminateFields.length === 0
      ? []
      : [[t('compare', 'differs.notDecidable'), row.indeterminateFields.join(', ')] as const]),
    ...(row.ambiguous ? [[t('compare', 'row.pairing'), ambiguousNote.trim()] as const] : []),
  ];
}

/** A size cell: one side of one quantity, under the absent/unknown rule. */
function sizeColumn<Row>(
  key: string,
  header: string,
  pick: (row: Row) => ByteDeltaDto,
  side: Side,
  kindOf: (row: Row) => string,
  unit: SizeUnit,
): Column<Row> {
  return {
    key,
    header,
    numeric: true,
    cell: (row) => <SideValue change={pick(row)} side={side} kind={kindOf(row)} unit={unit} />,
  };
}

/** A delta cell; sortable when the shell orders by it. */
function deltaColumn<Row>(
  key: string,
  header: string,
  pick: (row: Row) => ByteDeltaDto,
  unit: SizeUnit,
  sortable: boolean,
  firstDirection: 'asc' | 'desc' = 'asc',
): Column<Row> {
  return {
    key,
    header,
    numeric: true,
    sortable,
    firstDirection,
    cell: (row) => <DeltaCell change={pick(row)} unit={unit} />,
  };
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
  onFocusConsumed,
  onInspect,
  onReadingChange,
}: {
  readonly diffId: string;
  readonly unit: SizeUnit;
  readonly focus: Focus | null;
  readonly onFocusConsumed: () => void;
  readonly onInspect: (evidence: FactEvidence, trigger: HTMLElement) => void;
  readonly onReadingChange: () => void;
}) {
  const { t } = useI18n();
  const kindOf = (row: SectionChangeRowDto) => row.changeKind;
  const columns: Column<SectionChangeRowDto>[] = [
    { key: 'changeKind', header: t('compare', 'col.change'), sortable: true, firstDirection: 'asc', cell: (row) => <Kind kind={row.changeKind} /> },
    {
      key: 'key',
      header: t('compare', 'col.name'),
      sortable: true,
      firstDirection: 'asc',
      cell: (row) => (
        <span className={styles['wrap']}>
          {row.key}
          {row.nameKnown ? null : <span className={styles['note']}>{t('compare', 'section.noNameNote')}</span>}
          {row.ambiguous ? <span className={styles['note']}>{t('compare', 'section.ambiguousNote')}</span> : null}
        </span>
      ),
    },
    sizeColumn('oldFile', t('compare', 'col.oldFile'), (row) => row.fileSize, 'base', kindOf, unit),
    sizeColumn('newFile', t('compare', 'col.newFile'), (row) => row.fileSize, 'target', kindOf, unit),
    deltaColumn('delta', t('compare', 'col.deltaFile'), (row) => row.fileSize, unit, true),
    sizeColumn('oldRam', t('compare', 'col.oldRam'), (row) => row.memorySize, 'base', kindOf, unit),
    sizeColumn('newRam', t('compare', 'col.newRam'), (row) => row.memorySize, 'target', kindOf, unit),
    deltaColumn('deltaRam', t('compare', 'col.deltaRam'), (row) => row.memorySize, unit, false),
    { key: 'memorySize', header: t('compare', 'col.ramSize'), numeric: true, sortable: true, firstDirection: 'asc', cell: (row) => sizeText(row.memorySize, unit) },
    { key: 'fileSize', header: t('compare', 'col.fileSize'), numeric: true, sortable: true, firstDirection: 'asc', cell: (row) => sizeText(row.fileSize, unit) },
    { key: 'differs', header: t('compare', 'col.alsoDiffers'), cell: (row) => <Differs row={row} /> },
  ];
  return (
    <ChangeTable<SectionChangeRowDto, SectionChangeSortDto>
      resetKey={diffId}
      noun={t('compare', 'contributors.sections')}
      regionLabel={t('compare', 'section.region')}
      heading={t('compare', 'section.heading')}
      caption={t('compare', 'section.caption')}
      pagerLabel={t('compare', 'section.pagerLabel')}
      loadingLabel={t('compare', 'section.loading')}
      errorLabel={t('compare', 'section.errorLabel')}
      errorHeading={t('compare', 'section.errorHeading')}
      initialSort="key"
      initialDirection="asc"
      columns={columns}
      rowKey={(row, index) => `${row.key}:${String(index)}`}
      rowHeader="key"
      rowName={(row) => row.key}
      query={(request) => querySectionChanges({ diffId, ...request, limit: null })}
      focus={focus}
      onFocusConsumed={onFocusConsumed}
      onReadingChange={onReadingChange}
      detail={(row) =>
        changeDetail(
          t,
          unit,
          diffId,
          row.key,
          row.changeKind,
          [
            [t('compare', 'row.fileSize'), row.fileSize],
            [t('compare', 'row.memorySize'), row.memorySize],
          ],
          differsNotes(t, row, t('compare', 'section.ambiguousNote')),
        )
      }
      onInspect={onInspect}
    />
  );
}

/** The full symbol list, with the same filters and the fields a symbol row carries. */
function SymbolChanges({
  diffId,
  unit,
  focus,
  onFocusConsumed,
  onInspect,
  onReadingChange,
}: {
  readonly diffId: string;
  readonly unit: SizeUnit;
  readonly focus: Focus | null;
  readonly onFocusConsumed: () => void;
  readonly onInspect: (evidence: FactEvidence, trigger: HTMLElement) => void;
  readonly onReadingChange: () => void;
}) {
  const { t } = useI18n();
  const kindOf = (row: SymbolChangeRowDto) => row.changeKind;
  const columns: Column<SymbolChangeRowDto>[] = [
    { key: 'changeKind', header: t('compare', 'col.change'), sortable: true, firstDirection: 'asc', cell: (row) => <Kind kind={row.changeKind} /> },
    {
      key: 'name',
      header: t('compare', 'col.name'),
      sortable: true,
      firstDirection: 'asc',
      cell: (row) => (
        <span className={styles['wrap']}>
          {row.name}
          {row.ambiguous ? <span className={styles['note']}>{t('compare', 'symbol.ambiguousNote')}</span> : null}
        </span>
      ),
    },
    { key: 'kindBinding', header: t('compare', 'col.kindBinding'), cell: (row) => `${row.kind} / ${row.binding}` },
    { key: 'oldAddress', header: t('compare', 'col.oldAddress'), numeric: true, cell: (row) => sideAddress(row, 'base') },
    { key: 'newAddress', header: t('compare', 'col.newAddress'), numeric: true, cell: (row) => sideAddress(row, 'target') },
    sizeColumn('oldSize', t('compare', 'col.oldSize'), (row) => row.size, 'base', kindOf, unit),
    sizeColumn('newSize', t('compare', 'col.newSize'), (row) => row.size, 'target', kindOf, unit),
    deltaColumn('sizeDelta', t('compare', 'col.deltaSize'), (row) => row.size, unit, true),
    { key: 'size', header: t('compare', 'col.size'), numeric: true, sortable: true, firstDirection: 'asc', cell: (row) => sizeText(row.size, unit) },
    {
      key: 'section',
      header: t('compare', 'col.section'),
      cell: (row) => (
        <span className={styles['mono']}>
          {row.base === null ? ABSENT : row.base.sectionRef}
          {' → '}
          {row.target === null ? ABSENT : row.target.sectionRef}
        </span>
      ),
    },
    { key: 'differs', header: t('compare', 'col.alsoDiffers'), cell: (row) => <Differs row={row} /> },
  ];
  return (
    <ChangeTable<SymbolChangeRowDto, SymbolChangeSortDto>
      resetKey={diffId}
      noun={t('compare', 'contributors.symbols')}
      regionLabel={t('compare', 'symbol.region')}
      heading={t('compare', 'symbol.heading')}
      caption={t('compare', 'symbol.caption')}
      pagerLabel={t('compare', 'symbol.pagerLabel')}
      loadingLabel={t('compare', 'symbol.loading')}
      errorLabel={t('compare', 'symbol.errorLabel')}
      errorHeading={t('compare', 'symbol.errorHeading')}
      initialSort="name"
      initialDirection="asc"
      columns={columns}
      rowKey={(row, index) => `${row.name}:${String(index)}`}
      rowHeader="name"
      rowName={(row) => row.name}
      query={(request) => querySymbolChanges({ diffId, ...request, limit: null })}
      focus={focus}
      onFocusConsumed={onFocusConsumed}
      onReadingChange={onReadingChange}
      detail={(row) =>
        changeDetail(
          t,
          unit,
          diffId,
          row.name,
          row.changeKind,
          [[t('compare', 'row.size'), row.size]],
          differsNotes(t, row, t('compare', 'symbol.ambiguousNote')),
        )
      }
      onInspect={onInspect}
    />
  );
}

/**
 * ADR-0033 D8: which object files and libraries the change came from.
 *
 * Shown only when both builds stored GNU ld MAP object rows; otherwise this view says, in Core's words, why
 * there is no attribution (D-R3-7). The shortlist is Core's ranking and the table is the authoritative list,
 * paged by the shell like the section and symbol tables. An object present on one side only reads
 * "Not present" on the other, never 0.
 */
function ObjectChanges({
  summary,
  unit,
  onInspect,
  onReadingChange,
}: {
  readonly summary: CompareSummaryDto;
  readonly unit: SizeUnit;
  readonly onInspect: (evidence: FactEvidence, trigger: HTMLElement) => void;
  readonly onReadingChange: () => void;
}) {
  const { t } = useI18n();
  const attribution = summary.objectChanges;
  const diffId = summary.diffId;
  const [libraries, setLibraries] = useState(false);
  const [focus, setFocus] = useState<Focus | null>(null);
  const [seenDiffId, setSeenDiffId] = useState(diffId);

  if (seenDiffId !== diffId) {
    setSeenDiffId(diffId);
    setLibraries(false);
    setFocus(null);
  }

  if (!attribution.available) {
    return (
      <Notice tone="unknown" level={2} title={t('compare', 'objects.unavailableTitle')}>
        <p>{attribution.reason ?? t('compare', 'notice.attributionUnavailable')}</p>
      </Notice>
    );
  }

  const pick = (asLibrary: boolean) => (key: string) => {
    setLibraries(asLibrary);
    setFocus({ key, libraries: asLibrary });
  };
  const tally = (counts: ObjectChangeCountsDto) =>
    t('compare', 'objects.tally', {
      changed: String(counts.changed),
      added: String(counts.added),
      removed: String(counts.removed),
      unchanged: String(counts.unchanged),
    });
  const kindOf = (row: ObjectChangeRowDto) => row.changeKind;
  const columns: Column<ObjectChangeRowDto>[] = [
    { key: 'changeKind', header: t('compare', 'col.change'), sortable: true, firstDirection: 'asc', cell: (row) => <Kind kind={row.changeKind} /> },
    {
      key: 'key',
      header: t('compare', libraries ? 'col.library' : 'col.object'),
      sortable: true,
      firstDirection: 'asc',
      cell: (row) => (
        <span className={cx(styles['mono'], styles['wrap'])}>
          {row.key}
          {row.ambiguous ? <span className={styles['note']}>{t('compare', 'objects.ambiguousNote')}</span> : null}
        </span>
      ),
    },
    sizeColumn('oldFlash', t('compare', 'col.oldFlash'), (row) => row.nonvolatile, 'base', kindOf, unit),
    sizeColumn('newFlash', t('compare', 'col.newFlash'), (row) => row.nonvolatile, 'target', kindOf, unit),
    deltaColumn('flashDelta', t('compare', 'col.deltaFlash'), (row) => row.nonvolatile, unit, true, 'desc'),
    sizeColumn('oldRam', t('compare', 'col.oldRam'), (row) => row.runtimeRam, 'base', kindOf, unit),
    sizeColumn('newRam', t('compare', 'col.newRam'), (row) => row.runtimeRam, 'target', kindOf, unit),
    deltaColumn('ramDelta', t('compare', 'col.deltaRam'), (row) => row.runtimeRam, unit, true, 'desc'),
  ];

  return (
    <ChangeTable<ObjectChangeRowDto, ObjectChangeSortDto>
      resetKey={`${diffId}:${libraries ? 'libraries' : 'objects'}`}
      noun={t('compare', libraries ? 'contributors.libraries' : 'contributors.objects')}
      regionLabel={t('compare', 'objects.region')}
      heading={t('compare', 'objects.heading')}
      caption={t('compare', 'objects.caption')}
      pagerLabel={t('compare', libraries ? 'contributors.libraries' : 'contributors.objects')}
      errorLabel={t('compare', 'objects.errorLabel')}
      errorHeading={t('compare', 'objects.errorHeading')}
      initialSort="flashDelta"
      initialDirection="desc"
      columns={columns}
      rowKey={(row) => `${row.changeKind}:${row.key}`}
      rowHeader="key"
      rowName={(row) => row.key}
      query={(request) => queryObjectChanges({ diffId, libraries, ...request, limit: null })}
      focus={focus !== null && (focus.libraries ?? false) === libraries ? focus : null}
      onFocusConsumed={() => {
        setFocus(null);
      }}
      onReadingChange={onReadingChange}
      detail={(row) =>
        changeDetail(
          t,
          unit,
          diffId,
          row.key,
          row.changeKind,
          [
            [t('compare', 'row.flash'), row.nonvolatile],
            [t('compare', 'row.ram'), row.runtimeRam],
          ],
          row.ambiguous ? [[t('compare', 'row.pairing'), t('compare', 'objects.ambiguousNote').trim()]] : [],
        )
      }
      onInspect={onInspect}
      before={
        <>
          <p className={styles['hint']}>{t('compare', 'objects.basisHint')}</p>
          <p className={styles['status']}>
            {t('compare', 'objects.objectsLine', { tally: tally(attribution.objects) })}
            {' · '}
            {t('compare', 'objects.librariesLine', { tally: tally(attribution.libraries) })}
          </p>
          <div className={styles['ranking']}>
            <div className={styles['list']}>
              <h3>{t('compare', 'ranking.growthHeading')}</h3>
              <Contributors
                nounKey="contributors.objects"
                rows={attribution.topObjects}
                unit={unit}
                emptyKey="contributors.emptyObjectsGrowth"
                onPick={pick(false)}
              />
              <Contributors
                nounKey="contributors.libraries"
                rows={attribution.topLibraries}
                unit={unit}
                emptyKey="contributors.emptyLibrariesGrowth"
                onPick={pick(true)}
              />
            </div>
            <div className={styles['list']}>
              <h3>{t('compare', 'ranking.additionsHeading')}</h3>
              <Contributors
                nounKey="contributors.objects"
                rows={attribution.addedObjects}
                unit={unit}
                emptyKey="contributors.emptyObjectsAdded"
                onPick={pick(false)}
              />
            </div>
          </div>
        </>
      }
      extra={
        <span className={styles['cuts']} role="group" aria-label={t('compare', 'objects.viewGroup')}>
          {([false, true] as const).map((value) => (
            <button
              key={String(value)}
              type="button"
              className={styles['cut']}
              aria-pressed={libraries === value}
              onClick={() => {
                setLibraries(value);
                setFocus(null);
              }}
            >
              {t('compare', value ? 'objects.viewLibraries' : 'objects.viewObjects')}
            </button>
          ))}
        </span>
      }
    />
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

/**
 * One side of a change row.
 *
 * `Not present` is reserved for the side the diff says the row did not exist on. Every other empty
 * cell is a number that was never recorded, which is Unknown and not absence (prompt §23, D-R3-2).
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

function symbolAddition(row: SymbolChangeRowDto): ContributorDto {
  return {
    key: row.name,
    changeKind: row.changeKind,
    delta: null,
    bytes: row.size.target ?? row.size.base,
  };
}
