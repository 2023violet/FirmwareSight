/**
 * The Release page: one stored build, the policy it is judged against, and the record a reviewer
 * leaves beside it.
 *
 * Six rules decide the shape of this screen.
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
 *    one, and the SubjectBar says the record on screen is the previous one (prompt §49).
 * 5. **This is policy readiness, nothing more.** The page never says a build is safe to ship, legally
 *    compliant or certified: the Gate answers one policy, and the wording on the screen stops there
 *    (prompt §43).
 * 6. **A bundle attaches under Release, and only after a PASS.** P4 added no fifth stage to the rail; the
 *    Release Bundle is a view of this page, disabled with its reason while the disposition says otherwise,
 *    and its preview, destination token and typed errors all come from the same engine the command line
 *    drives (prompt §48, §35).
 *
 * R4 gives the page the anatomy of Analyze and Compare (`06_DELIVERY/18`): the SubjectBar names the build
 * the run on screen judged, the verdict leads, the run settings collapse to one line once a run exists,
 * and the findings, the evidence and the bundle are three views. Edits and decisions open in a Sheet;
 * evidence opens in the Inspector.
 */

import { useCallback, useEffect, useLayoutEffect, useRef, useState, type RefObject } from 'react';

import styles from './Release.module.css';
import { BuildPicker } from './components/BuildPicker';
import { Button, LinkButton } from './components/Button';
import { StatusStrip } from './components/Chip';
import { DataTable, type Column } from './components/DataTable';
import { DeltaValue } from './components/DeltaValue';
import { ErrorPanel } from './components/ErrorPanel';
import { ExportMenu } from './components/ExportMenu';
import type { FactEvidence } from './components/FactValue';
import { Inspector } from './components/Inspector';
import { Page } from './components/Layout';
import { LeadAnswer } from './components/LeadAnswer';
import { Notice } from './components/Notice';
import { PageHeader } from './components/PageHeader';
import { Sheet } from './components/Sheet';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import { StateBadge, type StateName } from './components/StateBadge';
import { SubjectBar, type SubjectFreshness } from './components/SubjectBar';
import { Tabs, tabPanelProps } from './components/Tabs';
import { evidenceBasisCaption } from './evidenceBasis';
import { formatSize, truncateMiddle, type SizeUnit } from './format';
import { useI18n, type Translate } from './i18n';
import type { Catalog } from './i18n/en';
import { gateVerdictSentence, severityState } from './stateWords';
import {
  acceptReview,
  attachReleaseFile,
  chooseBundleDestination,
  exportReleaseBundle,
  exportGateVisual,
  getGateRun,
  getReleaseAttachments,
  inspectProjectReadiness,
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
  BundleFileRowDto,
  BundlePreviewDto,
  CandidatePageDto,
  CompareCandidateDto,
  ErrorEnvelopeDto,
  GateArtifactRowDto,
  GateBudgetRowDto,
  GateFindingRowDto,
  GateGrowthRowDto,
  GateRunDto,
  ProjectContextDto,
  ProjectReadinessDto,
  ProjectPolicyDto,
  ReleaseAttachmentDto,
  ReleaseAttachmentsDto,
  UnknownDispositionDto,
  VisualFormatDto,
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
type ReleaseKey = keyof Catalog['release'];

/**
 * Findings are grouped and read in this order: what blocks, then what asks, then what is unknown.
 *
 * The order lives here and the words live in the catalog: the five states are Core's, but the plain-language
 * label beside each badge and the sentence under each group heading are interface copy (`ADR-0032` D5).
 * PASS and N/A start collapsed (spec §6.4): they are what needs no decision, and their counts stay in view.
 */
const STATE_GROUPS: readonly {
  readonly state: StateName;
  readonly labelKey: GateKey;
  readonly hintKey: GateKey;
  readonly startOpen: boolean;
}[] = [
  { state: 'BLOCK', labelKey: 'group.block.label', hintKey: 'group.block.hint', startOpen: true },
  { state: 'REVIEW', labelKey: 'group.review.label', hintKey: 'group.review.hint', startOpen: true },
  { state: 'UNKNOWN', labelKey: 'group.unknown.label', hintKey: 'group.unknown.hint', startOpen: true },
  { state: 'PASS', labelKey: 'group.pass.label', hintKey: 'group.pass.hint', startOpen: false },
  { state: 'N/A', labelKey: 'group.na.label', hintKey: 'group.na.hint', startOpen: false },
];

/** The five Gate words, so a stored value outside them is shown rather than dropped. */
const STATE_NAMES: readonly string[] = ['BLOCK', 'REVIEW', 'UNKNOWN', 'PASS', 'N/A'];

/** A three-way choice: what the reader said, or nothing — and nothing is the shell's to interpret. */
type TriState = 'unset' | 'yes' | 'no';

/** The three views of a run (spec §3.2). */
type View = 'findings' | 'evidence' | 'bundle';
const VIEWS: readonly View[] = ['findings', 'evidence', 'bundle'];
const VIEW_LABEL: Readonly<Record<View, ReleaseKey>> = {
  findings: 'views.findings',
  evidence: 'views.evidence',
  bundle: 'views.bundle',
};

type ExportFormat = VisualFormatDto;

/** The fact in the Inspector, with the control that opened it so focus can go back there. */
type Inspect = (evidence: FactEvidence, trigger: HTMLElement) => void;

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
  // A run the page is handed names its own build and baseline, so returning to Release with a run on screen
  // does not read as "the settings moved" for a run the reader never moved away from (R4 D-R4-7).
  const [snapshotId, setSnapshotId] = useState<string | null>(() => run?.snapshotId ?? null);
  const [baselineId, setBaselineId] = useState<string | null>(() => run?.baselineSnapshotId ?? null);
  const [draft, setDraft] = useState<ProjectPolicyDto | null>(null);
  // The sheet and the draft are separate: closing the sheet (Esc, a click beside it) keeps every edited
  // field, and only a save or a different project retires the draft (R4 review S2-2).
  const [policyOpen, setPolicyOpen] = useState(false);
  const [policyError, setPolicyError] = useState<ErrorEnvelopeDto | null>(null);
  const [policyNote, setPolicyNote] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [running, setRunning] = useState(false);
  const [restored, setRestored] = useState(false);
  const [reloadError, setReloadError] = useState<ErrorEnvelopeDto | null>(null);
  const [view, setView] = useState<View>('findings');
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [attachmentCount, setAttachmentCount] = useState<number | null>(null);
  const [inspected, setInspected] = useState<{
    readonly evidence: FactEvidence;
    readonly trigger: HTMLElement;
  } | null>(null);
  const inspect = useCallback<Inspect>((evidence, trigger) => {
    setInspected({ evidence, trigger });
  }, []);
  const closeInspector = useCallback(() => {
    setInspected(null);
  }, []);
  const editPolicyButton = useRef<HTMLButtonElement | null>(null);

  const [projectReadiness, setProjectReadiness] = useState<{
    readonly status: 'empty' | 'loading' | 'refreshing' | 'available' | 'error';
    readonly value: ProjectReadinessDto | null;
    readonly error: ErrorEnvelopeDto | null;
  }>({ status: 'empty', value: null, error: null });
  const readinessRequest = useRef(0);
  const projectIdentity = project === null ? null : `${project.projectName}\u0000${project.policySha256}`;
  const projectIdentityRef = useRef(projectIdentity);
  projectIdentityRef.current = projectIdentity;
  // A draft belongs to the project it was seeded from.
  useEffect(() => {
    setDraft(null);
  }, [projectIdentity]);

  const refreshProjectReadiness = useCallback(async () => {
    const request = ++readinessRequest.current;
    if (project === null) {
      setProjectReadiness({ status: 'empty', value: null, error: null });
      return;
    }
    const identity = `${project.projectName}\u0000${project.policySha256}`;
    setProjectReadiness((previous) => ({
      status: previous.value === null ? 'loading' : 'refreshing',
      value: previous.value,
      error: null,
    }));
    const outcome = await inspectProjectReadiness();
    if (request !== readinessRequest.current || projectIdentityRef.current !== identity) {
      return;
    }
    if (!outcome.ok) {
      setProjectReadiness((previous) => ({ status: 'error', value: previous.value, error: outcome.envelope }));
      return;
    }
    setProjectReadiness(
      outcome.value === null
        ? { status: 'empty', value: null, error: null }
        : { status: 'available', value: outcome.value, error: null },
    );
  }, [project]);

  useEffect(() => {
    if (project === null) {
      readinessRequest.current += 1;
      setProjectReadiness({ status: 'empty', value: null, error: null });
      return;
    }
    void refreshProjectReadiness();
    return () => {
      readinessRequest.current += 1;
    };
  }, [project, refreshProjectReadiness]);

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
        setPolicyOpen(false);
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
  const verdictRef = useRef<HTMLElement | null>(null);
  const focusVerdict = useRef(false);

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
      focusVerdict.current = true;
      onRunChange(outcome.value);
      setRestored(false);
      setSettingsOpen(false);
      setView('findings');
      return;
    }
    setError(outcome.envelope);
  }, [baselineId, onRunChange, snapshotId]);

  // A run a person just asked for moves focus to its verdict, so the keyboard lands where the eye does.
  useEffect(() => {
    if (run !== null && focusVerdict.current) {
      focusVerdict.current = false;
      verdictRef.current?.focus();
    }
  }, [run]);

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
    setReloadError(null);
    const outcome = await getGateRun(run.runId);
    if (outcome.ok && outcome.value !== null) {
      onRunChange(outcome.value);
      setRestored(true);
      return;
    }
    // A failed read of the stored record is not a failed Gate run: the run on screen is still the latest
    // one, so the SubjectBar keeps its word and the error stays with the record (R4 review S3).
    if (!outcome.ok) {
      setReloadError(outcome.envelope);
    }
  }, [onRunChange, run]);

  /**
   * Invalidate the run on screen because the attachment set it judged is no longer the set the page holds.
   *
   * The set is part of what a run identity binds (`04_TECH/28` §3), so a verdict about three files is not a
   * verdict about two. The page drops the record — and with it the bundle preview, which belongs to the
   * run — instead of leaving a `PASS` standing for bytes it never judged. Pressing Run Gate again makes a
   * new record of the set that is actually held.
   */
  const invalidateRun = useCallback(() => {
    onRunChange(null);
    setRestored(false);
  }, [onRunChange]);

  // The Gate visual export (M1 AC24, R4 D-R4-6). A layout effect, so the counter moves in the same commit
  // that shows a new run (CI #123): a late Save-dialog answer can never be read as this run's.
  const [exporting, setExporting] = useState(false);
  const [exportNote, setExportNote] = useState<string | null>(null);
  const [exportError, setExportError] = useState<ErrorEnvelopeDto | null>(null);
  // Its own state, which the run-change effect below does not reset: Rust's 6002 answer can land before the
  // new run renders, and that render must not wipe the one sentence saying nothing was written (R4 review
  // S2-3). The next export clears it.
  const [exportStale, setExportStale] = useState(false);
  const exportRequest = useRef(0);
  const exportSubject = run === null ? null : `${run.runId}\u0000${run.snapshotId}`;
  useLayoutEffect(() => {
    exportRequest.current += 1;
    setExporting(false);
    setExportNote(null);
    setExportError(null);
    return () => {
      exportRequest.current += 1;
    };
  }, [exportSubject]);

  const exportAs = useCallback(
    async (format: ExportFormat) => {
      if (run === null) {
        return;
      }
      const request = ++exportRequest.current;
      setExporting(true);
      setExportStale(false);
      const outcome = await exportGateVisual(run.runId, run.snapshotId, format);
      const staleAnswer = !outcome.ok && outcome.envelope.code === 'ERR-EXPORT-6002';
      if (request !== exportRequest.current) {
        // The run changed while the Save dialog was open. Rust refuses that export rather than writing the
        // run the reader moved away from; the screen says so. Any other late outcome belongs to the old run.
        if (staleAnswer) {
          setExportStale(true);
        }
        return;
      }
      setExporting(false);
      if (staleAnswer) {
        // Rust is the authority that the run changed, even when its answer arrives before this page has
        // rendered the change (CI #123): it is the stale note, never an error.
        setExportError(null);
        setExportNote(null);
        setExportStale(true);
        return;
      }
      if (!outcome.ok) {
        setExportError(outcome.envelope);
        setExportNote(null);
        return;
      }
      setExportError(null);
      setExportNote(
        outcome.value.status === 'written'
          ? t('release', 'visualExport.written', {
              name: outcome.value.fileName ?? t('release', 'visualExport.fallbackName'),
            })
          : outcome.value.status === 'cancelled'
            ? t('release', 'visualExport.cancelled')
            : outcome.value.status === 'kept-existing'
              ? t('release', 'visualExport.keptExisting')
              : t('release', 'visualExport.unknownOutcome', { status: outcome.value.status }),
      );
    },
    [run, t],
  );

  const rows = candidates?.rows ?? [];
  const chosen = (id: string | null): CompareCandidateDto | undefined =>
    rows.find((row) => row.snapshotId === id);
  const effectiveBaseline = baselineId === snapshotId ? null : baselineId;
  // The settings name the next run; the SubjectBar names the run on screen. When the two part, the run on
  // screen is the previous result, kept (spec §3.1).
  // A loaded policy whose canonical hash differs from the one the run was judged under also makes the run a
  // previous answer; Rust compares the same two hashes for the run's policy label (R4 review S3).
  const moved =
    run !== null &&
    (snapshotId !== run.snapshotId ||
      effectiveBaseline !== run.baselineSnapshotId ||
      (project !== null && project.policySha256 !== run.policySha256));
  const freshness: SubjectFreshness = run !== null && error !== null ? 'gateFailed' : moved ? 'settingsMoved' : 'latest';

  // Evidence belongs to one run, in one view, in one unit: a change of any of them closes the Inspector.
  const shownRun = run?.runId ?? null;
  useEffect(() => {
    setInspected(null);
  }, [shownRun, view, unit, freshness]);

  const runButton = (
    <Button variant="primary" disabled={snapshotId === null || running} onClick={() => void gate()}>
      {running ? t('release', 'builds.judging') : t('release', 'builds.run')}
    </Button>
  );

  // M1 T4: the attempt is set up in the order a person does it - which build, which policy, which files -
  // numbered from 1, and judged by one action after the last step.
  const steps = (
    <>
      <section className={styles['section']} aria-labelledby="fs-builds-heading">
        <h2 id="fs-builds-heading">{t('release', 'stepLabel', { n: 1 })}{' · '}{t('release', 'builds.heading')}</h2>
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
              <BuildPicker
                label={t('release', 'builds.currentLabel')}
                options={rows.map(buildOption)}
                value={snapshotId}
                onChange={setSnapshotId}
              />
              <BuildFacts row={chosen(snapshotId)} unit={unit} />
            </div>
            <div className={styles['picker']}>
              <BuildPicker
                label={t('release', 'builds.baselineLabel')}
                noneLabel={t('release', 'builds.noneOption')}
                options={rows.filter((row) => row.snapshotId !== snapshotId).map(buildOption)}
                value={effectiveBaseline}
                onChange={setBaselineId}
              />
              <BuildFacts row={chosen(effectiveBaseline)} unit={unit} />
              {effectiveBaseline === null ? (
                <span className={styles['hint']}>{t('release', 'builds.baselineHint')}</span>
              ) : null}
            </div>
          </div>
        )}
      </section>

      <section className={styles['section']} aria-labelledby="fs-policy-heading">
        <h2 id="fs-policy-heading">{t('release', 'stepLabel', { n: 2 })}{' · '}{t('release', 'policy.heading')}</h2>
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
            ref={editPolicyButton}
            ariaHasPopup="dialog"
            onClick={() => {
              setPolicyError(null);
              setInspected(null);
              setDraft((kept) => kept ?? seed ?? blankPolicy(project?.projectName ?? ''));
              setPolicyOpen(true);
            }}
          >
            {t('release', 'policy.edit')}
          </Button>
          {policyNote === null ? null : <span className={styles['status']}>{policyNote}</span>}
        </div>
        {policyError === null || policyOpen ? null : (
          <ErrorPanel
            envelope={policyError}
            label={t('release', 'policy.errorLabel')}
            heading={t('release', 'policy.errorHeading')}
          />
        )}
        <p className={styles['hint']}>
          {project === null
            ? t('release', 'policy.hintNoProject')
            : t('release', 'policy.hintEdit', { file: project.configFileName })}
        </p>
      </section>

      <ProjectReadinessPanel
        state={projectReadiness}
        projectLoaded={project !== null}
        onRefresh={() => void refreshProjectReadiness()}
      />

      <Attachments
        gateRunId={run?.runId ?? null}
        unit={unit}
        step={3}
        onSetChanged={invalidateRun}
        onCount={setAttachmentCount}
      />
    </>
  );

  const subjectRow = run === null ? undefined : chosen(run.snapshotId);

  return (
    <div className={styles['withInspector']}>
      <Page>
        <PageHeader
          title={t('gate', 'title')}
          subhead={t('gate', 'subhead')}
          actions={
            run === null ? undefined : (
              <ExportMenu<ExportFormat>
                label={t('release', 'visualExport.menu')}
                regionLabel={t('release', 'visualExport.region')}
                hint={t('release', 'visualExport.hint')}
                items={[
                  { key: 'png', label: t('release', 'visualExport.png') },
                  { key: 'svg', label: t('release', 'visualExport.svg') },
                  { key: 'html', label: t('release', 'visualExport.html') },
                ]}
                disabled={exporting}
                onExport={(format) => {
                  void exportAs(format);
                }}
              />
            )
          }
        />

        {exporting ? (
          <p className={styles['status']} role="status" aria-live="polite">
            {t('release', 'visualExport.waiting')}
          </p>
        ) : null}
        {exportNote === null ? null : (
          <p className={styles['status']} role="status" aria-live="polite">
            {exportNote}
          </p>
        )}
        {exportStale ? (
          <p className={styles['status']} role="status" aria-live="polite">
            {t('release', 'visualExport.staleNote')}
          </p>
        ) : null}
        {exportError === null ? null : (
          <ErrorPanel
            envelope={exportError}
            label={t('release', 'visualExport.errorLabel')}
            heading={t('release', 'visualExport.errorHeading')}
          />
        )}

        {run === null ? null : (
          <>
            <SubjectBar
              subject={{
                kind: 'single',
                build: {
                  fileName: subjectRow?.fileName ?? run.snapshotId,
                  sha256: subjectRow?.sha256 ?? null,
                },
              }}
              freshness={freshness}
              actions={<SizeUnitSwitch unit={unit} onSelect={onUnitChange} />}
            />
            {error === null ? null : (
              <Notice tone="stale" level={2} title={t('gate', 'staleTitle')}>
                <p role="status">{t('gate', 'staleRun', { id: truncateMiddle(run.runId, 10) })}</p>
              </Notice>
            )}
            {error === null && moved ? (
              <Notice tone="stale" level={2} title={t('gate', 'staleTitle')}>
                <p role="status">{t('gate', 'staleSetup')}</p>
              </Notice>
            ) : null}
            <Verdict run={run} focusRef={verdictRef} />
          </>
        )}

        {/* The settings that make the next run. With no run they are the page; with one they are a line that
            opens them in place. The steps keep one position in the tree either way, so a first run or a dropped
            one does not remount them, and nothing typed or attached - nor the focus on Attach - is lost
            (R4 review S3). */}
        <section
          className={run === null ? styles['setup'] : styles['settings']}
          aria-label={run === null ? undefined : t('release', 'settings.region')}
        >
          {run === null ? null : (
            <div className={styles['settingsLine']}>
              <span className={styles['axis']}>{t('release', 'settings.region')}</span>
              <span className={styles['settingsFacts']}>
                <span className={styles['monoSmall']}>
                  {settingsBuild(chosen(snapshotId), 'settings.build', 'settings.noBuild', t)}
                </span>
                <span className={styles['monoSmall']}>
                  {settingsBuild(chosen(effectiveBaseline), 'settings.baseline', 'settings.noBaseline', t)}
                </span>
                <span className={styles['monoSmall']}>
                  {project === null
                    ? t('release', 'settings.defaultPolicy')
                    : t('release', 'settings.policy', { hash: project.policySha256.slice(0, 12) })}
                </span>
                {attachmentCount === null ? null : (
                  <span>{t('release', 'settings.attachments', { count: attachmentCount })}</span>
                )}
              </span>
              <span className={styles['settingsActions']}>
                <Button
                  ariaExpanded={settingsOpen}
                  ariaControls="fs-release-steps"
                  onClick={() => {
                    setSettingsOpen((now) => !now);
                  }}
                >
                  {settingsOpen ? t('release', 'settings.close') : t('release', 'settings.edit')}
                </Button>
                {runButton}
              </span>
            </div>
          )}
          <div id="fs-release-steps" className={styles['steps']} hidden={run !== null && !settingsOpen}>
            {steps}
          </div>
          {run !== null ? null : (
            <section className={cx(styles['section'], styles['runBar'])} aria-label={t('release', 'builds.run')}>
              <div className={styles['pickerActions']}>
                {runButton}
                <SizeUnitSwitch unit={unit} onSelect={onUnitChange} />
                {snapshotId === null ? (
                  <span className={styles['hint']}>{t('release', 'runNeedsBuild')}</span>
                ) : null}
              </div>
            </section>
          )}
        </section>

        {error === null ? null : (
          <ErrorPanel
            envelope={error}
            label={t('gate', 'run.errorLabel')}
            heading={t('gate', 'run.errorHeading')}
          />
        )}

        {run === null ? null : (
          <>
            <Tabs
              idBase="fs-release"
              label={t('release', 'views.label')}
              items={VIEWS.map((key) => ({ key, label: t('release', VIEW_LABEL[key]) }))}
              current={view}
              onSelect={setView}
            />
            <div {...tabPanelProps('fs-release', view)} className={styles['panel']}>
              {view === 'findings' ? (
                <Findings run={run} onAccept={accept} onInspect={inspect} onSheetOpen={closeInspector} />
              ) : view === 'evidence' ? (
                <>
                  <RunRecord run={run} restored={restored} onReload={() => void reload()} />
                  {reloadError === null ? null : (
                    <ErrorPanel
                      envelope={reloadError}
                      label={t('gate', 'reload.errorLabel')}
                      heading={t('gate', 'reload.errorHeading')}
                    />
                  )}
                  <Evidence run={run} unit={unit} onInspect={inspect} />
                </>
              ) : null}
              {/* The bundle keeps its plan while the reader reads the findings: it is hidden, not unmounted,
                  and a new run still clears it (§28). */}
              <div hidden={view !== 'bundle'}>
                <BundleSection run={run} unit={unit} />
              </div>
            </div>
          </>
        )}
      </Page>

      <Sheet
        open={policyOpen && draft !== null}
        title={t('release', 'policy.sheetTitle')}
        closeLabel={t('release', 'policy.close')}
        returnFocus={editPolicyButton.current}
        onClose={() => {
          setPolicyOpen(false);
        }}
        actions={
          <>
            <Button variant="primary" disabled={draft === null || saving} onClick={() => void save()}>
              {saving
                ? t('release', 'policy.saving')
                : project === null
                  ? t('release', 'policy.saveAs', { file: 'firmwaresight.toml' })
                  : t('release', 'policy.save')}
            </Button>
            <span className={styles['status']}>
              {project === null
                ? t('release', 'policy.hintNoProject')
                : t('release', 'policy.hintEdit', { file: project.configFileName })}
            </span>
          </>
        }
      >
        {policyError === null ? null : (
          <ErrorPanel
            envelope={policyError}
            label={t('release', 'policy.errorLabel')}
            heading={t('release', 'policy.errorHeading')}
          />
        )}
        {draft === null ? null : <PolicyForm draft={draft} onChange={setDraft} />}
      </Sheet>

      <Inspector
        evidence={inspected?.evidence ?? null}
        returnFocus={inspected?.trigger ?? null}
        onClose={closeInspector}
      />
    </div>
  );
}

