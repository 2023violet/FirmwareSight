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
 *
 * U1P-R1 added the question this page could not previously ask. It holds a run and an analysis that arrived
 * through different doors, on different pages, at different times, and it used to treat "I have a run" as
 * "I have an answer for the build above". `readinessScope` is the one derived read that separates them, and
 * it changes no fact: the run keeps its own verdict on the page that made it, and this page stops borrowing it
 * for a build the gate never saw.
 *
 * U1P-R2 answers the same question for the rest of the page. The Gate card learned to hold its tongue about an
 * unanalyzed selection while the capability band and the figures kept speaking for the previous build, and the
 * MAP cell took its pill from one artifact and its sentence from another. A reader who selects a second
 * `firmware.elf` now meets a band that names that file and says it has not been analyzed, and everything that
 * belongs to the earlier result is gathered under `Previous analysis`, in the region that carries its snapshot
 * identity. `Analyze.tsx` already makes exactly this separation for its own report; this page was the last one
 * that had not.
 */

import styles from './Overview.module.css';
import type { ReactNode } from 'react';

import { Button, LinkButton } from './components/Button';
import { Chip, StatusStrip } from './components/Chip';
import { Page } from './components/Layout';
import { PageHeader } from './components/PageHeader';
import { EmptyState, Band, Panel, SummaryCard } from './components/Panel';
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

/**
 * Whether the Gate run in hand is about the build this page is describing.
 *
 * Derived, read-only, and derived from identities the shell already owns: nothing here computes a verdict, and
 * nothing here decides which build a run judged - `GateRunDto.snapshotId` already said that, in Core, when the
 * run was made. U1P-R1 adds this one question because the page could not previously ask it: `App` kept
 * `analyzedSelectionId` for `Analyze` and never passed it here, so a run made for one build rendered as the
 * shipping answer about another.
 */
type ReadinessScope =
  | { readonly kind: 'current' }
  | { readonly kind: 'otherBuild' }
  | { readonly kind: 'selectionPending' }
  | { readonly kind: 'noAnalysis' }
  | { readonly kind: 'otherPolicy' };

/**
 * The selection this page is holding but has no analysis for, or `null` when the selection is the one that
 * produced `summary`.
 *
 * The same identity rule `Analyze.tsx:136` uses for its own pending badge: the shell's selection handle, never
 * a file name, because two artifacts chosen in one session can both be called `firmware.elf`. U1P-R1 read it
 * only for the Gate verdict; U1P-R2 reads it once for the whole page, because a capability pill and a key
 * figure are exactly as much another build's facts as a verdict is. Returning the selection rather than a
 * boolean lets the caller render it without re-deriving the same comparison.
 */
function pendingSelectionOf(
  selection: SelectionDto | null,
  analyzedSelectionId: string | null,
): SelectionDto | null {
  return selection !== null && selection.selectionId !== analyzedSelectionId ? selection : null;
}

function readinessScope(
  gateRun: GateRunDto,
  summary: AnalysisSummaryDto | null,
  selection: SelectionDto | null,
  analyzedSelectionId: string | null,
  project: ProjectContextDto | null,
): ReadinessScope {
  if (pendingSelectionOf(selection, analyzedSelectionId) !== null) {
    return { kind: 'selectionPending' };
  }
  if (summary === null) {
    return { kind: 'noAnalysis' };
  }
  if (gateRun.snapshotId !== summary.identity.snapshotId) {
    return { kind: 'otherBuild' };
  }
  // Both values are real 64-hex fingerprints, and a default-policy run fingerprints the default policy, so a
  // difference here is a fact rather than a missing value. With no project loaded there is nothing to compare
  // and nothing is claimed.
  if (project !== null && project.policySha256 !== gateRun.policySha256) {
    return { kind: 'otherPolicy' };
  }
  return { kind: 'current' };
}

/** The neutral word for the answer this page cannot give, plus the sentence that explains why. */
const NOT_CURRENT: Record<
  Exclude<ReadinessScope['kind'], 'current'>,
  { readonly label: string; readonly headline: string; readonly retained: string }
