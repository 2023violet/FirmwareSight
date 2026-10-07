/**
 * The Release page: one stored build, the policy it is judged against, and the record a reviewer
 * leaves beside it.
 *
 * Five rules decide the shape of this screen.
 *
 * 1. **Nothing is computed here.** Every state, severity, count, aggregate, budget margin and delta
 *    arrives from the shell (`AGENTS.md` 3, `04_TECH/14` 3). This page chooses words, order and layout,
 *    and holds no copy of a default policy either: an unset field is sent as unset, and the shell is the
 *    only thing that knows what the product's default is (prompt §41, §52).
 * 2. **A state is icon + words.** The five Gate states are rendered by `StateBadge`, never by colour
 *    alone, and `UNKNOWN` keeps its neutral hollow icon so it cannot be read as a pass (DESIGN.md 5, 9;
 *    prompt §47).
 * 3. **An accepted review stays a review.** Acceptance adds an audit row beside the finding; the row
 *    still reads REVIEW and only the aggregate moves (prompt §48, `04_TECH/27`).
 * 4. **A failed run does not delete the last good one.** The GateRun moves only when the shell returns
 *    one, and the surviving record says it is previous (prompt §49).
 * 5. **This is policy readiness, nothing more.** The page never says a build is safe to ship, legally
 *    compliant or certified: the Gate answers one policy, and the wording on the screen stops there
 *    (prompt §43).
 * 6. **A bundle attaches under Release, and only after a PASS.** P4 added no fifth stage to the rail; the
 *    Release Bundle is section 6 of this page, disabled with its reason while the disposition says otherwise,
 *    and its preview, destination token and typed errors all come from the same engine the command line
 *    drives (prompt §48, §35).
 */

import { useCallback, useEffect, useRef, useState } from 'react';

import styles from './Release.module.css';
import { Button, LinkButton } from './components/Button';
import { StatusStrip } from './components/Chip';
import { ErrorPanel } from './components/ErrorPanel';
import { Page, ScrollArea } from './components/Layout';
import { PageHeader } from './components/PageHeader';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import { StateBadge, type StateName } from './components/StateBadge';
import { evidenceBasisCaption } from './evidenceBasis';
import { formatDelta, formatSize, truncateMiddle, type SizeUnit } from './format';
import { gateVerdictSentence, severityState } from './stateWords';
import {
  acceptReview,
  chooseBundleDestination,
  exportReleaseBundle,
  getGateRun,
  listCompareCandidates,
  openProjectConfig,
  prepareReleaseBundle,
  runReleaseGate,
  saveProjectPolicy,
} from './ipc/bridge';
import type {
  AcceptReviewRequestDto,
  BundleDestinationDto,
  BundleExportDto,
  BundlePreviewDto,
  CandidatePageDto,
  CompareCandidateDto,
  ErrorEnvelopeDto,
  GateBudgetRowDto,
  GateFindingRowDto,
  GateGrowthRowDto,
  GateRunDto,
  ProjectContextDto,
  ProjectPolicyDto,
  UnknownDispositionDto,
} from './ipc/types';
import { cx } from './styles/classnames';

/** The artifact kinds a policy may require. Presence is all P3 checks; see the hint under the group. */
const ARTIFACT_KINDS: readonly string[] = ['elf', 'map', 'bin', 'hex'];

/** The seven rules that can lose their evidence, in the order `[gate.on_unknown]` lists them. */
const UNKNOWN_RULES: readonly {
  readonly key: keyof ProjectPolicyDto['onUnknown'];
  readonly label: string;
}[] = [
  { key: 'gitClean', label: 'git.clean' },
  { key: 'commitMatchesRelease', label: 'release.commit_matches_expected' },
  { key: 'versionMatch', label: 'release.version_matches_policy' },
  { key: 'flashBudget', label: 'memory.flash_budget' },
  { key: 'ramBudget', label: 'memory.ram_budget' },
  { key: 'baselineGrowth', label: 'diff.growth' },
  { key: 'releaseNotes', label: 'release.notes' },
];

/** Findings are grouped and read in this order: what blocks, then what asks, then what is unknown. */
const STATE_GROUPS: readonly {
  readonly state: StateName;
  readonly heading: string;
  readonly hint: string;
}[] = [
  {
    state: 'BLOCK',
    heading: 'Block',
    hint: 'The rule evaluated and failed in a way that stops this run.',
  },
  {
    state: 'REVIEW',
    heading: 'Review',
    hint: 'The rule evaluated and needs a named person to answer it.',
  },
  {
    state: 'UNKNOWN',
    heading: 'Unknown',
    hint: 'The evidence this rule needs is missing, so it could not be evaluated.',
  },
  {
    state: 'PASS',
    heading: 'Pass',
    hint: 'The rule evaluated and is satisfied.',
  },
  {
    state: 'N/A',
    heading: 'Not applicable',
    hint: 'The policy or this build puts this rule outside the run.',
  },
];

/** The five Gate words, so a stored value outside them is shown rather than dropped. */
const STATE_NAMES: readonly string[] = ['BLOCK', 'REVIEW', 'UNKNOWN', 'PASS', 'N/A'];

/** A three-way choice: what the reader said, or nothing — and nothing is the shell's to interpret. */
type TriState = 'unset' | 'yes' | 'no';

