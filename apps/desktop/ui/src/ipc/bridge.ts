// Typed surface between the WebView and the shell.
//
// Two rules hold here: nothing is `any` (P0 prompt 42, ADR-0019), and a rejection carries the
// same `ErrorEnvelope` shape the CLI prints, so one failure looks the same on both surfaces
// instead of becoming a bare string in one and an object in the other.
//
// Calls resolve to an outcome rather than throwing. The `unknown` a rejected `invoke` produces
// is narrowed exactly once, here, so no component has to catch-and-guess.

import { invoke } from '@tauri-apps/api/core';

import type {
  AcceptReviewOutcomeDto,
  AcceptReviewRequestDto,
  AnalysisSummaryDto,
  BundleDestinationDto,
  BundleExportDto,
  BundlePlanRequestDto,
  BundlePreviewDto,
  CandidatePageDto,
  CandidatePageRequestDto,
  CompareRequestDto,
  CompareSummaryDto,
  ErrorEnvelopeDto,
  EvidencePageDto,
  EvidenceRequestDto,
  ExportOutcomeDto,
  GateRunDto,
  GateRunRequestDto,
  ProjectContextDto,
  ProjectPolicyDto,
  SectionChangePageDto,
  SectionChangeQueryDto,
  SectionPageDto,
  SectionRequestDto,
  SelectionDto,
  SymbolChangePageDto,
  SymbolChangeQueryDto,
  SymbolPageDto,
  SymbolRequestDto,
} from './types';

/** The commands the shell actually registers. A typo here is a compile error, not a surprise. */
const SELECT_ARTIFACT = 'select_artifact';
const ATTACH_MAP = 'attach_map';
const CLEAR_MAP = 'clear_map';
const ANALYZE_SELECTION = 'analyze_selection';
const QUERY_SECTIONS = 'query_sections';
const QUERY_SYMBOLS = 'query_symbols';
const QUERY_EVIDENCE = 'query_evidence';
const LIST_COMPARE_CANDIDATES = 'list_compare_candidates';
const COMPARE_SNAPSHOTS = 'compare_snapshots';
const QUERY_SECTION_CHANGES = 'query_section_changes';
const QUERY_SYMBOL_CHANGES = 'query_symbol_changes';
const EXPORT_COMPARE_JSON = 'export_compare_json';
const EXPORT_COMPARE_HTML = 'export_compare_html';
const OPEN_PROJECT_CONFIG = 'open_project_config';
const SAVE_PROJECT_POLICY = 'save_project_policy';
const RUN_RELEASE_GATE = 'run_release_gate';
const ACCEPT_REVIEW = 'accept_review';
const GET_GATE_RUN = 'get_gate_run';
const PREPARE_RELEASE_BUNDLE = 'prepare_release_bundle';
const CHOOSE_BUNDLE_DESTINATION = 'choose_bundle_destination';
const EXPORT_RELEASE_BUNDLE = 'export_release_bundle';

export type IpcOutcome<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly envelope: ErrorEnvelopeDto };

/**
 * Ask the shell to open the native artifact dialog.
 *
 * `null` means the user cancelled. It is a successful call with no selection, not a failure, so
 * the screen keeps whatever it was showing.
 */
export async function selectArtifact(): Promise<IpcOutcome<SelectionDto | null>> {
  return await call<SelectionDto | null>(SELECT_ARTIFACT);
}

/**
 * Ask the shell to open the native MAP dialog for the given selection.
 *
 * `null` means the user cancelled and the selection keeps whatever MAP it had.
 */
export async function attachMap(
  selectionId: string,
): Promise<IpcOutcome<SelectionDto | null>> {
  return await call<SelectionDto | null>(ATTACH_MAP, { selectionId });
}

/** Detach the MAP from a selection. The artifact itself stays selected. */
export async function clearMap(selectionId: string): Promise<IpcOutcome<SelectionDto>> {
  return await call<SelectionDto>(CLEAR_MAP, { selectionId });
}

/**
 * Analyze the staged selection.
 *
 * The only argument is the opaque handle the shell issued. A path is never sent, because the
 * WebView has no business naming one (`AGENTS.md` 7).
 */
export async function analyzeSelection(
  selectionId: string,
): Promise<IpcOutcome<AnalysisSummaryDto>> {
  return await call<AnalysisSummaryDto>(ANALYZE_SELECTION, { selectionId });
}

