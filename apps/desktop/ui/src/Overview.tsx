/**
 * The Overview page: what this workspace is holding, whether anything blocks the build, and what to
 * open next.
 *
 * U1's one new surface, and it is deliberately made of nothing that did not already exist. Every fact on
 * this screen is a fact the shell already holds - the last analysis the reader proved, the gate run the
 * Release page last loaded, the project policy that was opened - or a statement Core already made about
 * them. No command is added for it, no number is computed here, and every action is navigation to the
 * page that can actually do the thing. That is the difference between a summary and a second source of
 * truth, and it is the reason this page can name the run its verdict came from without being able to
 * change what that run said.
 *
 * The reference screen (`assets/ui-mockups/FS-UI-01-Overview.png`) shows a "Re-run Release Gate" button
 * that executes the gate. This page does not: running the gate belongs to Release, where the policy, the
 * build and the baseline are chosen and where an overwrite is confirmed. The button navigates, and
 * `U1_UI_GAP_AUDIT.md` §D records that choice rather than burying it.
 */

import styles from './Overview.module.css';
import type { ReactNode } from 'react';

import { Button, LinkButton } from './components/Button';
import { Chip, StatusStrip } from './components/Chip';
import { Page } from './components/Layout';
import { PageHeader } from './components/PageHeader';
import { EmptyState, Panel, SummaryCard, SummaryRow } from './components/Panel';
import { StateBadge } from './components/StateBadge';
import { formatSize, truncateMiddle, type SizeUnit } from './format';
import type {
  AnalysisSummaryDto,
  GateRunDto,
  ProjectContextDto,
  SelectionDto,
} from './ipc/types';
import {
  capabilityState,
  gateStateName,
  gateVerdictSentence,
  severityState,
} from './stateWords';

/** The page that can carry out a next step. Overview moves the reader; it never acts for them. */
export type OverviewTarget = 'analyze' | 'compare' | 'release' | 'history';