export function Release({
  unit,
  onUnitChange,
  lastAnalyzedSnapshotId,
  project,
  onProjectChange,
  run,
  onRunChange,
  onGoToAnalyze,
}: {
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
  readonly lastAnalyzedSnapshotId: string | null;
  readonly project: ProjectContextDto | null;
  readonly onProjectChange: (project: ProjectContextDto | null) => void;
  readonly run: GateRunDto | null;
  readonly onRunChange: (run: GateRunDto | null) => void;
  readonly onGoToAnalyze: () => void;
}) {
  const [candidates, setCandidates] = useState<CandidatePageDto | null>(null);
  const [candidateError, setCandidateError] = useState<ErrorEnvelopeDto | null>(null);
  const [snapshotId, setSnapshotId] = useState<string | null>(null);
  const [baselineId, setBaselineId] = useState<string | null>(null);
  const [draft, setDraft] = useState<ProjectPolicyDto | null>(null);
  const [policyError, setPolicyError] = useState<ErrorEnvelopeDto | null>(null);
  const [policyNote, setPolicyNote] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [running, setRunning] = useState(false);
  const [restored, setRestored] = useState(false);

  /**
   * The policy the editor starts from, in the order the facts are known: a loaded config, then the
   * policy the last run actually judged with, then nothing stated at all.
   *
   * The third case sends a form whose fields are all unset. There is no fourth case where this page
   * invents a default, because `04_TECH/08:60-62`'s defaults live in Rust and an unset field is what
   * asks for them.
   */
  const seed = project?.policy ?? run?.policy ?? null;

  /** The stored builds Release may gate: analyzed, persisted, and read from SQLite, not from disk. */
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
   * Name a default build (prompt §44).
   *
   * The build Analyze last proved is the one the reader most likely means; with none, the newest stored
   * build is offered. A baseline is never chosen by itself: it stays empty until asked for.
   */
  useEffect(() => {
    if (snapshotId !== null || candidates === null || candidates.rows.length === 0) {
      return;
    }
    const preferred =
      candidates.rows.find((row) => row.snapshotId === lastAnalyzedSnapshotId) ??
      candidates.rows[0];
    if (preferred !== undefined) {
      setSnapshotId(preferred.snapshotId);
    }
  }, [candidates, lastAnalyzedSnapshotId, snapshotId]);

  const open = useCallback(async () => {
    const outcome = await openProjectConfig();
    if (outcome.ok) {
      // Cancel resolves to `null`, which is a decision, not a failure: the page keeps its project.
      if (outcome.value !== null) {
        onProjectChange(outcome.value);
        setPolicyError(null);
        setPolicyNote(`Loaded ${outcome.value.configFileName}`);
      }
      return;
    }
    setPolicyError(outcome.envelope);
  }, [onProjectChange]);

  const save = useCallback(async () => {
    if (draft === null) {
      return;
    }
    setSaving(true);
    setPolicyError(null);
    const outcome = await saveProjectPolicy(draft);
    setSaving(false);
    if (outcome.ok) {
      if (outcome.value !== null) {
        onProjectChange(outcome.value);
        setPolicyNote(`Saved ${outcome.value.configFileName}`);
        setDraft(null);
      }
      return;
    }
    setPolicyError(outcome.envelope);
  }, [draft, onProjectChange]);

  /**
   * Judge the chosen build, and keep the previous run on the failure path.
   *
   * A config warning is not a failure and never reaches here: the run still completes and its warnings
   * travel inside its own payload (prompt §49).
   */
  const gate = useCallback(async () => {
    if (snapshotId === null) {
      return;
    }
    setRunning(true);
    setError(null);
    const outcome = await runReleaseGate({
      snapshotId,
      baselineSnapshotId: baselineId === snapshotId ? null : baselineId,
    });
    setRunning(false);
    if (outcome.ok) {
      onRunChange(outcome.value);
      setRestored(false);
      return;
    }
    setError(outcome.envelope);
  }, [baselineId, onRunChange, snapshotId]);

  /**
   * Accept one review. The finding keeps its own state; the row gains an acceptance and the aggregate is
   * the one the shell computed.
   */
  const accept = useCallback(
    async (request: AcceptReviewRequestDto): Promise<ErrorEnvelopeDto | null> => {
      if (run === null) {
        return null;
      }
      const outcome = await acceptReview(request);
      if (!outcome.ok) {
        return outcome.envelope;
      }
      const acceptance = outcome.value;
      onRunChange({
        ...run,
        dispositionEffectiveSeverity: acceptance.dispositionEffectiveSeverity,
        findings: run.findings.map((finding) =>
          finding.id === acceptance.findingId
            ? {
                ...finding,
                state: acceptance.state,
                acceptable: false,
                acceptance: acceptance.acceptance,
              }
            : finding,
        ),
      });
      return null;
    },
    [onRunChange, run],
  );

  /** Read the stored record back, and say so: a historical run shows no rebuilt numbers (prompt §46). */
  const reload = useCallback(async () => {
    if (run === null) {
      return;
    }
    const outcome = await getGateRun(run.runId);
    if (outcome.ok && outcome.value !== null) {
      onRunChange(outcome.value);
      setRestored(true);
      return;
    }
    if (!outcome.ok) {
      setError(outcome.envelope);
    }
  }, [onRunChange, run]);

  const chosen = (id: string | null): CompareCandidateDto | undefined =>
    candidates?.rows.find((row) => row.snapshotId === id);

  return (
    <Page>
      <PageHeader
        title="Release Gate"
        subhead="FirmwareSight policy readiness for one stored build, judged against one project policy. The answer below is that policy’s verdict and nothing more: no claim on this page reaches beyond the rules the project itself wrote."
      />

      {/* U1 §B.5: the verdict used to arrive at section 3, after the reader had scrolled past a policy
          editor and a build picker to find out whether the build can ship. The counts below are
          `GateCountsDto` verbatim and the severity is Core's disposition aggregate; this band displays
          both and decides neither. */}
      {run === null ? null : <Verdict run={run} />}

      <section className={styles['section']} aria-labelledby="fs-policy-heading">
        <h2 id="fs-policy-heading">1 · Project policy</h2>
        <div className={styles['row']}>
          <span className={styles['term']}>Config</span>
          <span className={styles['value']}>
            {project === null ? (
              <span>
                No project policy is loaded. A run judges FirmwareSight&rsquo;s stated default policy, and
                the run below shows which values those were.
              </span>
            ) : (
              <>
                <span>{project.projectName}</span>
                <span className={styles['monoSmall']}>{project.configFileName}</span>
                <span className={styles['note']}>
                  schema_version {project.configSchemaVersion} · policy{' '}
                  <span className={styles['mono']}>{project.policySha256}</span>
                </span>
              </>
            )}
          </span>
        </div>
        {project !== null && project.unknownKeys.length > 0 ? (
          <p className={styles['warning']} role="status">
            This config holds keys this build does not understand: {project.unknownKeys.join(', ')}. They
            are listed rather than dropped, and a save to this file is refused until they are resolved.
          </p>
        ) : null}
        {project !== null && project.warnings.length > 0 ? (
          <ul className={styles['noticeList']} aria-label="Config warnings">
            {project.warnings.map((warning) => (
              <li className={styles['noticeItem']} key={warning}>
                Warning: {warning}
              </li>
            ))}
          </ul>
        ) : null}
        <div className={styles['actions']}>
          <Button onClick={() => void open()}>Open project config</Button>
          <Button
            onClick={() => {
              setDraft(draft === null ? (seed ?? blankPolicy(project?.projectName ?? '')) : null);
            }}
            ariaExpanded={draft !== null}
          >
            {draft === null ? 'Edit policy' : 'Close policy editor'}
          </Button>
          <Button
            variant="primary"
            disabled={draft === null || saving}
            onClick={() => void save()}
          >
            {saving ? 'Saving…' : project === null ? 'Save as firmwaresight.toml' : 'Save policy'}
          </Button>
          {policyNote === null ? null : <span className={styles['status']}>{policyNote}</span>}
        </div>
        {policyError === null ? null : (
          <ErrorPanel envelope={policyError} label="Config load or save failed" heading="Policy error" />
        )}
        {draft === null ? (
          <p className={styles['hint']}>
            {project === null
              ? 'Open a config to edit its policy, or choose Edit policy to write one for this project.'
              : `Editing writes to ${project.configFileName} in the project folder.`}
          </p>
        ) : (
          <PolicyForm draft={draft} onChange={setDraft} />
        )}
      </section>

      <section className={styles['section']} aria-labelledby="fs-builds-heading">
        <h2 id="fs-builds-heading">2 · Build and baseline</h2>
        {candidateError === null ? null : (
          <ErrorPanel envelope={candidateError} label="Build list failed" heading="Build list error" />
        )}
        {candidates !== null && candidates.rows.length === 0 ? (
          <p className={styles['empty']}>
            Nothing is stored yet, so there is nothing to gate.{' '}
            <LinkButton onClick={onGoToAnalyze}>Analyze a build</LinkButton> first, then come back.
          </p>
        ) : (
          <div className={styles['selectors']}>
            <div className={styles['picker']}>
              <label className={styles['pickerLabel']} htmlFor="fs-gate-current">
                Current build
              </label>
              <select
                id="fs-gate-current"
                className={styles['select']}
                value={snapshotId ?? ''}
                onChange={(event) => {
                  setSnapshotId(event.target.value === '' ? null : event.target.value);
                }}
              >
                <option value="">Choose a build</option>
                {candidates?.rows.map((row) => (
                  <option key={row.snapshotId} value={row.snapshotId}>
                    {row.fileName} · {truncateMiddle(row.snapshotId, 8)}
                  </option>
                ))}
              </select>
              <BuildFacts row={chosen(snapshotId ?? null)} unit={unit} />
            </div>
            <div className={styles['picker']}>
              <label className={styles['pickerLabel']} htmlFor="fs-gate-baseline">
                Baseline (optional)
              </label>
              <select
                id="fs-gate-baseline"
                className={styles['select']}
                value={baselineId ?? ''}
                onChange={(event) => {
                  setBaselineId(event.target.value === '' ? null : event.target.value);
                }}
              >
                <option value="">No baseline</option>
                {candidates?.rows
                  .filter((row) => row.snapshotId !== snapshotId)
                  .map((row) => (
                    <option key={row.snapshotId} value={row.snapshotId}>
                      {row.fileName} · {truncateMiddle(row.snapshotId, 8)}
                    </option>
                  ))}
              </select>
              <BuildFacts row={chosen(baselineId ?? null)} unit={unit} />
              {baselineId === null ? (
                <span className={styles['hint']}>
                  With growth thresholds configured and no baseline, the growth rule answers Unknown and
                  the run still happens.
                </span>
              ) : null}
            </div>
            <div className={styles['pickerActions']}>
              <Button
                variant="primary"
                disabled={snapshotId === null || running}
                onClick={() => void gate()}
              >
                {running ? 'Judging…' : 'Run Gate'}
              </Button>
              <SizeUnitSwitch unit={unit} onSelect={onUnitChange} />
            </div>
          </div>
        )}
      </section>

      {error === null ? null : (
        <ErrorPanel envelope={error} label="Gate run failed" heading="Gate run error" />
      )}
      {run === null ? null : (
        <>
          {error === null ? null : (
            <p className={styles['stale']} role="status">
              The record below is the last run that succeeded ({truncateMiddle(run.runId, 10)}). It is not
              the result of the attempt that failed.
            </p>
          )}
          <Readiness run={run} restored={restored} onReload={() => void reload()} />
          <Findings run={run} onAccept={accept} />
          {run.tables === null ? null : <Evidence run={run} unit={unit} />}
          <BundleSection run={run} unit={unit} />
        </>
      )}
    </Page>
  );
}

