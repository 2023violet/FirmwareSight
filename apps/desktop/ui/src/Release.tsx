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
 *    Release Bundle is a numbered section of this page, disabled with its reason while the disposition says otherwise,
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
import { useI18n, type Translate } from './i18n';
import type { Catalog } from './i18n/en';
import { gateVerdictSentence, severityState } from './stateWords';
import {
  acceptReview,
  attachReleaseFile,
  chooseBundleDestination,
  exportReleaseBundle,
  getGateRun,
  getReleaseAttachments,
  listCompareCandidates,
  openProjectConfig,
  prepareReleaseBundle,
  removeReleaseAttachment,
  runReleaseGate,
  saveProjectPolicy,
  setReleaseAttachmentKind,
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
  ReleaseAttachmentsDto,
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

/** A key the `gate` namespace actually holds, so a typo is a compile error rather than a runtime hole. */
type GateKey = keyof Catalog['gate'];

/**
 * Findings are grouped and read in this order: what blocks, then what asks, then what is unknown.
 *
 * The order lives here and the words live in the catalog: the five states are Core's, but the plain-language
 * label beside each badge and the sentence under each group heading are interface copy (`ADR-0032` D5).
 */
const STATE_GROUPS: readonly {
  readonly state: StateName;
  readonly labelKey: GateKey;
  readonly hintKey: GateKey;
}[] = [
  { state: 'BLOCK', labelKey: 'group.block.label', hintKey: 'group.block.hint' },
  { state: 'REVIEW', labelKey: 'group.review.label', hintKey: 'group.review.hint' },
  { state: 'UNKNOWN', labelKey: 'group.unknown.label', hintKey: 'group.unknown.hint' },
  { state: 'PASS', labelKey: 'group.pass.label', hintKey: 'group.pass.hint' },
  { state: 'N/A', labelKey: 'group.na.label', hintKey: 'group.na.hint' },
];

/** The same five groups, in the same order, with this screen's words attached. */
function stateGroups(
  t: Translate,
): readonly { readonly state: StateName; readonly heading: string; readonly hint: string }[] {
  return STATE_GROUPS.map((group) => ({
    state: group.state,
    heading: t('gate', group.labelKey),
    hint: t('gate', group.hintKey),
  }));
}

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
  const { t } = useI18n();
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
        setPolicyNote(t('release', 'policy.loaded', { file: outcome.value.configFileName }));
      }
      return;
    }
    setPolicyError(outcome.envelope);
  }, [onProjectChange, t]);

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
        setPolicyNote(t('release', 'policy.saved', { file: outcome.value.configFileName }));
        setDraft(null);
      }
      return;
    }
    setPolicyError(outcome.envelope);
  }, [draft, onProjectChange, t]);

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

  /**
   * Invalidate the run on screen because the attachment set it judged is no longer the set the page holds.
   *
   * The set is part of what a run identity binds (`04_TECH/28` §3), so a verdict about three files is not a
   * verdict about two. The page drops the record — and with it the bundle preview, which lives inside the
   * run region and holds the set its plan was built from — instead of leaving a `PASS` standing for bytes
   * it never judged. Pressing Run Gate again makes a new record of the set that is actually held.
   */
  const invalidateRun = useCallback(() => {
    onRunChange(null);
    setRestored(false);
  }, [onRunChange]);

  return (
    <Page>
      <PageHeader
        title={t('gate', 'title')}
        subhead={t('gate', 'subhead')}
      />

      {/* U1 §B.5: the verdict used to sit below the policy editor and the build picker, so a reader had
          to scroll past both to find out whether the build can ship. The counts below are
          `GateCountsDto` verbatim and the severity is Core's disposition aggregate; this band displays
          both and decides neither. */}
      {run === null ? null : <Verdict run={run} />}

      {/* U1P §11: once a run exists, the run's own content is the page. The grouped findings follow the
          verdict immediately, the run record and the evidence tables follow them, and the two sections
          that configured the attempt drop below in a quieter surface - still open, still labelled, still
          the only way to change the build or judge again (§11D forbids hiding a control a reader needs).
          Nothing inside these regions changed: the same components, the same Core words, the same order
          of the five state groups. */}
      {run === null ? null : (
        <>
          {error === null ? null : (
            <p className={styles['stale']} role="status">
              {t('gate', 'staleRun', { id: truncateMiddle(run.runId, 10) })}
            </p>
          )}
          <Findings run={run} onAccept={accept} />
          <Readiness run={run} restored={restored} onReload={() => void reload()} />
          {run.tables === null ? null : <Evidence run={run} unit={unit} />}
          <Attachments
            gateRunId={run.runId}
            unit={unit}
            onSetChanged={invalidateRun}
          />
          <BundleSection run={run} unit={unit} />
        </>
      )}

      {/* The same region before any run exists, because attaching is something a person does *before*
          judging: a policy that requires a BIN is not satisfiable from this page until a file is chosen,
          and a control that only appears after the verdict it feeds would be a dead end. */}
      {run === null ? (
        <Attachments gateRunId={null} unit={unit} onSetChanged={invalidateRun} />
      ) : null}

      <section
        className={cx(styles['section'], run === null ? undefined : styles['secondary'])}
        aria-labelledby="fs-policy-heading"
      >
        <h2 id="fs-policy-heading">{'5 · '}{t('release', 'policy.heading')}</h2>
        <div className={styles['row']}>
          <span className={styles['term']}>{t('release', 'policy.configTerm')}</span>
          <span className={styles['value']}>
            {project === null ? (
              <span>{t('release', 'policy.noProject')}</span>
            ) : (
              <>
                <span>{project.projectName}</span>
                <span className={styles['monoSmall']}>{project.configFileName}</span>
                <span className={styles['note']}>
                  {t('release', 'policy.schemaLine', { version: project.configSchemaVersion })}{' '}
                  <span className={styles['mono']}>{project.policySha256}</span>
                </span>
              </>
            )}
          </span>
        </div>
        {project !== null && project.unknownKeys.length > 0 ? (
          <p className={styles['warning']} role="status">
            {t('release', 'policy.unknownKeys', { keys: project.unknownKeys.join(', ') })}
          </p>
        ) : null}
        {project !== null && project.warnings.length > 0 ? (
          <ul className={styles['noticeList']} aria-label={t('release', 'warnings.configList')}>
            {project.warnings.map((warning) => (
              <li className={styles['noticeItem']} key={warning}>
                {t('release', 'warnings.item', { warning })}
              </li>
            ))}
          </ul>
        ) : null}
        <div className={styles['actions']}>
          <Button onClick={() => void open()}>{t('release', 'policy.open')}</Button>
          <Button
            onClick={() => {
              setDraft(draft === null ? (seed ?? blankPolicy(project?.projectName ?? '')) : null);
            }}
            ariaExpanded={draft !== null}
          >
            {draft === null ? t('release', 'policy.edit') : t('release', 'policy.close')}
          </Button>
          <Button
            variant="primary"
            disabled={draft === null || saving}
            onClick={() => void save()}
          >
            {saving
              ? t('release', 'policy.saving')
              : project === null
                ? t('release', 'policy.saveAs', { file: 'firmwaresight.toml' })
                : t('release', 'policy.save')}
          </Button>
          {policyNote === null ? null : <span className={styles['status']}>{policyNote}</span>}
        </div>
        {policyError === null ? null : (
          <ErrorPanel
            envelope={policyError}
            label={t('release', 'policy.errorLabel')}
            heading={t('release', 'policy.errorHeading')}
          />
        )}
        {draft === null ? (
          <p className={styles['hint']}>
            {project === null
              ? t('release', 'policy.hintNoProject')
              : t('release', 'policy.hintEdit', { file: project.configFileName })}
          </p>
        ) : (
          <PolicyForm draft={draft} onChange={setDraft} />
        )}
      </section>

      <section
        className={cx(styles['section'], run === null ? undefined : styles['secondary'])}
        aria-labelledby="fs-builds-heading"
      >
        <h2 id="fs-builds-heading">{'6 · '}{t('release', 'builds.heading')}</h2>
        {candidateError === null ? null : (
          <ErrorPanel
            envelope={candidateError}
            label={t('release', 'builds.errorLabel')}
            heading={t('release', 'builds.errorHeading')}
          />
        )}
        {candidates !== null && candidates.rows.length === 0 ? (
          <p className={styles['empty']}>
            {t('release', 'builds.emptyBefore')}{' '}
            <LinkButton onClick={onGoToAnalyze}>{t('release', 'builds.emptyLink')}</LinkButton>{' '}
            {t('release', 'builds.emptyAfter')}
          </p>
        ) : (
          <div className={styles['selectors']}>
            <div className={styles['picker']}>
              <label className={styles['pickerLabel']} htmlFor="fs-gate-current">
                {t('release', 'builds.currentLabel')}
              </label>
              <select
                id="fs-gate-current"
                className={styles['select']}
                value={snapshotId ?? ''}
                onChange={(event) => {
                  setSnapshotId(event.target.value === '' ? null : event.target.value);
                }}
              >
                <option value="">{t('release', 'builds.chooseOption')}</option>
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
                {t('release', 'builds.baselineLabel')}
              </label>
              <select
                id="fs-gate-baseline"
                className={styles['select']}
                value={baselineId ?? ''}
                onChange={(event) => {
                  setBaselineId(event.target.value === '' ? null : event.target.value);
                }}
              >
                <option value="">{t('release', 'builds.noneOption')}</option>
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
                <span className={styles['hint']}>{t('release', 'builds.baselineHint')}</span>
              ) : null}
            </div>
            <div className={styles['pickerActions']}>
              <Button
                variant="primary"
                disabled={snapshotId === null || running}
                onClick={() => void gate()}
              >
                {running ? t('release', 'builds.judging') : t('release', 'builds.run')}
              </Button>
              <SizeUnitSwitch unit={unit} onSelect={onUnitChange} />
            </div>
          </div>
        )}
      </section>

      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label={t('gate', 'run.errorLabel')}
          heading={t('gate', 'run.errorHeading')}
        />
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
  const { t } = useI18n();
  return (
    <section className={styles['section']} aria-labelledby="fs-readiness-heading">
      <h2 id="fs-readiness-heading">{'2 · '}{t('gate', 'readiness.heading')}</h2>
      <div className={styles['states']}>
        <StateBadge
          state={stateName(run.dispositionEffectiveSeverity)}
          label={t('gate', 'readiness.disposition', { severity: run.dispositionEffectiveSeverity })}
          note={t('gate', 'readiness.dispositionNote')}
        />
        <StateBadge
          state={stateName(run.overallEffectiveSeverity)}
          label={t('gate', 'readiness.asComputed', { severity: run.overallEffectiveSeverity })}
          note={t('gate', 'readiness.asComputedNote')}
        />
      </div>
      <div className={styles['counts']} role="group" aria-label={t('gate', 'readiness.countsGroup')}>
        <StateBadge state="BLOCK" label={t('gate', 'group.block.label')} count={run.counts.block} />
        <StateBadge state="REVIEW" label={t('gate', 'group.review.label')} count={run.counts.review} />
        <StateBadge state="UNKNOWN" label={t('gate', 'group.unknown.label')} count={run.counts.unknown} />
        <StateBadge state="PASS" label={t('gate', 'group.pass.label')} count={run.counts.pass} />
        <StateBadge state="N/A" label={t('gate', 'group.na.label')} count={run.counts.notApplicable} />
      </div>
      <dl className={styles['rows']}>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('gate', 'readiness.runTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{run.runId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('gate', 'readiness.buildTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{run.snapshotId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('gate', 'readiness.baselineTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>
            {run.baselineSnapshotId ?? t('gate', 'readiness.none')}
          </dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('gate', 'readiness.policyTerm')}</dt>
          <dd className={styles['value']}>
            <span>{run.policySource}</span>
            {run.projectName === null ? null : <span>{run.projectName}</span>}
            <span className={styles['monoSmall']}>{run.policySha256}</span>
          </dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('gate', 'readiness.storedTerm')}</dt>
          <dd className={cx(styles['value'], styles['monoSmall'])}>{run.createdAt}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('gate', 'readiness.workspaceTerm')}</dt>
          <dd className={styles['value']}>
            <span>{run.git.summary}</span>
            {run.git.headCommit === null ? null : (
              <span className={styles['monoSmall']}>
                HEAD {truncateMiddle(run.git.headCommit, 8)}
              </span>
            )}
            {run.git.exactTag === null ? null : (
              <span className={styles['monoSmall']}>
                {t('gate', 'readiness.exactTag', { tag: run.git.exactTag })}
              </span>
            )}
            {run.git.dirty === null ? null : (
              <span>{t('gate', run.git.dirty ? 'readiness.dirtyLine' : 'readiness.cleanLine')}</span>
            )}
            {run.git.reason === null ? null : (
              <span className={styles['note']}>{run.git.reason}</span>
            )}
          </dd>
        </div>
      </dl>
      <p className={styles['hint']}>{t('gate', 'readiness.gitHint')}</p>
      {run.recordNote === null ? null : (
        <p className={styles['stale']} role="status">
          {run.recordNote}
        </p>
      )}
      {run.warnings.length === 0 ? null : (
        <ul className={styles['noticeList']} aria-label={t('gate', 'warnings.runList')}>
          {run.warnings.map((warning) => (
            <li className={styles['noticeItem']} key={warning}>
              {t('gate', 'warnings.item', { warning })}
            </li>
          ))}
        </ul>
      )}
      <div className={styles['actions']}>
        {restored ? (
          <span className={styles['status']}>{t('gate', 'readiness.historyStatus')}</span>
        ) : (
          <Button onClick={onReload}>{t('gate', 'readiness.reload')}</Button>
        )}
        <span className={styles['status']}>{t('gate', 'readiness.storedRecordNote')}</span>
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
  const { t } = useI18n();
  return (
    <section className={styles['section']} aria-labelledby="fs-findings-heading">
      <h2 id="fs-findings-heading">{'1 · '}{t('gate', 'findings.heading')}</h2>
      {run.findings.length === 0 ? (
        <p className={styles['hint']}>{t('gate', 'findings.empty')}</p>
      ) : null}
      {stateGroups(t).map((group) => {
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
  const { t } = useI18n();
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
        <span className={styles['severity']}>
          {t('gate', 'findings.effectiveSeverity', { severity: finding.effectiveSeverity })}
        </span>
      </div>
      <p className={styles['summary']}>{finding.summary}</p>
      {finding.remediation === null ? null : (
        <p className={styles['remediation']}>
          <span className={styles['axis']}>{t('gate', 'findings.nextStep')}</span> {finding.remediation}
        </p>
      )}
      {finding.evidenceRefs.length === 0 ? (
        <p className={styles['note']}>{t('gate', 'findings.noEvidence')}</p>
      ) : (
        <ul className={styles['refs']} aria-label={t('gate', 'findings.evidenceList', { rule: finding.ruleId })}>
          {finding.evidenceRefs.map((ref) => (
            <li className={styles['monoSmall']} key={ref}>
              {ref}
            </li>
          ))}
        </ul>
      )}
      {finding.acceptance === null ? null : (
        <div className={styles['accepted']} role="status">
          <span className={styles['axis']}>{t('release', 'accept.recordTitle')}</span>
          <dl className={styles['audit']}>
            <div>
              <dt>{t('release', 'accept.by')}</dt>
              <dd>{finding.acceptance.actor}</dd>
            </div>
            <div>
              <dt>{t('release', 'accept.when')}</dt>
              <dd className={styles['monoSmall']}>{finding.acceptance.acceptedAt}</dd>
            </div>
            <div>
              <dt>{t('release', 'accept.reason')}</dt>
              <dd>{finding.acceptance.reason}</dd>
            </div>
            <div>
              <dt>{t('release', 'accept.state')}</dt>
              <dd>{finding.acceptance.originalState}</dd>
            </div>
          </dl>
          <p className={styles['note']}>
            {t('release', 'accept.recordNote', { state: finding.acceptance.originalState })}
          </p>
        </div>
      )}
      {finding.acceptable ? (
        <div className={styles['accept']}>
          {openForm ? (
            <div className={styles['acceptForm']}>
              <label className={styles['field']} htmlFor={`fs-actor-${finding.id}`}>
                <span className={styles['pickerLabel']}>{t('release', 'accept.actorLabel')}</span>
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
                <span className={styles['pickerLabel']}>{t('release', 'accept.reasonLabel')}</span>
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
                  {busy ? t('release', 'accept.recording') : t('release', 'accept.record')}
                </Button>
                <Button
                  onClick={() => {
                    setOpenForm(false);
                  }}
                >
                  {t('release', 'accept.cancel')}
                </Button>
                <span className={styles['status']}>{t('release', 'accept.requiredNote')}</span>
              </div>
              {error === null ? null : (
                <ErrorPanel
                  envelope={error}
                  label={t('release', 'accept.errorLabel', { rule: finding.ruleId })}
                  heading={t('release', 'accept.errorHeading')}
                />
              )}
            </div>
          ) : (
            <Button
              onClick={() => {
                setOpenForm(true);
              }}
            >
              {t('release', 'accept.button')}
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
/**
 * The three words Rust uses for which set satisfied a required kind (`GateArtifactRowDto.satisfiedBy`).
 * A label map, not a decision: the screen never works out satisfaction from `present`.
 */
const SATISFIED_BY: Readonly<Record<string, GateKey>> = {
  analyzed: 'evidence.satisfiedAnalyzed',
  attachment: 'evidence.satisfiedAttachment',
  none: 'evidence.satisfiedNone',
};

/** The same three words in the language on screen; a value outside the map is shown as Rust sent it. */
function satisfiedBy(value: string, t: Translate): string {
  const key = SATISFIED_BY[value];
  return key === undefined ? value : t('gate', key);
}

function Evidence({ run, unit }: { readonly run: GateRunDto; readonly unit: SizeUnit }) {
  const { t } = useI18n();
  const tables = run.tables;
  if (tables === null) {
    return null;
  }
  return (
    <section className={styles['section']} aria-labelledby="fs-evidence-heading">
      <h2 id="fs-evidence-heading">{'3 · '}{t('gate', 'evidence.heading')}</h2>

      <div className={styles['block']}>
        <h3>{t('gate', 'evidence.artifactsHeading')}</h3>
        <ScrollArea label={t('gate', 'evidence.artifactsTable')}>
          <table className={styles['table']}>
            <caption className={styles['caption']}>{t('gate', 'evidence.artifactsCaption')}</caption>
            <thead>
              <tr>
                <th scope="col">{t('gate', 'evidence.colKind')}</th>
                <th scope="col">{t('gate', 'evidence.colPolicy')}</th>
                <th scope="col">{t('gate', 'evidence.colSatisfiedBy')}</th>
                <th scope="col">{t('gate', 'evidence.colSha')}</th>
                <th scope="col">{t('gate', 'evidence.colSize')}</th>
              </tr>
            </thead>
            <tbody>
              {tables.artifacts.map((row) => (
                <tr key={row.kind}>
                  <th scope="row" className={styles['mono']}>
                    {row.kind}
                  </th>
                  <td>
                    {t('gate', row.required ? 'evidence.required' : 'evidence.notRequired')}
                  </td>
                  <td>{satisfiedBy(row.satisfiedBy, t)}</td>
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
        <h3>{t('gate', 'evidence.budgetsHeading')}</h3>
        {tables.budgets.length === 0 ? (
          <p className={styles['hint']}>{t('gate', 'evidence.noBudgets')}</p>
        ) : (
          <ScrollArea label={t('gate', 'evidence.budgetsTable')}>
            <table className={styles['table']}>
              <caption className={styles['caption']}>{t('gate', 'evidence.budgetsCaption')}</caption>
              <thead>
                <tr>
                  <th scope="col">{t('gate', 'evidence.colSide')}</th>
                  <th scope="col">{t('gate', 'evidence.colActual')}</th>
                  <th scope="col">{t('gate', 'evidence.colBudget')}</th>
                  <th scope="col">{t('gate', 'evidence.colMargin')}</th>
                  <th scope="col">{t('gate', 'evidence.colState')}</th>
                  <th scope="col">{t('gate', 'evidence.colEvidenceBasis')}</th>
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
        <h3>{t('gate', 'evidence.growthHeading')}</h3>
        {tables.growth.length === 0 ? (
          <p className={styles['hint']}>{t('gate', 'evidence.noGrowth')}</p>
        ) : (
          <ScrollArea label={t('gate', 'evidence.growthTable')}>
            <table className={styles['table']}>
              <caption className={styles['caption']}>{t('gate', 'evidence.growthCaption')}</caption>
              <thead>
                <tr>
                  <th scope="col">{t('gate', 'evidence.colSide')}</th>
                  <th scope="col">{t('gate', 'evidence.colOld')}</th>
                  <th scope="col">{t('gate', 'evidence.colNew')}</th>
                  <th scope="col">{t('gate', 'evidence.colDelta')}</th>
                  <th scope="col">{t('gate', 'evidence.colReviewThreshold')}</th>
                  <th scope="col">{t('gate', 'evidence.colState')}</th>
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
        <h3>{t('gate', 'evidence.notesHeading')}</h3>
        <div className={styles['row']}>
          <span className={styles['term']}>{t('gate', 'evidence.notesPathTerm')}</span>
          <span className={cx(styles['value'], styles['mono'])}>
            {tables.notes.relativePath ?? t('gate', 'evidence.noNotesPath')}
          </span>
        </div>
        <div className={styles['row']}>
          <span className={styles['term']}>{t('gate', 'evidence.colState')}</span>
          <span className={styles['value']}>
            <StateBadge
              state={stateName(tables.notes.state)}
              label={
                tables.notes.required
                  ? t('gate', 'evidence.notesRequired', {
                      presence: notesPresence(tables.notes.present, t),
                    })
                  : t('gate', 'evidence.notRequired')
              }
            />
            <span className={styles['severity']}>
              {t('gate', 'findings.effectiveSeverity', {
                severity: tables.notes.effectiveSeverity,
              })}
            </span>
          </span>
        </div>
        {tables.notes.sha256 === null ? null : (
          <div className={styles['row']}>
            <span className={styles['term']}>{t('gate', 'evidence.colSha')}</span>
            <span className={cx(styles['value'], styles['monoSmall'])}>{tables.notes.sha256}</span>
          </div>
        )}
        {tables.notes.reason === null ? null : <p className={styles['note']}>{tables.notes.reason}</p>}
        <p className={styles['hint']}>{t('gate', 'evidence.notesHint')}</p>
      </div>
    </section>
  );
}

function BudgetRow({ row, unit }: { readonly row: GateBudgetRowDto; readonly unit: SizeUnit }) {
  const { t } = useI18n();
  const margin =
    row.headroomBytes !== null
      ? t('gate', 'evidence.headroom', { value: formatSize(row.headroomBytes, unit) })
      : row.overBytes !== null
        ? t('gate', 'evidence.over', { value: formatSize(row.overBytes, unit) })
        : t('gate', 'evidence.unknownValue');
  return (
    <tr>
      <th scope="row">{row.label}</th>
      <td className={styles['mono']}>{formatSize(row.actualBytes, unit)}</td>
      <td className={styles['mono']}>{formatSize(row.budgetBytes, unit)}</td>
      <td className={styles['mono']}>{margin}</td>
      <td>
        <StateBadge state={stateName(row.state)} label={row.state} />
        <span className={styles['note']}>
          {' '}
          {t('gate', 'evidence.effective', { severity: row.effectiveSeverity })}
        </span>
      </td>
      <td className={styles['wrap']}>
        <span>
          {row.basis === null ? t('gate', 'evidence.noBasis') : evidenceBasisCaption(row.basis, t)}
        </span>
        <span className={styles['note']}>
          {' · '}
          {t('gate', row.exact ? 'evidence.completeAttribution' : 'evidence.floor')}
          {' · '}
          {t('gate', row.admissible ? 'evidence.maySupport' : 'evidence.cannotBlock')}
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
 * 4 · Release Bundle (prompt §48, §49, §50, §58).
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
/**
 * The three kinds a release may declare for an attached file. The shell owns what they mean —
 * `observe_attachment` refuses anything else, and no word here makes FirmwareSight read a container.
 */
const ATTACHMENT_KINDS: readonly string[] = ['bin', 'hex', 'unknown'];

/**
 * Release attachments: the byte files this release ships without analyzing them.
 *
 * Every value on these rows came from the shell: the length and the digest from one read of the chosen
 * file, the kind from what its name stated or the owner re-declared, and `provenance: unknown` because an
 * attached file's build origin is permanently unestablished (`ADR-0030` D-7). Nothing is hashed, sized or
 * classified on this side of the boundary (`AGENTS.md` 3).
 *
 * The file cell shows `displayName`, the leaf as its owner chose it. `fileName` is the same file's
 * archive-safe name, and it is a different string whenever the chosen name carries characters a portable
 * folder will not hold — `固件镜像.bin` files itself as `bin`, and showing that back to the person who
 * picked the file would be a name they never typed (`N01-F02`).
 *
 * The verbs are **Attach** and **Withdraw** beside the **Ship** flow, never Analyze, and §9's limitation
 * sentence sits in the caption — where a reader is forming the expectation — rather than in a tooltip.
 *
 * The region carries no number on purpose. The six numbered sections of this page are labels a reader
 * quotes in a screenshot and a test asserts by name, so inserting a number here would silently renumber
 * three regions that existing evidence already cites. This one is named by what it is.
 */
function Attachments({
  gateRunId,
  unit,
  onSetChanged,
}: {
  readonly gateRunId: string | null;
  readonly unit: SizeUnit;
  readonly onSetChanged: () => void;
}) {
  const { t } = useI18n();
  const [set, setSet] = useState<ReleaseAttachmentsDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [busy, setBusy] = useState(false);

  /** Re-read the set from the shell, asking it whether the run on screen still matches. */
  const load = useCallback(async (): Promise<void> => {
    const outcome = await getReleaseAttachments(gateRunId);
    if (outcome.ok) {
      setSet(outcome.value);
      setError(null);
      return;
    }
    setError(outcome.envelope);
  }, [gateRunId]);

  useEffect(() => {
    void load();
  }, [load]);

  /** Open the native dialog. A cancel is a decision: the set stays as it was and nothing is reported. */
  const attach = async (): Promise<void> => {
    setBusy(true);
    const outcome = await attachReleaseFile();
    setBusy(false);
    if (!outcome.ok) {
      setError(outcome.envelope);
      return;
    }
    if (outcome.value === null) {
      return;
    }
    await load();
    onSetChanged();
  };

  /** Re-declare one file's kind. The shell re-reads the same bytes, so the digest on the row does not move. */
  const declare = async (attachmentId: string, kind: string): Promise<void> => {
    setBusy(true);
    const outcome = await setReleaseAttachmentKind(attachmentId, kind);
    setBusy(false);
    if (!outcome.ok) {
      setError(outcome.envelope);
      return;
    }
    await load();
    onSetChanged();
  };

  /** Withdraw one file and show the set that remains. */
  const withdraw = async (attachmentId: string): Promise<void> => {
    setBusy(true);
    const outcome = await removeReleaseAttachment(attachmentId);
    setBusy(false);
    if (!outcome.ok) {
      setError(outcome.envelope);
      return;
    }
    await load();
    onSetChanged();
  };

  const rows = set?.attachments ?? [];
  return (
    <section className={styles['section']} aria-labelledby="fs-attachments-heading">
      <h2 id="fs-attachments-heading">{t('release', 'attachments.heading')}</h2>
      <ScrollArea label={t('release', 'attachments.table')}>
        <table className={styles['table']}>
          <caption className={styles['caption']}>{t('release', 'attachments.caption')}</caption>
          <thead>
            <tr>
              <th scope="col">{t('release', 'attachments.colFile')}</th>
              <th scope="col">{t('release', 'attachments.colKind')}</th>
              <th scope="col">{t('release', 'attachments.colSize')}</th>
              <th scope="col">{t('release', 'attachments.colSha')}</th>
              <th scope="col">{t('release', 'attachments.colEvidence')}</th>
              <th scope="col">{t('release', 'attachments.colAction')}</th>
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 ? (
              <tr>
                <td colSpan={6} className={styles['empty']}>
                  {t('release', 'attachments.empty')}
                </td>
              </tr>
            ) : (
              rows.map((row) => (
                <tr key={row.attachmentId}>
                  <th scope="row" className={styles['mono']}>
                    {row.displayName}
                  </th>
                  <td>
                    <select
                      className={styles['select']}
                      aria-label={t('release', 'attachments.kindPicker', { name: row.displayName })}
                      value={row.kind}
                      disabled={busy}
                      onChange={(event) => {
                        void declare(row.attachmentId, event.target.value);
                      }}
                    >
                      {ATTACHMENT_KINDS.map((kind) => (
                        <option key={kind} value={kind}>
                          {kind}
                        </option>
                      ))}
                    </select>
                  </td>
                  <td className={styles['monoSmall']}>{formatSize(row.byteSize, unit)}</td>
                  <td className={styles['monoSmall']}>{row.sha256}</td>
                  <td>
                    {t('release', 'attachments.bytesLine', {
                      basis: row.kindBasis,
                      provenance: row.provenance,
                    })}
                  </td>
                  <td>
                    <Button disabled={busy} onClick={() => void withdraw(row.attachmentId)}>
                      {t('release', 'attachments.withdraw')}
                    </Button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </ScrollArea>
      <div className={styles['actions']}>
        <Button disabled={busy} onClick={() => void attach()}>
          {t('release', 'attachments.attach')}
        </Button>
      </div>
      <p className={styles['hint']}>{t('release', 'attachments.hint')}</p>
      {set?.matchesGateRun === false ? (
        <p className={styles['stale']} role="status">
          {t('release', 'attachments.stale', {
            id: truncateMiddle(set.gateRunId ?? '', 10),
          })}
        </p>
      ) : null}
      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label={t('release', 'attachments.errorLabel')}
          heading={t('release', 'attachments.errorHeading')}
        />
      )}
    </section>
  );
}

function BundleSection({ run, unit }: { readonly run: GateRunDto; readonly unit: SizeUnit }) {
  const { t } = useI18n();
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
      setNote(t('release', 'bundle.noFolder'));
      return;
    }
    setNote(null);
    setDestination(result.value);
    setOutcome(null);
    setConfirm(null);
  }, [preview, t]);

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
      <h2 id="fs-bundle-heading">{'4 · '}{t('release', 'bundleHeading')}</h2>
      {ready ? (
        <p className={styles['hint']}>{t('release', 'bundle.readyHint')}</p>
      ) : (
        <p className={styles['stale']} role="status">
          {t('release', 'bundle.notReady', { severity: run.dispositionEffectiveSeverity })}
        </p>
      )}
      <div className={styles['actions']}>
        <Button
          variant="primary"
          disabled={!ready || busy}
          onClick={() => void prepare()}
        >
          {busy && preview === null
            ? t('release', 'bundle.preparing')
            : t('release', 'bundle.prepare')}
        </Button>
        <Button
          disabled={preview === null || busy}
          onClick={() => void choose()}
        >
          {busy && preview !== null && destination === null
            ? t('release', 'bundle.choosing')
            : t('release', 'bundle.choose')}
        </Button>
        <Button
          disabled={preview === null || destination === null || busy || foreignOccupied}
          onClick={() => void exportBundle(false)}
        >
          {busy && outcome === null ? t('release', 'bundle.writing') : t('release', 'bundle.export')}
        </Button>
      </div>
      {foreignOccupied && destination !== null ? (
        <p className={styles['stale']} role="status">
          <span className={styles['mono']}>{destination.bundleFolderName}</span>{' '}
          {t('release', 'bundle.foreignOccupied')}
        </p>
      ) : null}
      {!ready && !busy && preview === null ? (
        <p className={styles['hint']}>{t('release', 'bundle.prepareDisabledWhy')}</p>
      ) : null}
      {note === null ? null : (
        <p className={styles['status']} role="status">
          {note}
        </p>
      )}
      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label={t('release', 'bundle.errorLabel')}
          heading={t('release', 'bundle.errorHeading')}
        />
      )}
      {confirm === null ? null : (
        <div className={styles['block']} role="group" aria-labelledby="fs-bundle-confirm">
          <h3 id="fs-bundle-confirm">{t('release', 'bundle.replaceTitle')}</h3>
          <p className={styles['hint']}>
            <span className={styles['mono']}>{confirm}</span>{' '}
            {t('release', 'bundle.replaceWhy')}
          </p>
          <div className={styles['actions']}>
            <Button
              variant="primary"
              ref={confirmRef}

              disabled={busy}
              onClick={() => void exportBundle(true)}
            >
              {t('release', 'bundle.replaceButton', { name: confirm })}
            </Button>
            <Button
              onClick={() => {
                setConfirm(null);
                setError(null);
                setNote(t('release', 'bundle.kept'));
              }}
            >
              {t('release', 'bundle.keep')}
            </Button>
          </div>
        </div>
      )}
      {preview === null ? null : <BundlePreview preview={preview} unit={unit} />}
      {destination === null || preview === null ? null : (
        <p className={styles['status']} role="status">
          {t('release', 'bundle.destChosen')}{' '}
          <span className={styles['mono']}>{destination.bundleFolderName}</span>
          {'. '}
          {destination.exists
            ? t(
                'release',
                destination.recognizableBundle
                  ? 'bundle.destOccupiedBundle'
                  : 'bundle.destOccupiedForeign',
              )
            : t('release', 'bundle.destFresh')}
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
  const { t } = useI18n();
  return (
    <div className={styles['block']}>
      <h3>{t('release', 'bundle.previewHeading')}</h3>
      <dl className={styles['rows']}>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('release', 'bundle.releaseTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{preview.releaseId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('release', 'bundle.versionTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{preview.releaseVersion}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('release', 'bundle.currentBuildTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{preview.snapshotId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('release', 'bundle.baselineTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>
            {preview.baselineSnapshotId ?? t('release', 'bundle.noBaseline')}
          </dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('release', 'bundle.gateRunTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{preview.gateRunId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('release', 'bundle.acceptedReviewsTerm')}</dt>
          <dd className={styles['value']}>{preview.acceptedReviewCount}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('release', 'bundle.proposedFolderTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{preview.bundleFolderName}</dd>
        </div>
      </dl>
      <ScrollArea label={t('release', 'bundle.fileList')}>
        <table className={styles['table']}>
          <caption className={styles['caption']}>{t('release', 'bundle.filesCaption')}</caption>
          <thead>
            <tr>
              <th scope="col">{t('release', 'bundle.colFile')}</th>
              <th scope="col">{t('release', 'bundle.colRole')}</th>
              <th scope="col">{t('release', 'bundle.colSize')}</th>
              <th scope="col">{t('release', 'bundle.colSha')}</th>
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
        <ul className={styles['noticeList']} aria-label={t('release', 'warnings.planList')}>
          {preview.warnings.map((warning) => (
            <li className={styles['noticeItem']} key={warning}>
              {t('release', 'warnings.item', { warning })}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** What an export produced (§50): the bundle&rsquo;s own name and digest, and what is in it. */
function BundleResult({ outcome }: { readonly outcome: BundleExportDto }) {
  const { t } = useI18n();
  return (
    <div className={styles['block']}>
      <h3>{t('release', 'bundle.resultHeading')}</h3>
      <dl className={styles['rows']}>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('release', 'bundle.releaseTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{outcome.releaseId}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('release', 'bundle.folderTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{outcome.folderDisplayName}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('release', 'bundle.manifestTerm')}</dt>
          <dd className={cx(styles['value'], styles['mono'])}>{outcome.manifestSha256}</dd>
        </div>
        <div className={styles['row']}>
          <dt className={styles['term']}>{t('release', 'bundle.filesTerm')}</dt>
          <dd className={styles['value']}>
            {outcome.artifactCount === 1
              ? t('release', 'bundle.filesOne', { count: outcome.fileCount })
              : t('release', 'bundle.filesMany', {
                  count: outcome.fileCount,
                  artifacts: outcome.artifactCount,
                })}
          </dd>
        </div>
      </dl>
      <p className={styles['hint']} role="status">
        {outcome.replaced ? `${t('release', 'bundle.replacedNote')} ` : ''}
        {t('release', 'bundle.verifiedNote')}{' '}
        {outcome.recordWritten
          ? t('release', 'bundle.recordWrittenNote')
          : t('release', 'bundle.recordMissingNote')}
      </p>
    </div>
  );
}

function GrowthRow({ row, unit }: { readonly row: GateGrowthRowDto; readonly unit: SizeUnit }) {
  const { t } = useI18n();
  return (
    <tr>
      <th scope="row">{row.label}</th>
      <td className={styles['mono']}>{formatSize(row.oldBytes, unit)}</td>
      <td className={styles['mono']}>{formatSize(row.newBytes, unit)}</td>
      <td className={styles['mono']}>{formatDelta(row.deltaBytes, unit)}</td>
      <td className={styles['mono']}>{formatSize(row.thresholdBytes, unit)}</td>
      <td className={styles['wrap']}>
        <StateBadge state={stateName(row.state)} label={row.state} />
        <span className={styles['note']}>
          {' '}
          {t('gate', 'evidence.effective', { severity: row.effectiveSeverity })}
        </span>
        <span className={styles['note']}>
          {' '}
          {t('gate', 'evidence.deltaComparability', { comparability: row.comparability })}
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
  const { t } = useI18n();
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
        <legend>{t('release', 'form.fieldsLegend')}</legend>
        <label className={styles['field']} htmlFor="fs-policy-name">
          <span className={styles['pickerLabel']}>{t('release', 'form.projectName')}</span>
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
          <legend>{t('release', 'form.requiredArtifacts')}</legend>
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
          <p className={styles['hint']}>{t('release', 'form.requiredHint')}</p>
        </fieldset>

        <div className={styles['grid']}>
          <label className={styles['field']} htmlFor="fs-flash-budget">
            <span className={styles['pickerLabel']}>{t('release', 'form.flashBudget')}</span>
            <input
              id="fs-flash-budget"
              className={styles['input']}
              inputMode="numeric"
              placeholder={t('release', 'form.notStated')}
              value={draft.flashBudget ?? ''}
              onChange={(event) => {
                patch({ flashBudget: bytes(event.target.value) });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-ram-budget">
            <span className={styles['pickerLabel']}>{t('release', 'form.ramBudget')}</span>
            <input
              id="fs-ram-budget"
              className={styles['input']}
              inputMode="numeric"
              placeholder={t('release', 'form.notStated')}
              value={draft.ramBudget ?? ''}
              onChange={(event) => {
                patch({ ramBudget: bytes(event.target.value) });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-flash-growth">
            <span className={styles['pickerLabel']}>{t('release', 'form.flashGrowth')}</span>
            <input
              id="fs-flash-growth"
              className={styles['input']}
              inputMode="numeric"
              placeholder={t('release', 'form.notStated')}
              value={draft.flashGrowthReviewBytes ?? ''}
              onChange={(event) => {
                patch({ flashGrowthReviewBytes: bytes(event.target.value) });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-ram-growth">
            <span className={styles['pickerLabel']}>{t('release', 'form.ramGrowth')}</span>
            <input
              id="fs-ram-growth"
              className={styles['input']}
              inputMode="numeric"
              placeholder={t('release', 'form.notStated')}
              value={draft.ramGrowthReviewBytes ?? ''}
              onChange={(event) => {
                patch({ ramGrowthReviewBytes: bytes(event.target.value) });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-unknown-count">
            <span className={styles['pickerLabel']}>{t('release', 'form.unknownCount')}</span>
            <input
              id="fs-unknown-count"
              className={styles['input']}
              inputMode="numeric"
              placeholder={t('release', 'form.notStated')}
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
            label={t('release', 'form.requireClean')}
            value={toTri(draft.requireCleanGit)}
            onChange={(value) => {
              patch({ requireCleanGit: fromTri(value) });
            }}
          />
          <TriSelect
            id="fs-require-notes"
            label={t('release', 'form.requireNotes')}
            value={toTri(draft.requireReleaseNotes)}
            onChange={(value) => {
              patch({ requireReleaseNotes: fromTri(value) });
            }}
          />
          <label className={styles['field']} htmlFor="fs-notes-path">
            <span className={styles['pickerLabel']}>{t('release', 'form.notesPath')}</span>
            <input
              id="fs-notes-path"
              className={styles['input']}
              placeholder={t('release', 'form.notStated')}
              value={draft.releaseNotesPath ?? ''}
              onChange={(event) => {
                patch({ releaseNotesPath: event.target.value === '' ? null : event.target.value });
              }}
            />
          </label>
        </div>

        <div className={styles['grid']}>
          <label className={styles['field']} htmlFor="fs-version-source">
            <span className={styles['pickerLabel']}>{t('release', 'form.versionSource')}</span>
            <select
              id="fs-version-source"
              className={styles['select']}
              value={draft.versionSource ?? 'unset'}
              onChange={(event) => {
                const value = event.target.value;
                patch({ versionSource: value === 'unset' ? null : value });
              }}
            >
              <option value="unset">{t('release', 'form.versionSourceUnset')}</option>
              <option value="git_tag">{t('release', 'form.versionSourceGitTag')}</option>
            </select>
          </label>
          <label className={styles['field']} htmlFor="fs-version-pattern">
            <span className={styles['pickerLabel']}>{t('release', 'form.versionPattern')}</span>
            <input
              id="fs-version-pattern"
              className={styles['input']}
              placeholder={t('release', 'form.notStated')}
              value={draft.versionPattern ?? ''}
              onChange={(event) => {
                patch({ versionPattern: event.target.value === '' ? null : event.target.value });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-expected-version">
            <span className={styles['pickerLabel']}>{t('release', 'form.expectedVersion')}</span>
            <input
              id="fs-expected-version"
              className={styles['input']}
              placeholder={t('release', 'form.notStated')}
              value={draft.expectedVersion ?? ''}
              onChange={(event) => {
                patch({ expectedVersion: event.target.value === '' ? null : event.target.value });
              }}
            />
          </label>
          <label className={styles['field']} htmlFor="fs-expected-commit">
            <span className={styles['pickerLabel']}>{t('release', 'form.expectedCommit')}</span>
            <input
              id="fs-expected-commit"
              className={styles['input']}
              placeholder={t('release', 'form.notStated')}
              value={draft.expectedCommit ?? ''}
              onChange={(event) => {
                patch({ expectedCommit: event.target.value === '' ? null : event.target.value });
              }}
            />
          </label>
        </div>

        <fieldset className={styles['nested']}>
          <legend>{t('release', 'form.onUnknownLegend')}</legend>
          {UNKNOWN_RULES.map((rule) => (
            <div className={styles['unknownRow']} key={rule.key}>
              <span className={styles['monoSmall']}>{rule.label}</span>
              <select
                id={`fs-unknown-${rule.key}`}
                className={styles['select']}
                aria-label={t('release', 'form.onUnknownPicker', { rule: rule.label })}
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
                <option value="unset">{t('release', 'form.notStatedDefault')}</option>
                <option value="review">{t('release', 'form.dispositionReview')}</option>
                <option value="block">{t('release', 'form.dispositionBlock')}</option>
              </select>
            </div>
          ))}
          <p className={styles['hint']}>{t('release', 'form.onUnknownHint')}</p>
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
  const { t } = useI18n();
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
        <option value="unset">{t('release', 'form.notStatedDefault')}</option>
        <option value="yes">{t('release', 'form.yes')}</option>
        <option value="no">{t('release', 'form.no')}</option>
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
  const { t } = useI18n();
  if (row === undefined) {
    return <span className={styles['hint']}>{t('release', 'builds.noChosen')}</span>;
  }
  return (
    <span className={styles['sideFacts']}>
      <span className={styles['monoSmall']}>SHA-256 {truncateMiddle(row.sha256, 8)}</span>
      <span className={styles['mono']}>{formatSize(row.byteSize, unit)}</span>
      <span>
        {t('release', 'builds.nonvolatile', {
          value: formatSize(row.nonvolatile.bytes, unit),
          state: row.nonvolatile.state,
        })}{' · '}
        {t('release', 'builds.runtime', {
          value: formatSize(row.runtimeRam.bytes, unit),
          state: row.runtimeRam.state,
        })}
      </span>
      <span>{t('release', 'builds.stored', { at: row.importedAt })}</span>
    </span>
  );
}

/** The three ways the notes rule can have seen a file, in the words prompt §53 asks for. */
function notesPresence(present: boolean | null, t: Translate): string {
  if (present === true) {
    return t('gate', 'evidence.notesPresent');
  }
  if (present === false) {
    return t('gate', 'evidence.notesMissing');
  }
  return t('gate', 'evidence.notesUnobserved');
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
  const { t } = useI18n();
  const severity = severityState(run.dispositionEffectiveSeverity);

  return (
    <section className={styles['verdict']} aria-label={t('gate', 'verdict.group')}>
      <StateBadge variant="chip" state={severity} label={severity} />
      <p className={styles['verdictSentence']}>{gateVerdictSentence(run, t)}</p>
      <StatusStrip
        counts={{
          PASS: run.counts.pass,
          REVIEW: run.counts.review,
          BLOCK: run.counts.block,
          UNKNOWN: run.counts.unknown,
          'N/A': run.counts.notApplicable,
        }}
        label={t('gate', 'verdict.strip')}
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
