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

import { useCallback, useState, type ReactNode } from 'react';

import styles from './Analyze.module.css';
import { Details } from './Details';
import { GettingStartedPanel } from './GettingStarted';
import { ErrorPanel } from './components/ErrorPanel';
import { StateBadge, type StateName } from './components/StateBadge';
import { evidenceBasisCaption } from './evidenceBasis';
import { formatOptional, formatSize, truncateMiddle, type SizeUnit } from './format';
import { analyzeSelection, attachMap, clearMap, selectArtifact } from './ipc/bridge';
import type {
  AnalysisSummaryDto,
  BudgetDto,
  ErrorEnvelopeDto,
  SelectionDto,
} from './ipc/types';
import { cx } from './styles/classnames';

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

  return (
    <main className={styles['page']}>
      <header className={styles['header']}>
        <h1>Analyze</h1>
        <p className={styles['subhead']}>
          Choose a firmware artifact and FirmwareSight reports the Core facts it can prove. Nothing
          here changes the artifact. Comparing two builds that were already analyzed happens on
          Compare.
        </p>
      </header>

      <section className={styles['controls']} aria-label="Artifact selection">
        <button
          type="button"
          className={styles['control']}
          onClick={() => {
            void chooseArtifact();
          }}
          disabled={analyzing}
        >
          Choose firmware artifact
        </button>

        {selection === null ? null : (
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
              {selection.mapAttached ? (
                <button
                  type="button"
                  className={styles['control']}
                  onClick={() => {
                    void addMap();
                  }}
                  disabled={analyzing}
                >
                  Replace MAP
                </button>
              ) : (
                <button
                  type="button"
                  className={styles['control']}
                  onClick={() => {
                    void addMap();
                  }}
                  disabled={analyzing}
                >
                  Add MAP
                </button>
              )}
              {selection.mapAttached ? (
                <button
                  type="button"
                  className={styles['control']}
                  onClick={() => {
                    void removeMap();
                  }}
                  disabled={analyzing}
                >
                  Remove MAP
                </button>
              ) : null}
            </div>
          </>
        )}

        <button
          type="button"
          className={styles['primary']}
          onClick={() => {
            void analyze();
          }}
          disabled={analyzing || selection === null}
        >
          Analyze
        </button>
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
        <ErrorPanel envelope={error} label="Analysis error" heading="Analysis failed" />
      )}
      {summary === null && !analyzing && error === null ? (
        <>
          <section className={styles['status']} aria-label="No analysis yet">
            <p>Nothing has been analyzed in this session yet. Choose an artifact and run Analyze.</p>
          </section>
          {/* §13's first-use guidance, above the empty state it explains and below the controls that
              answer it. It is a panel, not a gate: every control on this page is reachable with it
              showing, and hiding it is one click that hides nothing else. */}
          {gettingStartedHidden ? null : <GettingStartedPanel onDismiss={onHideGettingStarted} />}
        </>
      ) : null}

      {summary === null ? null : (
        <Report
          summary={summary}
          stale={error !== null || pendingSelection}
          candidateName={selection?.fileName ?? null}
          pending={pendingSelection}
          unit={unit}
        />
      )}

      {/* One snapshot id, the last one the shell proved. A later attempt that failed cannot repoint
          these tables, because the last-good summary only moves on a summary. */}
      {summary === null ? null : (
        <Details snapshotId={summary.identity.snapshotId} unit={unit} onUnitChange={onUnitChange} />
      )}
    </main>
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
  const { artifact, memory, capabilities, evidenceSummary } = summary;

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

      <Section title="Artifact">
        <Row term="File" value={artifact.fileName} title={artifact.fileName} />
        <Row term="SHA-256" value={artifact.sha256} mono title={artifact.sha256} />
        <Row term="Size" value={formatSize(artifact.byteSize, unit)} mono />
        <Row
          term="Format"
          value={`${artifact.architecture} ${artifact.bitness}-bit ${artifact.endianness}`}
        />
        <Row term="Kind" value={artifact.kind} />
        <Row term="Parser" value={artifact.parserId} mono />
        <Row
          term="Entry"
          value={formatOptional(artifact.entryPoint)}
          mono={artifact.entryPoint !== null}
          note={artifact.entryPointUnknownReason ?? null}
        />
        <Row
          term="Build ID"
          value={truncateMiddle(formatOptional(artifact.buildId), 8)}
          mono={artifact.buildId !== null}
          note={artifact.buildIdUnknownReason ?? null}
        />
      </Section>

      <Section title="Memory">
        <BudgetRow
          term="Nonvolatile / load image"
          budget={memory.nonvolatileImageFootprint}
          unit={unit}
        />
        <BudgetRow term="Runtime RAM" budget={memory.runtimeRamFootprint} unit={unit} />
        <div className={styles['row']}>
          <span className={styles['term']}>Load evidence</span>
          <EvidenceQuality memory={memory} />
        </div>
        <Row
          term="Layout source"
          value={memory.layoutSource}
          note={`Accounting rule ${memory.accountingRule}`}
        />
        <Row
          term="Dual-accounted"
          value={
            memory.dualAccountedSections.length === 0
              ? 'none'
              : memory.dualAccountedSections.join(', ')
          }
          mono
          count={memory.dualAccountedSections.length}
        />
        <Row
          term="Device metadata excluded"
          value={formatSize(memory.excludedMetadataBytes, unit)}
          mono
        />
      </Section>

      <Section title="Capabilities">
        <CapabilityRow term="ELF" value={capabilities.elf} />
        <CapabilityRow term="Sections" value={capabilities.sections} />
        <CapabilityRow term="Symbols" value={capabilities.symbols} />
        <CapabilityRow term="Debug info" value={capabilities.debugInfo} />
        <CapabilityRow term="MAP" value={capabilities.map} />
        {/* L19: the scope here is one build's attribution capability, not a delta between builds. */}
        <CapabilityRow term="Object/module attribution" value={capabilities.objectAttribution} />
        <CapabilityRow term="Git" value={capabilities.git} />
      </Section>

      <Section title="Counts">
        <Row term="Sections" value={String(summary.sectionCount)} mono />
        <Row term="Symbols" value={String(summary.symbolCount)} mono />
        <div className={styles['row']}>
          <span className={styles['term']}>Evidence</span>
          <span className={styles['value']}>
            <StateBadge
              state="PASS"
              label={`${String(evidenceSummary.total)} recorded`}
              note={`${String(evidenceSummary.observed)} observed, ${String(evidenceSummary.derived)} derived, ${String(evidenceSummary.declared)} declared, ${String(evidenceSummary.unknown)} unknown`}
            />
          </span>
        </div>
      </Section>

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