/** One stored build as the pickers list it. */
function buildOption(row: CompareCandidateDto) {
  return { id: row.snapshotId, fileName: row.fileName, sha256: row.sha256, importedAt: row.importedAt };
}

/** One build in the settings line: file and short hash, or the words for none. */
function settingsBuild(
  row: CompareCandidateDto | undefined,
  some: ReleaseKey,
  none: ReleaseKey,
  t: Translate,
): string {
  return row === undefined ? t('release', none) : t('release', some, { name: `${row.fileName} · ${row.sha256.slice(0, 12)}` });
}

function ProjectReadinessPanel({
  state,
  projectLoaded,
  onRefresh,
}: {
  readonly state: {
    readonly status: 'empty' | 'loading' | 'refreshing' | 'available' | 'error';
    readonly value: ProjectReadinessDto | null;
    readonly error: ErrorEnvelopeDto | null;
  };
  readonly projectLoaded: boolean;
  readonly onRefresh: () => void;
}) {
  const { t } = useI18n();
  const value = state.value;
  const busy = state.status === 'loading' || state.status === 'refreshing';
  return (
    <section className={styles['section']} aria-labelledby="fs-project-readiness-heading">
      <h2 id="fs-project-readiness-heading">{t('release', 'projectReadiness.heading')}</h2>
      <p className={styles['hint']}>{t('release', 'projectReadiness.scope')}</p>
      {!projectLoaded ? (
        <p className={styles['status']} role="status">{t('release', 'projectReadiness.noProject')}</p>
      ) : busy ? (
        <p className={styles['status']} role="status" aria-live="polite">{t('release', 'projectReadiness.loading')}</p>
      ) : state.status === 'empty' ? (
        <p className={styles['status']} role="status">{t('release', 'projectReadiness.sessionChanged')}</p>
      ) : null}
      {state.status === 'error' ? (
        <>
          <p className={styles['stale']} role="status">{t('release', 'projectReadiness.refreshFailed')}</p>
          {state.error === null ? null : (
            <ErrorPanel
              envelope={state.error}
              label={t('release', 'projectReadiness.errorLabel')}
              heading={t('release', 'projectReadiness.errorHeading')}
            />
          )}
        </>
      ) : null}
      {value === null ? null : (
        <>
          <dl className={styles['rows']}>
            <div className={styles['row']}>
              <dt className={styles['term']}>{t('release', 'projectReadiness.project')}</dt>
              <dd className={styles['value']}>
                <span>{value.projectName}</span>
                <span className={styles['monoSmall']}>{value.policySha256}</span>
              </dd>
            </div>
            <div className={styles['row']}>
              <dt className={styles['term']}>{t('release', 'projectReadiness.git')}</dt>
              <dd className={styles['value']}>
                {value.gitAvailable ? <span>{t('release', 'projectReadiness.gitAvailable')}</span> : <StateBadge state="UNKNOWN" label={t('release', 'projectReadiness.unknown')} />}
                {value.gitHeadCommit === null ? null : <span className={styles['monoSmall']}>HEAD {truncateMiddle(value.gitHeadCommit, 8)}</span>}
                {value.gitClean === null ? <StateBadge state="UNKNOWN" label={t('release', 'projectReadiness.unknown')} /> : <span>{t('release', value.gitClean ? 'projectReadiness.clean' : 'projectReadiness.modified')}</span>}
                {value.gitUnknownReason === null ? null : <span className={styles['note']}>{value.gitUnknownReason}</span>}
              </dd>
            </div>
            <div className={styles['row']}>
              <dt className={styles['term']}>{t('release', 'projectReadiness.notes')}</dt>
              <dd className={styles['value']}>
                {value.releaseNotesStatus === 'unreadable' ? (
                  <StateBadge state="UNKNOWN" label={t('release', 'projectReadiness.notes.unreadable')} />
                ) : (
                  <span>{t('release', `projectReadiness.notes.${value.releaseNotesStatus}` as ReleaseKey)}</span>
                )}
                {value.releaseNotesSha256 === null ? null : <span className={styles['monoSmall']}>{value.releaseNotesSha256}</span>}
                {value.releaseNotesReason === null ? null : <span className={styles['note']}>{value.releaseNotesReason}</span>}
              </dd>
            </div>
            <div className={styles['row']}>
              <dt className={styles['term']}>{t('release', 'projectReadiness.required')}</dt>
              <dd className={styles['value']}>
                <span>{value.declaredArtifactRequirements.length === 0 ? t('release', 'projectReadiness.noneDeclared') : value.declaredArtifactRequirements.join(', ')}</span>
                <span className={styles['note']}>{t('release', 'projectReadiness.declaredOnly')}</span>
              </dd>
            </div>
          </dl>
          {!value.matchesLoadedPolicy ? <p className={styles['stale']} role="status">{t('release', 'projectReadiness.policyChanged')}</p> : null}
          {value.unknownKeyCount > 0 ? <p className={styles['warning']} role="status">{t('release', 'projectReadiness.unknownKeys', { count: value.unknownKeyCount })}</p> : null}
          {value.warnings.length === 0 ? null : (
            <ul className={styles['noticeList']} aria-label={t('release', 'projectReadiness.warnings')}>
              {value.warnings.map((warning, index) => <li className={styles['noticeItem']} key={`${index}-${warning}`}>{warning}</li>)}
            </ul>
          )}
          <p className={styles['hint']}>{t('release', 'projectReadiness.buildNotChecked')}</p>
        </>
      )}
      <div className={styles['actions']}>
        <Button disabled={!projectLoaded || busy} onClick={onRefresh}>
          {busy ? t('release', 'projectReadiness.refreshing') : t('release', 'projectReadiness.refresh')}
        </Button>
      </div>
    </section>
  );
}

