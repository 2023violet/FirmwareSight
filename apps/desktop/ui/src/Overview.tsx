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
 *
 * R2 (`06_DELIVERY/16`, ADR-0034 D1) changes the order and the weight, not the rules: the page answers first.
 * The ship question leads, styled as the page's one answer and carrying its one primary action; a session
 * with nothing analyzed leads with the step that fixes that instead. The SubjectBar replaces the header's meta
 * line as the place that names the build, and the comparison states its deltas as neutral facts (▲▼).
 */

import styles from './Overview.module.css';
import type { ReactNode } from 'react';


import { Button, LinkButton } from './components/Button';
import { Chip, StatusStrip } from './components/Chip';
import { DeltaValue } from './components/DeltaValue';
import { SubjectBar, type Subject } from './components/SubjectBar';
import { Page } from './components/Layout';
import { PageHeader } from './components/PageHeader';
import { MemoryUsage } from './components/MemoryUsage';
import { EmptyState, Band, Panel, SummaryCard } from './components/Panel';
import { StateBadge } from './components/StateBadge';
import { formatSize, truncateMiddle, type SizeUnit } from './format';
import { useI18n, type Translate } from './i18n';
import type { Catalog } from './i18n/en';
import type {
  AnalysisSummaryDto,
  ByteDeltaDto,
  CompareSummaryDto,
  GateRunDto,
  ProjectContextDto,
  SelectionDto,
} from './ipc/types';
import {
  capabilityState,
  gateStateName,
  gateVerdictSentence,
  severityState,
  analyzedWithMap,
  mapChangedSince,
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

/**
 * The neutral word for the answer this page cannot give, plus the sentence that explains why.
 *
 * The three parts are catalog keys rather than strings because a fourth kind would otherwise be a
 * sentence someone has to remember to translate: the kind is the key, and the type below will not
 * accept a kind without all three of its words.
 */
type NotCurrentKind = Exclude<ReadinessScope['kind'], 'current'>;

/** A key the `common` namespace actually holds, so a typo is a compile error rather than a runtime hole. */
type CommonKey = keyof Catalog['common'];

const NOT_CURRENT_KEYS: Record<
  NotCurrentKind,
  { readonly label: CommonKey; readonly headline: CommonKey; readonly retained: CommonKey }
> = {
  otherBuild: {
    label: 'overview.notCurrent.otherBuild.label',
    headline: 'overview.notCurrent.otherBuild.headline',
    retained: 'overview.notCurrent.otherBuild.retained',
  },
  selectionPending: {
    label: 'overview.notCurrent.selectionPending.label',
    headline: 'overview.notCurrent.selectionPending.headline',
    retained: 'overview.notCurrent.selectionPending.retained',
  },
  noAnalysis: {
    label: 'overview.notCurrent.noAnalysis.label',
    headline: 'overview.notCurrent.noAnalysis.headline',
    retained: 'overview.notCurrent.noAnalysis.retained',
  },
  otherPolicy: {
    label: 'overview.notCurrent.otherPolicy.label',
    headline: 'overview.notCurrent.otherPolicy.headline',
    retained: 'overview.notCurrent.otherPolicy.retained',
  },
};

/** One readiness state's three sentences, read out of the catalog on screen. */
function notCurrentWords(kind: NotCurrentKind, t: Translate) {
  const keys = NOT_CURRENT_KEYS[kind];
  return {
    label: t('common', keys.label),
    headline: t('common', keys.headline),
    retained: t('common', keys.retained),
  };
}

export function Overview({
  summary,
  selection,
  analyzedSelectionId,
  gateRun,
  project,
  comparison = null,
  unit,
  onOpen,
}: {
  readonly summary: AnalysisSummaryDto | null;
  readonly selection: SelectionDto | null;
  /** Which selection produced `summary`; it is the shell's handle, and it moves with the summary. */
  readonly analyzedSelectionId: string | null;
  readonly gateRun: GateRunDto | null;
  readonly project: ProjectContextDto | null;
  /** The comparison this session computed last (M1 F2), or `null` when none was run. */
  readonly comparison?: CompareSummaryDto | null;
  readonly unit: SizeUnit;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
  // One derivation, three surfaces: the subject, the input band and the figures all answer to the same
  // question, and §4.I is the reason the top of the page cannot keep naming the retained artifact whenever it
  // is not current.
  const { t } = useI18n();
  const pending = pendingSelectionOf(selection, analyzedSelectionId);

  /**
   * What the SubjectBar names. A pending selection is named by its file alone - this page has no hash for a
   * file nobody has analyzed - and the current summary by its file and hash. Nothing is named when nothing is
   * held, because an empty page has no subject.
   */
  const subject: Subject | null =
    pending !== null
      ? {
          kind: 'single',
          build: { fileName: pending.fileName, sha256: null, map: pending.mapAttached, mapFileName: pending.mapFileName },
        }
      : summary !== null
        ? {
            kind: 'single',
            build: {
              fileName: summary.artifact.fileName,
              sha256: summary.artifact.sha256,
              // The analysis's own MAP word (R2 review S2-1); the file name only while the selection agrees.
              map: analyzedWithMap(summary),
              mapFileName:
                analyzedWithMap(summary) === true && selection?.mapAttached === true ? selection.mapFileName : null,
            },
          }
        : null;

  /**
   * A session with no analysis leads with the step that makes one; otherwise the ship answer leads (R2 review
   * S2-3: a chosen but unanalyzed file is still "nothing analyzed", and its next step is Analyze).
   */
  const leadWithIntake = summary === null;

  const intake =
    summary === null ? (
      <EmptyState
        role="status"
        message={t('common', 'overview.emptyMessage')}
        nextStep={
          <Button variant="primary" onClick={() => onOpen('analyze')}>
            {selection === null
              ? t('common', 'overview.emptyAction')
              : t('common', 'overview.analyzeSelectedButton')}
          </Button>
        }
      />
    ) : pending !== null ? (
      <SelectedRow selection={pending} onOpen={onOpen} />
    ) : (
      <InputRow summary={summary} selection={selection} onOpen={onOpen} />
    );

  const readiness = (
    <Readiness
      gateRun={gateRun}
      summary={summary}
      selection={selection}
      analyzedSelectionId={analyzedSelectionId}
      project={project}
      lead={!leadWithIntake}
      onOpen={onOpen}
    />
  );

  return (
    <Page>
      <PageHeader
        title={t('common', 'overview.title')}
        meta={
          gateRun === null
            ? []
            : [
                <span key="gate">
                  {t('common', 'overview.metaGateRun', { id: truncateMiddle(gateRun.runId, 8) })}
                </span>,
              ]
        }
        actions={
          <>
            <Button onClick={() => onOpen('compare')}>{t('common', 'overview.compareButton')}</Button>
            <Button onClick={() => onOpen('history')}>{t('common', 'page.history')}</Button>
            {/* Secondary on purpose: the page's one primary move is the answer's own (ADR-0034 D1). */}
            <Button onClick={() => onOpen('release')}>{t('common', 'overview.openReleaseButton')}</Button>
          </>
        }
      />

      {subject === null ? null : (
        <SubjectBar
          subject={subject}
          freshness={
            pending !== null ? 'pending' : summary !== null && mapChangedSince(summary, selection) ? 'mapChanged' : 'latest'
          }
        />
      )}

      {leadWithIntake ? intake : readiness}
      {leadWithIntake ? readiness : intake}

      {summary === null || pending !== null ? null : <Facts summary={summary} unit={unit} />}
      {summary === null || pending === null ? null : (
        <PreviousAnalysis summary={summary} candidate={pending} unit={unit} />
      )}
      {comparison === null ? null : (
        <LatestComparison comparison={comparison} unit={unit} onOpen={onOpen} />
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
  t: Translate,
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
      <p className={styles['inputDetail']}>{t('common', 'overview.gitDetail')}</p>
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
  const { t } = useI18n();
  // One band, three cells, hairlines between them - which is what the reference draws and what this
  // component asked for since U1. The cells stay named regions because they are three separate answers a
  // reader may want to jump between, and a band is a layout, not an excuse to flatten semantics.
  // This band is only ever rendered when the summary describes the selection in hand. Its MAP pill is Core's
  // word for the analysis; the selection's MAP line can differ once a MAP is added or removed after analyzing,
  // which the SubjectBar names as "MAP changed since this analysis" (R2 review S2-1).
  // The wrapper is the U1P-R3 F2 fix and nothing else: it asks this one band to fill the line its cells wrap
  // onto, because at the frozen 1024 px width three capability cells land in a two-track row and the leftover
  // track showed the band's own background as a fourth cell that does not exist. `components/Panel` is not
  // touched, so the bands on Analyze, Compare, Release and History are the ones that were approved.
  return (
    <div className={styles['fillBand']}>
      <Band label={t('common', 'overview.inputsBand')}>
        {capabilityCells(
          summary,
          selection?.mapAttached
            ? (selection.mapFileName ?? t('common', 'overview.mapAttached'))
            : t('common', 'overview.mapNeeded'),
          {
            elf: <LinkButton onClick={() => onOpen('analyze')}>{t('common', 'overview.openAnalyze')}</LinkButton>,
            map: <LinkButton onClick={() => onOpen('analyze')}>{t('common', 'overview.attachMap')}</LinkButton>,
          },
          t,
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
  const { t } = useI18n();
  return (
    <Band label={t('common', 'overview.selectedBand')}>
      <Panel bare title={t('common', 'overview.artifactCell')}>
        <p className={styles['inputDetail']}>
          {t('common', 'overview.notAnalyzed', { name: selection.fileName })}
        </p>
        <LinkButton onClick={() => onOpen('analyze')}>{t('common', 'overview.openAnalyze')}</LinkButton>
      </Panel>
      <Panel bare title="MAP">
        <p className={styles['inputDetail']}>
          {selection.mapAttached
            ? t('common', 'overview.mapAttachedLine', {
                name: selection.mapFileName ?? t('common', 'overview.mapAttached'),
              })
            : t('common', 'overview.mapMissingLine')}
        </p>
        <LinkButton onClick={() => onOpen('analyze')}>
          {selection.mapAttached
            ? t('common', 'overview.changeMap')
            : t('common', 'overview.attachMap')}
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
  const { t } = useI18n();
  const { artifact } = summary;
  const sameName = candidate.fileName === artifact.fileName;
  const attribution = sameName
    ? t('common', 'overview.previousSameName')
    : t('common', 'overview.previousOtherName', { name: candidate.fileName });

  return (
    <Panel
      title={t('common', 'overview.previousTitle')}
      hint={t('common', 'overview.previousHint')}
    >
      <p className={styles['stale']} role="note">
        {t('common', 'overview.previousOf', { name: artifact.fileName }) + attribution}
      </p>
      <p className={styles['identity']}>
        {t('common', 'overview.snapshotLine', { id: summary.identity.snapshotId })}
      </p>
      <div className={styles['fillBand']}>
        <Band label={t('common', 'overview.previousInputsBand')}>
          {capabilityCells(
            summary,
            t('common', 'overview.previousAsReported', { name: artifact.fileName }),
            { elf: null, map: null },
            t,
          )}
        </Band>
      </div>
      <Band narrow label={t('common', 'overview.previousFiguresBand')}>
        {figureCells(summary, unit, t)}
      </Band>
    </Panel>
  );
}

/**
 * M1 F2: the comparison this session computed last, in one row, so a reader who compared and came
 * back sees what moved without re-running it.
 *
 * Every figure is the Compare summary's own (a signed delta from Rust, or Unknown with its reason);
 * nothing is recomputed here. The comparison lives for the session, like the diff handle behind it,
 * and the row says so.
 */
function LatestComparison({
  comparison,
  unit,
  onOpen,
}: {
  readonly comparison: CompareSummaryDto;
  readonly unit: SizeUnit;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
  const { t } = useI18n();
  const { base, target, memory, objectChanges } = comparison;
  const delta = (change: ByteDeltaDto) =>
    change.delta === null ? (
      <span className={styles['stale']}>
        {t('common', 'memory.unknown')}
        {change.reason === null ? null : ` (${change.reason})`}
      </span>
    ) : (
      // A change is a fact, not a judgement: ▲ or ▼ in primary text, never the status colours (ADR-0034 D1.4).
      <DeltaValue delta={change.delta} format={(bytes) => formatSize(bytes, unit)} />
    );
  const topObject = objectChanges.available ? (objectChanges.topObjects[0] ?? null) : null;

  return (
    <Panel
      title={t('common', 'overview.compareTitle')}
      hint={t('common', 'overview.compareHint')}
    >
      <p className={styles['identity']}>
        {t('common', 'overview.comparePair', {
          base: `${base.fileName} · ${truncateMiddle(base.sha256, 6)}`,
          target: `${target.fileName} · ${truncateMiddle(target.sha256, 6)}`,
        })}
      </p>
      <dl className={styles['compareFacts']}>
        <dt>{t('common', 'overview.compareFlash')}</dt>
        <dd>{delta(memory.nonvolatile)}</dd>
        <dt>{t('common', 'overview.compareRam')}</dt>
        <dd>{delta(memory.runtimeRam)}</dd>
        <dt>{t('common', 'overview.compareComparability')}</dt>
        <dd className={styles['identity']}>{memory.comparability}</dd>
        {topObject === null || topObject.delta === null ? null : (
          <>
            <dt>{t('common', 'overview.compareTopObject')}</dt>
            <dd className={styles['identity']}>
              {topObject.key}{' '}
              <DeltaValue delta={topObject.delta} format={(bytes) => formatSize(bytes, unit)} />
            </dd>
          </>
        )}
      </dl>
      <div>
        <Button onClick={() => onOpen('compare')}>{t('common', 'overview.compareOpen')}</Button>
      </div>
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
  lead,
  onOpen,
}: {
  readonly gateRun: GateRunDto | null;
  readonly summary: AnalysisSummaryDto | null;
  readonly selection: SelectionDto | null;
  readonly analyzedSelectionId: string | null;
  readonly project: ProjectContextDto | null;
  /** Whether this is the page's answer (R2): the lead styling, and the page's one primary action. */
  readonly lead: boolean;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
  const { t } = useI18n();
  const className = lead ? styles['lead'] : undefined;
  if (gateRun === null) {
    return (
      <Panel
        title={t('common', 'overview.shipTitle')}
        hint={t('common', 'overview.shipNoRunHint')}
        className={className}
      >
        <div className={styles['verdict']}>
          <StateBadge variant="chip" state="UNKNOWN" label={t('common', 'overview.shipNoRunLabel')} />
          <p className={styles['verdictSentence']}>{t('common', 'overview.shipNoRunMessage')}</p>
        </div>
        <div className={styles['nextSteps']}>
          <Button variant={lead ? 'primary' : 'secondary'} onClick={() => onOpen('release')}>
            {t('common', 'overview.shipNoRunAction')}
          </Button>
        </div>
      </Panel>
    );
  }

  const scope = readinessScope(gateRun, summary, selection, analyzedSelectionId, project);
  return scope.kind === 'current'
    ? <CurrentReadiness gateRun={gateRun} summary={summary} className={className} onOpen={onOpen} />
    : <NotCurrentReadiness gateRun={gateRun} summary={summary} scope={scope} project={project} className={className} onOpen={onOpen} />;
}

/** The run's own answer, shown as the page's answer - which is only reachable when the subjects match. */
function CurrentReadiness({
  gateRun,
  summary,
  className,
  onOpen,
}: {
  readonly gateRun: GateRunDto;
  readonly summary: AnalysisSummaryDto | null;
  readonly className: string | undefined;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
  const { t } = useI18n();
  const state = severityState(gateRun.dispositionEffectiveSeverity);
  const open = gateRun.findings.filter((finding) => finding.state !== 'PASS' && finding.state !== 'N/A');

  return (
    <Panel
      title={t('common', 'overview.shipTitle')}
      className={className}
      hint={
        <>
          {t('common', 'overview.latestEvidence')}{' '}
          <Chip mono>{truncateMiddle(gateRun.runId, 8)}</Chip> · {gateRun.createdAt}
        </>
      }
    >
      <div
        className={styles['verdict']}
        role="group"
        aria-label={t('common', 'overview.readinessGroup')}
      >
        <StateBadge variant="chip" state={state} label={state} />
        <p className={styles['verdictSentence']}>{gateVerdictSentence(gateRun, t)}</p>
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
          label={t('common', 'overview.findingsStrip')}
        />
        {/* U1P-R1 §5.A: the verdict names the build it judged, in full, as text inside the answer itself. A
            truncated value is not an answer a reader can check, and a `title` attribute is not reachable by a
            keyboard reader, so this line is never shortened. */}
        <p className={styles['identity']}>
          {t('common', 'overview.judgedSnapshot', { id: gateRun.snapshotId })}
        </p>
      </div>

      {open.length === 0 ? (
        <p className={styles['quiet']}>{t('common', 'overview.noOpenFindings')}</p>
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
              {t('common', 'overview.moreFindings', { count: open.length - 4 })}
            </li>
          ) : null}
        </ul>
      )}

      <div className={styles['nextSteps']}>
        <span className={styles['nextLabel']}>{t('common', 'overview.nextSteps')}</span>
        {/* Both labels name the page rather than repeating the header's `Open Release Gate`: two buttons
            with one accessible name on one screen is a reader's question, not a reader's answer. */}
        <Button variant={className === undefined ? 'secondary' : 'primary'} onClick={() => onOpen('release')}>
          {state === 'PASS'
            ? t('common', 'overview.openRunButton')
            : t('common', 'overview.resolveButton')}
        </Button>
        {summary !== null && capabilityState(summary.capabilities.map) !== 'PASS' ? (
          <Button onClick={() => onOpen('analyze')}>{t('common', 'overview.attachMapButton')}</Button>
        ) : null}
      </div>

      <p className={styles['scope']}>{t('common', 'overview.scopeNote')}</p>
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
  className,
  onOpen,
}: {
  readonly gateRun: GateRunDto;
  readonly summary: AnalysisSummaryDto | null;
  readonly scope: Exclude<ReadinessScope, { readonly kind: 'current' }>;
  readonly project: ProjectContextDto | null;
  readonly className: string | undefined;
  readonly onOpen: (target: OverviewTarget) => void;
}) {
  const { t } = useI18n();
  const words = notCurrentWords(scope.kind, t);

  return (
    <Panel
      title={t('common', 'overview.shipTitle')}
      className={className}
      hint={
        <>
          {t('common', 'overview.latestEvidence')}{' '}
          <Chip mono>{truncateMiddle(gateRun.runId, 8)}</Chip> · {gateRun.createdAt}
        </>
      }
    >
      <div
        className={styles['verdict']}
        role="group"
        aria-label={t('common', 'overview.readinessGroup')}
      >
        <StateBadge variant="chip" state="UNKNOWN" label={words.label} />
        <p className={styles['verdictSentence']}>{words.headline}</p>
      </div>

      <div className={styles['pastRun']} role="group" aria-label={words.retained}>
        <p className={styles['identity']}>
          {t('common', 'overview.judgedSnapshot', { id: gateRun.snapshotId })}
        </p>
        {summary === null ? null : (
          <p className={styles['identity']}>
            {t('common', 'overview.describedSnapshot', { id: summary.identity.snapshotId })}
          </p>
        )}
        {scope.kind === 'otherPolicy' && project !== null ? (
          <p className={styles['identity']}>
            {t('common', 'overview.policyPair', {
              run: gateRun.policySha256,
              loaded: project.policySha256,
            })}
          </p>
        ) : null}
        <p className={styles['quiet']}>{t('common', 'overview.retainedQuiet')}</p>
      </div>

      <div className={styles['nextSteps']}>
        <span className={styles['nextLabel']}>{t('common', 'overview.nextSteps')}</span>
        {/* Navigation only in every arm: choosing a build and running the gate is the Release page's work, and
            arriving there must never imply a run happened. §4.H is why the pending case leads with Analyze.
            Its primary control used to read "Choose this build on Release Gate", and the build Release
            preselects when the reader clicks is the retained one - so the strongest label on the page promised
            a choice about a file it was not about to choose. */}
        {scope.kind === 'selectionPending' ? (
          <>
            <Button variant={className === undefined ? 'secondary' : 'primary'} onClick={() => onOpen('analyze')}>
              {t('common', 'overview.analyzeSelectedButton')}
            </Button>
            <Button onClick={() => onOpen('release')}>{t('common', 'overview.viewPreviousRunButton')}</Button>
          </>
        ) : (
          <>
            <Button variant={className === undefined ? 'secondary' : 'primary'} onClick={() => onOpen('release')}>
              {t('common', 'overview.chooseBuildButton')}
            </Button>
            {summary === null ? (
              <Button onClick={() => onOpen('analyze')}>{t('common', 'overview.analyzeArtifactButton')}</Button>
            ) : null}
          </>
        )}
      </div>

      <p className={styles['scope']}>{t('common', 'overview.scopeNote')}</p>
    </Panel>
  );
}

/** The four figures, always read from one summary; the band around them decides whose figures they are. */
function figureCells(
  summary: AnalysisSummaryDto,
  unit: SizeUnit,
  t: Translate,
): ReactNode[] {
  const { memory, capabilities, evidenceSummary } = summary;
  const flash = memory.nonvolatileImageFootprint;
  const ram = memory.runtimeRamFootprint;
  // The context lines are Core's own words about a figure - a classification, a state, a reason - and
  // `ADR-0032` D5 keeps them as written in both languages. A translated classification would be a
  // second name for one fact.
  const footprintContext = (
    value: { classification: string; state: string; reason: string | null },
  ) =>
    `${value.classification} · ${value.state}${value.reason === null ? '' : ` · ${value.reason}`}`;

  return [
    <SummaryCard
      key="flash"
      label={t('common', 'overview.flashLabel')}
      value={flash.bytes === null ? t('common', 'overview.unknownValue') : formatSize(flash.bytes, unit)}
      context={footprintContext(flash)}
    />,
    <SummaryCard
      key="ram"
      label={t('common', 'overview.ramLabel')}
      value={ram.bytes === null ? t('common', 'overview.unknownValue') : formatSize(ram.bytes, unit)}
      context={footprintContext(ram)}
    />,
    <SummaryCard
      key="symbols"
      label={t('common', 'overview.symbolsLabel')}
      value={String(summary.symbolCount)}
      context={t('common', 'overview.capabilityLine', { state: capabilities.symbols })}
    />,
    <SummaryCard
      key="evidence"
      label={t('common', 'overview.evidenceLabel')}
      value={String(evidenceSummary.total)}
      context={t('common', 'overview.evidenceLine', {
        observed: evidenceSummary.observed,
        derived: evidenceSummary.derived,
        declared: evidenceSummary.declared,
        unknown: evidenceSummary.unknown,
      })}
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
  const { t } = useI18n();
  return (
    <>
      <Band narrow label={t('common', 'overview.keyFiguresBand')}>{figureCells(summary, unit, t)}</Band>
      <Panel title={t('common', 'memory.title')} hint={t('common', 'memory.hint')}>
        <MemoryUsage regions={summary.memory.regionUsage} unit={unit} />
      </Panel>
    </>
  );
}
