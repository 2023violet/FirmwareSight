import { useCallback, useState, type ReactNode } from 'react';

import { Details } from './Details';
import { StateBadge, type StateName } from './components/StateBadge';
import { formatOptional, formatSize, truncateMiddle, type SizeUnit } from './format';
import { analyzeSelection, attachMap, clearMap, selectArtifact } from './ipc/bridge';
import type {
  AnalysisSummaryDto,
  BudgetDto,
  ErrorEnvelopeDto,
  SelectionDto,
} from './ipc/types';
import styles from './App.module.css';
import { cx } from './styles/classnames';

export function App() {
  const [selection, setSelection] = useState<SelectionDto | null>(null);
  const [lastGood, setLastGood] = useState<AnalysisSummaryDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [analyzing, setAnalyzing] = useState(false);
  // US-001's unit switch. One piece of state for the whole screen, because a summary in bytes next
  // to a table in KiB would be two answers to one question. It changes no query and no data.
  const [unit, setUnit] = useState<SizeUnit>('bytes');

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
    setSelection(outcome.value);
    setError(null);
  }, []);

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
      setSelection(outcome.value);
    }
  }, [selection]);

  const removeMap = useCallback(async () => {
    if (selection === null) {
      return;
    }
    const outcome = await clearMap(selection.selectionId);
    if (!outcome.ok) {
      setError(outcome.envelope);
      return;
    }
    setSelection(outcome.value);
  }, [selection]);

  /**
   * Analyze, and keep the previous result on the failure path.
   *
   * A failed analysis is not a snapshot: `lastGood` only ever moves when the shell returns a
   * summary. What changes on failure is the error, plus the label on the surviving report - a
   * reader must not mistake the previous artifact's numbers for the candidate that just failed.
   */
  const analyze = useCallback(async () => {
    if (selection === null) {
      return;
    }
    setAnalyzing(true);
    const outcome = await analyzeSelection(selection.selectionId);
    setAnalyzing(false);

    if (outcome.ok) {
      setLastGood(outcome.value);
      setError(null);
      return;
    }
    setError(outcome.envelope);
  }, [selection]);

  const summary = lastGood;

  return (
    <main className={styles['page']}>
      <header className={styles['header']}>
        <h1>Analyze</h1>
        <p className={styles['subhead']}>
          Choose a firmware artifact and FirmwareSight reports the Core facts it can prove. No diff,
          no gate, no release action exists in this build.
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
      {error === null ? null : <ErrorPanel envelope={error} />}
      {summary === null && !analyzing && error === null ? (
        <section className={styles['status']} aria-label="No analysis yet">
          <p>Nothing has been analyzed in this session yet. Choose an artifact and run Analyze.</p>
        </section>
      ) : null}

      {summary === null ? null : (
        <Report
          summary={summary}
          stale={error !== null}
          candidateName={selection?.fileName ?? null}
          unit={unit}
        />
      )}

      {/* One snapshot id, the last one the shell proved. A later attempt that failed cannot repoint
          these tables, because `lastGood` only moves on a summary. */}
      {summary === null ? null : (
        <Details snapshotId={summary.identity.snapshotId} unit={unit} onUnitChange={setUnit} />
      )}
    </main>
  );
}

/**
 * The report, and the honesty flag that comes with it.
 *
 * `stale` is set when a later analysis failed: the numbers below are the last ones FirmwareSight
 * could prove, not a result for the file now sitting in the selector. Naming the candidate it does
 * not describe is what keeps a preserved result from being read as a fresh verdict.
 */
function Report({
  summary,
  stale,
  candidateName,
  unit,
}: {
  readonly summary: AnalysisSummaryDto;
  readonly stale: boolean;
  readonly candidateName: string | null;
  readonly unit: SizeUnit;
}) {
  const { artifact, memory, capabilities, evidenceSummary } = summary;

  return (
    <section
      className={styles['report']}
      aria-label={stale ? 'Last good analysis' : 'Analysis summary'}
    >
      {stale ? (
        <p className={styles['stale']} role="note">
          Previous analysis of {artifact.fileName}.
          {candidateName === null || candidateName === artifact.fileName
            ? ' The failed attempt above produced no result, so nothing here was replaced.'
            : ` It is not an analysis of ${candidateName}.`}
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
        <CapabilityRow term="Object attribution" value={capabilities.objectAttribution} />
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
  if (memory.admissibleForHardBlock) {
    return (
      <StateBadge
        state="PASS"
        label="Admissible for a hard limit"
        note={`Weakest basis ${basis}; region evidence came from the linker.`}
      />
    );
  }
  return (
    <StateBadge
      state="UNKNOWN"
      label="Not admissible for a hard limit"
      note={`Weakest basis ${basis} is name- or flag-derived. Supply the linker MAP to strengthen it.`}
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

function ErrorPanel({ envelope }: { readonly envelope: ErrorEnvelopeDto }) {
  return (
    <section className={styles['error']} role="alert" aria-label="Analysis error">
      <h2>Analysis failed</h2>
      <dl className={styles['errorList']}>
        <div>
          <dt>What happened</dt>
          <dd>{envelope.message}</dd>
        </div>
        <div>
          <dt>Code</dt>
          <dd className={styles['monoSmall']}>{envelope.code}</dd>
        </div>
        {envelope.details === null ? null : (
          <div>
            <dt>Why we know</dt>
            <dd className={styles['monoSmall']}>{envelope.details}</dd>
          </div>
        )}
        {envelope.remediation === null ? null : (
          <div>
            <dt>What to do</dt>
            <dd>{envelope.remediation}</dd>
          </div>
        )}
        <div>
          <dt>Diagnostics ID</dt>
          <dd className={styles['monoSmall']}>{envelope.operationId}</dd>
        </div>
      </dl>
    </section>
  );
}
