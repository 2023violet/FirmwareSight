/**
 * The Analyze page: choose one artifact, get the Core facts FirmwareSight can prove, then read the
 * stored details one bounded page at a time.
 *
 * The two pieces of state that outlive a page switch - which artifact is selected and the last
 * summary the shell proved - belong to the shell, not to this component. Compare is allowed to
 * prefer the last analyzed snapshot as its target (prompt §18), and a reader who navigates away and
 * back must not find that their session forgot what it just analyzed. Everything else - the error,
 * the in-flight flag - is this page's own business.
 */

import { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';

import styles from './Analyze.module.css';
import { Button } from './components/Button';
import { DataTable, type Column } from './components/DataTable';
import { ErrorPanel } from './components/ErrorPanel';
import { ExportMenu } from './components/ExportMenu';
import { asEvidenceClass, FactValue, type FactEvidence } from './components/FactValue';
import { Inspector } from './components/Inspector';
import { Page } from './components/Layout';
import { LeadAnswer } from './components/LeadAnswer';
import { Notice } from './components/Notice';
import { PageHeader, PageSection } from './components/PageHeader';
import { EmptyState, FactList, FactRow, Figure, Qualifier } from './components/Panel';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import { StateBadge, type StateName } from './components/StateBadge';
import { SubjectBar, type Subject, type SubjectFreshness } from './components/SubjectBar';
import { Tabs, tabPanelProps } from './components/Tabs';
import { Details, LargestSections, type DetailsView } from './Details';
import { evidenceBasisCaption } from './evidenceBasis';
import { formatOptional, formatSize, truncateMiddle, type SizeUnit } from './format';
import { GettingStartedPanel } from './GettingStarted';
import { useI18n, type Translate } from './i18n';
import type { Catalog } from './i18n/en';
import { analyzeSelection, attachMap, clearMap, exportAnalysisMarkdown, exportAnalysisVisual, selectArtifact } from './ipc/bridge';
import type {
  AnalysisSummaryDto,
  ErrorEnvelopeDto,
  MemoryRegionDto,
  MemorySummaryDto,
  SelectionDto,
} from './ipc/types';
import { analyzedWithMap, capabilityState, mapChangedSince } from './stateWords';

export function Analyze({
  unit,
  onUnitChange,
  selection,
  onSelectionChange,
  analyzedSelectionId,
  lastGood,
  gettingStartedHidden,
  onHideGettingStarted,
  onLastGoodChange,
  autoAnalyzeId = null,
  onAutoAnalyzeStarted,
  chooseRequested = false,
  onChooseStarted,
}: {
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
  readonly selection: SelectionDto | null;
  readonly onSelectionChange: (selection: SelectionDto | null) => void;
  readonly analyzedSelectionId: string | null;
  readonly lastGood: AnalysisSummaryDto | null;
  /** Whether the reader has hidden the first-use panel for this session (prompt §13). */
  readonly gettingStartedHidden: boolean;
  readonly onHideGettingStarted: () => void;
  readonly onLastGoodChange: (summary: AnalysisSummaryDto, selectionId: string) => void;
  /** A selection the shell staged from a drop and wants analyzed once, as if Analyze had been pressed. */
  readonly autoAnalyzeId?: string | null;
  readonly onAutoAnalyzeStarted?: () => void;
  /** The shell asked for the firmware chooser once (`Ctrl+O` or the command palette, R5). */
  readonly chooseRequested?: boolean;
  readonly onChooseStarted?: () => void;
}) {
  const { t } = useI18n();
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [analyzing, setAnalyzing] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [exportMessage, setExportMessage] = useState<string | null>(null);
  /** Shown outside the export region, which is hidden while a newer selection waits to be analyzed. */
  const [staleExportNote, setStaleExportNote] = useState(false);
  const [exportError, setExportError] = useState<ErrorEnvelopeDto | null>(null);

  /**
   * Every dialog is the shell's, so the UI only ever learns the name and the handle. A cancelled
   * dialog answers `null`, which leaves the screen exactly as it was: refusing to choose is a
   * normal thing to do, not a failure to report.
   */
  const chooseArtifact = useCallback(async () => {
    const outcome = await selectArtifact();
    if (!outcome.ok) {
      setError(outcome.envelope);
      return;
    }
    if (outcome.value === null) {
      return;
    }
    onSelectionChange(outcome.value);
    setError(null);
  }, [onSelectionChange]);

  // A chooser the shell asked for opens once, as if the reader had pressed the button.
  useEffect(() => {
    if (!chooseRequested) {
      return;
    }
    onChooseStarted?.();
    void chooseArtifact();
  }, [chooseRequested, chooseArtifact, onChooseStarted]);

  const addMap = useCallback(async () => {
    if (selection === null) {
      return;
    }
    const outcome = await attachMap(selection.selectionId);
    if (!outcome.ok) {
      setError(outcome.envelope);
      return;
    }
    if (outcome.value !== null) {
      onSelectionChange(outcome.value);
    }
  }, [onSelectionChange, selection]);

  const removeMap = useCallback(async () => {
    if (selection === null) {
      return;
    }
    const outcome = await clearMap(selection.selectionId);
    if (!outcome.ok) {
      setError(outcome.envelope);
      return;
    }
    onSelectionChange(outcome.value);
  }, [onSelectionChange, selection]);

  /**
   * Analyze, and keep the previous result on the failure path.
   *
   * A failed analysis is not a snapshot: the last-good summary only ever moves when the shell
   * returns a summary. What changes on failure is the error, plus the label on the surviving report -
   * a reader must not mistake the previous artifact's numbers for the candidate that just failed.
   */
  const analyze = useCallback(async () => {
    if (selection === null) {
      return;
    }
    setAnalyzing(true);
    setStaleExportNote(false);
    const outcome = await analyzeSelection(selection.selectionId);
    setAnalyzing(false);

    if (outcome.ok) {
      onLastGoodChange(outcome.value, selection.selectionId);
      setError(null);
      return;
    }
    setError(outcome.envelope);
  }, [onLastGoodChange, selection]);

  // A drop asks for one analysis of the selection it staged. The request is consumed when it starts, so a
  // re-render or a later selection never repeats it.
  useEffect(() => {
    if (autoAnalyzeId === null || selection?.selectionId !== autoAnalyzeId || analyzing) {
      return;
    }
    onAutoAnalyzeStarted?.();
    void analyze();
  }, [autoAnalyzeId, selection, analyzing, analyze, onAutoAnalyzeStarted]);

  const summary = lastGood;

  /**
   * The report on screen was produced by a different selection than the one now in the row.
   *
   * Identity is the shell's selection handle and never the file name: two artifacts chosen in one
   * session can both be called `firmware.elf`, which is precisely the case that has to be caught.
   * Attaching or removing a MAP answers with the same handle, so it cannot move this flag.
   */
  const pendingSelection = selection !== null && selection.selectionId !== analyzedSelectionId;

  /**
   * The summary that is entitled to be read as the current one, which is not always `lastGood`.
   *
   * U1-V2-06: a failed attempt, or a selection that has not been analyzed yet, leaves the previous
   * snapshot in `lastGood`. Keeping it is correct and is what the report below does under an
   * explicit "Previous analysis of …" note. Presenting its capability words at the top of the page
   * as if they described the file now in the row is not: green "ELF supported / MAP provided" above a
   * row reading `MAP: Not provided` states a current fact the current attempt never earned. So the
   * top of this page reads from this value, and the retained evidence reads from `summary`.
   *
   * This is a presentation distinction over facts the shell already returned. It defines no domain
   * state, maps nothing onto the five Gate states, and mutates nothing.
   */
  const currentSummary =
    summary !== null && error === null && !pendingSelection ? summary : null;

  const exportRequest = useRef(0);
  // A layout effect, not a passive one: it runs in the same commit that shows the new selection or analysis.
  // A passive effect left a window - the new subject on screen, the counter not yet moved - in which a late
  // Save-dialog answer was taken as current and its ERR-EXPORT-6002 shown as an error instead of the stale
  // note (CI #123, Windows). The counter must move before anything else can be awaited.
  useLayoutEffect(() => {
    exportRequest.current += 1;
    setExporting(false);
    setExportError(null);
    setExportMessage(null);
    return () => { exportRequest.current += 1; };
  }, [selection?.selectionId, currentSummary?.identity.snapshotId]);

  const exportVisual = async (format: 'png' | 'svg' | 'html' | 'markdown') => {
    if (currentSummary === null || selection === null || analyzing || exporting) return;
    const request = ++exportRequest.current;
    setExporting(true);
    setStaleExportNote(false);
    const result =
      format === 'markdown'
        ? await exportAnalysisMarkdown(selection.selectionId, currentSummary.identity.snapshotId)
        : await exportAnalysisVisual(selection.selectionId, currentSummary.identity.snapshotId, format);
    if (request !== exportRequest.current) {
      // The analysis on screen changed while the Save dialog was open; Rust refused to write the old one
      // (M1, AC24). Say so rather than stay silent.
      if (!result.ok && result.envelope.code === 'ERR-EXPORT-6002') {
        setStaleExportNote(true);
      }
      return;
    }
    setExporting(false);
    if (!result.ok && result.envelope.code === 'ERR-EXPORT-6002') {
      // Rust is the authority on whether the subject changed: ERR-EXPORT-6002 says it did, even when the
      // answer arrives before this page has rendered the new subject. It is the stale note, never an error.
      setExportError(null);
      setExportMessage(null);
      setStaleExportNote(true);
      return;
    }
    if (!result.ok) {
      setExportError(result.envelope);
      setExportMessage(null);
      return;
    }
    setExportError(null);
    setExportMessage(result.value.status === 'written'
      ? t('analyze', 'export.written', { name: result.value.fileName ?? t('analyze', 'export.fallbackName') })
      : result.value.status === 'cancelled'
        ? t('analyze', 'export.cancelled')
        : result.value.status === 'kept-existing'
          ? t('analyze', 'export.keptExisting')
          : t('analyze', 'export.unknownOutcome', { status: result.value.status }));
  };

  /** Which view the reader is on. It survives a new analysis; the tables inside start fresh. */
  const [view, setView] = useState<View>('memory');
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
  // Evidence belongs to one snapshot, read as current or as retained. A new analysis, a result becoming
  // retained (another file chosen, or a failed attempt) and a change of view all close the Inspector, rather
  // than leave a fact on screen beside tables that no longer describe it in the same terms (R2 review S2-2).
  const shownSnapshot = summary?.identity.snapshotId ?? null;
  const shownAsRetained = summary !== null && (error !== null || pendingSelection);
  useEffect(() => {
    setInspected(null);
  }, [shownSnapshot, shownAsRetained, view]);

  /** The shell's freshness word for the SubjectBar: one place that says whose numbers these are. */
  const freshness: SubjectFreshness =
    summary === null
      ? 'latest'
      : error !== null
        ? 'failed'
        : pendingSelection
          ? 'retained'
          : mapChangedSince(summary, selection)
            ? 'mapChanged'
            : 'latest';
  const stale = summary !== null && (error !== null || pendingSelection);

  return (
    <div className={styles['withInspector']}>
      <Page>
        <PageHeader
          title={t('analyze', 'title')}
          subhead={t('analyze', 'header.subhead')}
          actions={
            <>
              {currentSummary === null ? null : (
                <ExportMenu
                  label={t('analyze', 'export.menu')}
                  regionLabel={t('analyze', 'export.region')}
                  hint={t('analyze', 'export.hint')}
                  items={[
                    { key: 'png', label: t('analyze', 'export.png') },
                    { key: 'svg', label: t('analyze', 'export.svg') },
                    { key: 'html', label: t('analyze', 'export.html') },
                    { key: 'markdown', label: t('analyze', 'export.markdown') },
                  ]}
                  disabled={exporting || analyzing}
                  onExport={(format) => {
                    void exportVisual(format);
                  }}
                />
              )}
              <Button
                variant={selection === null ? 'primary' : 'secondary'}
                disabled={analyzing}
                onClick={() => {
                  void chooseArtifact();
                }}
              >
                {t('analyze', 'intake.chooseArtifact')}
              </Button>
              <Button
                variant={selection === null ? 'secondary' : 'primary'}
                disabled={analyzing || selection === null}
                onClick={() => {
                  void analyze();
                }}
              >
                {t('analyze', 'intake.analyzeButton')}
              </Button>
            </>
          }
        />

        <SubjectBar
          subject={subjectOf(selection, currentSummary, summary)}
          freshness={selection !== null && summary === null ? 'pending' : freshness}
          actions={
            <>
              {selection === null ? null : (
                <>
                  <Button
                    disabled={analyzing}
                    onClick={() => {
                      void addMap();
                    }}
                  >
                    {selection.mapAttached ? t('analyze', 'intake.replaceMap') : t('analyze', 'intake.addMap')}
                  </Button>
                  {selection.mapAttached ? (
                    <Button
                      variant="ghost"
                      disabled={analyzing}
                      onClick={() => {
                        void removeMap();
                      }}
                    >
                      {t('analyze', 'intake.removeMap')}
                    </Button>
                  ) : null}
                </>
              )}
              <SizeUnitSwitch unit={unit} onSelect={onUnitChange} />
            </>
          }
        />

        {analyzing ? (
          // The only asynchronous fact on the screen. Without a live region it is visible to sighted
          // users and silent for everyone else, because the button's own disabled state says nothing
          // about when the work finished.
          <p className={styles['status']} role="status" aria-live="polite">
            {t('analyze', 'status.analyzing')}
          </p>
        ) : null}
        {staleExportNote ? (
          <Notice tone="stale" level={2} title={t('analyze', 'export.staleNote')} />
        ) : null}
        {exporting ? (
          <p className={styles['status']} role="status" aria-live="polite">
            {t('analyze', 'export.waiting')}
          </p>
        ) : null}
        {exportMessage === null ? null : (
          <p className={styles['status']} role="status" aria-live="polite">
            {exportMessage}
          </p>
        )}
        {exportError === null ? null : (
          <ErrorPanel
            envelope={exportError}
            label={t('analyze', 'export.errorLabel')}
            heading={t('analyze', 'export.errorHeading')}
          />
        )}
        {error === null ? null : (
          <ErrorPanel
            envelope={error}
            label={t('analyze', 'status.errorRegion')}
            heading={t('analyze', 'status.errorHeading')}
            action={
              <Button
                variant="primary"
                disabled={analyzing}
                onClick={() => {
                  void chooseArtifact();
                }}
              >
                {t('analyze', 'status.chooseAnother')}
              </Button>
            }
          />
        )}

        {summary === null && !analyzing && error === null ? (
          <>
            <EmptyState
              label={t('analyze', 'status.emptyRegion')}
              message={t('analyze', 'status.emptyMessage')}
            />
            {/* §13's first-use guidance, below the controls that answer it. It is a panel, not a gate:
                every control on this page is reachable with it showing. */}
            {gettingStartedHidden ? null : <GettingStartedPanel onDismiss={onHideGettingStarted} />}
          </>
        ) : null}

        {/* Everything below describes `summary`. When that is a retained result, the stale Notice says so
            first and the region names say it again, so a screen-reader user jumping straight to a table
            hears whose numbers they are (the U1R contract). */}
        {summary === null ? null : (
          <section
            className={styles['result']}
            aria-label={stale ? t('analyze', 'report.groupStale') : t('analyze', 'report.groupCurrent')}
          >
            {stale ? (
              <Notice tone="stale" level={2} title={t('analyze', 'status.retainedTitle')}>
                <p role="note">{retainedSentence(summary, selection?.fileName ?? null, pendingSelection, t)}</p>
              </Notice>
            ) : null}
            <AnalysisLead
              summary={summary}
              retained={stale}
              unit={unit}
              inspecting={inspected?.evidence ?? null}
              onInspect={inspect}
            />
            <BuildLine summary={summary} unit={unit} />
            <Tabs
              idBase="fs-analyze"
              label={t('analyze', 'views.label')}
              items={VIEWS.map((key) => ({ key, label: t('analyze', VIEW_LABEL[key]) }))}
              current={view}
              onSelect={setView}
            />
            <div {...tabPanelProps('fs-analyze', view)} className={styles['panel']}>
              {view === 'memory' ? (
                <MemoryView summary={summary} unit={unit} onInspect={inspect} />
              ) : view === 'evidence' ? (
                <>
                  <BuildFacts summary={summary} />
                  <Details
                    key={`${summary.identity.snapshotId}-evidence`}
                    snapshotId={summary.identity.snapshotId}
                    view="evidence"
                    unit={unit}
                    onInspect={inspect}
                    stale={stale}
                  />
                  <SnapshotFooter summary={summary} />
                </>
              ) : (
                <Details
                  key={`${summary.identity.snapshotId}-${view}`}
                  snapshotId={summary.identity.snapshotId}
                  view={view}
                  unit={unit}
                  onInspect={inspect}
                  stale={stale}
                />
              )}
            </div>
          </section>
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

/** The five views, in the order the spec reads them (ADR-0034 D2, `03_DESIGN/09` §6.2). */
type View = 'memory' | DetailsView;
const VIEWS: readonly View[] = ['memory', 'sections', 'symbols', 'objects', 'evidence'];
const VIEW_LABEL: Readonly<Record<View, AnalyzeKey>> = {
  memory: 'views.memory',
  sections: 'details.tab.sections',
  symbols: 'details.tab.symbols',
  objects: 'details.tab.objects',
  evidence: 'details.tab.evidence',
};

type AnalyzeKey = keyof Catalog['analyze'];

/**
 * What the SubjectBar names. The current analysis when there is one; otherwise the file in the row, with no
 * hash, because a hash this page has not computed for that file would be borrowed from another one.
 */
function subjectOf(
  selection: SelectionDto | null,
  current: AnalysisSummaryDto | null,
  retained: AnalysisSummaryDto | null,
): Subject | null {
  if (current !== null) {
    // The MAP word is the analysis's own; the selection only lends the file name when it still agrees.
    const withMap = analyzedWithMap(current);
    return {
      kind: 'single',
      build: {
        fileName: current.artifact.fileName,
        sha256: current.artifact.sha256,
        map: withMap,
        mapFileName: withMap === true && selection?.mapAttached === true ? selection.mapFileName : null,
      },
    };
  }
  if (selection !== null) {
    return {
      kind: 'single',
      build: { fileName: selection.fileName, sha256: null, map: selection.mapAttached, mapFileName: selection.mapFileName },
    };
  }
  if (retained !== null) {
    return { kind: 'single', build: { fileName: retained.artifact.fileName, sha256: retained.artifact.sha256, map: null } };
  }
  return null;
}

/**
 * The retained-result sentence, word for word the one the dossier used to carry.
 *
 * Two artifacts can share one leaf name, which is the case this screen exists for, so naming the candidate
 * is skipped when the names are equal: it would otherwise read "not an analysis of firmware.elf" under
 * "Previous analysis of firmware.elf" and say nothing.
 */
function retainedSentence(
  summary: AnalysisSummaryDto,
  candidateName: string | null,
  pending: boolean,
  t: Translate,
): string {
  const name = summary.artifact.fileName;
  const sameName = candidateName === null || candidateName === name;
  const attribution = pending
    ? candidateName === null
      ? t('analyze', 'report.previousNotAnalyzed')
      : sameName
        ? t('analyze', 'report.previousSameName')
        : t('analyze', 'report.previousOtherName', { name: candidateName })
    : sameName
      ? t('analyze', 'report.previousNoResult')
      : t('analyze', 'report.previousNotCandidate', { name: candidateName });
  return t('analyze', 'report.previousOf', { name }) + attribution;
}

/**
 * The page's one answer: Flash used, as a value the reader can trace, measured against RAM, the strength of
 * the load evidence and the amount of evidence behind it. It replaces the four-card result band.
 */
function AnalysisLead({
  summary,
  retained,
  unit,
  inspecting,
  onInspect,
}: {
  readonly summary: AnalysisSummaryDto;
  /** A retained result names itself as one, in the lead's own accessible name too (R2 review S3). */
  readonly retained: boolean;
  readonly unit: SizeUnit;
  readonly inspecting: FactEvidence | null;
  readonly onInspect: (evidence: FactEvidence, trigger: HTMLElement) => void;
}) {
  const { t } = useI18n();
  const { memory, evidenceSummary } = summary;
  const flash = memory.nonvolatileImageFootprint;
  const ram = memory.runtimeRamFootprint;
  const quality = evidenceQuality(memory, t);
  const evidence: FactEvidence = {
    field: 'nonvolatile_image_footprint_bytes',
    value: formatSize(flash.bytes, unit),
    classification: asEvidenceClass(flash.classification),
    source: memory.layoutSource,
    rule: memory.accountingRule,
    raw: flash.bytes === null ? t('analyze', 'budget.unknown') : String(flash.bytes),
    details: [
      [t('analyze', 'lead.accuracy'), budgetWord(flash.state, t)],
      ...(flash.reason === null ? [] : [[t('analyze', 'lead.reason'), flash.reason] as const]),
      ...(flash.unattributed.length === 0
        ? []
        : [[t('analyze', 'lead.unattributed'), flash.unattributed.join(', ')] as const]),
    ],
  };
  return (
    <LeadAnswer
      sentence={t('analyze', retained ? 'lead.flashRetained' : 'lead.flash')}
      figure={
        <FactValue
          evidence={evidence}
          selected={inspecting?.field === evidence.field}
          onInspect={onInspect}
        />
      }
      state={
        flash.state === 'exact' ? undefined : (
          <StateBadge state={budgetState(flash)} label={budgetWord(flash.state, t)} />
        )
      }
      reference={t('analyze', 'lead.reference', {
        ram: formatSize(ram.bytes, unit),
        ramWord: budgetWord(ram.state, t),
        quality: quality.label,
        count: evidenceSummary.total,
      })}
      note={t('analyze', 'result.evidenceLine', {
        observed: evidenceSummary.observed,
        derived: evidenceSummary.derived,
        declared: evidenceSummary.declared,
        unknown: evidenceSummary.unknown,
      })}
    />
  );
}

/**
 * Whose numbers these are, in one line under the answer: file, hash, size and format. A retained result keeps
 * this line inside its own region, so it can never be read as the identity of the file now in the row.
 */
function BuildLine({ summary, unit }: { readonly summary: AnalysisSummaryDto; readonly unit: SizeUnit }) {
  const { t } = useI18n();
  const { artifact } = summary;
  return (
    <p className={styles['buildLine']} role="group" aria-label={t('analyze', 'build.line')}>
      <span title={artifact.fileName}>{artifact.fileName}</span>
      <span className={styles['monoSmall']} title={artifact.sha256}>
        sha256 {truncateMiddle(artifact.sha256, 6)}
      </span>
      <span className={styles['monoSmall']}>{formatSize(artifact.byteSize, unit)}</span>
      <span>{`${artifact.architecture} ${artifact.bitness}-bit ${artifact.endianness}`}</span>
      <span>{t('analyze', 'header.metaSections', { count: summary.sectionCount })}</span>
      <span>{t('analyze', 'header.metaSymbols', { count: summary.symbolCount })}</span>
    </p>
  );
}

/** Memory: the regions, how the bytes were counted, and where most of them are. */
function MemoryView({
  summary,
  unit,
  onInspect,
}: {
  readonly summary: AnalysisSummaryDto;
  readonly unit: SizeUnit;
  readonly onInspect: (evidence: FactEvidence, trigger: HTMLElement) => void;
}) {
  const { t } = useI18n();
  const { memory } = summary;
  const quality = evidenceQuality(memory, t);
  return (
    <div className={styles['view']}>
      <RegionUsageTable regions={memory.regionUsage} onInspect={onInspect} />
      <PageSection title={t('analyze', 'memory.accounting')}>
        <FactList>
          <FactRow term={t('analyze', 'result.loadEvidenceLabel')}>
            <StateBadge state={quality.state} label={quality.label} />
            <Qualifier>{quality.note}</Qualifier>
          </FactRow>
          <FactRow term={t('analyze', 'report.layoutSourceTerm')}>
            {memory.layoutSource}
            <Qualifier>{t('analyze', 'report.accountingRule', { rule: memory.accountingRule })}</Qualifier>
          </FactRow>
          <FactRow term={t('analyze', 'report.dualAccountedTerm')}>
            {memory.dualAccountedSections.length === 0
              ? t('analyze', 'report.noneValue')
              : memory.dualAccountedSections.join(', ')}
            <Qualifier>
              {t('analyze', 'report.dualAccountedCount', { count: memory.dualAccountedSections.length })}
            </Qualifier>
          </FactRow>
          <FactRow term={t('analyze', 'report.excludedMetadataTerm')}>
            <Figure>{formatSize(memory.excludedMetadataBytes, unit)}</Figure>
          </FactRow>
        </FactList>
      </PageSection>
      <LargestSections snapshotId={summary.identity.snapshotId} unit={unit} />
    </div>
  );
}

/** Evidence › the build itself: what was read, and what this build can and cannot prove. */
function BuildFacts({ summary }: { readonly summary: AnalysisSummaryDto }) {
  const { t } = useI18n();
  const { artifact, capabilities } = summary;
  return (
    <div className={styles['buildFacts']}>
      <PageSection title={t('analyze', 'report.artifactSection')}>
        <FactList>
          {/* File, size and format are in the build line above; the full hash is spelled out here. */}
          <FactRow term="SHA-256" title={artifact.sha256}>
            <Figure>{artifact.sha256}</Figure>
          </FactRow>
          <FactRow term={t('analyze', 'report.kindTerm')}>{artifact.kind}</FactRow>
          <FactRow term={t('analyze', 'report.parserTerm')}>
            <Figure>{artifact.parserId}</Figure>
          </FactRow>
          <FactRow term={t('analyze', 'report.entryTerm')}>
            {formatOptional(artifact.entryPoint)}
            {artifact.entryPointUnknownReason === null ? null : (
              <Qualifier>{artifact.entryPointUnknownReason}</Qualifier>
            )}
          </FactRow>
          <FactRow term={t('analyze', 'report.buildIdTerm')}>
            {truncateMiddle(formatOptional(artifact.buildId), 8)}
            {artifact.buildIdUnknownReason === null ? null : (
              <Qualifier>{artifact.buildIdUnknownReason}</Qualifier>
            )}
          </FactRow>
        </FactList>
      </PageSection>
      <PageSection title={t('analyze', 'report.capabilitiesSection')}>
        <FactList>
          <CapabilityRow term="ELF" value={capabilities.elf} />
          <CapabilityRow term={t('analyze', 'report.sectionsTerm')} value={capabilities.sections} />
          <CapabilityRow term={t('analyze', 'report.symbolsTerm')} value={capabilities.symbols} />
          <CapabilityRow term={t('analyze', 'report.debugInfoTerm')} value={capabilities.debugInfo} />
          <CapabilityRow term="MAP" value={capabilities.map} />
          {/* L19: the scope here is one build's attribution capability, not a delta between builds. */}
          <CapabilityRow term={t('analyze', 'report.attributionTerm')} value={capabilities.objectAttribution} />
          <CapabilityRow term="Git" value={capabilities.git} />
        </FactList>
      </PageSection>
    </div>
  );
}

function SnapshotFooter({ summary }: { readonly summary: AnalysisSummaryDto }) {
  return (
    <footer className={styles['footer']}>
      <span className={styles['monoSmall']}>{summary.identity.snapshotId}</span>
      <span>
        {summary.identity.schema} · {summary.identity.schemaStability} · normalization{' '}
        {summary.identity.normalizationVersion} · fwsight {summary.identity.createdByFwsightVersion}
      </span>
    </footer>
  );
}


/**
 * The linker's regions as the shared table (R3, D-R2-2 closed). Each row's section evidence - origin, source,
 * evidence id and every section the region holds, with its addresses and basis - opens in the Inspector
 * rather than in a disclosure inside the row, so the table keeps one row height and the evidence gets room.
 */
function RegionUsageTable({
  regions,
  onInspect,
}: {
  readonly regions: readonly MemoryRegionDto[];
  readonly onInspect: (evidence: FactEvidence, trigger: HTMLElement) => void;
}) {
  const { t } = useI18n();

  if (regions.length === 0) {
    return (
      <p className={styles['regionUnknown']} role="status">
        {t('analyze', 'region.none')}
      </p>
    );
  }

  const columns: Column<MemoryRegionDto>[] = [
    { key: 'name', header: t('analyze', 'region.name'), cell: (region) => <span title={region.name}>{region.name}</span> },
    { key: 'kind', header: t('analyze', 'region.kind'), cell: (region) => regionKind(region.kind, t) },
    { key: 'capacity', header: t('analyze', 'region.capacity'), numeric: true, cell: (region) => regionBytes(region.capacityBytes, t) },
    {
      key: 'mapped',
      header: t('analyze', 'region.mapped'),
      numeric: true,
      cell: (region) =>
        region.usedBytes === null
          ? t('analyze', 'region.unknown')
          : region.state === 'partial'
            ? t('analyze', 'region.atLeast', { bytes: region.usedBytes })
            : regionBytes(region.usedBytes, t),
    },
    {
      key: 'status',
      header: t('analyze', 'region.status'),
      cell: (region) => (
        <>
          <span>{regionState(region.state, t)}</span>
          <span className={styles['regionStatusDetail']}>{regionRisk(region.risk, t)}</span>
          {region.reasons.length === 0 ? null : (
            <span className={styles['regionStatusDetail']}>{region.reasons.join('; ')}</span>
          )}
        </>
      ),
    },
    {
      key: 'remaining',
      header: t('analyze', 'region.remaining'),
      cell: (region) =>
        region.remainingBytes === null ? (
          <span>
            {t('analyze', 'region.unknown')}
            <span className={styles['regionStatusDetail']}>{region.remainingReason}</span>
          </span>
        ) : (
          regionBytes(region.remainingBytes, t)
        ),
    },
  ];

  return (
    <DataTable
      caption={t('analyze', 'region.title')}
      columns={columns}
      rows={regions}
      rowKey={(region) => region.evidenceId}
      rowHeader="name"
      onRowAction={(region, trigger) => {
        onInspect(regionEvidence(region, t), trigger);
      }}
      rowActionLabel={(region) => t('analyze', 'region.showEvidence', { name: region.name })}
    />
  );
}

/** One region's evidence, in the shape the Inspector shows: the region, then each section it holds. */
function regionEvidence(region: MemoryRegionDto, t: Translate): FactEvidence {
  return {
    field: region.name,
    // The same words the table's Mapped cell uses: a partial region is "at least N bytes", never plain N.
    value:
      region.usedBytes === null
        ? t('analyze', 'region.unknown')
        : region.state === 'partial'
          ? t('analyze', 'region.atLeast', { bytes: region.usedBytes })
          : regionBytes(region.usedBytes, t),
    // Mapped bytes are a sum of section intervals the linker regions bound: derived from observed rows, not
    // observed themselves (AGENTS.md §8, R3 review S2-2). No evidence at all is Unknown.
    classification: region.evidenceSource === 'none' || region.usedBytes === null ? 'unknown' : 'derived',
    source: region.evidenceSource,
    locator: region.evidenceId,
    rule: t('analyze', 'region.origin') + ' ' + region.origin,
    details: region.sections.map((section): readonly [string, string] => [
      section.name ?? t('analyze', 'region.unknown'),
      [
        t('analyze', 'region.sectionAddress', {
          kind: addressKind(section.addressKind, t),
          address: section.address,
          bytes: section.bytes,
        }),
        t('analyze', 'region.vmaLma', {
          vma: section.virtualAddress ?? t('analyze', 'region.unknown'),
          lma: section.loadAddress ?? t('analyze', 'region.unknown'),
        }),
        ...(section.virtualAddressUnknownReason === null
          ? []
          : [t('analyze', 'region.virtualReason', { reason: section.virtualAddressUnknownReason })]),
        ...(section.loadAddressUnknownReason === null
          ? []
          : [t('analyze', 'region.loadReason', { reason: section.loadAddressUnknownReason })]),
        `${section.locator} · ${section.evidenceBasis}`,
      ].join(' · '),
    ]),
  };
}

function regionBytes(value: string | null, t: Translate): string {
  return value === null ? t('analyze', 'region.unknown') : `${value} ${t('analyze', 'region.bytes')}`;
}

function regionKind(value: string, t: Translate): string {
  switch (value) {
    case 'nonvolatile': return t('analyze', 'region.kindNonvolatile');
    case 'volatile-ram': return t('analyze', 'region.kindRam');
    default: return t('analyze', 'region.unknown');
  }
}

function regionState(value: string, t: Translate): string {
  switch (value) {
    case 'exact': return t('analyze', 'region.stateExact');
    case 'partial': return t('analyze', 'region.statePartial');
    default: return t('analyze', 'region.unknown');
  }
}

function regionRisk(value: string, t: Translate): string {
  switch (value) {
    case 'within-capacity': return t('analyze', 'region.riskWithin');
    case 'exceeded': return t('analyze', 'region.riskExceeded');
    case 'review': return t('analyze', 'region.riskReview');
    default: return t('analyze', 'region.unknown');
  }
}

function addressKind(value: string, t: Translate): string {
  return value === 'load' ? t('analyze', 'region.loadAddress') : t('analyze', 'region.virtualAddress');
}

/**
 * How strong the load evidence behind the two budgets is.
 *
 * Returned as data rather than as markup because the same answer is the Analyze result band's third cell
 * and the dossier has no room for it twice. The words are Core's: the weakest basis is what caps the
 * pair, and admissibility for a hard limit is a property of that evidence, not a mood.
 */
function evidenceQuality(memory: MemorySummaryDto, t: Translate): {
  readonly state: StateName;
  readonly label: string;
  readonly note: string;
} {
  const basis = memory.weakestEvidenceBasis;
  if (basis === null) {
    return {
      state: 'UNKNOWN',
      label: t('analyze', 'evidence.noneLabel'),
      note: t('analyze', 'evidence.noneNote'),
    };
  }
  const caption = evidenceBasisCaption(basis, t);
  return memory.admissibleForHardBlock
    ? {
        state: 'PASS',
        label: t('analyze', 'evidence.admissibleLabel'),
        note: t('analyze', 'evidence.admissibleNote', { basis: caption }),
      }
    : {
        state: 'UNKNOWN',
        label: t('analyze', 'evidence.notAdmissibleLabel'),
        note: t('analyze', 'evidence.notAdmissibleNote', { basis: caption }),
      };
}

/** A budget's own evidence quality, in the five presentation states. `partial` asks for a look, not a pass. */
function budgetState(budget: MemorySummaryDto['nonvolatileImageFootprint']): StateName {
  return budget.state === 'exact' ? 'PASS' : budget.state === 'partial' ? 'REVIEW' : 'UNKNOWN';
}

function budgetWord(state: string, t: Translate): string {
  switch (state) {
    case 'exact':
      return t('analyze', 'budget.exact');
    case 'partial':
      return t('analyze', 'budget.partial');
    default:
      return t('analyze', 'budget.unknown');
  }
}

function CapabilityRow({ term, value }: { readonly term: string; readonly value: string }) {
  return (
    <FactRow term={term}>
      <StateBadge state={capabilityState(value)} label={value} />
    </FactRow>
  );
}
