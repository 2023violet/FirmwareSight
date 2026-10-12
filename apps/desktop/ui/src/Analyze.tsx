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

import { useCallback, useState } from 'react';

import styles from './Analyze.module.css';
import { Button } from './components/Button';
import { Chip } from './components/Chip';
import { ErrorPanel } from './components/ErrorPanel';
import { Page } from './components/Layout';
import { PageHeader, PageSection } from './components/PageHeader';
import { Band, EmptyState, FactList, FactRow, Figure, Qualifier, SummaryCard } from './components/Panel';
import { StateBadge, type StateName } from './components/StateBadge';
import { Details } from './Details';
import { evidenceBasisCaption } from './evidenceBasis';
import { formatOptional, formatSize, truncateMiddle, type SizeUnit } from './format';
import { GettingStartedPanel } from './GettingStarted';
import { useI18n, type Translate } from './i18n';
import { analyzeSelection, attachMap, clearMap, selectArtifact } from './ipc/bridge';
import type {
  AnalysisSummaryDto,
  ErrorEnvelopeDto,
  MemorySummaryDto,
  SelectionDto,
} from './ipc/types';
import { capabilityState } from './stateWords';

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
}) {
  const { t } = useI18n();
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [analyzing, setAnalyzing] = useState(false);

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
    const outcome = await analyzeSelection(selection.selectionId);
    setAnalyzing(false);

    if (outcome.ok) {
      onLastGoodChange(outcome.value, selection.selectionId);
      setError(null);
      return;
    }
    setError(outcome.envelope);
  }, [onLastGoodChange, selection]);

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

  return (
    <Page>
      <PageHeader
        title={t('analyze', 'title')}
        meta={
          currentSummary === null
            ? [
                selection === null
                  ? t('analyze', 'header.metaNone')
                  : t('analyze', 'header.metaSelected', { name: selection.fileName }),
              ]
            : [
                t('analyze', 'header.metaArtifact', { name: currentSummary.artifact.fileName }),
                <span key="sha">sha256 {truncateMiddle(currentSummary.artifact.sha256, 6)}</span>,
                t('analyze', 'header.metaSections', { count: currentSummary.sectionCount }),
                t('analyze', 'header.metaSymbols', { count: currentSummary.symbolCount }),
              ]
        }
        subhead={t('analyze', 'header.subhead')}
        actions={
          <>
            <Button
              disabled={analyzing}
              onClick={() => {
                void chooseArtifact();
              }}
            >
              {t('analyze', 'intake.chooseArtifact')}
            </Button>
            <Button
              variant="primary"
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

      {/* The three inputs, stated as pills rather than as rows deep in a list: what this screen can
          actually prove about the build is the first thing a reader needs, and the words are Core's
          capability answers mapped to the five states, not a judgement made here. They describe the
          current attempt only — see `currentSummary`. When it has no right to speak, the strip says
          so in neutral words and points at the retained report instead of borrowing its pills. */}
      <div className={styles['states']} role="group" aria-label={t('analyze', 'status.capabilitiesGroup')}>
        {currentSummary === null ? (
          <>
            <Chip>
              {error !== null
                ? t('analyze', 'status.chipFailed')
                : selection === null
                  ? t('analyze', 'status.chipNoArtifact')
                  : t('analyze', 'status.chipSelected', { name: selection.fileName })}
            </Chip>
            {summary === null ? null : <Chip>{t('analyze', 'status.chipRetained')}</Chip>}
          </>
        ) : (
          <>
            <StateBadge
              variant="chip"
              state={capabilityState(currentSummary.capabilities.elf)}
              label={t('analyze', 'status.capabilityElf', { value: currentSummary.capabilities.elf })}
            />
            <StateBadge
              variant="chip"
              state={capabilityState(currentSummary.capabilities.map)}
              label={t('analyze', 'status.capabilityMap', { value: currentSummary.capabilities.map })}
            />
            <StateBadge
              variant="chip"
              state={capabilityState(currentSummary.capabilities.git)}
              label={t('analyze', 'status.capabilityGit', { value: currentSummary.capabilities.git })}
            />
          </>
        )}
      </div>

      <section className={styles['controls']} aria-label={t('analyze', 'intake.selectionGroup')}>
        {selection === null ? (
          <p className={styles['meta']}>{t('analyze', 'intake.noArtifact')}</p>
        ) : (
          <>
            <p className={styles['selected']} title={selection.fileName}>
              {selection.fileName}
            </p>
            <p className={styles['meta']}>
              {selection.mapAttached
                ? t('analyze', 'intake.mapLine', {
                    name: selection.mapFileName ?? t('analyze', 'intake.mapAttached'),
                  })
                : t('analyze', 'intake.mapNotProvided')}
            </p>
            <div className={styles['actions']}>
              <Button
                disabled={analyzing}
                onClick={() => {
                  void addMap();
                }}
              >
                {selection.mapAttached
                  ? t('analyze', 'intake.replaceMap')
                  : t('analyze', 'intake.addMap')}
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
            </div>
          </>
        )}
      </section>

      {analyzing ? (
        // The only asynchronous fact on the screen. Without a live region it is visible to sighted
        // users and silent for everyone else, because the button's own disabled state says nothing
        // about when the work finished.
        <p className={styles['status']} role="status" aria-live="polite">
          {t('analyze', 'status.analyzing')}
        </p>
      ) : null}
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
          {/* §13's first-use guidance, above the empty state it explains and below the controls that
              answer it. It is a panel, not a gate: every control on this page is reachable with it
              showing, and hiding it is one click that hides nothing else. */}
          {gettingStartedHidden ? null : <GettingStartedPanel onDismiss={onHideGettingStarted} />}
        </>
      ) : null}

      {/* U1P §9 and §13, in one place: the analysis comes before the dossier, and everything on screen
          that describes a past attempt sits on a quieter surface than the failure above it. The band is
          the four figures that answer "was this worth reading", `Details` is the actual sections and
          symbols, and the identity dossier moves below them as the reference puts it. Under a failed
          attempt or a selection that has not been analyzed, all three keep saying whose numbers they are
          — the band's region name, the report's own sentence, the details' accessible name — which is
          the U1R contract, restated in a new position rather than loosened. */}
      {summary === null ? null : (
        <div className={error === null && !pendingSelection ? undefined : styles['retained']}>
          <ResultBand summary={summary} stale={error !== null || pendingSelection} unit={unit} />
          <Details
            snapshotId={summary.identity.snapshotId}
            unit={unit}
            onUnitChange={onUnitChange}
            stale={error !== null || pendingSelection}
          />
          <Report
            summary={summary}
            stale={error !== null || pendingSelection}
            candidateName={selection?.fileName ?? null}
            pending={pendingSelection}
            unit={unit}
          />
        </div>
      )}
    </Page>
  );
}