function EvidenceQuality({ memory }: { readonly memory: AnalysisSummaryDto['memory'] }) {
  const basis = memory.weakestEvidenceBasis;
  if (basis === null) {
    return (
      <StateBadge
        state="UNKNOWN"
        label="No load evidence"
        note="No allocatable section carried an address and flag pair. Re-run with a linker MAP."
      />
    );
  }
  const caption = evidenceBasisCaption(basis);
  if (memory.admissibleForHardBlock) {
    return (
      <StateBadge
        state="PASS"
        label="Admissible for a hard limit"
        note={`Weakest basis ${caption}; region evidence came from the linker.`}
      />
    );
  }
  return (
    <StateBadge
      state="UNKNOWN"
      label="Not admissible for a hard limit"
      note={`Weakest basis ${caption} is name- or flag-derived. Supply the linker MAP to strengthen it.`}
    />
  );
}

function BudgetRow({
  term,
  budget,
  unit,
}: {
  readonly term: string;
  readonly budget: BudgetDto;
  readonly unit: SizeUnit;
}) {
  const state: StateName =
    budget.state === 'exact' ? 'PASS' : budget.state === 'partial' ? 'REVIEW' : 'UNKNOWN';
  return (
    <div className={styles['row']}>
      <span className={styles['term']}>{term}</span>
      <span className={styles['value']}>
        <StateBadge
          state={state}
          label={budgetWord(budget.state)}
          count={budget.state === 'partial' ? budget.unattributed.length : undefined}
          note={budget.reason ?? undefined}
        />
        {/* The figure sits beside the state word rather than inside it: a bare number with no
            evidence label would be a number without context (DESIGN.md 9). */}
        <span className={styles['mono']}>{formatSize(budget.bytes, unit)}</span>
      </span>
    </div>
  );
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
  const state: StateName = capabilityState(value);
  return (
    <div className={styles['row']}>
      <span className={styles['term']}>{term}</span>
      <span className={styles['value']}>
        <StateBadge state={state} label={value} />
      </span>
    </div>
  );
}

/**
 * Presentation mapping only: the shell already decided availability, this turns the word into
 * one of the five frozen states. `not-provided` is N/A rather than UNKNOWN because the user
 * chose not to supply that input; it is not missing evidence.
 */
function capabilityState(value: string): StateName {
  switch (value) {
    case 'supported':
    case 'available':
    case 'provided':
      return 'PASS';
    case 'partial':
      return 'REVIEW';
    case 'unsupported':
      return 'BLOCK';
    case 'unavailable':
    case 'not-provided':
      return 'N/A';
    default:
      return 'UNKNOWN';
  }
}

function Section({ title, children }: { readonly title: string; readonly children: ReactNode }) {
  return (
    <section className={styles['section']} aria-label={title}>
      <h2>{title}</h2>
      <div className={styles['rows']}>{children}</div>
    </section>
  );
}

function Row({
  term,
  value,
  mono = false,
  note,
  title,
  count,
}: {
  readonly term: string;
  readonly value: string;
  readonly mono?: boolean | undefined;
  readonly note?: string | null;
  readonly title?: string | undefined;
  readonly count?: number | undefined;
}) {
  return (
    <div className={styles['row']}>
      <span className={styles['term']}>{term}</span>
      <span className={cx(styles['value'], mono ? styles['mono'] : undefined)} title={title}>
        {value}
        {count === undefined ? null : <span className={styles['count']}>{count}</span>}
        {note === undefined || note === null || note.length === 0 ? null : (
          <span className={styles['note']}>{note}</span>
        )}
      </span>
    </div>
  );
}