/**
 * The run record (prompt §43 item 5): both aggregates, which build, baseline and policy the run judged,
 * when it was stored and what the workspace looked like. The aggregate with and without the accepted
 * reviews are both Core&rsquo;s answers; neither is derived here.
 */
function RunRecord({
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
    <section className={styles['block']} aria-labelledby="fs-readiness-heading">
      <h2 id="fs-readiness-heading">{t('gate', 'readiness.heading')}</h2>
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

/**
 * Findings grouped BLOCK → REVIEW → UNKNOWN → PASS → N/A. Every group heading carries its count, so a
 * collapsed group still says how many rows it holds.
 */
function Findings({
  run,
  onAccept,
  onInspect,
  onSheetOpen,
}: {
  readonly run: GateRunDto;
  readonly onAccept: (request: AcceptReviewRequestDto) => Promise<ErrorEnvelopeDto | null>;
  readonly onInspect: Inspect;
  readonly onSheetOpen: () => void;
}) {
  const { t } = useI18n();
  const [open, setOpen] = useState<ReadonlySet<StateName>>(
    () => new Set(STATE_GROUPS.filter((group) => group.startOpen).map((group) => group.state)),
  );
  return (
    <section className={styles['block']} aria-labelledby="fs-findings-heading">
      <h2 id="fs-findings-heading">{t('gate', 'findings.heading')}</h2>
      {run.findings.length === 0 ? (
        <p className={styles['hint']}>{t('gate', 'findings.empty')}</p>
      ) : null}
      {STATE_GROUPS.map((group) => {
        const rows = run.findings.filter((finding) => finding.state === group.state);
        if (rows.length === 0) {
          return null;
        }
        const heading = t('gate', group.labelKey);
        const expanded = open.has(group.state);
        const listId = `fs-findings-${group.state === 'N/A' ? 'na' : group.state.toLowerCase()}`;
        return (
          <div className={styles['group']} key={group.state}>
            <div className={styles['groupLine']}>
              <h3 className={styles['groupHead']}>
                <StateBadge state={group.state} label={heading} count={rows.length} />
              </h3>
              <span className={styles['hint']}>{t('gate', group.hintKey)}</span>
              {group.startOpen ? null : (
                <Button
                  variant="ghost"
                  ariaExpanded={expanded}
                  onClick={() => {
                    setOpen((now) => {
                      const next = new Set(now);
                      if (next.has(group.state)) {
                        next.delete(group.state);
                      } else {
                        next.add(group.state);
                      }
                      return next;
                    });
                  }}
                >
                  {expanded
                    ? t('gate', 'findings.collapse', { group: heading })
                    : t('gate', 'findings.expand', { group: heading, count: rows.length })}
                </Button>
              )}
            </div>
            {expanded ? (
              <ol className={styles['findings']} id={listId}>
                {rows.map((finding) => (
                  <FindingRow
                    finding={finding}
                    key={finding.id}
                    runId={run.runId}
                    onAccept={onAccept}
                    onInspect={onInspect}
                    onSheetOpen={onSheetOpen}
                  />
                ))}
              </ol>
            ) : null}
          </div>
        );
      })}
    </section>
  );
}

/** What the Inspector shows for a finding: its state, severity and every evidence locator it quotes. */
function findingEvidence(finding: GateFindingRowDto, t: Translate): FactEvidence {
  const refs: (readonly [string, string])[] =
    finding.evidenceRefs.length === 0
      ? [[t('gate', 'findings.evidenceNone'), t('gate', 'findings.noEvidence')]]
      : finding.evidenceRefs.map((ref, index) => [t('gate', 'findings.evidenceTerm', { n: index + 1 }), ref] as const);
  return {
    field: finding.ruleId,
    value: finding.state,
    // A finding is the policy's judgement of observed facts: derived, and Unknown when its evidence is missing.
    classification: finding.state === 'UNKNOWN' ? 'unknown' : 'derived',
    rule: finding.ruleId,
    details: [[t('gate', 'findings.severityTerm'), finding.effectiveSeverity], ...refs],
    ...(finding.evidenceRefs.length === 0 ? {} : { raw: finding.evidenceRefs.join('\n') }),
  };
}

/**
 * One finding, and the acceptance beside it.
 *
 * The Accept control exists only where Rust said `acceptable`, which is only a REVIEW row. An accepted
 * row still renders its own state: the acceptance is an audit fact next to it, never a rewrite of it
 * (prompt §48). The name and the reason are asked for in a Sheet, because they are the record.
 */
function FindingRow({
  finding,
  runId,
  onAccept,
  onInspect,
  onSheetOpen,
}: {
  readonly finding: GateFindingRowDto;
  readonly runId: string;
  readonly onAccept: (request: AcceptReviewRequestDto) => Promise<ErrorEnvelopeDto | null>;
  readonly onInspect: Inspect;
  /** A sheet is modal; the Inspector closes first so two focus traps never compete (R4 review S3). */
  readonly onSheetOpen: () => void;
}) {
  const { t } = useI18n();
  const [openForm, setOpenForm] = useState(false);
  const [actor, setActor] = useState('');
  const [reason, setReason] = useState('');
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [busy, setBusy] = useState(false);
  const basisButton = useRef<HTMLButtonElement | null>(null);
  const acceptButton = useRef<HTMLButtonElement | null>(null);
  const actorField = useRef<HTMLInputElement | null>(null);
  const record = useRef<HTMLDivElement | null>(null);

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
    // A refusal is shown even if the reader closed the sheet while it was in flight (R4 review S3).
    setError(failure);
    setOpenForm(true);
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
      {finding.acceptance === null ? null : (
        <div className={styles['accepted']} role="status" ref={record} tabIndex={-1}>
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
      <div className={styles['actions']}>
        <Button
          ref={basisButton}
          variant="ghost"
          onClick={() => {
            if (basisButton.current !== null) {
              onInspect(findingEvidence(finding, t), basisButton.current);
            }
          }}
        >
          {t('gate', 'findings.showBasis', { rule: finding.ruleId })}
        </Button>
        {finding.acceptable ? (
          <Button
            ref={acceptButton}
            ariaHasPopup="dialog"
            onClick={() => {
              setError(null);
              onSheetOpen();
              setOpenForm(true);
            }}
          >
            {t('release', 'accept.button')}
          </Button>
        ) : null}
      </div>
      <Sheet
        open={openForm}
        title={t('release', 'accept.sheetTitle', { rule: finding.ruleId })}
        returnFocus={acceptButton.current}
        initialFocus={actorField}
        fallbackFocus={record}
        onClose={() => {
          setOpenForm(false);
        }}
        actions={
          <>
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
          </>
        }
      >
        <p className={styles['summary']}>{finding.summary}</p>
        <label className={styles['field']} htmlFor={`fs-actor-${finding.id}`}>
          <span className={styles['pickerLabel']}>{t('release', 'accept.actorLabel')}</span>
          <input
            ref={actorField}
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
            rows={3}
            value={reason}
            onChange={(event) => {
              setReason(event.target.value);
            }}
          />
        </label>
        <p className={styles['hint']}>{t('release', 'accept.requiredNote')}</p>
        {error === null ? null : (
          <ErrorPanel
            envelope={error}
            label={t('release', 'accept.errorLabel', { rule: finding.ruleId })}
            heading={t('release', 'accept.errorHeading')}
          />
        )}
      </Sheet>
    </li>
  );
}

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

/** A budget row's margin in words: headroom, excess, or Unknown when neither was measured. */
function budgetMargin(row: GateBudgetRowDto, unit: SizeUnit, t: Translate): string {
  return row.headroomBytes !== null
    ? t('gate', 'evidence.headroom', { value: formatSize(row.headroomBytes, unit) })
    : row.overBytes !== null
      ? t('gate', 'evidence.over', { value: formatSize(row.overBytes, unit) })
      : t('gate', 'evidence.unknownValue');
}

/** A budget row's evidence basis in words: what the actual figure rests on and what it may decide. */
function budgetBasis(row: GateBudgetRowDto, t: Translate): string {
  return [
    row.basis === null ? t('gate', 'evidence.noBasis') : evidenceBasisCaption(row.basis, t),
    t('gate', row.exact ? 'evidence.completeAttribution' : 'evidence.floor'),
    t('gate', row.admissible ? 'evidence.maySupport' : 'evidence.cannotBlock'),
    ...(row.reason === null ? [] : [row.reason]),
  ].join(' · ');
}

function budgetEvidence(row: GateBudgetRowDto, unit: SizeUnit, t: Translate): FactEvidence {
  return {
    field: row.label,
    value: sizeCell(row.actualBytes, unit, t),
    // The actual figure is a sum Core made of observed sections; with no figure it is Unknown.
    classification: row.actualBytes === null ? 'unknown' : 'derived',
    rule: row.ruleId,
    details: [
      [t('gate', 'evidence.colBudget'), sizeCell(row.budgetBytes, unit, t)],
      [t('gate', 'evidence.colMargin'), budgetMargin(row, unit, t)],
      [t('gate', 'evidence.colState'), `${row.state} · ${t('gate', 'evidence.effective', { severity: row.effectiveSeverity })}`],
      [t('gate', 'evidence.colEvidenceBasis'), budgetBasis(row, t)],
    ],
  };
}

function growthEvidence(row: GateGrowthRowDto, unit: SizeUnit, t: Translate): FactEvidence {
  return {
    field: row.label,
    value: row.deltaBytes === null ? t('gate', 'evidence.unknownValue') : signedSize(row.deltaBytes, unit),
    classification: row.deltaBytes === null ? 'unknown' : 'derived',
    rule: row.ruleId,
    details: [
      [t('gate', 'evidence.colOld'), sizeCell(row.oldBytes, unit, t)],
      [t('gate', 'evidence.colNew'), sizeCell(row.newBytes, unit, t)],
      [t('gate', 'evidence.colReviewThreshold'), sizeCell(row.thresholdBytes, unit, t)],
      [t('gate', 'evidence.colState'), `${row.state} · ${t('gate', 'evidence.effective', { severity: row.effectiveSeverity })}`],
      [
        t('gate', 'evidence.comparabilityTerm'),
        row.reason === null ? row.comparability : `${row.comparability}: ${row.reason}`,
      ],
    ],
  };
}

/** A delta in the Inspector's plain text: ▲ or ▼ and the magnitude, the same glyphs the tables use. */
function signedSize(delta: number, unit: SizeUnit): string {
  return delta === 0 ? formatSize(0, unit) : `${delta > 0 ? '▲' : '▼'}${formatSize(Math.abs(delta), unit)}`;
}

/** A size where `null` is a fact never measured: the word Unknown, never a number. */
function sizeCell(value: number | null, unit: SizeUnit, t: Translate): string {
  return value === null ? t('gate', 'evidence.unknownValue') : formatSize(value, unit);
}

/**
 * The numbers behind the findings: required artifacts, the two budgets, growth against the baseline, and
 * the release notes file (prompt §51–§53).
 *
 * Every figure is Rust&rsquo;s. The unit switch only re-labels a byte count, and a row whose state is
 * Unknown shows the word `Unknown` with its reason rather than a number that was never measured.
 */
function Evidence({
  run,
  unit,
  onInspect,
}: {
  readonly run: GateRunDto;
  readonly unit: SizeUnit;
  readonly onInspect: Inspect;
}) {
  const { t } = useI18n();
  const tables = run.tables;
  if (tables === null) {
    return (
      <Notice tone="unknown" level={2} title={t('gate', 'evidence.noTablesTitle')}>
        <p>{t('gate', 'evidence.noTables')}</p>
      </Notice>
    );
  }

  const artifactColumns: readonly Column<GateArtifactRowDto>[] = [
    { key: 'kind', header: t('gate', 'evidence.colKind'), cell: (row) => <span className={styles['mono']}>{row.kind}</span> },
    { key: 'policy', header: t('gate', 'evidence.colPolicy'), cell: (row) => t('gate', row.required ? 'evidence.required' : 'evidence.notRequired') },
    { key: 'satisfied', header: t('gate', 'evidence.colSatisfiedBy'), cell: (row) => satisfiedBy(row.satisfiedBy, t) },
    { key: 'sha', header: t('gate', 'evidence.colSha'), cell: (row) => <span className={styles['monoSmall']}>{row.sha256 === null ? '—' : truncateMiddle(row.sha256, 8)}</span> },
    { key: 'size', header: t('gate', 'evidence.colSize'), numeric: true, cell: (row) => formatSize(row.byteSize, unit) },
  ];
  const budgetColumns: readonly Column<GateBudgetRowDto>[] = [
    { key: 'side', header: t('gate', 'evidence.colSide'), cell: (row) => row.label },
    { key: 'actual', header: t('gate', 'evidence.colActual'), numeric: true, cell: (row) => sizeCell(row.actualBytes, unit, t) },
    { key: 'budget', header: t('gate', 'evidence.colBudget'), numeric: true, cell: (row) => sizeCell(row.budgetBytes, unit, t) },
    { key: 'margin', header: t('gate', 'evidence.colMargin'), numeric: true, cell: (row) => budgetMargin(row, unit, t) },
    {
      key: 'state',
      header: t('gate', 'evidence.colState'),
      cell: (row) => (
        <>
          <StateBadge state={stateName(row.state)} label={row.state} />
          <span className={styles['note']}> {t('gate', 'evidence.effective', { severity: row.effectiveSeverity })}</span>
        </>
      ),
    },
    {
      key: 'basis',
      header: t('gate', 'evidence.colEvidenceBasis'),
      cell: (row) => (
        <span className={styles['wrap']}>
          <span>{row.basis === null ? t('gate', 'evidence.noBasis') : evidenceBasisCaption(row.basis, t)}</span>
          <span className={styles['note']}>
            {' · '}
            {t('gate', row.exact ? 'evidence.completeAttribution' : 'evidence.floor')}
            {' · '}
            {t('gate', row.admissible ? 'evidence.maySupport' : 'evidence.cannotBlock')}
          </span>
          {row.reason === null ? null : <span className={styles['note']}>{' · ' + row.reason}</span>}
        </span>
      ),
    },
  ];
  const growthColumns: readonly Column<GateGrowthRowDto>[] = [
    { key: 'side', header: t('gate', 'evidence.colSide'), cell: (row) => row.label },
    { key: 'old', header: t('gate', 'evidence.colOld'), numeric: true, cell: (row) => sizeCell(row.oldBytes, unit, t) },
    { key: 'new', header: t('gate', 'evidence.colNew'), numeric: true, cell: (row) => sizeCell(row.newBytes, unit, t) },
    {
      key: 'delta',
      header: t('gate', 'evidence.colDelta'),
      numeric: true,
      // An unmeasured delta is Unknown, not "not present": both sides exist, the comparison did not hold.
      cell: (row) =>
        row.deltaBytes === null ? t('gate', 'evidence.unknownValue') : (
          <DeltaValue delta={row.deltaBytes} format={(bytes) => formatSize(bytes, unit)} />
        ),
    },
    { key: 'threshold', header: t('gate', 'evidence.colReviewThreshold'), numeric: true, cell: (row) => sizeCell(row.thresholdBytes, unit, t) },
    {
      key: 'state',
      header: t('gate', 'evidence.colState'),
      cell: (row) => (
        <span className={styles['wrap']}>
          <StateBadge state={stateName(row.state)} label={row.state} />
          <span className={styles['note']}> {t('gate', 'evidence.effective', { severity: row.effectiveSeverity })}</span>
          <span className={styles['note']}>
            {' '}
            {t('gate', 'evidence.deltaComparability', { comparability: row.comparability })}
            {row.reason === null ? '' : `: ${row.reason}`}
          </span>
        </span>
      ),
    },
  ];

  return (
    <section className={styles['block']} aria-labelledby="fs-evidence-heading">
      <h2 id="fs-evidence-heading">{t('gate', 'evidence.heading')}</h2>

      <div className={styles['block']}>
        <h3>{t('gate', 'evidence.artifactsHeading')}</h3>
        <DataTable
          caption={t('gate', 'evidence.artifactsCaption')}
          columns={artifactColumns}
          rows={tables.artifacts}
          rowKey={(row) => row.kind}
          density="compact"
        />
      </div>

      <div className={styles['block']}>
        <h3>{t('gate', 'evidence.budgetsHeading')}</h3>
        {tables.budgets.length === 0 ? (
          <p className={styles['hint']}>{t('gate', 'evidence.noBudgets')}</p>
        ) : (
          <DataTable
            caption={t('gate', 'evidence.budgetsCaption')}
            columns={budgetColumns}
            rows={tables.budgets}
            rowKey={(row) => row.ruleId}
            density="compact"
            onRowAction={(row, trigger) => {
              onInspect(budgetEvidence(row, unit, t), trigger);
            }}
            rowActionLabel={(row) => t('gate', 'evidence.showBudget', { side: row.label })}
          />
        )}
      </div>

      <div className={styles['block']}>
        <h3>{t('gate', 'evidence.growthHeading')}</h3>
        {tables.growth.length === 0 ? (
          <p className={styles['hint']}>{t('gate', 'evidence.noGrowth')}</p>
        ) : (
          <DataTable
            caption={t('gate', 'evidence.growthCaption')}
            columns={growthColumns}
            rows={tables.growth}
            rowKey={(row) => row.side}
            density="compact"
            onRowAction={(row, trigger) => {
              onInspect(growthEvidence(row, unit, t), trigger);
            }}
            rowActionLabel={(row) => t('gate', 'evidence.showGrowth', { side: row.label })}
          />
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
 */
function Attachments({
  gateRunId,
  unit,
  step,
  onSetChanged,
  onCount,
}: {
  readonly gateRunId: string | null;
  readonly unit: SizeUnit;
  /** The setup step this section is, when it is shown as one. */
  readonly step?: number;
  readonly onSetChanged: () => void;
  /** How many files the set holds, for the run-settings line. */
  readonly onCount: (count: number) => void;
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

  const count = set?.attachments.length ?? null;
  useEffect(() => {
    if (count !== null) {
      onCount(count);
    }
  }, [count, onCount]);

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

  const columns: readonly Column<ReleaseAttachmentDto>[] = [
    { key: 'file', header: t('release', 'attachments.colFile'), cell: (row) => <span className={styles['mono']}>{row.displayName}</span> },
    {
      key: 'kind',
      header: t('release', 'attachments.colKind'),
      cell: (row) => (
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
      ),
    },
    { key: 'size', header: t('release', 'attachments.colSize'), numeric: true, cell: (row) => formatSize(row.byteSize, unit) },
    { key: 'sha', header: t('release', 'attachments.colSha'), cell: (row) => <span className={styles['monoSmall']}>{row.sha256}</span> },
    {
      key: 'evidence',
      header: t('release', 'attachments.colEvidence'),
      cell: (row) => t('release', 'attachments.bytesLine', { basis: row.kindBasis, provenance: row.provenance }),
    },
    {
      key: 'action',
      header: t('release', 'attachments.colAction'),
      cell: (row) => (
        <Button disabled={busy} onClick={() => void withdraw(row.attachmentId)}>
          {t('release', 'attachments.withdraw')}
        </Button>
      ),
    },
  ];

  return (
    <section className={styles['section']} aria-labelledby="fs-attachments-heading">
      <h2 id="fs-attachments-heading">
        {step === undefined ? null : (
          <>
            {t('release', 'stepLabel', { n: step })}
            {' · '}
          </>
        )}
        {t('release', 'attachments.heading')}
      </h2>
      <DataTable
        caption={t('release', 'attachments.caption')}
        columns={columns}
        rows={set?.attachments ?? []}
        rowKey={(row) => row.attachmentId}
        density="compact"
        empty={t('release', 'attachments.empty')}
      />
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

/**
 * The Release Bundle view (prompt §48, §49, §50, §58).
 *
 * Three facts decide what this view can do, and none of them is computed here: the disposition is Core's
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
  const { t } = useI18n();
  const [preview, setPreview] = useState<BundlePreviewDto | null>(null);
  const [destination, setDestination] = useState<BundleDestinationDto | null>(null);
  const [outcome, setOutcome] = useState<BundleExportDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  /** The bundle folder name awaiting an explicit replace decision (§30). */
  const [confirm, setConfirm] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [note, setNote] = useState<string | null>(null);
  const confirmRef = useRef<HTMLButtonElement | null>(null);
  const exportButton = useRef<HTMLButtonElement | null>(null);

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
      // A refusal retires the question the sheet was asking. Leaving it up would keep asserting a
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

  const keep = () => {
    setConfirm(null);
    setError(null);
    setNote(t('release', 'bundle.kept'));
  };

  const fileColumns: readonly Column<BundleFileRowDto>[] = [
    { key: 'path', header: t('release', 'bundle.colFile'), cell: (file) => <span className={styles['mono']}>{file.path}</span> },
    { key: 'role', header: t('release', 'bundle.colRole'), cell: (file) => <span className={styles['mono']}>{file.role}</span> },
    { key: 'size', header: t('release', 'bundle.colSize'), numeric: true, cell: (file) => formatSize(file.byteSize, unit) },
    { key: 'sha', header: t('release', 'bundle.colSha'), cell: (file) => <span className={styles['monoSmall']}>{truncateMiddle(file.sha256, 8)}</span> },
  ];

  return (
    <section className={styles['block']} aria-labelledby="fs-bundle-heading">
      <h2 id="fs-bundle-heading">{t('release', 'bundleHeading')}</h2>
      {ready ? (
        <p className={styles['hint']}>{t('release', 'bundle.readyHint')}</p>
      ) : (
        <Notice tone="info" level={3} title={t('release', 'bundle.notReadyTitle')}>
          <p role="status">{t('release', 'bundle.notReady', { severity: run.dispositionEffectiveSeverity })}</p>
        </Notice>
      )}
      <div className={styles['actions']}>
        <Button variant="primary" disabled={!ready || busy} onClick={() => void prepare()}>
          {busy && preview === null ? t('release', 'bundle.preparing') : t('release', 'bundle.prepare')}
        </Button>
        <Button disabled={preview === null || busy} onClick={() => void choose()}>
          {busy && preview !== null && destination === null
            ? t('release', 'bundle.choosing')
            : t('release', 'bundle.choose')}
        </Button>
        <Button
          ref={exportButton}
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
      <Sheet
        open={confirm !== null}
        title={t('release', 'bundle.replaceTitle')}
        returnFocus={exportButton.current}
        initialFocus={confirmRef}
        onClose={keep}
        actions={
          confirm === null ? undefined : (
            <>
              <Button variant="primary" ref={confirmRef} disabled={busy} onClick={() => void exportBundle(true)}>
                {t('release', 'bundle.replaceButton', { name: confirm })}
              </Button>
              <Button onClick={keep}>{t('release', 'bundle.keep')}</Button>
            </>
          )
        }
      >
        <p className={styles['hint']}>
          <span className={styles['mono']}>{confirm}</span> {t('release', 'bundle.replaceWhy')}
        </p>
      </Sheet>
      {preview === null ? null : (
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
          <DataTable
            caption={t('release', 'bundle.filesCaption')}
            columns={fileColumns}
            rows={preview.files}
            rowKey={(file) => file.path}
            density="compact"
          />
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
      )}
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

/**
 * The policy editor, shown in a Sheet. Every field is a named policy value; no field takes a path, a
 * document or a command.
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
          <NumberField id="fs-flash-budget" label={t('release', 'form.flashBudget')} value={draft.flashBudget} onChange={(value) => { patch({ flashBudget: bytes(value) }); }} />
          <NumberField id="fs-ram-budget" label={t('release', 'form.ramBudget')} value={draft.ramBudget} onChange={(value) => { patch({ ramBudget: bytes(value) }); }} />
          <NumberField id="fs-flash-growth" label={t('release', 'form.flashGrowth')} value={draft.flashGrowthReviewBytes} onChange={(value) => { patch({ flashGrowthReviewBytes: bytes(value) }); }} />
          <NumberField id="fs-ram-growth" label={t('release', 'form.ramGrowth')} value={draft.ramGrowthReviewBytes} onChange={(value) => { patch({ ramGrowthReviewBytes: bytes(value) }); }} />
          <NumberField id="fs-unknown-count" label={t('release', 'form.unknownCount')} value={draft.unknownEvidenceReviewCount} onChange={(value) => { patch({ unknownEvidenceReviewCount: bytes(value) }); }} />
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
          <TextField id="fs-notes-path" label={t('release', 'form.notesPath')} value={draft.releaseNotesPath} onChange={(value) => { patch({ releaseNotesPath: value }); }} />
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
          <TextField id="fs-version-pattern" label={t('release', 'form.versionPattern')} value={draft.versionPattern} onChange={(value) => { patch({ versionPattern: value }); }} />
          <TextField id="fs-expected-version" label={t('release', 'form.expectedVersion')} value={draft.expectedVersion} onChange={(value) => { patch({ expectedVersion: value }); }} />
          <TextField id="fs-expected-commit" label={t('release', 'form.expectedCommit')} value={draft.expectedCommit} onChange={(value) => { patch({ expectedCommit: value }); }} />
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

/** A byte or count field: empty is "not stated", which the shell resolves against its own default. */
function NumberField({
  id,
  label,
  value,
  onChange,
}: {
  readonly id: string;
  readonly label: string;
  readonly value: number | null;
  readonly onChange: (value: string) => void;
}) {
  const { t } = useI18n();
  return (
    <label className={styles['field']} htmlFor={id}>
      <span className={styles['pickerLabel']}>{label}</span>
      <input
        id={id}
        className={cx(styles['input'], styles['mono'])}
        inputMode="numeric"
        placeholder={t('release', 'form.notStated')}
        value={value ?? ''}
        onChange={(event) => {
          onChange(event.target.value);
        }}
      />
    </label>
  );
}

/** A text field: empty is "not stated", sent as `null`. */
function TextField({
  id,
  label,
  value,
  onChange,
}: {
  readonly id: string;
  readonly label: string;
  readonly value: string | null;
  readonly onChange: (value: string | null) => void;
}) {
  const { t } = useI18n();
  return (
    <label className={styles['field']} htmlFor={id}>
      <span className={styles['pickerLabel']}>{label}</span>
      <input
        id={id}
        className={styles['input']}
        placeholder={t('release', 'form.notStated')}
        value={value ?? ''}
        onChange={(event) => {
          onChange(event.target.value === '' ? null : event.target.value);
        }}
      />
    </label>
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
 * A stored build&rsquo;s own recorded facts, shown under the picker that names it. A reader choosing
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
 * The verdict, first on the page: the one answer Release gives (ADR-0034 D1 principle 1).
 *
 * `DESIGN.md` 9 forbids this sentence from reaching past policy readiness into a legal, security or
 * compliance claim, and the standing note under it says so. When accepted reviews change the aggregate,
 * the sentence carries both severities, because showing only one of them would let a reader believe the
 * build cleared on its own.
 */
function Verdict({
  run,
  focusRef,
}: {
  readonly run: GateRunDto;
  readonly focusRef: RefObject<HTMLElement | null>;
}) {
  const { t } = useI18n();
  const severity = severityState(run.dispositionEffectiveSeverity);
  return (
    <LeadAnswer
      label={t('gate', 'verdict.group')}
      focusRef={focusRef}
      state={<StateBadge variant="chip" state={severity} label={severity} />}
      sentence={gateVerdictSentence(run, t)}
      reference={
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
      }
      note={t('gate', 'verdict.note')}
    />
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