> = {
  otherBuild: {
    label: 'Not assessed for this build',
    headline:
      'The latest Gate run judged a different build, so this page has no shipping answer about the build it is describing.',
    retained: 'Retained run for a different build',
  },
  selectionPending: {
    label: 'Not assessed for the selected artifact',
    headline:
      'The artifact you selected has not been analyzed, so the run below cannot be its shipping answer.',
    retained: 'Retained run for the previous build',
  },
  noAnalysis: {
    label: 'No analysis in this session',
    headline:
      'This session has analyzed nothing, so there is no build here to pair the stored run with.',
    retained: 'Retained run with no build to compare against',
  },
  otherPolicy: {
    label: 'Not assessed under the policy loaded now',
    headline:
      'The run below was judged under a different policy fingerprint, so its result does not answer for the policy loaded now.',
    retained: 'Retained run under a different policy',
  },
};

export function Overview({
  summary,
  selection,
  analyzedSelectionId,
  gateRun,
  project,
  unit,
  onOpen,
}: {
  readonly summary: AnalysisSummaryDto | null;
  readonly selection: SelectionDto | null;
  /** Which selection produced `summary`; it is the shell's handle, and it moves with the summary. */
  readonly analyzedSelectionId: string | null;
  readonly gateRun: GateRunDto | null;
  readonly project: ProjectContextDto | null;
  readonly unit: SizeUnit;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
  // One derivation, three surfaces: the header, the input band and the figures all answer to the same question,
  // and §4.I is the reason the header cannot keep naming the retained artifact whenever it is not current.
  const pending = pendingSelectionOf(selection, analyzedSelectionId);

  const meta: ReactNode[] = [];
  if (project !== null) {
    meta.push(<span key="project">{project.projectName}</span>);
  }
  if (summary !== null && pending === null) {
    meta.push(<span key="artifact">{`artifact ${summary.artifact.fileName}`}</span>);
    meta.push(<span key="snapshot">snapshot {truncateMiddle(summary.identity.snapshotId, 8)}</span>);
  } else if (selection !== null) {
    meta.push(<span key="selected">selected {selection.fileName}, not analyzed yet</span>);
    if (summary !== null) {
      meta.push(
        <span key="previous">{`previous analysis ${truncateMiddle(summary.identity.snapshotId, 8)}`}</span>,
      );
    }
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
      ) : pending !== null ? (
        <SelectedRow selection={pending} onOpen={onOpen} />
      ) : (
        <InputRow summary={summary} selection={selection} onOpen={onOpen} />
      )}

      <Readiness
        gateRun={gateRun}
        summary={summary}
        selection={selection}
        analyzedSelectionId={analyzedSelectionId}
        project={project}
        onOpen={onOpen}
      />

      {summary === null || pending !== null ? null : <Facts summary={summary} unit={unit} />}
      {summary === null || pending === null ? null : (
        <PreviousAnalysis summary={summary} candidate={pending} unit={unit} />
      )}
    </Page>
  );
}

/**
 * The three capability cells, every one of them stated from a single summary.
 *
 * `mapDetail` and the two actions are parameters because the same three cells answer for two different
 * subjects - the build that is current, and a result retained from an earlier one - and U1P-V2-02 was exactly
 * the page mixing one subject's pill with the other's sentence. A caller now has to say, per cell, which
 * artifact it is talking about; the shape will not let it borrow one line from somewhere else.
 */