/**
 * FirmwareSight policy readiness (prompt §43 item 5). The aggregate with and without the accepted
 * reviews are both Core&rsquo;s answers; neither is derived here.
 */
function Readiness({
  run,
  restored,
  onReload,
}: {
  readonly run: GateRunDto;
  readonly restored: boolean;
  readonly onReload: () => void;
}) {
  return (
    <section className={styles['section']} aria-labelledby="fs-readiness-heading">
      <h2 id="fs-readiness-heading">3 · FirmwareSight policy readiness</h2>
      <div className={styles['states']}>
        <StateBadge
          state={stateName(run.dispositionEffectiveSeverity)}
          label={`Disposition: ${run.dispositionEffectiveSeverity}`}
          note="The aggregate with this run's accepted reviews counted."
        />
        <StateBadge
          state={stateName(run.overallEffectiveSeverity)}
          label={`As computed: ${run.overallEffectiveSeverity}`}
          note="The aggregate before any review was accepted."
        />
      </div>
      <div className={styles['counts']} role="group" aria-label="Findings by state">
        <StateBadge state="BLOCK" label="Block" count={run.counts.block} />
        <StateBadge state="REVIEW" label="Review" count={run.counts.review} />
        <StateBadge state="UNKNOWN" label="Unknown" count={run.counts.unknown} />
        <StateBadge state="PASS" label="Pass" count={run.counts.pass} />
        <StateBadge state="N/A" label="Not applicable" count={run.counts.notApplicable} />
      </div>
      <dl className={styles['rows']}>
        <div className={styles['row']}>
          <dt className={styles['term']}>Run</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{run.runId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Build</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{run.snapshotId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Baseline</dt>
          <dd className={cx(styles['value'], styles['mono'])}>
            {run.baselineSnapshotId ?? 'None'}
          </dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Policy</dt>
          <dd className={styles['value']}>
            <span>{run.policySource}</span>
            {run.projectName === null ? null : <span>{run.projectName}</span>}
            <span className={styles['monoSmall']}>{run.policySha256}</span>
          </dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Stored</dt>
          <dd className={cx(styles['value'], styles['monoSmall'])}>{run.createdAt}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Workspace</dt>
          <dd className={styles['value']}>
            <span>{run.git.summary}</span>
            {run.git.headCommit === null ? null : (
              <span className={styles['monoSmall']}>
                HEAD {truncateMiddle(run.git.headCommit, 8)}
              </span>
            )}
            {run.git.exactTag === null ? null : (
              <span className={styles['monoSmall']}>
                exact tag {run.git.exactTag} on workspace HEAD
              </span>
            )}
            {run.git.dirty === null ? null : (
              <span>{run.git.dirty ? 'dirty' : 'clean'} (workspace, not artifact)</span>
            )}
            {run.git.reason === null ? null : (
              <span className={styles['note']}>{run.git.reason}</span>
            )}
          </dd>
        </div>
      </dl>
      <p className={styles['hint']}>
        Git facts describe the workspace FirmwareSight read. They are not proof of how this artifact was
        built, and nothing here says it was built from a commit.
      </p>
      {run.recordNote === null ? null : (
        <p className={styles['stale']} role="status">
          {run.recordNote}
        </p>
      )}
      {run.warnings.length === 0 ? null : (
        <ul className={styles['noticeList']} aria-label="Run warnings">
          {run.warnings.map((warning) => (
            <li className={styles['noticeItem']} key={warning}>
              Warning: {warning}
            </li>
          ))}
        </ul>
      )}
      <div className={styles['actions']}>
        {restored ? (
          <span className={styles['status']}>Showing the record as history stores it.</span>
        ) : (
          <Button onClick={onReload}>
            Read the stored record
          </Button>
        )}
        <span className={styles['status']}>
          A stored record keeps findings, evidence and accepted reviews. The numbers a policy produced
          stay inside each finding.
        </span>
      </div>
    </section>
  );
}

/** Findings grouped BLOCK → REVIEW → UNKNOWN → PASS → N/A, each row showing rule, state and evidence. */
function Findings({
  run,
  onAccept,
}: {
  readonly run: GateRunDto;
  readonly onAccept: (request: AcceptReviewRequestDto) => Promise<ErrorEnvelopeDto | null>;
}) {
  return (
    <section className={styles['section']} aria-labelledby="fs-findings-heading">
      <h2 id="fs-findings-heading">4 · Findings by state</h2>
      {run.findings.length === 0 ? (
        <p className={styles['hint']}>This run returned no findings.</p>
      ) : null}
      {STATE_GROUPS.map((group) => {
        const rows = run.findings.filter((finding) => finding.state === group.state);
        if (rows.length === 0) {
          return null;
        }
        return (
          <div className={styles['group']} key={group.state}>
            <h3 className={styles['groupHead']}>
              <StateBadge state={group.state} label={group.heading} count={rows.length} />
            </h3>
            <p className={styles['hint']}>{group.hint}</p>
            <ol className={styles['findings']}>
              {rows.map((finding) => (
                <FindingRow finding={finding} key={finding.id} runId={run.runId} onAccept={onAccept} />
              ))}
            </ol>
          </div>
        );
      })}
    </section>
  );
}

/**
 * One finding, and the acceptance beside it.
 *
 * The Accept control exists only where Rust said `acceptable`, which is only a REVIEW row. An accepted
 * row still renders its own state: the acceptance is an audit fact next to it, never a rewrite of it
 * (prompt §48).
 */
function FindingRow({
  finding,
  runId,
  onAccept,
}: {
  readonly finding: GateFindingRowDto;
  readonly runId: string;
  readonly onAccept: (request: AcceptReviewRequestDto) => Promise<ErrorEnvelopeDto | null>;
}) {
  const [openForm, setOpenForm] = useState(false);
  const [actor, setActor] = useState('');
  const [reason, setReason] = useState('');
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = useCallback(async () => {
    setBusy(true);
    const failure = await onAccept({ runId, findingId: finding.id, actor, reason });
    setBusy(false);
    if (failure === null) {
      setOpenForm(false);
      setActor('');
      setReason('');
      setError(null);
      return;
    }
    setError(failure);
  }, [actor, finding.id, onAccept, reason, runId]);

  return (
    <li className={styles['finding']}>
      <div className={styles['findingHead']}>
        <span className={styles['mono']}>{finding.ruleId}</span>
        <StateBadge state={stateName(finding.state)} label={finding.state} />
        <span className={styles['severity']}>effective severity: {finding.effectiveSeverity}</span>
      </div>
      <p className={styles['summary']}>{finding.summary}</p>
      {finding.remediation === null ? null : (
        <p className={styles['remediation']}>
          <span className={styles['axis']}>Next step</span> {finding.remediation}
        </p>
      )}
      {finding.evidenceRefs.length === 0 ? (
        <p className={styles['note']}>This finding quotes no evidence locator.</p>
      ) : (
        <ul className={styles['refs']} aria-label={`Evidence for ${finding.ruleId}`}>
          {finding.evidenceRefs.map((ref) => (
            <li className={styles['monoSmall']} key={ref}>
              {ref}
            </li>
          ))}
        </ul>
      )}
      {finding.acceptance === null ? null : (
        <div className={styles['accepted']} role="status">
          <span className={styles['axis']}>Accepted</span>
          <dl className={styles['audit']}>
            <div>
              <dt>By</dt>
              <dd>{finding.acceptance.actor}</dd>
            </div>
            <div>
              <dt>When</dt>
              <dd className={styles['monoSmall']}>{finding.acceptance.acceptedAt}</dd>
            </div>
            <div>
              <dt>Reason</dt>
              <dd>{finding.acceptance.reason}</dd>
            </div>
            <div>
              <dt>Accepted state</dt>
              <dd>{finding.acceptance.originalState}</dd>
            </div>
          </dl>
          <p className={styles['note']}>
            An acceptance is a record: it cannot be edited or deleted, and this row stays{' '}
            {finding.acceptance.originalState}.
          </p>
        </div>
      )}
      {finding.acceptable ? (
        <div className={styles['accept']}>
          {openForm ? (
            <div className={styles['acceptForm']}>
              <label className={styles['field']} htmlFor={`fs-actor-${finding.id}`}>
                <span className={styles['pickerLabel']}>Who accepts this review</span>
                <input
                  id={`fs-actor-${finding.id}`}
                  className={styles['input']}
                  value={actor}
                  onChange={(event) => {
                    setActor(event.target.value);
                  }}
                />
              </label>
              <label className={styles['field']} htmlFor={`fs-reason-${finding.id}`}>
                <span className={styles['pickerLabel']}>Why it is acceptable</span>
                <textarea
                  id={`fs-reason-${finding.id}`}
                  className={styles['input']}
                  rows={2}
                  value={reason}
                  onChange={(event) => {
                    setReason(event.target.value);
                  }}
                />
              </label>
              <div className={styles['actions']}>
                <Button
                  variant="primary"
                  disabled={busy || actor.trim().length === 0 || reason.trim().length === 0}
                  onClick={() => void submit()}
                >
                  {busy ? 'Recording…' : 'Record acceptance'}
                </Button>
                <Button
                  onClick={() => {
                    setOpenForm(false);
                  }}
                >
                  Cancel
                </Button>
                <span className={styles['status']}>
                  Both are required. An acceptance without a name and a reason is not an audit record.
                </span>
              </div>
              {error === null ? null : (
                <ErrorPanel
                  envelope={error}
                  label={`Acceptance of ${finding.ruleId} failed`}
                  heading="Acceptance refused"
                />
              )}
            </div>
          ) : (
            <Button
              onClick={() => {
                setOpenForm(true);
              }}
            >
              Accept review
            </Button>
          )}
        </div>
      ) : null}
    </li>
  );
}

/**
 * The numbers behind the findings: required artifacts, the two budgets, growth against the baseline, and
 * the release notes file (prompt §51–§53).
 *
 * Every figure is Rust&rsquo;s. The unit switch only re-labels a byte count, and a row whose state is
 * Unknown shows the word `Unknown` with its reason rather than a number that was never measured.
 */
function Evidence({ run, unit }: { readonly run: GateRunDto; readonly unit: SizeUnit }) {
  const tables = run.tables;
  if (tables === null) {
    return null;
  }
  return (
    <section className={styles['section']} aria-labelledby="fs-evidence-heading">
      <h2 id="fs-evidence-heading">5 · What the findings were judged on</h2>

      <div className={styles['block']}>
        <h3>Required artifacts</h3>
        <ScrollArea label="Required artifact table">
          <table className={styles['table']}>
            <caption className={styles['caption']}>
              Policy requirement against what this build holds. P3 analyzes ELF and reads GNU ld MAP
              files; a required BIN or HEX is checked for presence only.
            </caption>
            <thead>
              <tr>
                <th scope="col">Kind</th>
                <th scope="col">Policy</th>
                <th scope="col">Build</th>
                <th scope="col">SHA-256</th>
                <th scope="col">Size</th>
              </tr>
            </thead>
            <tbody>
              {tables.artifacts.map((row) => (
                <tr key={row.kind}>
                  <th scope="row" className={styles['mono']}>
                    {row.kind}
                  </th>
                  <td>{row.required ? 'Required' : 'Not required'}</td>
                  <td>{row.present ? 'Present' : 'Missing'}</td>
                  <td className={styles['monoSmall']}>
                    {row.sha256 === null ? '—' : truncateMiddle(row.sha256, 8)}
                  </td>
                  <td className={styles['mono']}>{formatSize(row.byteSize, unit)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </ScrollArea>
      </div>

      <div className={styles['block']}>
        <h3>Memory budgets</h3>
        {tables.budgets.length === 0 ? (
          <p className={styles['hint']}>
            No FLASH or RAM budget is configured, so neither rule has anything to measure against.
          </p>
        ) : (
          <ScrollArea label="Memory budget table">
            <table className={styles['table']}>
              <caption className={styles['caption']}>
                Actual against budget, with the evidence the actual figure rests on. A floor is not a
                measurement of the whole, so a partial total never blocks.
              </caption>
              <thead>
                <tr>
                  <th scope="col">Side</th>
                  <th scope="col">Actual</th>
                  <th scope="col">Budget</th>
                  <th scope="col">Headroom / over</th>
                  <th scope="col">State</th>
                  <th scope="col">Evidence basis</th>
                </tr>
              </thead>
              <tbody>
                {tables.budgets.map((row) => (
                  <BudgetRow key={row.ruleId} row={row} unit={unit} />
                ))}
              </tbody>
            </table>
          </ScrollArea>
        )}
      </div>

      <div className={styles['block']}>
        <h3>Growth against the baseline</h3>
        {tables.growth.length === 0 ? (
          <p className={styles['hint']}>
            No `[diff]` growth threshold is configured, so nothing is compared with a baseline here.
          </p>
        ) : (
          <ScrollArea label="Growth against the baseline table">
            <table className={styles['table']}>
              <caption className={styles['caption']}>
                The deltas are the same ones Compare shows: Core computed them once and this page moves
                them.
              </caption>
              <thead>
                <tr>
                  <th scope="col">Side</th>
                  <th scope="col">Old</th>
                  <th scope="col">New</th>
                  <th scope="col">Delta</th>
                  <th scope="col">Review threshold</th>
                  <th scope="col">State</th>
                </tr>
              </thead>
              <tbody>
                {tables.growth.map((row) => (
                  <GrowthRow key={row.side} row={row} unit={unit} />
                ))}
              </tbody>
            </table>
          </ScrollArea>
        )}
      </div>

      <div className={styles['block']}>
        <h3>Release notes</h3>
        <div className={styles['row']}>
          <span className={styles['term']}>Path</span>
          <span className={cx(styles['value'], styles['mono'])}>
            {tables.notes.relativePath ?? 'No notes path is required'}
          </span>
        </div>
        <div className={styles['row']}>
          <span className={styles['term']}>State</span>
          <span className={styles['value']}>
            <StateBadge
              state={stateName(tables.notes.state)}
              label={
                tables.notes.required
                  ? `Required · ${notesPresence(tables.notes.present)}`
                  : 'Not required'
              }
            />
            <span className={styles['severity']}>
              effective severity: {tables.notes.effectiveSeverity}
            </span>
          </span>
        </div>
        {tables.notes.sha256 === null ? null : (
          <div className={styles['row']}>
            <span className={styles['term']}>SHA-256</span>
            <span className={cx(styles['value'], styles['monoSmall'])}>{tables.notes.sha256}</span>
          </div>
        )}
        {tables.notes.reason === null ? null : <p className={styles['note']}>{tables.notes.reason}</p>}
        <p className={styles['hint']}>
          The path is relative to the project folder, and the folder itself is never shown. There is no
          editor here: FirmwareSight reads the notes file, it does not write one.
        </p>
      </div>
    </section>
  );
}

function BudgetRow({ row, unit }: { readonly row: GateBudgetRowDto; readonly unit: SizeUnit }) {
  const margin =
    row.headroomBytes !== null
      ? `${formatSize(row.headroomBytes, unit)} headroom`
      : row.overBytes !== null
        ? `${formatSize(row.overBytes, unit)} over`
        : 'Unknown';
  return (
    <tr>
      <th scope="row">{row.label}</th>
      <td className={styles['mono']}>{formatSize(row.actualBytes, unit)}</td>
      <td className={styles['mono']}>{formatSize(row.budgetBytes, unit)}</td>
      <td className={styles['mono']}>{margin}</td>
      <td>
        <StateBadge state={stateName(row.state)} label={row.state} />
        <span className={styles['note']}> effective {row.effectiveSeverity}</span>
      </td>
      <td className={styles['wrap']}>
        <span>{row.basis === null ? 'no basis recorded' : evidenceBasisCaption(row.basis)}</span>
        <span className={styles['note']}>
          {' · '}
          {row.exact ? 'complete attribution' : 'floor'}
          {' · '}
          {row.admissible ? 'may support a hard verdict' : 'cannot block on this evidence'}
        </span>
        {row.reason === null ? null : <span className={styles['note']}>{' · ' + row.reason}</span>}
      </td>
    </tr>
  );
}

/**
 * Which refusals mean the preview is no longer an authorization for anything (§28, §49).
 *
 * Each one says a fact moved after the plan was built — the workspace, the source bytes, the notes, or the
 * plan itself expiring — so the screen clears the preview and the destination rather than letting a person
 * press Export again over a stale authorization.
 */
const STALE_PLAN_CODES: readonly string[] = [
  'ERR-BUNDLE-6102',
  'ERR-BUNDLE-6103',
  'ERR-BUNDLE-6104',
  'ERR-BUNDLE-6105',
];

/**
 * 6 · Release Bundle (prompt §48, §49, §50, §58).
 *
 * Three facts decide what this section can do, and none of them is computed here: the disposition is Core's
 * aggregate, the plan's contents are the bundle engine's, and the destination is a folder a person chose in a
 * dialog this page cannot see. The order is the flow §48 fixes — Prepare, preview, choose a destination,
 * export, verification result — and a refusal is shown as the typed error the shell returned, with the code
 * and the remediation the engine wrote for it.
 *
 * The plan is dropped whenever the run changes: a preview authorized against one run is not an authorization
 * for the next (§28). Copying the release id or the manifest digest is deliberately not offered either — the
 * shipped build grants no clipboard or shell surface (`AGENTS.md` 7), so the two values are rendered as
 * selectable text instead of a button that would fail.
 */
function BundleSection({ run, unit }: { readonly run: GateRunDto; readonly unit: SizeUnit }) {
  const [preview, setPreview] = useState<BundlePreviewDto | null>(null);
  const [destination, setDestination] = useState<BundleDestinationDto | null>(null);
  const [outcome, setOutcome] = useState<BundleExportDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  /** The bundle folder name awaiting an explicit replace decision (§30). */
  const [confirm, setConfirm] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [note, setNote] = useState<string | null>(null);
  const confirmRef = useRef<HTMLButtonElement>(null);

  const ready = run.dispositionEffectiveSeverity === 'PASS';

  /**
   * A destination the engine has already told us is occupied by something it did not write.
   *
   * `recognizableBundle` comes from the same `is_recognizable_bundle` the write path consults before
   * it replaces anything, so this page cannot disagree with the engine about what counts as a
   * bundle. That makes the folder unreplaceable by rule, which means asking the release owner to
   * authorize replacing it would be a question with no possible answer.
   */
  const foreignOccupied =
    destination !== null && destination.exists && !destination.recognizableBundle;

  // §28: a new run invalidates whatever was prepared against the old one.
  useEffect(() => {
    setPreview(null);
    setDestination(null);
    setOutcome(null);
    setConfirm(null);
    setError(null);
    setNote(null);
  }, [run.runId]);

  // Focus goes to the decision the screen just asked for, so the keyboard path matches the visual one.
  useEffect(() => {
    if (confirm !== null) {
      confirmRef.current?.focus();
    }
  }, [confirm]);

  const prepare = useCallback(async () => {
    setBusy(true);
    setError(null);
    setNote(null);
    const outcome = await prepareReleaseBundle({
      snapshotId: run.snapshotId,
      baselineSnapshotId: run.baselineSnapshotId,
      gateRunId: run.runId,
    });
    setBusy(false);
    if (outcome.ok) {
      setPreview(outcome.value);
      setDestination(null);
      setOutcome(null);
      setConfirm(null);
      return;
    }
    setPreview(null);
    setError(outcome.envelope);
  }, [run.baselineSnapshotId, run.runId, run.snapshotId]);

  const choose = useCallback(async () => {
    if (preview === null) {
      return;
    }
    setBusy(true);
    setError(null);
    const result = await chooseBundleDestination(preview.planId);
    setBusy(false);
    if (!result.ok) {
      setError(result.envelope);
      return;
    }
    // Cancel is a decision, not a failure: the preview stays and the screen says what happened.
    if (result.value === null) {
      setNote('No folder was chosen. The bundle has not been written.');
      return;
    }
    setNote(null);
    setDestination(result.value);
    setOutcome(null);
    setConfirm(null);
  }, [preview]);

  const exportBundle = useCallback(
    async (overwrite: boolean) => {
      if (preview === null || destination === null) {
        return;
      }
      setBusy(true);
      const result = await exportReleaseBundle(preview.planId, destination.destinationToken, overwrite);
      setBusy(false);
      if (result.ok) {
        setOutcome(result.value);
        setError(null);
        setConfirm(null);
        return;
      }
      setError(result.envelope);
      // A refusal retires the question the card was asking. Leaving it up would keep asserting a
      // fact the engine just denied, which is what E2E-F002 recorded after `ERR-BUNDLE-6107`.
      setConfirm(null);
      if (result.envelope.code === 'ERR-BUNDLE-6106') {
        // §30: the first attempt against an occupied destination asks, and only asks. Nothing is
        // replaced until the release owner presses Replace — and only a folder this engine wrote is
        // ever offered that decision, because the engine refuses to replace any other kind.
        if (destination.recognizableBundle) {
          setConfirm(destination.bundleFolderName);
        }
        return;
      }
      if (STALE_PLAN_CODES.includes(result.envelope.code)) {
        setPreview(null);
        setDestination(null);
        setConfirm(null);
      }
    },
    [destination, preview],
  );

  return (
    <section className={styles['section']} aria-labelledby="fs-bundle-heading">
      <h2 id="fs-bundle-heading">6 · Release Bundle</h2>
      {ready ? (
        <p className={styles['hint']}>
          This run&rsquo;s disposition is PASS, so the release can be packaged into a folder you choose.
          The bundle is the analysis, the comparison, the Gate result, the accepted reviews, the Release
          Notes and the current artifacts — readable without FirmwareSight and checkable against its own
          SHA256SUMS.
        </p>
      ) : (
        <p className={styles['stale']} role="status">
          No bundle is prepared: the disposition is {run.dispositionEffectiveSeverity}. A release bundle is
          written only for a PASS disposition, so resolve what the findings above report, accept what a
          named reviewer accepts, and run the Gate again.
        </p>
      )}
      <div className={styles['actions']}>
        <Button
          variant="primary"
          disabled={!ready || busy}
          onClick={() => void prepare()}
        >
          {busy && preview === null ? 'Preparing…' : 'Prepare bundle'}
        </Button>
        <Button
          disabled={preview === null || busy}
          onClick={() => void choose()}
        >
          {busy && preview !== null && destination === null
            ? 'Choosing…'
            : 'Choose destination folder'}
        </Button>
        <Button
          disabled={preview === null || destination === null || busy || foreignOccupied}
          onClick={() => void exportBundle(false)}
        >
          {busy && outcome === null ? 'Writing…' : 'Export bundle'}
        </Button>
      </div>
      {foreignOccupied && destination !== null ? (
        <p className={styles['stale']} role="status">
          <span className={styles['mono']}>{destination.bundleFolderName}</span> is already there, and
          it is not a bundle this engine wrote, so nothing will be replaced and Export is disabled.
          Choose another destination folder to export.
        </p>
      ) : null}
      {!ready && !busy && preview === null ? (
        <p className={styles['hint']}>
          Prepare is disabled because of the disposition above, not because of anything this page decided.
        </p>
      ) : null}
      {note === null ? null : (
        <p className={styles['status']} role="status">
          {note}
        </p>
      )}
      {error === null ? null : (
        <ErrorPanel envelope={error} label="Bundle step failed" heading="Release bundle error" />
      )}
      {confirm === null ? null : (
        <div className={styles['block']} role="group" aria-labelledby="fs-bundle-confirm">
          <h3 id="fs-bundle-confirm">Replace the existing bundle?</h3>
          <p className={styles['hint']}>
            <span className={styles['mono']}>{confirm}</span> already holds a FirmwareSight release bundle.
            Replacing it moves that bundle aside and removes it only once the new one is written and
            verified; keeping it writes nothing.
          </p>
          <div className={styles['actions']}>
            <Button
              variant="primary"
              ref={confirmRef}

              disabled={busy}
              onClick={() => void exportBundle(true)}
            >
              Replace the existing bundle named {confirm}
            </Button>
            <Button
              onClick={() => {
                setConfirm(null);
                setError(null);
                setNote('Kept the bundle that was there. Nothing was written.');
              }}
            >
              Keep it
            </Button>
          </div>
        </div>
      )}
      {preview === null ? null : <BundlePreview preview={preview} unit={unit} />}
      {destination === null || preview === null ? null : (
        <p className={styles['status']} role="status">
          Destination chosen for <span className={styles['mono']}>{destination.bundleFolderName}</span>.
          {destination.exists
            ? ` A folder of that name is already there${
                destination.recognizableBundle
                  ? ', and it reads as a FirmwareSight bundle.'
                  : ', and it is not a bundle this engine wrote — it will not be replaced.'
              }`
            : ' Nothing of that name is there yet.'}
        </p>
      )}
      {outcome === null ? null : <BundleResult outcome={outcome} />}
    </section>
  );
}

/**
 * The preview a release owner authorizes (§49): what the release is, which two builds it stands on, and
 * every file it will hold with the digest the engine computed for it.
 *
 * No field here is editable, and none is assembled by this page: the list is the plan&rsquo;s own, in the
 * order Core&rsquo;s bundle-path rule sorts it, which is why a reader sees the same order the manifest
 * records.
 */
function BundlePreview({
  preview,
  unit,
}: {
  readonly preview: BundlePreviewDto;
  readonly unit: SizeUnit;
}) {
  return (
    <div className={styles['block']}>
      <h3>Bundle preview</h3>
      <dl className={styles['rows']}>
        <div className={styles['row']}>
          <dt className={styles['term']}>Release</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{preview.releaseId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Version</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{preview.releaseVersion}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Current build</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{preview.snapshotId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Baseline</dt>
          <dd className={cx(styles['value'], styles['mono'])}>
            {preview.baselineSnapshotId ?? 'None'}
          </dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Gate run</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{preview.gateRunId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Accepted reviews</dt>
          <dd className={styles['value']}>{preview.acceptedReviewCount}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Proposed folder</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{preview.bundleFolderName}</dd>
        </div>
      </dl>
      <ScrollArea label="Bundle file list">
        <table className={styles['table']}>
          <caption className={styles['caption']}>
            Every file the bundle will hold, named as it will appear inside it. The digests are the ones the
            plan computed before anything was written, and SHA256SUMS and the manifest repeat them.
          </caption>
          <thead>
            <tr>
              <th scope="col">File</th>
              <th scope="col">Role</th>
              <th scope="col">Size</th>
              <th scope="col">SHA-256</th>
            </tr>
          </thead>
          <tbody>
            {preview.files.map((file) => (
              <tr key={file.path}>
                <th scope="row" className={styles['mono']}>
                  {file.path}
                </th>
                <td className={styles['mono']}>{file.role}</td>
                <td className={styles['mono']}>{formatSize(file.byteSize, unit)}</td>
                <td className={styles['monoSmall']}>{truncateMiddle(file.sha256, 8)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </ScrollArea>
      {preview.warnings.length === 0 ? null : (
        <ul className={styles['noticeList']} aria-label="Plan warnings">
          {preview.warnings.map((warning) => (
            <li className={styles['noticeItem']} key={warning}>
              Warning: {warning}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** What an export produced (§50): the bundle&rsquo;s own name and digest, and what is in it. */
function BundleResult({ outcome }: { readonly outcome: BundleExportDto }) {
  return (
    <div className={styles['block']}>
      <h3>Bundle created</h3>
      <dl className={styles['rows']}>
        <div className={styles['row']}>
          <dt className={styles['term']}>Release</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{outcome.releaseId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Folder</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{outcome.folderDisplayName}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Manifest SHA-256</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{outcome.manifestSha256}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>Files</dt>
          <dd className={styles['value']}>
            {outcome.fileCount} ({outcome.artifactCount} artifact
            {outcome.artifactCount === 1 ? '' : 's'})
          </dd>
        </div>
      </dl>
      <p className={styles['hint']} role="status">
        {outcome.replaced
          ? 'This export replaced the bundle that was there, under your confirmation. '
          : ''}
        The written folder was verified against its own SHA256SUMS and manifest before it was put in
        place.{' '}
        {outcome.recordWritten
          ? 'The release is recorded in this database.'
          : 'The release record was not written; the bundle itself is complete.'}
      </p>
    </div>
  );
}

function GrowthRow({ row, unit }: { readonly row: GateGrowthRowDto; readonly unit: SizeUnit }) {
  return (
    <tr>
      <th scope="row">{row.label}</th>
      <td className={styles['mono']}>{formatSize(row.oldBytes, unit)}</td>
      <td className={styles['mono']}>{formatSize(row.newBytes, unit)}</td>
      <td className={styles['mono']}>{formatDelta(row.deltaBytes, unit)}</td>
      <td className={styles['mono']}>{formatSize(row.thresholdBytes, unit)}</td>
      <td className={styles['wrap']}>
        <StateBadge state={stateName(row.state)} label={row.state} />
        <span className={styles['note']}> effective {row.effectiveSeverity}</span>
        <span className={styles['note']}>
          {' · delta '}
          {row.comparability}
          {row.reason === null ? '' : `: ${row.reason}`}
        </span>
      </td>
    </tr>
  );
}

/**
 * The policy editor. Every field is a named policy value; no field takes a path, a document or a
 * command.
 *
 * Each control has three positions: the two it can state, and &ldquo;not stated&rdquo;. The third one is
 * what keeps the defaults out of this file.
 */
function PolicyForm({
  draft,
  onChange,
}: {
  readonly draft: ProjectPolicyDto;
  readonly onChange: (draft: ProjectPolicyDto) => void;
}) {
  const patch = (part: Partial<ProjectPolicyDto>): void => {
    onChange({ ...draft, ...part });
  };
  const bytes = (value: string): number | null => {
    const trimmed = value.trim();
    if (trimmed === '') {
      return null;
    }
    const parsed = Number(trimmed);
    return Number.isFinite(parsed) && parsed >= 0 ? Math.trunc(parsed) : null;
  };

  return (
    <form
      className={styles['form']}
      onSubmit={(event) => {
        // The Save control is the one that writes; a form here exists for labels and the keyboard, not
        // for a second submit path.
        event.preventDefault();
      }}
    >
      <fieldset className={styles['fields']}>
        <legend>Policy fields</legend>
        <label className={styles['field']} htmlFor="fs-policy-name">
          <span className={styles['pickerLabel']}>Project name</span>
          <input
            id="fs-policy-name"
            className={styles['input']}
            value={draft.projectName}
            onChange={(event) => {
              patch({ projectName: event.target.value });
            }}
          />
        </label>

        <fieldset className={styles['nested']}>
          <legend>Required artifacts</legend>
          {ARTIFACT_KINDS.map((kind) => {
            const checked = draft.requiredArtifactKinds?.includes(kind) ?? false;
            return (
              <label className={styles['radio']} key={kind} htmlFor={`fs-required-${kind}`}>
                <input
                  id={`fs-required-${kind}`}
                  type="checkbox"
                  checked={checked}
                  onChange={(event) => {
                    const current = draft.requiredArtifactKinds ?? [];
                    patch({
                      requiredArtifactKinds: event.target.checked
                        ? [...current, kind]
                        : current.filter((entry) => entry !== kind),
                    });
                  }}
                />
                <span>{kind}</span>
              </label>
            );
          })}
          <p className={styles['hint']}>
            Leaving none checked states nothing, and the documented default applies: a build must hold an
            ELF file. A required kind the build does not hold blocks the run.
          </p>
        </fieldset>

        <div className={styles['grid']}>
          <label className={styles['field']} htmlFor="fs-flash-budget">
            <span className={styles['pickerLabel']}>FLASH budget (bytes)</span>
            <input
              id="fs-flash-budget"
              className={styles['input']}
              inputMode="numeric"
              placeholder="not stated"
              value={draft.flashBudget ?? ''}
              onChange={(event) => {
                patch({ flashBudget: bytes(event.target.value) });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-ram-budget">
            <span className={styles['pickerLabel']}>RAM budget (bytes)</span>
            <input
              id="fs-ram-budget"
              className={styles['input']}
              inputMode="numeric"
              placeholder="not stated"
              value={draft.ramBudget ?? ''}
              onChange={(event) => {
                patch({ ramBudget: bytes(event.target.value) });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-flash-growth">
            <span className={styles['pickerLabel']}>FLASH growth review threshold (bytes)</span>
            <input
              id="fs-flash-growth"
              className={styles['input']}
              inputMode="numeric"
              placeholder="not stated"
              value={draft.flashGrowthReviewBytes ?? ''}
              onChange={(event) => {
                patch({ flashGrowthReviewBytes: bytes(event.target.value) });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-ram-growth">
            <span className={styles['pickerLabel']}>RAM growth review threshold (bytes)</span>
            <input
              id="fs-ram-growth"
              className={styles['input']}
              inputMode="numeric"
              placeholder="not stated"
              value={draft.ramGrowthReviewBytes ?? ''}
              onChange={(event) => {
                patch({ ramGrowthReviewBytes: bytes(event.target.value) });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-unknown-count">
            <span className={styles['pickerLabel']}>Unknown evidence review count</span>
            <input
              id="fs-unknown-count"
              className={styles['input']}
              inputMode="numeric"
              placeholder="not stated"
              value={draft.unknownEvidenceReviewCount ?? ''}
              onChange={(event) => {
                patch({ unknownEvidenceReviewCount: bytes(event.target.value) });
              }}
            />
          </label>
        </div>

        <div className={styles['grid']}>
          <TriSelect
            id="fs-clean-git"
            label="Require a clean workspace (git.clean)"
            value={toTri(draft.requireCleanGit)}
            onChange={(value) => {
              patch({ requireCleanGit: fromTri(value) });
            }}
          />
          <TriSelect
            id="fs-require-notes"
            label="Require release notes"
            value={toTri(draft.requireReleaseNotes)}
            onChange={(value) => {
              patch({ requireReleaseNotes: fromTri(value) });
            }}
          />
          <label className={styles['field']} htmlFor="fs-notes-path">
            <span className={styles['pickerLabel']}>Release notes path (relative to the project)</span>
            <input
              id="fs-notes-path"
              className={styles['input']}
              placeholder="not stated"
              value={draft.releaseNotesPath ?? ''}
              onChange={(event) => {
                patch({ releaseNotesPath: event.target.value === '' ? null : event.target.value });
              }}
            />
          </label>
        </div>

        <div className={styles['grid']}>
          <label className={styles['field']} htmlFor="fs-version-source">
            <span className={styles['pickerLabel']}>Version source</span>
            <select
              id="fs-version-source"
              className={styles['select']}
              value={draft.versionSource ?? 'unset'}
              onChange={(event) => {
                const value = event.target.value;
                patch({ versionSource: value === 'unset' ? null : value });
              }}
            >
              <option value="unset">Not stated (no [version] section)</option>
              <option value="git_tag">Git tag on workspace HEAD</option>
            </select>
          </label>
          <label className={styles['field']} htmlFor="fs-version-pattern">
            <span className={styles['pickerLabel']}>Version pattern (regular expression)</span>
            <input
              id="fs-version-pattern"
              className={styles['input']}
              placeholder="not stated"
              value={draft.versionPattern ?? ''}
              onChange={(event) => {
                patch({ versionPattern: event.target.value === '' ? null : event.target.value });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-expected-version">
            <span className={styles['pickerLabel']}>Expected version</span>
            <input
              id="fs-expected-version"
              className={styles['input']}
              placeholder="not stated"
              value={draft.expectedVersion ?? ''}
              onChange={(event) => {
                patch({ expectedVersion: event.target.value === '' ? null : event.target.value });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-expected-commit">
            <span className={styles['pickerLabel']}>Expected commit</span>
            <input
              id="fs-expected-commit"
              className={styles['input']}
              placeholder="not stated"
              value={draft.expectedCommit ?? ''}
              onChange={(event) => {
                patch({ expectedCommit: event.target.value === '' ? null : event.target.value });
              }}
            />
          </label>
        </div>

        <fieldset className={styles['nested']}>
          <legend>When evidence is missing (on_unknown)</legend>
          {UNKNOWN_RULES.map((rule) => (
            <div className={styles['unknownRow']} key={rule.key}>
              <span className={styles['monoSmall']}>{rule.label}</span>
              <select
                id={`fs-unknown-${rule.key}`}
                className={styles['select']}
                aria-label={`What ${rule.label} becomes when its evidence is missing`}
                value={draft.onUnknown[rule.key] ?? 'unset'}
                onChange={(event) => {
                  const value = event.target.value;
                  onChange({
                    ...draft,
                    onUnknown: {
                      ...draft.onUnknown,
                      [rule.key]: value === 'unset' ? null : (value as UnknownDispositionDto),
                    },
                  });
                }}
              >
                <option value="unset">Not stated (the documented default applies)</option>
                <option value="review">Review</option>
                <option value="block">Block</option>
              </select>
            </div>
          ))}
          <p className={styles['hint']}>
            A missing fact never becomes a pass. It becomes a review or a block, and that choice is
            recorded here rather than assumed by the screen.
          </p>
        </fieldset>
      </fieldset>
    </form>
  );
}

function TriSelect({
  id,
  label,
  value,
  onChange,
}: {
  readonly id: string;
  readonly label: string;
  readonly value: TriState;
  readonly onChange: (value: TriState) => void;
}) {
  return (
    <label className={styles['field']} htmlFor={id}>
      <span className={styles['pickerLabel']}>{label}</span>
      <select
        id={id}
        className={styles['select']}
        value={value}
        onChange={(event) => {
          onChange(event.target.value as TriState);
        }}
      >
        <option value="unset">Not stated (the documented default applies)</option>
        <option value="yes">Yes</option>
        <option value="no">No</option>
      </select>
    </label>
  );
}

/**
 * A stored build&rsquo;s own recorded facts, shown beside the selector that names it. A reader choosing
 * between two builds chooses on these, and they are Rust&rsquo;s numbers, not a re-derivation (US-001).
 */
function BuildFacts({
  row,
  unit,
}: {
  readonly row: CompareCandidateDto | undefined;
  readonly unit: SizeUnit;
}) {
  if (row === undefined) {
    return <span className={styles['hint']}>No build is chosen.</span>;
  }
  return (
    <span className={styles['sideFacts']}>
      <span className={styles['monoSmall']}>SHA-256 {truncateMiddle(row.sha256, 8)}</span>
      <span className={styles['mono']}>{formatSize(row.byteSize, unit)}</span>
      <span>
        nonvolatile {formatSize(row.nonvolatile.bytes, unit)} ({row.nonvolatile.state}) · runtime{' '}
        {formatSize(row.runtimeRam.bytes, unit)} ({row.runtimeRam.state})
      </span>
      <span>stored {row.importedAt}</span>
    </span>
  );
}

/** The three ways the notes rule can have seen a file, in the words prompt §53 asks for. */
function notesPresence(present: boolean | null): string {
  if (present === true) {
    return 'Present';
  }
  if (present === false) {
    return 'Missing';
  }
  return 'Not observed';
}

/**
 * The verdict band, first on the page.
 *
 * `DESIGN.md` 9 forbids this sentence from reaching past policy readiness into a legal, security or
 * compliance claim, and the page's own subhead already says so; the band repeats nothing for that reason.
 * What it does add is the case the numbered sections handled worst: when accepted reviews change the
 * aggregate, the two severities are different facts, and showing only one of them would let a reader
 * believe the build cleared on its own.
 */
function Verdict({ run }: { readonly run: GateRunDto }) {
  const severity = severityState(run.dispositionEffectiveSeverity);

  return (
    <section className={styles['verdict']} aria-label="Gate verdict">
      <StateBadge variant="chip" state={severity} label={severity} />
      <p className={styles['verdictSentence']}>{gateVerdictSentence(run)}</p>
      <StatusStrip
        counts={{
          PASS: run.counts.pass,
          REVIEW: run.counts.review,
          BLOCK: run.counts.block,
          UNKNOWN: run.counts.unknown,
          'N/A': run.counts.notApplicable,
        }}
        label="Findings by state in this run"
      />
    </section>
  );
}

/**
 * The five Gate state words, exactly.
 *
 * A value outside this set is shown as `UNKNOWN` rather than dropped: the alternative is a finding with
 * no state on the screen, which would hide the one thing a reader has to inspect.
 */
function stateName(value: string): StateName {
  return STATE_NAMES.includes(value) ? (value as StateName) : 'UNKNOWN';
}

const toTri = (value: boolean | null): TriState =>
  value === null ? 'unset' : value ? 'yes' : 'no';

const fromTri = (value: TriState): boolean | null => (value === 'unset' ? null : value === 'yes');

/**
 * A policy with nothing stated in it, for the first config a project will ever have.
 *
 * Every field is `null`, which the shell resolves against the defaults it owns. This function names no
 * default value at all, on purpose: it is the reason the front end cannot drift from the backend.
 */
function blankPolicy(projectName: string): ProjectPolicyDto {
  return {
    projectName,
    requiredArtifactKinds: null,
    flashBudget: null,
    ramBudget: null,
    requireCleanGit: null,
    requireReleaseNotes: null,
    releaseNotesPath: null,
    versionSource: null,
    versionPattern: null,
    expectedVersion: null,
    expectedCommit: null,
    flashGrowthReviewBytes: null,
    ramGrowthReviewBytes: null,
    unknownEvidenceReviewCount: null,
    onUnknown: {
      gitClean: null,
      commitMatchesRelease: null,
      versionMatch: null,
      flashBudget: null,
      ramBudget: null,
      baselineGrowth: null,
      releaseNotes: null,
    },
  };
}