/**
 * Ask for one bounded page of sections of a snapshot already in history.
 *
 * The request names a snapshot id and a page: the shell owns the limit, so asking for more than it
 * will produce is not a way to get more of it.
 */
export async function querySections(
  request: SectionRequestDto,
): Promise<IpcOutcome<SectionPageDto>> {
  return await call<SectionPageDto>(QUERY_SECTIONS, { request });
}

/** Ask for one bounded page of symbols. */
export async function querySymbols(request: SymbolRequestDto): Promise<IpcOutcome<SymbolPageDto>> {
  return await call<SymbolPageDto>(QUERY_SYMBOLS, { request });
}

/** Ask for one bounded page of evidence items. */
export async function queryEvidence(
  request: EvidenceRequestDto,
): Promise<IpcOutcome<EvidencePageDto>> {
  return await call<EvidencePageDto>(QUERY_EVIDENCE, { request });
}

/**
 * Ask for one bounded page of builds Compare may start from.
 *
 * Which project's history is in scope is not askable: Rust fixes it, so this cannot be widened into
 * a listing of someone else's builds (prompt §17).
 */
export async function listCompareCandidates(
  request: CandidatePageRequestDto,
): Promise<IpcOutcome<CandidatePageDto>> {
  return await call<CandidatePageDto>(LIST_COMPARE_CANDIDATES, { request });
}

/**
 * Compare two stored builds and get the bounded summary.
 *
 * The whole diff stays in Rust and comes back as a session-local handle; the change tables are read
 * one page at a time against that handle (prompt §20). The request names two snapshot ids and nothing
 * else - no path, no table, no statement.
 */
export async function compareSnapshots(
  request: CompareRequestDto,
): Promise<IpcOutcome<CompareSummaryDto>> {
  return await call<CompareSummaryDto>(COMPARE_SNAPSHOTS, { request });
}

/** Ask for one bounded page of section changes of a comparison this session computed. */
export async function querySectionChanges(
  request: SectionChangeQueryDto,
): Promise<IpcOutcome<SectionChangePageDto>> {
  return await call<SectionChangePageDto>(QUERY_SECTION_CHANGES, { request });
}

/** Ask for one bounded page of symbol changes. */
export async function querySymbolChanges(
  request: SymbolChangeQueryDto,
): Promise<IpcOutcome<SymbolChangePageDto>> {
  return await call<SymbolChangePageDto>(QUERY_SYMBOL_CHANGES, { request });
}

/**
 * Export the comparison as JSON, asking the shell to open the save dialog.
 *
 * `cancelled` and `kept-existing` are statuses, not errors: the person chose a folder and then
 * declined, which is a normal thing to do. The path they picked never comes back to this side of the
 * boundary, and nothing here sends one out (prompt §35, §36).
 */
export async function exportCompareJson(diffId: string): Promise<IpcOutcome<ExportOutcomeDto>> {
  return await call<ExportOutcomeDto>(EXPORT_COMPARE_JSON, { diffId });
}

/** Export the comparison as one self-contained HTML file. */
export async function exportCompareHtml(diffId: string): Promise<IpcOutcome<ExportOutcomeDto>> {
  return await call<ExportOutcomeDto>(EXPORT_COMPARE_HTML, { diffId });
}

/**
 * Ask the shell to open the native dialog for a project's `firmwaresight.toml`.
 *
 * `null` means the person cancelled, which is a decision and not a failure: the page keeps the project
 * it already had. What comes back is the policy and the file's name — never the folder the file lives
 * in, which the WebView has no use for (prompt §40).
 */
export async function openProjectConfig(): Promise<IpcOutcome<ProjectContextDto | null>> {
  return await call<ProjectContextDto | null>(OPEN_PROJECT_CONFIG);
}

/**
 * Write the edited policy.
 *
 * With a project loaded the shell writes that project's own file; without one it opens a save dialog.
 * Either way the front end sends a structured policy and no path, so there is no command here that
 * could be widened into "write this arbitrary file" (`AGENTS.md` 7, prompt §41).
 */
export async function saveProjectPolicy(
  policy: ProjectPolicyDto,
): Promise<IpcOutcome<ProjectContextDto | null>> {
  return await call<ProjectContextDto | null>(SAVE_PROJECT_POLICY, { policy });
}

/**
 * Judge a stored build against the policy in force and store the run.
 *
 * The request names two snapshot ids and nothing else. The whole finding set crosses the boundary
 * because the MVP has ten rules, which is a bounded number by construction (prompt §46).
 */
