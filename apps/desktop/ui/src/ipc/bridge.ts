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
  ErrorEnvelopeDto,
  EvidencePageDto,
  EvidenceRequestDto,
  SectionPageDto,
  SectionRequestDto,
  SelectionDto,
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