function capabilityCells(
  summary: AnalysisSummaryDto,
  mapDetail: ReactNode,
  actions: { readonly elf: ReactNode; readonly map: ReactNode },
): ReactNode[] {
  const { capabilities, artifact } = summary;
  return [
    <Panel
      key="elf"
      bare
      title="ELF"
      hint={<StateBadge variant="chip" state={capabilityState(capabilities.elf)} label={capabilities.elf} />}
    >
      <p className={styles['inputDetail']}>
        {`${artifact.fileName} · ${formatSize(artifact.byteSize, 'bytes')} · sha256 ${truncateMiddle(artifact.sha256, 6)}`}
      </p>
      {actions.elf}
    </Panel>,
    <Panel
      key="map"
      bare
      title="MAP"
      hint={<StateBadge variant="chip" state={capabilityState(capabilities.map)} label={capabilities.map} />}
    >
      <p className={styles['inputDetail']}>{mapDetail}</p>
      {actions.map}
    </Panel>,
    <Panel
      key="git"
      bare
      title="Git"
      hint={<StateBadge variant="chip" state={capabilityState(capabilities.git)} label={capabilities.git} />}
    >
      {/* U1P-R3 §8 found this sentence crediting the fact to the wrong source. The pill is `capabilities.git`,
          which the analysis path leaves at Core's default (`firmwaresight-artifact/src/pipeline.rs` builds
          every snapshot from `Capabilities::elf_only()`); no gate rule writes it, and `git.clean` reads the
          *opened project's* workspace rather than any repository this artifact came from. The cell now names
          the one thing that actually produced the value, and leaves the rule's verdict on the page that
          judged it. */}
      <p className={styles['inputDetail']}>
        Core&rsquo;s analysis capability for Git provenance. The <code>git.clean</code> rule is the
        gate&rsquo;s separate question, about the opened project.
      </p>
    </Panel>,
  ];
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
  // One band, three cells, hairlines between them - which is what the reference draws and what this
  // component asked for since U1. The cells stay named regions because they are three separate answers a
  // reader may want to jump between, and a band is a layout, not an excuse to flatten semantics.
  // This band is only ever rendered when the summary describes the selection in hand, so its two facts -
  // Core's capability and the selection's MAP attachment - are the same artifact's by construction.
  // The wrapper is the U1P-R3 F2 fix and nothing else: it asks this one band to fill the line its cells wrap
  // onto, because at the frozen 1024 px width three capability cells land in a two-track row and the leftover
  // track showed the band's own background as a fourth cell that does not exist. `components/Panel` is not
  // touched, so the bands on Analyze, Compare, Release and History are the ones that were approved.
  return (
    <div className={styles['fillBand']}>
      <Band label="Input capabilities">
        {capabilityCells(
          summary,
          selection?.mapAttached
            ? (selection.mapFileName ?? 'attached')
            : 'Symbol-level analysis depends on it',
          {
            elf: <LinkButton onClick={() => onOpen('analyze')}>Open Analyze</LinkButton>,
            map: <LinkButton onClick={() => onOpen('analyze')}>Attach one on Analyze</LinkButton>,
          },
        )}
      </Band>
    </div>
  );
}

/**
 * The selected artifact, stated with only what a selection can prove: its name and whether a MAP came with it.
 *
 * There is deliberately no capability pill here. An unanalyzed file is not an `UNKNOWN` verdict - it is the
 * absence of one, and §4.J refuses to let this page name a state Core has not been asked about. The two lines
 * below are the whole of what the shell knows before Analyze runs, and the action is the one move that changes
 * them.
 */
