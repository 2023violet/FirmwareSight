import { useCallback, useEffect, useState, type ReactNode } from 'react';

import { StateBadge, type StateName } from './components/StateBadge';
import { formatBytes, formatOptional, truncateMiddle } from './format';
import { getAnalysisSummary, listFixtures } from './ipc/bridge';
import type {
  AnalysisSummaryDto,
  BudgetDto,
  ErrorEnvelopeDto,
  FixtureKey,
  FixtureOptionDto,
} from './ipc/types';
import styles from './App.module.css';
import { cx } from './styles/classnames';

type Phase =
  | { readonly kind: 'loading' }
  | { readonly kind: 'idle' }
  | { readonly kind: 'ready'; readonly summary: AnalysisSummaryDto }
  | { readonly kind: 'failed'; readonly envelope: ErrorEnvelopeDto };

/** The closed set, taken from the generated union rather than a second hand-written list. */
const FIXTURE_KEYS: readonly FixtureKey[] = ['p0_basic', 'p0_dual_region'];

function toFixtureKey(value: string): FixtureKey | null {
  return FIXTURE_KEYS.find((key) => key === value) ?? null;
}

/**
 * The shell owns the fixture labels, so nothing here repeats them. Before the first reply there
 * is nothing to name, and the only honest option is the loading placeholder - keyed to the
 * current selection so the control never holds a value it did not offer.
 */
function fixtureOptions(
  fixtures: readonly FixtureOptionDto[],
  selected: FixtureKey,
): readonly FixtureOptionDto[] {
  return fixtures.length === 0
    ? [{ key: selected, label: 'Loading fixtures...' }]
    : fixtures;
}

export function App() {
  const [fixtures, setFixtures] = useState<readonly FixtureOptionDto[]>([]);
  const [selected, setSelected] = useState<FixtureKey>('p0_dual_region');
  const [phase, setPhase] = useState<Phase>({ kind: 'idle' });

  useEffect(() => {
    let cancelled = false;
    void listFixtures().then((outcome) => {
      if (cancelled) {
        return;
      }
      if (!outcome.ok) {
        setPhase({ kind: 'failed', envelope: outcome.envelope });
        return;
      }
      const unknown = outcome.value.filter((option) => toFixtureKey(option.key) === null);
      if (unknown.length > 0) {
        // The Rust enum and the generated union are meant to be the same contract. If the shell
        // offers a key the types do not name, that is drift, and saying so beats quietly
        // dropping an option the user could have picked.
        setPhase({
          kind: 'failed',
          envelope: internalError(
            'The shell offered a fixture this build does not know.',
            unknown.map((option) => option.key).join(', '),
            'Rebuild after running cargo test -p firmwaresight-desktop to refresh the generated IPC types.',
          ),
        });
        return;
      }
      setFixtures(outcome.value);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const analyze = useCallback(async () => {
    setPhase({ kind: 'loading' });
    const outcome = await getAnalysisSummary(selected);
    setPhase(
      outcome.ok
        ? { kind: 'ready', summary: outcome.value }
        : { kind: 'failed', envelope: outcome.envelope },
    );
  }, [selected]);

  const summary = phase.kind === 'ready' ? phase.summary : null;

  return (
    <main className={styles['page']}>
      <header className={styles['header']}>
        <h1>P0 Technical Summary</h1>
        <p className={styles['subhead']}>
          Same Core facts the CLI prints, read through the Desktop IPC boundary. No diff, no gate,
          no release action exists in this build.
        </p>
      </header>

      <section className={styles['controls']} aria-label="Artifact selection">
        <label className={styles['field']}>
          <span>Fixture</span>
          <select
            className={styles['select']}
            value={selected}
            disabled={phase.kind === 'loading'}
            onChange={(event) => {
              const next = toFixtureKey(event.target.value);
              if (next !== null) {
                setSelected(next);
              }
            }}
          >
            {fixtureOptions(fixtures, selected).map((option) => (
              <option key={option.key} value={option.key}>
                {option.label}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          className={styles['primary']}
          onClick={() => {
            void analyze();
          }}
          disabled={phase.kind === 'loading'}
        >
          Analyze
        </button>
      </section>

      {phase.kind === 'loading' ? (
        // The only asynchronous fact on the screen. Without a live region it is visible to sighted
        // users and silent for everyone else, because the button's own disabled state says nothing
        // about when the work finished.
        <p className={styles['status']} role="status" aria-live="polite">
          Analyzing…
        </p>
      ) : null}
      {phase.kind === 'failed' ? <ErrorPanel envelope={phase.envelope} /> : null}
      {phase.kind === 'idle' ? (
        <p className={styles['status']}>
          Nothing has been analyzed in this session yet. Choose a fixture and run Analyze.
        </p>
      ) : null}

      {summary === null ? null : <Report summary={summary} />}
    </main>
  );
}

/**
 * An error the UI found for itself rather than one the shell reported. It carries the internal
 * code because that is what it is; the shell's own codes never pass through here.
 */
function internalError(message: string, details: string, remediation: string): ErrorEnvelopeDto {
  return {
    code: 'ERR-INTERNAL-9001',
    message,
    operationId: 'unavailable',
    details,
    remediation,
  };
}

function Report({ summary }: { readonly summary: AnalysisSummaryDto }) {
  const { artifact, memory, capabilities, evidenceSummary } = summary;

  return (
    <div className={styles['report']}>
      <Section title="Artifact">
        <Row term="SHA-256" value={artifact.sha256} mono title={artifact.sha256} />
        <Row term="Size" value={formatBytes(artifact.byteSize)} mono />
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
        <BudgetRow term="Nonvolatile / load image" budget={memory.nonvolatileImageFootprint} />
        <BudgetRow term="Runtime RAM" budget={memory.runtimeRamFootprint} />
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
        <Row term="Device metadata excluded" value={formatBytes(memory.excludedMetadataBytes)} mono />
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
          {summary.identity.normalizationVersion} · fwsight {summary.identity.createdByFwsightVersion}
        </span>
      </footer>
    </div>
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

function BudgetRow({ term, budget }: { readonly term: string; readonly budget: BudgetDto }) {
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
        <span className={styles['mono']}>{formatBytes(budget.bytes)}</span>
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
