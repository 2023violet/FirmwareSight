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
  AnalysisSummaryDto,
  CandidatePageDto,
  CandidatePageRequestDto,
  CompareRequestDto,
  CompareSummaryDto,
  ErrorEnvelopeDto,
  EvidencePageDto,
  EvidenceRequestDto,
  ExportOutcomeDto,
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