export async function runReleaseGate(
  request: GateRunRequestDto,
): Promise<IpcOutcome<GateRunDto>> {
  return await call<GateRunDto>(RUN_RELEASE_GATE, { request });
}

/**
 * Accept one REVIEW finding, naming the person and the reason.
 *
 * The row it returns still reads REVIEW. Only the aggregate moves, and only for the finding this
 * acceptance names (prompt §48).
 */
export async function acceptReview(
  request: AcceptReviewRequestDto,
): Promise<IpcOutcome<AcceptReviewOutcomeDto>> {
  return await call<AcceptReviewOutcomeDto>(ACCEPT_REVIEW, { request });
}

/**
 * Read a stored run back.
 *
 * `null` means this database has no run with that id, which the screen labels as "not in history"
 * rather than rendering as an error (prompt §49).
 */
export async function getGateRun(runId: string): Promise<IpcOutcome<GateRunDto | null>> {
  return await call<GateRunDto | null>(GET_GATE_RUN, { runId });
}

/**
 * Prepare one Release Bundle plan and return the bounded preview a person authorizes.
 *
 * Nothing is written: §26 ends at a preview plus a session-local plan id, and the destination is not
 * part of the question yet. The request names three ids and no path — the build, the optional
 * baseline, and the Gate run the release is standing on (prompt §26, §27).
 */
export async function prepareReleaseBundle(
  request: BundlePlanRequestDto,
): Promise<IpcOutcome<BundlePreviewDto>> {
  return await call<BundlePreviewDto>(PREPARE_RELEASE_BUNDLE, { request });
}

/**
 * Ask the shell to open the native folder dialog for one prepared bundle.
 *
 * `null` means the person cancelled, which is a decision and not a failure: the plan stays prepared and
 * the preview stays on screen. What comes back is a token, the bundle's own folder name, and whether
 * that name is already taken — never the folder the person chose, which is why this side of the
 * boundary cannot write anywhere by itself (prompt §29).
 */
export async function chooseBundleDestination(
  planId: string,
): Promise<IpcOutcome<BundleDestinationDto | null>> {
  return await call<BundleDestinationDto | null>(CHOOSE_BUNDLE_DESTINATION, { planId });
}

/**
 * Write the bundle the preview authorized.
 *
 * `overwrite` is the release owner's explicit confirmation and nothing more: with `false` an occupied
 * destination is refused, and with `true` the shell still refuses a folder it did not write as a bundle
 * (prompt §30, §35). The three arguments are all handles this session issued.
 */
export async function exportReleaseBundle(
  planId: string,
  destinationToken: string,
  overwrite: boolean,
): Promise<IpcOutcome<BundleExportDto>> {
  return await call<BundleExportDto>(EXPORT_RELEASE_BUNDLE, {
    planId,
    destinationToken,
    overwrite,
  });
}

async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<IpcOutcome<T>> {
  try {
    return { ok: true, value: await invoke<T>(command, args) };
  } catch (reason: unknown) {
    return { ok: false, envelope: toEnvelope(reason) };
  }
}

/**
 * Narrow a rejection to the error envelope.
 *
 * A value crossing the Tauri boundary is JSON, so the shape is checked rather than asserted.
 * Anything that fails the check is reported as an internal error with its raw form attached -
 * guessing a code for it would write a fact that was never observed.
 */
export function toEnvelope(reason: unknown): ErrorEnvelopeDto {
  if (isErrorEnvelope(reason)) {
    return reason;
  }
  return {
    code: 'ERR-INTERNAL-9001',
    message: 'FirmwareSight hit an internal error.',
    operationId: 'unavailable',
    details: describe(reason),
    remediation: 'Report this message; no artifact data is needed.',
  };
}

export function isErrorEnvelope(value: unknown): value is ErrorEnvelopeDto {
  if (typeof value !== 'object' || value === null) {
    return false;
  }
  const candidate: Record<string, unknown> = { ...value };
  return (
    typeof candidate['code'] === 'string' &&
    typeof candidate['message'] === 'string' &&
    typeof candidate['operationId'] === 'string'
  );
}

function describe(reason: unknown): string {
  if (typeof reason === 'string') {
    return reason;
  }
  if (reason instanceof Error) {
    return reason.message;
  }
  try {
    return JSON.stringify(reason) ?? String(reason);
  } catch {
    // Circular or otherwise unserializable; the type name is still honest evidence.
    return `unrepresentable ${typeof reason}`;
  }
}