export function Overview({
  summary,
  selection,
  gateRun,
  project,
  unit,
  onOpen,
}: {
  readonly summary: AnalysisSummaryDto | null;
  readonly selection: SelectionDto | null;
  readonly gateRun: GateRunDto | null;
  readonly project: ProjectContextDto | null;
  readonly unit: SizeUnit;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
  const meta: ReactNode[] = [];
  if (project !== null) {
    meta.push(<span key="project">{project.projectName}</span>);
  }
  if (summary !== null) {
    meta.push(<span key="artifact">{`artifact ${summary.artifact.fileName}`}</span>);
    meta.push(<span key="snapshot">snapshot {truncateMiddle(summary.identity.snapshotId, 8)}</span>);
  } else if (selection !== null) {
    meta.push(<span key="selected">selected {selection.fileName}, not analyzed yet</span>);
  } else {
    meta.push(<span key="none">no artifact selected</span>);
  }
  if (gateRun !== null) {
    meta.push(<span key="gate">gate run {truncateMiddle(gateRun.runId, 8)}</span>);
  }

  return (
    <Page>
      <PageHeader
        title="Overview"
        meta={meta}
        actions={
          <>
            <Button onClick={() => onOpen('compare')}>Compare builds</Button>
            <Button onClick={() => onOpen('history')}>Bundle &amp; History</Button>
            <Button variant="primary" onClick={() => onOpen('release')}>
              Open Release Gate
            </Button>
          </>
        }
      />

      {summary === null ? (
        <EmptyState
          role="status"
          message="Nothing has been analyzed in this session yet, so there is nothing to summarize."
          nextStep={
            <Button variant="primary" onClick={() => onOpen('analyze')}>
              Choose an artifact
            </Button>
          }
        />
      ) : (
        <InputRow summary={summary} selection={selection} onOpen={onOpen} />
      )}

      <Readiness gateRun={gateRun} summary={summary} onOpen={onOpen} />

      {summary === null ? null : <Facts summary={summary} gateRun={gateRun} unit={unit} />}
    </Page>
  );
}

/** The three inputs the product reads, each with the capability Core reported for it. */
function InputRow({
  summary,
  selection,
  onOpen,
}: {
  readonly summary: AnalysisSummaryDto;
  readonly selection: SelectionDto | null;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
  const { capabilities, artifact } = summary;

  return (
    <SummaryRow label="Input capabilities">
      <Panel
        title="ELF"
        hint={<StateBadge variant="chip" state={capabilityState(capabilities.elf)} label={capabilities.elf} />}
      >
        <p className={styles['inputDetail']}>
          {`${artifact.fileName} · ${formatSize(artifact.byteSize, 'bytes')} · sha256 ${truncateMiddle(artifact.sha256, 6)}`}
        </p>
        <LinkButton onClick={() => onOpen('analyze')}>Open Analyze</LinkButton>
      </Panel>
      <Panel
        title="MAP"
        hint={<StateBadge variant="chip" state={capabilityState(capabilities.map)} label={capabilities.map} />}
      >
        <p className={styles['inputDetail']}>
          {selection?.mapAttached
            ? (selection.mapFileName ?? 'attached')
            : 'Symbol-level analysis depends on it'}
        </p>
        <LinkButton onClick={() => onOpen('analyze')}>Attach one on Analyze</LinkButton>
      </Panel>
      <Panel
        title="Git"
        hint={<StateBadge variant="chip" state={capabilityState(capabilities.git)} label={capabilities.git} />}
      >
        <p className={styles['inputDetail']}>
          Commit provenance is judged by the <code>git.clean</code> rule, from the repository this build
          came from.
        </p>
      </Panel>
    </SummaryRow>
  );
}

/**
 * The ship question, answered with the run's own aggregate.
 *
 * `DESIGN.md` 9 forbids presenting this as a legal, security or compliance conclusion, so the sentence
 * that limits it is part of the card rather than a footnote a reader has to remember to look for. The
 * counts come from `GateCountsDto` and the severity is Core's disposition aggregate; the sentence is
 * `gateVerdictSentence`, the same builder the gate page uses, so the two pages cannot state one run two
 * ways.
 */
function Readiness({
  gateRun,
  summary,
  onOpen,
}: {
  readonly gateRun: GateRunDto | null;
  readonly summary: AnalysisSummaryDto | null;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
  if (gateRun === null) {
    return (
      <Panel title="Can we ship now?" hint="no gate run in this session">
        <EmptyState
          message="FirmwareSight has not run the gate in this session, so there is no verdict to show."
          nextStep={
            <Button variant="primary" onClick={() => onOpen('release')}>
              Run it on Release Gate
            </Button>
          }
        />
      </Panel>
    );
  }

  const state = severityState(gateRun.dispositionEffectiveSeverity);
  const open = gateRun.findings.filter((finding) => finding.state !== 'PASS' && finding.state !== 'N/A');

  return (
    <Panel
      title="Can we ship now?"
      hint={
        <>
          Latest evidence: run <Chip mono>{truncateMiddle(gateRun.runId, 8)}</Chip> · {gateRun.createdAt}
        </>
      }
    >
      <div className={styles['verdict']}>
        <StateBadge variant="chip" state={state} label={state} />
        <p className={styles['verdictSentence']}>{gateVerdictSentence(gateRun)}</p>
        {/* The same strip the gate page puts under its verdict: a verdict word without the counts it was
            aggregated from is an assertion, and this page states a verdict from a run it cannot open. */}
        <StatusStrip
          counts={{
            PASS: gateRun.counts.pass,
            REVIEW: gateRun.counts.review,
            BLOCK: gateRun.counts.block,
            UNKNOWN: gateRun.counts.unknown,
            'N/A': gateRun.counts.notApplicable,
          }}
          label="Findings by state in this run"
        />
      </div>

      {open.length === 0 ? (
        <p className={styles['quiet']}>
          No finding is open: every rule that could be evaluated passed, and none is waiting on a
          disposition.
        </p>
      ) : (
        <ul className={styles['findings']}>
          {open.slice(0, 4).map((finding) => (
            <li key={finding.id} className={styles['finding']}>
              <StateBadge
                state={gateStateName(finding.state)}
                label={finding.ruleId}
                note={finding.summary}
              />
            </li>
          ))}
          {open.length > 4 ? (
            <li className={styles['more']}>
              {`${String(open.length - 4)} more finding(s) on the Release Gate page.`}
            </li>
          ) : null}
        </ul>
      )}

      <div className={styles['nextSteps']}>
        <span className={styles['nextLabel']}>Next steps</span>
        {/* Both labels name the page rather than repeating the header's `Open Release Gate`: two buttons
            with one accessible name on one screen is a reader's question, not a reader's answer. */}
        <Button variant="primary" onClick={() => onOpen('release')}>
          {state === 'PASS' ? 'Open the run on Release Gate' : 'Resolve on Release Gate'}
        </Button>
        {summary !== null && capabilityState(summary.capabilities.map) !== 'PASS' ? (
          <Button onClick={() => onOpen('analyze')}>Attach a MAP file</Button>
        ) : null}
      </div>

      <p className={styles['scope']}>
        This is FirmwareSight policy readiness for the run named above. It is not a legal, security or
        product-compliance conclusion, and it never replaces a person&rsquo;s decision.
      </p>
    </Panel>
  );
}

/** The four numbers a reader scans before they read anything. */
function Facts({
  summary,
  gateRun,
  unit,
}: {
  readonly summary: AnalysisSummaryDto;
  readonly gateRun: GateRunDto | null;
  readonly unit: SizeUnit;
}) {
  const flash = summary.memory.nonvolatileImageFootprint;
  const evidence = summary.evidenceSummary;

  return (
    <SummaryRow>
      <SummaryCard
        label="Flash footprint"
        value={flash.bytes === null ? 'Unknown' : formatSize(flash.bytes, unit)}
        context={`${flash.classification} · ${flash.state}${flash.reason === null ? '' : ` · ${flash.reason}`}`}
      />
      <SummaryCard
        label="Symbols"
        value={String(summary.symbolCount)}
        context={`capability ${summary.capabilities.symbols}`}
      />
      <SummaryCard
        label="Evidence"
        value={String(evidence.total)}
        context={`${String(evidence.observed)} observed · ${String(evidence.derived)} derived · ${String(evidence.declared)} declared · ${String(evidence.unknown)} unknown`}
      />
      <SummaryCard
        label="Last gate"
        value={gateRun === null ? '—' : truncateMiddle(gateRun.runId, 8)}
        context={gateRun === null ? 'no run in this session' : gateRun.createdAt}
      />
    </SummaryRow>
  );
}
