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
        title="Analyze"
        meta={
          currentSummary === null
            ? [selection === null ? 'no artifact selected' : `selected ${selection.fileName}`]
            : [
                `artifact ${currentSummary.artifact.fileName}`,
                <span key="sha">sha256 {truncateMiddle(currentSummary.artifact.sha256, 6)}</span>,
                `${String(currentSummary.sectionCount)} sections`,
                `${String(currentSummary.symbolCount)} symbols`,
              ]
        }
        subhead="Choose a firmware artifact and FirmwareSight reports the Core facts it can prove. Nothing
          here changes the artifact."
        actions={
          <>
            <Button
              disabled={analyzing}
              onClick={() => {
                void chooseArtifact();
              }}
            >
              Choose firmware artifact
            </Button>
            <Button
              variant="primary"
              disabled={analyzing || selection === null}
              onClick={() => {
                void analyze();
              }}
            >
              Analyze
            </Button>
          </>
        }
      />

      {/* The three inputs, stated as pills rather than as rows deep in a list: what this screen can
          actually prove about the build is the first thing a reader needs, and the words are Core's
          capability answers mapped to the five states, not a judgement made here. They describe the
          current attempt only — see `currentSummary`. When it has no right to speak, the strip says
          so in neutral words and points at the retained report instead of borrowing its pills. */}
      <div className={styles['states']} role="group" aria-label="Input capabilities">
        {currentSummary === null ? (
          <>
            <Chip>
              {error !== null
                ? 'Current analysis failed'
                : selection === null
                  ? 'No artifact selected yet'
                  : `${selection.fileName} selected, not analyzed yet`}
            </Chip>
            {summary === null ? null : <Chip>Previous result retained below</Chip>}
          </>
        ) : (
          <>
            <StateBadge
              variant="chip"
              state={capabilityState(currentSummary.capabilities.elf)}
              label={`ELF ${currentSummary.capabilities.elf}`}
            />
            <StateBadge
              variant="chip"
              state={capabilityState(currentSummary.capabilities.map)}
              label={`MAP ${currentSummary.capabilities.map}`}
            />
            <StateBadge
              variant="chip"
              state={capabilityState(currentSummary.capabilities.git)}
              label={`Git ${currentSummary.capabilities.git}`}
            />
          </>
        )}
      </div>

      <section className={styles['controls']} aria-label="Artifact selection">
        {selection === null ? (
          <p className={styles['meta']}>
            No artifact is selected. The Analyze action stays unavailable until one is.
          </p>
        ) : (
          <>
            <p className={styles['selected']} title={selection.fileName}>
              {selection.fileName}
            </p>
            <p className={styles['meta']}>
              {selection.mapAttached
                ? `MAP: ${selection.mapFileName ?? 'attached'}`
                : 'MAP: Not provided'}
            </p>
            <div className={styles['actions']}>
              <Button
                disabled={analyzing}
                onClick={() => {
                  void addMap();
                }}
              >
                {selection.mapAttached ? 'Replace MAP' : 'Add MAP'}
              </Button>
              {selection.mapAttached ? (
                <Button
                  variant="ghost"
                  disabled={analyzing}
                  onClick={() => {
                    void removeMap();
                  }}
                >
                  Remove MAP
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
          Analyzing…
        </p>
      ) : null}
      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label="Analysis error"
          heading="Analysis failed"
          action={
            <Button
              variant="primary"
              disabled={analyzing}
              onClick={() => {
                void chooseArtifact();
              }}
            >
              Choose another artifact
            </Button>
          }
        />
      )}
      {summary === null && !analyzing && error === null ? (
        <>
          <EmptyState
            label="No analysis yet"
            message="Nothing has been analyzed in this session yet. Choose an artifact and run Analyze."
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
  const { memory, evidenceSummary } = summary;
  const quality = evidenceQuality(memory);
  const flash = memory.nonvolatileImageFootprint;
  const ram = memory.runtimeRamFootprint;

  const budgetCell = (label: string, budget: typeof flash, rule: string) => (
    <SummaryCard
      label={label}
      state={
        <StateBadge
          state={budgetState(budget)}
          label={budgetWord(budget.state)}
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
      aria-label={stale ? 'Last good analysis result' : 'Analysis result'}
    >
      <Band narrow label="Result figures">
        {budgetCell('Flash footprint', flash, memory.accountingRule)}
        {budgetCell('Runtime RAM', ram, memory.layoutSource)}
        <SummaryCard
          label="Load evidence"
          state={<StateBadge state={quality.state} label={quality.label} />}
          context={quality.note}
        />
        <SummaryCard
          label="Evidence recorded"
          value={<Figure>{String(evidenceSummary.total)}</Figure>}
          context={`${String(evidenceSummary.observed)} observed · ${String(evidenceSummary.derived)} derived · ${String(evidenceSummary.declared)} declared · ${String(evidenceSummary.unknown)} unknown`}
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
  const { artifact, memory, capabilities } = summary;

  // The second half of the note. Two artifacts can share one leaf name, which is the case this
  // screen exists for, so naming the candidate has to be skipped when the names are equal: it
  // would otherwise read "not an analysis of firmware.elf" under "Previous analysis of
  // firmware.elf" and say nothing.
  const sameName = candidateName === null || candidateName === artifact.fileName;
  const attribution = pending
    ? candidateName === null
      ? ' The selection now in this row has not been analyzed yet.'
      : sameName
        ? ' The selection now in this row is a different file with the same name, and it has not been analyzed yet.'
        : ` It is not an analysis of ${candidateName}: that selection has not been analyzed yet.`
    : sameName
      ? ' The failed attempt above produced no result, so nothing here was replaced.'
      : ` It is not an analysis of ${candidateName}.`;

  return (
    <section
      className={styles['report']}
      aria-label={stale ? 'Last good analysis' : 'Analysis summary'}
    >
      {stale ? (
        <p className={styles['stale']} role="note">
          Previous analysis of {artifact.fileName}.
          {attribution}
        </p>
      ) : null}

      <PageSection title="Artifact">
        <FactList>
          <FactRow term="File" title={artifact.fileName}>
            {artifact.fileName}
          </FactRow>
          <FactRow term="SHA-256" title={artifact.sha256}>
            <Figure>{artifact.sha256}</Figure>
          </FactRow>
          <FactRow term="Size">
            <Figure>{formatSize(artifact.byteSize, unit)}</Figure>
          </FactRow>
          <FactRow term="Format">
            {`${artifact.architecture} ${artifact.bitness}-bit ${artifact.endianness}`}
          </FactRow>
          <FactRow term="Kind">{artifact.kind}</FactRow>
          <FactRow term="Parser">
            <Figure>{artifact.parserId}</Figure>
          </FactRow>
          <FactRow term="Entry">
            {formatOptional(artifact.entryPoint)}
            {artifact.entryPointUnknownReason === null ? null : (
              <Qualifier>{artifact.entryPointUnknownReason}</Qualifier>
            )}
          </FactRow>
          <FactRow term="Build ID">
            {truncateMiddle(formatOptional(artifact.buildId), 8)}
            {artifact.buildIdUnknownReason === null ? null : (
              <Qualifier>{artifact.buildIdUnknownReason}</Qualifier>
            )}
          </FactRow>
        </FactList>
      </PageSection>

      {/* The two budgets and the strength of the evidence behind them are stated once, in the result band
          above the tables. What stays here is the accounting that explains those figures. */}
      <PageSection title="Memory">
        <FactList>
          <FactRow term="Layout source">
            {memory.layoutSource}
            <Qualifier>{`Accounting rule ${memory.accountingRule}`}</Qualifier>
          </FactRow>
          <FactRow term="Dual-accounted">
            {memory.dualAccountedSections.length === 0
              ? 'none'
              : memory.dualAccountedSections.join(', ')}
            <Qualifier>{`${String(memory.dualAccountedSections.length)} section(s)`}</Qualifier>
          </FactRow>
          <FactRow term="Device metadata excluded">
            <Figure>{formatSize(memory.excludedMetadataBytes, unit)}</Figure>
          </FactRow>
        </FactList>
      </PageSection>

      <PageSection title="Capabilities">
        <FactList>
          <CapabilityRow term="ELF" value={capabilities.elf} />
          <CapabilityRow term="Sections" value={capabilities.sections} />
          <CapabilityRow term="Symbols" value={capabilities.symbols} />
          <CapabilityRow term="Debug info" value={capabilities.debugInfo} />
          <CapabilityRow term="MAP" value={capabilities.map} />
          {/* L19: the scope here is one build's attribution capability, not a delta between builds. */}
          <CapabilityRow term="Object/module attribution" value={capabilities.objectAttribution} />
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
function evidenceQuality(memory: MemorySummaryDto): {
  readonly state: StateName;
  readonly label: string;
  readonly note: string;
} {
  const basis = memory.weakestEvidenceBasis;
  if (basis === null) {
    return {
      state: 'UNKNOWN',
      label: 'No load evidence',
      note: 'No allocatable section carried an address and flag pair. Re-run with a linker MAP.',
    };
  }
  const caption = evidenceBasisCaption(basis);
  return memory.admissibleForHardBlock
    ? {
        state: 'PASS',
        label: 'Admissible for a hard limit',
        note: `Weakest basis ${caption}; region evidence came from the linker.`,
      }
    : {
        state: 'UNKNOWN',
        label: 'Not admissible for a hard limit',
        note: `Weakest basis ${caption} is name- or flag-derived. Supply the linker MAP to strengthen it.`,
      };
}

/** A budget's own evidence quality, in the five presentation states. `partial` asks for a look, not a pass. */
function budgetState(budget: MemorySummaryDto['nonvolatileImageFootprint']): StateName {
  return budget.state === 'exact' ? 'PASS' : budget.state === 'partial' ? 'REVIEW' : 'UNKNOWN';
}

function budgetWord(state: string): string {
  switch (state) {
    case 'exact':
      return 'Exact';
    case 'partial':
      return 'Partial';
    default:
      return 'Unknown';
  }
}

function CapabilityRow({ term, value }: { readonly term: string; readonly value: string }) {
  return (
    <FactRow term={term}>
      <StateBadge state={capabilityState(value)} label={value} />
    </FactRow>
  );
}