/**
 * The result figures, stated once.
 *
 * Four facts answer "was this worth analyzing, and how good is the evidence": what the build costs in
 * flash and in RAM, whether the load evidence is strong enough to bind a limit, and how much recorded
 * evidence stands behind the screen. Each is Core's own word and number, projected into the five
 * presentation states by the same mapping the rest of the product uses; nothing here is computed.
 */
function ResultBand({
  summary,
  stale,
  unit,
}: {
  readonly summary: AnalysisSummaryDto;
  readonly stale: boolean;
  readonly unit: SizeUnit;
}) {
  const { t } = useI18n();
  const { memory, evidenceSummary } = summary;
  const quality = evidenceQuality(memory, t);
  const flash = memory.nonvolatileImageFootprint;
  const ram = memory.runtimeRamFootprint;

  const budgetCell = (label: string, budget: typeof flash, rule: string) => (
    <SummaryCard
      label={label}
      state={
        <StateBadge
          state={budgetState(budget)}
          label={budgetWord(budget.state, t)}
          count={budget.state === 'partial' ? budget.unattributed.length : undefined}
          note={budget.reason ?? undefined}
        />
      }
      value={<Figure>{formatSize(budget.bytes, unit)}</Figure>}
      context={`${budget.classification} · ${rule}`}
    />
  );

  return (
    <section
      className={styles['result']}
      aria-label={
        stale ? t('analyze', 'result.groupStale') : t('analyze', 'result.groupCurrent')
      }
    >
      <Band narrow label={t('analyze', 'result.band')}>
        {budgetCell(t('analyze', 'result.flashLabel'), flash, memory.accountingRule)}
        {budgetCell(t('analyze', 'result.ramLabel'), ram, memory.layoutSource)}
        <SummaryCard
          label={t('analyze', 'result.loadEvidenceLabel')}
          state={<StateBadge state={quality.state} label={quality.label} />}
          context={quality.note}
        />
        <SummaryCard
          label={t('analyze', 'result.evidenceRecordedLabel')}
          value={<Figure>{String(evidenceSummary.total)}</Figure>}
          context={t('analyze', 'result.evidenceLine', {
            observed: evidenceSummary.observed,
            derived: evidenceSummary.derived,
            declared: evidenceSummary.declared,
            unknown: evidenceSummary.unknown,
          })}
        />
      </Band>
    </section>
  );
}