function SelectedRow({
  selection,
  onOpen,
}: {
  readonly selection: SelectionDto;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
  return (
    <Band label="Selected artifact">
      <Panel bare title="Artifact">
        <p className={styles['inputDetail']}>{`${selection.fileName} · not analyzed yet`}</p>
        <LinkButton onClick={() => onOpen('analyze')}>Open Analyze</LinkButton>
      </Panel>
      <Panel bare title="MAP">
        <p className={styles['inputDetail']}>
          {selection.mapAttached
            ? `MAP attached: ${selection.mapFileName ?? 'attached'}`
            : 'No MAP attached to this selection'}
        </p>
        <LinkButton onClick={() => onOpen('analyze')}>
          {selection.mapAttached ? 'Change it on Analyze' : 'Attach one on Analyze'}
        </LinkButton>
      </Panel>
    </Band>
  );
}

/**
 * Everything this page holds about some other artifact, inside one region that says so.
 *
 * §4.B allows retained history only under an explicit heading carrying its own snapshot identity, and §4.I
 * requires that label to sit in the immediate visual vicinity of the figures rather than in a warning a reader
 * has to go looking for. The attribution follows `Analyze.tsx`'s accepted device, including the case that
 * motivated it: two artifacts can share one leaf name, and "it is not an analysis of firmware.elf" under
 * "Previous analysis of firmware.elf" would state nothing.
 */
function PreviousAnalysis({
  summary,
  candidate,
  unit,
}: {
  readonly summary: AnalysisSummaryDto;
  readonly candidate: SelectionDto;
  readonly unit: SizeUnit;
}) {
  const { artifact } = summary;
  const sameName = candidate.fileName === artifact.fileName;
  const attribution = sameName
    ? ' The artifact selected now has the same name and has not been analyzed; nothing here describes it.'
    : ` It is not an analysis of ${candidate.fileName}: that selection has not been analyzed yet.`;

  return (
    <Panel title="Previous analysis" hint="retained from this session">
      <p className={styles['stale']} role="note">
        {`Previous analysis of ${artifact.fileName}.${attribution}`}
      </p>
      <p className={styles['identity']}>{`Snapshot ${summary.identity.snapshotId}`}</p>
      <div className={styles['fillBand']}>
        <Band label="Previous input capabilities">
          {capabilityCells(
            summary,
            `As reported by the analysis of ${artifact.fileName}.`,
            { elf: null, map: null },
          )}
        </Band>
      </div>
      <Band narrow label="Previous key figures">
        {figureCells(summary, unit)}
      </Band>
    </Panel>
  );
}

/**
 * The ship question, and the one thing that has to be true before this page may answer it: the run in hand
 * has to be about the build the page is describing.
 *
 * `DESIGN.md` 9 forbids presenting this as a legal, security or compliance conclusion, so the sentence
 * that limits it is part of the card rather than a footnote a reader has to remember to look for. The
 * counts come from `GateCountsDto` and the severity is Core's disposition aggregate; the sentence is
 * `gateVerdictSentence`, the same builder the gate page uses, so the two pages cannot state one run two
 * ways. U1P-R1 does not change any of that - it stops the page from attributing that aggregate to a
 * build the gate never looked at.
 */
function Readiness({
  gateRun,
  summary,
  selection,
  analyzedSelectionId,
  project,
  onOpen,
}: {
  readonly gateRun: GateRunDto | null;
  readonly summary: AnalysisSummaryDto | null;
  readonly selection: SelectionDto | null;
  readonly analyzedSelectionId: string | null;
  readonly project: ProjectContextDto | null;
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

  const scope = readinessScope(gateRun, summary, selection, analyzedSelectionId, project);
  return scope.kind === 'current'
    ? <CurrentReadiness gateRun={gateRun} summary={summary} onOpen={onOpen} />
    : <NotCurrentReadiness gateRun={gateRun} summary={summary} scope={scope} project={project} onOpen={onOpen} />;
}

/** The run's own answer, shown as the page's answer - which is only reachable when the subjects match. */
function CurrentReadiness({
  gateRun,
  summary,
  onOpen,
}: {
  readonly gateRun: GateRunDto;
  readonly summary: AnalysisSummaryDto | null;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
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
      <div className={styles['verdict']} role="group" aria-label="Readiness for the build this page describes">
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
        {/* U1P-R1 §5.A: the verdict names the build it judged, in full, as text inside the answer itself. A
            truncated value is not an answer a reader can check, and a `title` attribute is not reachable by a
            keyboard reader, so this line is never shortened. */}
        <p className={styles['identity']}>{`Judged snapshot ${gateRun.snapshotId}`}</p>
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

/**
 * The page has a run and cannot answer for the build it is describing: §5.H's invariant lives here.
 *
 * What this state says is the whole of what it knows - which run exists, which snapshot that run judged, which
 * snapshot the page is describing, and that the answer belongs to Release Gate. What it deliberately does not
 * do is repeat the run's verdict sentence or its counts. §5.B permits showing historical details if they are
 * labelled and subordinate, and the smaller truthful shape was chosen over the more informative-looking one on
 * purpose: a green word that a reader has to scroll back up to disown is exactly the failure this unit exists
 * to close. Nothing is deleted - the run and every finding of it are still on the page that made them.
 */
function NotCurrentReadiness({
  gateRun,
  summary,
  scope,
  project,
  onOpen,
}: {
  readonly gateRun: GateRunDto;
  readonly summary: AnalysisSummaryDto | null;
  readonly scope: Exclude<ReadinessScope, { readonly kind: 'current' }>;
  readonly project: ProjectContextDto | null;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
  const words = NOT_CURRENT[scope.kind];

  return (
    <Panel
      title="Can we ship now?"
      hint={
        <>
          Latest evidence: run <Chip mono>{truncateMiddle(gateRun.runId, 8)}</Chip> · {gateRun.createdAt}
        </>
      }
    >
      <div className={styles['verdict']} role="group" aria-label="Readiness for the build this page describes">
        <StateBadge variant="chip" state="UNKNOWN" label={words.label} />
        <p className={styles['verdictSentence']}>{words.headline}</p>
      </div>

      <div className={styles['pastRun']} role="group" aria-label={words.retained}>
        <p className={styles['identity']}>{`Judged snapshot ${gateRun.snapshotId}`}</p>
        {summary === null ? null : (
          <p className={styles['identity']}>{`Snapshot this page describes ${summary.identity.snapshotId}`}</p>
        )}
        {scope.kind === 'otherPolicy' && project !== null ? (
          <p className={styles['identity']}>
            {`Run policy ${gateRun.policySha256} · loaded policy ${project.policySha256}`}
          </p>
        ) : null}
        <p className={styles['quiet']}>
          Its verdict and its findings stay on the Release Gate page, where the run was made, and nothing here
          restates them for a build the gate did not judge.
        </p>
      </div>

      <div className={styles['nextSteps']}>
        <span className={styles['nextLabel']}>Next steps</span>
        {/* Navigation only in every arm: choosing a build and running the gate is the Release page's work, and
            arriving there must never imply a run happened. §4.H is why the pending case leads with Analyze.
            Its primary control used to read "Choose this build on Release Gate", and the build Release
            preselects when the reader clicks is the retained one - so the strongest label on the page promised
            a choice about a file it was not about to choose. */}
        {scope.kind === 'selectionPending' ? (
          <>
            <Button variant="primary" onClick={() => onOpen('analyze')}>
              Analyze selected artifact
            </Button>
            <Button onClick={() => onOpen('release')}>View previous Gate run</Button>
          </>
        ) : (
          <>
            <Button variant="primary" onClick={() => onOpen('release')}>
              Choose this build on Release Gate
            </Button>
            {summary === null ? (
              <Button onClick={() => onOpen('analyze')}>Analyze an artifact</Button>
            ) : null}
          </>
        )}
      </div>

      <p className={styles['scope']}>
        This is FirmwareSight policy readiness for the run named above. It is not a legal, security or
        product-compliance conclusion, and it never replaces a person&rsquo;s decision.
      </p>
    </Panel>
  );
}

/** The four figures, always read from one summary; the band around them decides whose figures they are. */
function figureCells(summary: AnalysisSummaryDto, unit: SizeUnit): ReactNode[] {
  const { memory, capabilities, evidenceSummary } = summary;
  const flash = memory.nonvolatileImageFootprint;
  const ram = memory.runtimeRamFootprint;

  return [
    <SummaryCard
      key="flash"
      label="Flash footprint"
      value={flash.bytes === null ? 'Unknown' : formatSize(flash.bytes, unit)}
      context={`${flash.classification} · ${flash.state}${flash.reason === null ? '' : ` · ${flash.reason}`}`}
    />,
    <SummaryCard
      key="ram"
      label="Runtime RAM"
      value={ram.bytes === null ? 'Unknown' : formatSize(ram.bytes, unit)}
      context={`${ram.classification} · ${ram.state}${ram.reason === null ? '' : ` · ${ram.reason}`}`}
    />,
    <SummaryCard
      key="symbols"
      label="Symbols"
      value={String(summary.symbolCount)}
      context={`capability ${capabilities.symbols}`}
    />,
    <SummaryCard
      key="evidence"
      label="Evidence"
      value={String(evidenceSummary.total)}
      context={`${String(evidenceSummary.observed)} observed · ${String(evidenceSummary.derived)} derived · ${String(evidenceSummary.declared)} declared · ${String(evidenceSummary.unknown)} unknown`}
    />,
  ];
}

/** The four figures a reader scans before they read anything, subordinate to the verdict above them. */
function Facts({
  summary,
  unit,
}: {
  readonly summary: AnalysisSummaryDto;
  readonly unit: SizeUnit;
}) {
  return <Band narrow label="Key figures">{figureCells(summary, unit)}</Band>;
}