/**
 * The report, and the honesty flag that comes with it.
 *
 * `stale` is set when a later analysis failed **or** when the selection has moved on without one:
 * either way the numbers below are the last ones FirmwareSight could prove, not a result for the
 * file now sitting in the row. Naming the candidate it does not describe is what keeps a preserved
 * result from being read as a fresh verdict.
 */
function Report({
  summary,
  stale,
  pending,
  candidateName,
  unit,
}: {
  readonly summary: AnalysisSummaryDto;
  readonly stale: boolean;
  readonly pending: boolean;
  readonly candidateName: string | null;
  readonly unit: SizeUnit;
}) {
  const { t } = useI18n();
  const { artifact, memory, capabilities } = summary;

  // The second half of the note. Two artifacts can share one leaf name, which is the case this
  // screen exists for, so naming the candidate has to be skipped when the names are equal: it
  // would otherwise read "not an analysis of firmware.elf" under "Previous analysis of
  // firmware.elf" and say nothing.
  const sameName = candidateName === null || candidateName === artifact.fileName;
  const attribution = pending
    ? candidateName === null
      ? t('analyze', 'report.previousNotAnalyzed')
      : sameName
        ? t('analyze', 'report.previousSameName')
        : t('analyze', 'report.previousOtherName', { name: candidateName })
    : sameName
      ? t('analyze', 'report.previousNoResult')
      : t('analyze', 'report.previousNotCandidate', { name: candidateName });

  return (
    <section
      className={styles['report']}
      aria-label={stale ? t('analyze', 'report.groupStale') : t('analyze', 'report.groupCurrent')}
    >
      {stale ? (
        <p className={styles['stale']} role="note">
          {t('analyze', 'report.previousOf', { name: artifact.fileName }) + attribution}
        </p>
      ) : null}

      <PageSection title={t('analyze', 'report.artifactSection')}>
        <FactList>
          <FactRow term={t('analyze', 'report.fileTerm')} title={artifact.fileName}>
            {artifact.fileName}
          </FactRow>
          <FactRow term="SHA-256" title={artifact.sha256}>
            <Figure>{artifact.sha256}</Figure>
          </FactRow>
          <FactRow term={t('analyze', 'report.sizeTerm')}>
            <Figure>{formatSize(artifact.byteSize, unit)}</Figure>
          </FactRow>
          <FactRow term={t('analyze', 'report.formatTerm')}>
            {`${artifact.architecture} ${artifact.bitness}-bit ${artifact.endianness}`}
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

      {/* The two budgets and the strength of the evidence behind them are stated once, in the result band
          above the tables. What stays here is the accounting that explains those figures. */}
      <PageSection title={t('analyze', 'report.memorySection')}>
        <FactList>
          <FactRow term={t('analyze', 'report.layoutSourceTerm')}>
            {memory.layoutSource}
            <Qualifier>{t('analyze', 'report.accountingRule', { rule: memory.accountingRule })}</Qualifier>
          </FactRow>
          <FactRow term={t('analyze', 'report.dualAccountedTerm')}>
            {memory.dualAccountedSections.length === 0
              ? t('analyze', 'report.noneValue')
              : memory.dualAccountedSections.join(', ')}
            <Qualifier>
              {t('analyze', 'report.dualAccountedCount', {
                count: memory.dualAccountedSections.length,
              })}
            </Qualifier>
          </FactRow>
          <FactRow term={t('analyze', 'report.excludedMetadataTerm')}>
            <Figure>{formatSize(memory.excludedMetadataBytes, unit)}</Figure>
          </FactRow>
        </FactList>
      </PageSection>

      <PageSection title={t('analyze', 'report.capabilitiesSection')}>
        <FactList>
          <CapabilityRow term="ELF" value={capabilities.elf} />
          <CapabilityRow term={t('analyze', 'report.sectionsTerm')} value={capabilities.sections} />
          <CapabilityRow term={t('analyze', 'report.symbolsTerm')} value={capabilities.symbols} />
          <CapabilityRow
            term={t('analyze', 'report.debugInfoTerm')}
            value={capabilities.debugInfo}
          />
          <CapabilityRow term="MAP" value={capabilities.map} />
          {/* L19: the scope here is one build's attribution capability, not a delta between builds. */}
          <CapabilityRow
            term={t('analyze', 'report.attributionTerm')}
            value={capabilities.objectAttribution}
          />
          <CapabilityRow term="Git" value={capabilities.git} />
        </FactList>
      </PageSection>

      <footer className={styles['footer']}>
        <span className={styles['monoSmall']}>{summary.identity.snapshotId}</span>
        <span>
          {summary.identity.schema} · {summary.identity.schemaStability} · normalization{' '}
          {summary.identity.normalizationVersion} · fwsight{' '}
          {summary.identity.createdByFwsightVersion}
        </span>
      </footer>
    </section>
  );
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
